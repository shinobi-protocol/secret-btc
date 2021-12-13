import { Block } from 'bitcoinjs-lib';

export default interface BtcClientInterface {
    getBlockHeight(id: string): Promise<number>;
    getBlockHeader(id: string): Promise<Block>;

    getBestBlockHeight(): Promise<number>;
    getBestBlockHeader(): Promise<Block>;
}
