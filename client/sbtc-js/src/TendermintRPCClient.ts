import axios, { AxiosError } from 'axios';
import { Type } from './contracts/sfps/types';

/**
 * Tendermint RPC provides api about tendermint consensus
 * https://docs.tendermint.com/master/rpc/
 */
export class TendermintRPCClient {
    url: string;
    constructor(url: string) {
        this.url = url;
    }
    public async getBlock(height?: number): Promise<BlockResponse> {
        return await this.request<BlockResponse>('/block', { height });
    }
    /*
    public async getLatestBlocks(): Promise<BlockChainResponse> {
        return await this.request<BlockChainResponse>('/blockchain');
    }
    */
    public async getTxsInBlock(height: number): Promise<Tx[]> {
        const txs = [];
        const perPage = 100;
        for (let page = 1; ; page++) {
            const response = await this.request<TxsInBlockResponse>(
                '/tx_search',
                {
                    query: `"tx.height=${height.toString()}"`,
                    page,
                    per_page: perPage,
                }
            );
            txs.push(...response.txs);
            if (txs.length === parseInt(response.total_count)) {
                break;
            }
        }
        return txs;
    }
    public async getBlockByHash(hash: string): Promise<BlockResponse> {
        return await this.request<BlockResponse>('/block_by_hash', {
            hash: '0x' + hash,
        });
    }
    public async getValidators(height?: number): Promise<Validator[]> {
        const validators = [];
        const perPage = 100;
        for (let page = 1; ; page++) {
            const response = await this.request<ValidatorsResponse>(
                '/validators',
                {
                    height,
                    page,
                    per_page: perPage,
                }
            );
            validators.push(...response.validators);
            if ((validators.length, parseInt(response.total))) {
                break;
            }
        }
        return validators;
    }

    private async request<R>(
        path: string,
        params?: Record<string, unknown>
    ): Promise<R> {
        const isRetryable = (e: AxiosError) => {
            return !!(
                e.response &&
                e.response.status === 429 &&
                e.response.headers['retry-after']
            ); //eslint-disable-line
        };
        const isAxiosError = (error: any): error is AxiosError => {
            return !!error.isAxiosError; //eslint-disable-line
        };

        for (;;) {
            try {
                const ret: { data: { result: R } } = await axios(
                    this.url + path,
                    {
                        params,
                    }
                );
                return ret.data.result;
            } catch (e) {
                console.log('Tendermint RPC returns error: ', e);
                if (!isAxiosError(e)) throw e;
                if (isRetryable(e)) {
                    const waitTime = e.response!.headers['retry-after']; //eslint-disable-line
                    console.log(
                        'Tendermint RPC returns error: 429',
                        waitTime * 1000
                    );
                    await new Promise((resolve) =>
                        setTimeout(resolve, waitTime * 1000)
                    ); //eslint-disable-line
                } else {
                    throw e;
                }
            }
        }
    }
}

export interface BlockResponse {
    block_id: Blockid;
    block: Block;
}

export interface Block {
    header: Header;
    data: Data;
    evidence: Evidence;
    last_commit: Lastcommit;
}

export interface Lastcommit {
    height: string;
    round: number;
    block_id: Blockid;
    signatures: Signature[];
}

export interface Signature {
    block_id_flag: number;
    validator_address: string;
    timestamp: string;
    signature: string;
}

export interface Evidence {
    evidence?: any;
}

export interface Data {
    txs?: any;
}

export interface Header {
    version: Version;
    chain_id: string;
    height: string;
    time: string;
    last_block_id: Blockid;
    last_commit_hash: string;
    data_hash: string;
    validators_hash: string;
    next_validators_hash: string;
    consensus_hash: string;
    app_hash: string;
    last_results_hash: string;
    evidence_hash: string;
    proposer_address: string;
}

export interface Version {
    block: string;
    app: string;
}

export interface Blockid {
    hash: string;
    parts: Parts;
}

export interface Parts {
    total: number;
    hash: string;
}

export interface ValidatorsResponse {
    block_height: string;
    validators: Validator[];
    count: string;
    total: string;
}

export interface Validator {
    address: string;
    pub_key: Pubkey;
    voting_power: string;
    proposer_priority: string;
}

export interface Pubkey {
    type: Type;
    value: string;
}

interface Event {
    type: string;
    attributes: Attribute[];
}

interface Attribute {
    key: string;
    value: string;
}

export interface BlockChainResponse {
    last_height: string;
    block_metas: BlockMeta[];
}

export interface BlockMeta {
    block_id: BlockID;
    block_size: number;
    header: Header;
    num_txs: string;
}

export interface BlockID {
    hash: string;
    parts: Parts;
}

export interface Version {
    block: string;
    app: string;
}

export interface TxsInBlockResponse {
    txs: Tx[];
    total_count: string;
}

export interface Tx {
    hash: string;
    height: string;
    index: number;
    tx_result: TxResult;
    tx: string;
}

export interface TxResult {
    code: number;
    data: string;
    log: string;
    info: string;
    gas_wanted: string;
    gas_used: string;
    events: Event[];
    codespace: string;
}
