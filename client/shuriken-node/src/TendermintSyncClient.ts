/* eslint-disable @typescript-eslint/no-non-null-assertion */
import { ExecuteResult } from 'sbtc-js/build/contracts/ContractClient';
import { SFPSClient } from 'sbtc-js/build/contracts/sfps/SFPSClient';
import { HandleMsg } from 'sbtc-js/build/contracts/shuriken/types';
import { ShurikenClient } from 'sbtc-js/build/contracts/shuriken/ShurikenClient';
import { Header, TendermintRPCClient } from 'sbtc-js/build/TendermintRPCClient';
import { Logger } from 'winston';
import { PrefixedLogger } from './PrefixedLogger';

export default class TendermintSyncClient {
    shurikenClient: ShurikenClient;
    sfpsClient: SFPSClient;
    tendermintClient: TendermintRPCClient;
    networkBestHeader?: Header;
    contractBestHeader?: Header;
    maxInterval?: number;
    blocksPerTx: number;
    checkedHeight?: number;
    gasUsedOfLastOutOfGas?: number;
    headersForSync: Header[] = [];
    logger: PrefixedLogger;

    constructor(
        shurikenClient: ShurikenClient,
        sfpsClient: SFPSClient,
        tendermintClient: TendermintRPCClient,
        blocksPerTx: number,
        logger: Logger
    ) {
        this.shurikenClient = shurikenClient;
        this.sfpsClient = sfpsClient;
        this.tendermintClient = tendermintClient;
        this.blocksPerTx = blocksPerTx;
        this.logger = new PrefixedLogger(logger, '[TendermintSyncClient]');
    }
    public async fetchNetworkBestHeader(): Promise<void> {
        this.networkBestHeader = (
            await this.tendermintClient.getBlock()
        ).block.header;
    }

    public async estimateGasUsed(): Promise<number | undefined> {
        if (this.gasUsedOfLastOutOfGas) {
            const estimated = Math.min(
                this.gasUsedOfLastOutOfGas * 1.5,
                9999999
            );
            this.gasUsedOfLastOutOfGas = undefined;
            return estimated;
        }
        const searchBlockRange = 1000;
        const currentBlock = (await this.shurikenClient.secretNetworkClient.query.tendermint.getLatestBlock({})).block!;
        const query = `wasm.contract_address='${this.shurikenClient.contractAddress
            }' AND message.signer='${this.shurikenClient.senderAddress()}' AND tx.minheight=${Math.max(
                parseInt(currentBlock.header!.height!, 10) - searchBlockRange,
                1
            )}`;
        try {
            const result = await this.shurikenClient.secretNetworkClient.query.txsQuery(
                query
            );
            console.log('txquery', query);
            console.log('txquery result', JSON.stringify(result));
            const proxyGasUsedArray = result
                .filter(
                    (tx) =>
                        tx.jsonLog![0].events.filter(
                            (event) =>
                                event.type === 'wasm' &&
                                event.attributes.filter(
                                    (event) =>
                                        event.key === 'contract_address' &&
                                        event.value ===
                                        this.sfpsClient.contractAddress
                                ).length
                        ).length &&
                        tx.jsonLog![0].events.filter(
                            (event) =>
                                event.type === 'message' &&
                                event.attributes.filter(
                                    (event) =>
                                        event.key === 'action' &&
                                        event.value === 'execute'
                                )
                        ).length
                )
                .map((tx) => tx.gasUsed!);
            return proxyGasUsedArray.length
                ? Math.min(
                    Math.ceil(Math.max(...proxyGasUsedArray) * 1.1),
                    9999999
                )
                : undefined;
        } catch (e) {
            /// if search query does not match, rest client throws following error
            if ((e as Error).message === 'Unexpected response data format') {
                return undefined;
            } else {
                throw e;
            }
        }
    }

    public async fetchContractBestHeader(): Promise<void> {
        this.contractBestHeader = (
            await this.tendermintClient.getBlockByHash(
                await this.sfpsClient.currentHighestHeaderHash()
            )
        ).block.header;
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
                    await this.tendermintClient.getBlock(height + 1)
                ).block.last_commit;
                const validators = await this.tendermintClient.getValidators(
                    height
                );
                return {
                    signed_header: {
                        header: header,
                        commit: commit,
                    },
                    validators: validators,
                };
            })
        );
        console.log('msg length: ', JSON.stringify(lightBlocks).length);
        const committedHashes = await this.sfpsClient.verifySubsequentLightBlocks(this.contractBestHeader!, lightBlocks);
        console.log('commitedHashes', JSON.stringify(committedHashes));
        return await this.shurikenClient.proxySFPSAppendSubsequentHashes(
            committedHashes,
            headers[headers.length - 1],
            await this.estimateGasUsed()
        );
    }

    public async syncTendermintHeaders(): Promise<
        ExecuteResult<HandleMsg, void>[]
    > {
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
            const header = (await this.tendermintClient.getBlock(height)).block
                .header;
            this.logger.log(`check ${height} `);
            if (
                header.validators_hash != header.next_validators_hash ||
                height - lastHeight == this.maxInterval
            ) {
                this.logger.log(`sync ${height} `);
                this.headersForSync.push(header);
                lastHeight = parseInt(header.height);
            }
            this.checkedHeight = height;
        }
        while (this.headersForSync.length >= this.blocksPerTx) {
            const headers = this.headersForSync.slice(0, this.blocksPerTx);
            this.headersForSync = this.headersForSync.slice(this.blocksPerTx);
            try {
                const result = await this.syncHeaders(headers);
                results.push(result);
            } catch (e) {
                const searchResult = /gasWanted: [0-9]+, gasUsed: [0-9]+: out of gas./.exec(
                    (e as Error).message
                );
                if (searchResult) {
                    this.gasUsedOfLastOutOfGas = parseInt(
                        /[0-9]+/.exec(searchResult[0])![1]
                    );
                    this.logger.log(
                        'out of gas. gasUsed = ' +
                        this.gasUsedOfLastOutOfGas.toString()
                    );
                    this.headersForSync = headers.concat(this.headersForSync);
                } else {
                    throw e;
                }
            }
        }
        this.logger.log('Finish Sync Job');
        return results;
    }
}
