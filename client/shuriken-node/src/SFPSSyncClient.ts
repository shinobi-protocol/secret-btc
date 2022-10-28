/* eslint-disable @typescript-eslint/no-non-null-assertion */
import { Bech32 } from 'secretjs';
import { PublicKey } from 'secretjs/dist/protobuf_stuff/tendermint/crypto/keys';
import { Validator } from 'secretjs/dist/protobuf_stuff/tendermint/types/validator';
import { Validator as ValidatorResponse } from 'secretjs/dist/protobuf_stuff/cosmos/base/tendermint/v1beta1/query';
import { ExecuteResult } from 'sbtc-js/build/contracts/ContractClient';
import { SFPSClient } from 'sbtc-js/build/contracts/sfps/SFPSClient';
import { HandleMsg } from 'sbtc-js/build/contracts/shuriken/types';
import { ShurikenClient } from 'sbtc-js/build/contracts/shuriken/ShurikenClient';
import { Logger } from 'winston';
import { Header } from 'secretjs/dist/protobuf_stuff/tendermint/types/types';
import { PrefixedLogger } from './PrefixedLogger';

export default class SFPSSyncClient {
    shurikenClient: ShurikenClient;
    sfpsClient: SFPSClient;
    networkBestHeader?: Header;
    contractBestHeader?: Header;
    contractBestHeaderIndex?: number;
    maxInterval?: number;
    blocksPerTx: number;
    checkedHeight?: number;
    gasUsedOfLastOutOfGas?: number;
    headersForSync: Header[] = [];
    logger: PrefixedLogger;

    constructor(
        shurikenClient: ShurikenClient,
        sfpsClient: SFPSClient,
        blocksPerTx: number,
        logger: Logger
    ) {
        this.shurikenClient = shurikenClient;
        this.sfpsClient = sfpsClient;
        this.blocksPerTx = blocksPerTx;
        this.logger = new PrefixedLogger(logger, '[SFPSSyncClient]');
    }
    public async fetchNetworkBestHeader(): Promise<void> {
        this.networkBestHeader = (
            await this.sfpsClient.secretNetworkClient.query.tendermint.getLatestBlock(
                {}
            )
        ).block!.header!;
    }

    public async fetchContractBestHeader(): Promise<void> {
        this.contractBestHeader = (
            await this.sfpsClient.secretNetworkClient.query.tendermint.getBlockByHeight(
                {
                    height: (
                        await this.sfpsClient.currentHighestHeaderHeight()
                    ).toString(),
                }
            )
        ).block!.header!;
        this.contractBestHeaderIndex = (await this.sfpsClient.hashListLength())-1;
        if (
            !this.checkedHeight ||
            parseInt(this.contractBestHeader.height) > this.checkedHeight
        ) {
            this.checkedHeight = parseInt(this.contractBestHeader.height);
        }
    }

    public async fetchMaxInterval(): Promise<void> {
        this.maxInterval = await this.sfpsClient.maxInterval();
    }

    public async syncHeaders(
        headers: Header[]
    ): Promise<ExecuteResult<HandleMsg, void>> {
        const lightBlocks = await Promise.all(
            headers.map(async (header) => {
                const height = parseInt(header.height);
                const commit = (
                    await this.shurikenClient.secretNetworkClient.query.tendermint.getBlockByHeight(
                        { height: (height + 1).toString() }
                    )
                ).block!.lastCommit!;
                const validatorsResponse: ValidatorResponse[] = (
                    await this.shurikenClient.secretNetworkClient.query.tendermint.getValidatorSetByHeight(
                        { height: header.height, pagination: { limit: '200' } }
                    )
                ).validators;
                const validators = parseValidatorsResponse(validatorsResponse);
                const totalVotingPower = validators.reduce(
                    (previous, current) =>
                        previous + parseInt(current.votingPower),
                    0
                );
                return {
                    signedHeader: {
                        header: header,
                        commit: commit,
                    },
                    validatorSet: {
                        validators,
                        totalVotingPower: totalVotingPower.toString(),
                    },
                };
            })
        );
        console.log('msg length: ', JSON.stringify(lightBlocks).length);
        const committedHashes = await this.sfpsClient.verifySubsequentLightBlocks(
            this.contractBestHeader!,
            this.contractBestHeaderIndex!,
            lightBlocks
        );
        console.log('committedHashes', JSON.stringify(committedHashes));
        return await this.shurikenClient.proxySFPSAppendSubsequentHashes(
            committedHashes,
        );
    }

    public async syncSFPSHeaders(): Promise<ExecuteResult<HandleMsg, void>[]> {
        this.logger.log('Start Sync Job');
        await this.fetchMaxInterval();
        await this.fetchContractBestHeader();
        await this.fetchNetworkBestHeader();
        const maxHeight = parseInt(this.networkBestHeader!.height);
        this.logger.log(`Best height on network: ${maxHeight}`);
        const results = [];
        const contractHeight = parseInt(this.contractBestHeader!.height);
        this.logger.log(`Best height on contract: ${contractHeight} `);
        let lastHeight = contractHeight;
        while (this.headersForSync.length > 0) {
            const height = parseInt(
                this.headersForSync[this.headersForSync.length - 1].height
            );
            if (lastHeight > height) {
                this.headersForSync = this.headersForSync.slice(1);
            } else {
                lastHeight = height;
                break;
            }
        }
        for (
            let height = this.checkedHeight! + 1;
            height < maxHeight && this.headersForSync.length < this.blocksPerTx;
            height++
        ) {
            const header = (
                await this.shurikenClient.secretNetworkClient.query.tendermint.getBlockByHeight(
                    { height: height.toString() }
                )
            ).block!.header!;
            this.logger.log(`check ${height} `);
            if (
                header.validatorsHash.toString() !=
                    header.nextValidatorsHash.toString() ||
                height - lastHeight == this.maxInterval
            ) {
                this.logger.log(`sync ${height} `);
                this.headersForSync.push(header);
                this.logger.log(`headersForSync ${this.headersForSync.length}`);
                lastHeight = parseInt(header.height);
            }
            this.checkedHeight = height;
        }
        while (this.headersForSync.length >= this.blocksPerTx) {
            const headers = this.headersForSync.slice(0, this.blocksPerTx);
            this.headersForSync = this.headersForSync.slice(this.blocksPerTx);
            const result = await this.syncHeaders(headers);
            results.push(result);
        }
        this.logger.log('Finish Sync Job');
        return results;
    }
}

interface PubKey {
    typeUrl: string;
    value: string | Uint8Array;
}

function parseValidatorsResponse(
    validatorsResponse: ValidatorResponse[]
): Validator[] {
    return validatorsResponse.map((validator) => {
        const pubKeyResponse = validator.pubKey! as PubKey;
        const address = Bech32.decode(validator.address).data;
        const pubKey = ((): PublicKey => {
            if (pubKeyResponse.typeUrl === '/cosmos.crypto.ed25519.PubKey') {
                return {
                    ed25519: pubKeyResponse.value.slice(2) as Uint8Array,
                    secp256k1: undefined,
                };
            } else {
                return {
                    ed25519: undefined,
                    secp256k1: pubKeyResponse.value.slice(2) as Uint8Array,
                };
            }
        })();
        return {
            address,
            pubKey,
            votingPower: validator.votingPower,
            proposerPriority: validator.proposerPriority,
        };
    });
}
