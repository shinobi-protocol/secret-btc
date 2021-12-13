import { Block } from 'bitcoinjs-lib';
import { RegtestUtils } from 'regtest-client';
import BtcClientInterface from './BtcClientInterface';

// TODO Packaging
export default class RegtestUtilClient
    extends RegtestUtils
    implements BtcClientInterface {
    protected url;
    constructor(url = 'http://127.0.0.1:8080/1') {
        super({ APIURL: url });
        this.url = url;
    }
    getBlockHeight(id: string): Promise<number> {
        return this.dhttp({
            method: 'GET',
            url: `${this.url}/b/${id}/height`,
        }) as Promise<number>;
    }
    async getBlockHeader(id: string): Promise<Block> {
        return Block.fromHex(
            (await this.dhttp({
                method: 'GET',
                url: `${this.url}/b/${id}/header`,
            })) as string
        );
    }
    getBestBlockHeight(): Promise<number> {
        return this.height();
    }
    async getBlock(id: string): Promise<Block> {
        return Block.fromHex(
            (await this.dhttp({
                method: 'GET',
                url: `${this.url}/b/${id}/block`,
            })) as string
        );
    }
    async getBlockByTxId(id: string): Promise<Block> {
        const blockId = (await this.dhttp({
            method: 'GET',
            url: `${this.url}/t/${id}/block`,
        })) as string;
        return Block.fromHex(
            (await this.dhttp({
                method: 'GET',
                url: `${this.url}/b/${blockId}/block`,
            })) as string
        );
    }
    async getBestBlockHeader(): Promise<Block> {
        return Block.fromHex(
            (await this.dhttp({
                method: 'GET',
                url: `${this.url}/b/best/header`,
            })) as string
        );
    }
}
