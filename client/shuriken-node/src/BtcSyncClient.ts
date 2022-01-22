import { ExecuteResult } from 'sbtc-js/build/contracts/ContractClient';
import { ShurikenClient } from 'sbtc-js/build/contracts/shuriken/ShurikenClient';
import { HandleMsg } from 'sbtc-js/build/contracts/shuriken/types';
import { Logger } from 'winston';
import BtcClientInterface from './BtcClientInterface';
import { chunkArray } from './chunkArray';
import { PrefixedLogger } from './PrefixedLogger';
import { BitcoinSPVClient } from 'sbtc-js/build/contracts/bitcoin_spv/BitcoinSPVClient';
import { Block } from 'sbtc-js/node_modules/bitcoinjs-lib';

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

    public async estimateGasUsed(): Promise<number | undefined> {
        const searchBlockRange = 1000;
        const currentBlock = await this.shurikenClient.signingCosmWasmClient.getBlock();
        const query = `wasm.contract_address=${
            this.shurikenClient.contractAddress
        }&message.signer=${this.shurikenClient.senderAddress()}&tx.minheight=${Math.max(
            currentBlock.header.height - searchBlockRange,
            1
        )}`;
        try {
            const result = await this.shurikenClient.signingCosmWasmClient.restClient.txsQuery(
                `${query}&limit=100`
            );
            const proxyGasUsedArray = result.txs
                .filter(
                    (tx) =>
                        tx.logs![0].events.filter(
                            (event) =>
                                event.type === 'wasm' &&
                                event.attributes.filter(
                                    (event) =>
                                        event.key === 'contract_address' &&
                                        event.value ===
                                            this.bitcoinSPVClient
                                                .contractAddress
                                ).length
                        ).length &&
                        tx.logs![0].events.filter(
                            (event) =>
                                event.type === 'message' &&
                                event.attributes.filter(
                                    (event) =>
                                        event.key === 'action' &&
                                        event.value === 'execute'
                                )
                        ).length
                )
                .map((tx) => parseInt(tx.gas_used!, 10));
            return proxyGasUsedArray.length
                ? Math.ceil(Math.max(...proxyGasUsedArray) * 1.1)
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

    public async syncBitcoinHeaders(): Promise<
        ExecuteResult<HandleMsg, void>[]
    > {
        this.logger.log('Start Sync Job');
        const spvBestHash = (
            await this.bitcoinSPVClient.bestHeaderHash()
        ).toString('hex');
        const spvBestHeight = await this.btcClient.getBlockHeight(spvBestHash);
        this.logger.log(`SPV Best Header: ${spvBestHeight} ${spvBestHash}`);
        const btcBestHeight = await this.btcClient.getBestBlockHeight();
        const btcBestId = (await this.btcClient.getBestBlockHeader()).getId();
        this.logger.log(`Bitcoin Best Header: ${btcBestHeight} ${btcBestId}`);

        const headers: Block[] = [];
        const prevHash = Buffer.alloc(32);
        let id = btcBestId;
        for (;;) {
            // extend header chain
            if (id === spvBestHash) {
                this.logger.log('extend header chain');
                break;
            }
            // detect chain folk parent
            if (
                headers.length > btcBestHeight - spvBestHeight &&
                id ===
                    (
                        await this.bitcoinSPVClient.blockHeader(
                            btcBestHeight - headers.length
                        )
                    ).getId()
            ) {
                this.logger.log('detect chain folk parent');
                break;
            }
            const header = await this.btcClient.getBlockHeader(id);
            headers.push(header);
            header.prevHash?.copy(prevHash);
            id = prevHash.reverse().toString('hex');
        }
        headers.reverse();
        let tipHeight = btcBestHeight - headers.length;
        const chunkedHeaders = chunkArray(headers, this.blocksPerTx);
        const results = [];
        for (const chunk of chunkedHeaders) {
            tipHeight += chunk.length;
            const result = await this.shurikenClient.proxyBitcoinSPVAddHeaders(
                tipHeight,
                chunk,
                await this.estimateGasUsed()
            );
            results.push(result);
        }
        this.logger.log('Finish Sync Job');
        return results;
    }
}
