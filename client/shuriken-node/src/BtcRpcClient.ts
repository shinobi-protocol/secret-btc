import axios from 'axios';
import { Block } from 'bitcoinjs-lib';
import BtcClientInterface from './BtcClientInterface';

export default class BtcRpcClient implements BtcClientInterface {
    url: string;
    user: string;
    password: string;
    constructor(url: string, user: string, password: string) {
        this.url = url;
        this.user = user;
        this.password = password;
    }
    public async getBlockHeight(id: string): Promise<number> {
        interface Answer {
            height: number;
        }
        return (await this.request<Answer>('getblockheader', [id, true]))
            .height;
    }
    public async getBlockHeader(id: string): Promise<Block> {
        return Block.fromHex(
            await this.request<string>('getblockheader', [id, false])
        );
    }
    public async getBestBlockHeight(): Promise<number> {
        return this.getBlockHeight(await this.getBestBlockHash());
    }
    public async getBestBlockHeader(): Promise<Block> {
        return this.getBlockHeader(await this.getBestBlockHash());
    }

    private getBestBlockHash(): Promise<string> {
        return this.request<string>('getbestblockhash', []);
    }

    /*
    public async getBlockHash(height: number): Promise<Buffer> {
        try {
            return Buffer.from(
                await this.request<string>('getblockhash', [height]),
                'hex'
            );
        } catch (error: unknown) {
            interface Err {
                response: {
                    data: {
                        error: {
                            code: number;
                        };
                    };
                };
            }
            const e = error as Err;
            if (
                e.response &&
                e.response.data &&
                e.response.data.error &&
                e.response.data.error.code === -8
            ) {
                throw new BlockHeightOutOfRangeError(height);
            }
            throw error;
        }
    }

    public async getBlock(hash: Buffer): Promise<Block> {
        return Block.fromHex(
            await this.request<string>('getblock', [hash.toString('hex'), 0])
        );
    }

    public async getBlockHeader(hash: Buffer): Promise<Block> {
        return Block.fromHex(
            await this.request<string>('getblockheader', [
                hash.toString('hex'),
                false,
            ])
        );
    }

    public async getBlockHeight(hash: Buffer): Promise<number> {
        interface Answer {
            height: number;
        }
        return (
            await this.request<Answer>('getblock', [hash.toString('hex'), 1])
        ).height;
    }

    public async getChainTips(): Promise<ChainTip[]> {
        return await this.request('getchaintips', []);
    }
    */

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    private async request<R>(method: string, params: any[]): Promise<R> {
        const id = Math.random().toString();
        const ret: { data: { result: R } } = await axios.post(
            this.url,
            {
                jsonrpc: '1.0',
                id: id,
                method: method,
                params: params,
            },
            {
                headers: { 'Content-Type': 'text/plain' },
                auth: {
                    username: this.user,
                    password: this.password,
                },
            }
        );
        return ret.data.result;
    }
}
