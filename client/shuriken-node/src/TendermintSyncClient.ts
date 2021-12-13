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
    checkedHeight?: number;
    logger: PrefixedLogger;

    constructor(
        shurikenClient: ShurikenClient,
        sfpsClient: SFPSClient,
        tendermintClient: TendermintRPCClient,
        logger: Logger
    ) {
        this.shurikenClient = shurikenClient;
        this.sfpsClient = sfpsClient;
        this.tendermintClient = tendermintClient;
        this.logger = new PrefixedLogger(logger, '[TendermintSyncClient]');
    }

    public async fetchNetworkBestHeader(): Promise<void> {
        this.networkBestHeader = (
            await this.tendermintClient.getBlock()
        ).block.header;
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

    public async syncHeader(
        header: Header
    ): Promise<ExecuteResult<HandleMsg, void>> {
        const height = parseInt(header.height);
        const commit = (await this.tendermintClient.getBlock(height + 1)).block
            .last_commit;
        const validators = await this.tendermintClient.getValidators(height);
        const lightBlock = {
            signed_header: {
                header: header,
                commit: commit,
            },
            validators: validators,
        };

        return await this.shurikenClient.proxySFPSAddLightBlock(
            this.contractBestHeader!,
            lightBlock
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
        for (
            let height = this.checkedHeight! + 1;
            height < maxHeight;
            height++
        ) {
            const header = (await this.tendermintClient.getBlock(height)).block
                .header;
            const contractHeight = parseInt(this.contractBestHeader!.height);
            const authorizedValidators = this.contractBestHeader!
                .next_validators_hash;
            if (
                authorizedValidators != header.next_validators_hash ||
                height - contractHeight == this.maxInterval
            ) {
                this.logger.log(`sync ${height}`);
                const result = await this.syncHeader(header);
                results.push(result);
                this.contractBestHeader = header;
            }
        }
        this.logger.log('Finish Sync Job');
        this.checkedHeight = maxHeight - 1;
        return results;
    }
}
