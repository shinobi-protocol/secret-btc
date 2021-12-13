import { ExecuteResult } from 'sbtc-js/build/contracts/ContractClient';
import { ShurikenClient } from 'sbtc-js/build/contracts/shuriken/ShurikenClient';
import { HandleMsg } from 'sbtc-js/build/contracts/shuriken/types';
import { Logger } from 'winston';
import BtcClientInterface from './BtcClientInterface';
import { chunkArray } from './chunkArray';
import { PrefixedLogger } from './PrefixedLogger';
import { BitcoinSPVClient } from 'sbtc-js/build/contracts/bitcoin_spv/BitcoinSPVClient';

export default class BtcSyncClient {
    shurikenClient: ShurikenClient;
    bitcoinSPVClient: BitcoinSPVClient;
    btcClient: BtcClientInterface;
    blocksPerTx: number;
    logger: PrefixedLogger;

    constructor(
        shurikenClient: ShurikenClient,
        bitcoinSPVClient: BitcoinSPVClient,
        btcClient: BtcClientInterface,
        blocksPerTx: number,
        logger: Logger
    ) {
        this.shurikenClient = shurikenClient;
        this.bitcoinSPVClient = bitcoinSPVClient;
        this.btcClient = btcClient;
        this.blocksPerTx = blocksPerTx;
        this.logger = new PrefixedLogger(logger, '[BtcSyncClient]');
    }
    public async syncBitcoinHeaders(): Promise<
        ExecuteResult<HandleMsg, void>[]
    > {
        this.logger.log('Start Sync Job');
        const spvBestHash = (
            await this.bitcoinSPVClient.bestHeaderHash()
        ).toString('hex');
        let spvBestHeight = await this.btcClient.getBlockHeight(spvBestHash);
        this.logger.log(`SPV Best Header: ${spvBestHeight} ${spvBestHash}`);
        const btcBestHeight = await this.btcClient.getBestBlockHeight();
        const btcBestId = (await this.btcClient.getBestBlockHeader()).getId();
        this.logger.log(`Bitcoin Best Header: ${btcBestHeight} ${btcBestId}`);

        const headers = [];
        let header = await this.btcClient.getBlockHeader(btcBestId);
        const prevHash = Buffer.alloc(32);
        header.prevHash?.copy(prevHash);
        let prevId = prevHash.reverse().toString('hex');
        while (prevId !== undefined && header.getId() !== spvBestHash) {
            headers.push(header);
            header = await this.btcClient.getBlockHeader(prevId);
            header.prevHash?.copy(prevHash);
            prevId = prevHash.reverse().toString('hex');
        }
        headers.reverse();
        const chunkedHeaders = chunkArray(headers, this.blocksPerTx);
        const results = [];
        for (const chunk of chunkedHeaders) {
            spvBestHeight += chunk.length;
            const result = await this.shurikenClient.proxyBitcoinSPVAddHeaders(
                spvBestHeight,
                chunk
            );
            results.push(result);
        }
        this.logger.log('Finish Sync Job');
        return results;
    }
}
