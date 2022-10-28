/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-var-requires */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-non-null-assertion */
import { Account } from 'secretjs';
import { Block, networks, Transaction } from 'bitcoinjs-lib';
import { BitcoinSPVHandleMsg, QueryMsg, QueryAnswer } from './types';
import {
    ContractClient,
    ExecuteResult as GenericExecuteResult,
} from '../ContractClient';
import { MerkleProof } from './BitcoinMerkleTree';

type ExecuteResult<ANSWER> = GenericExecuteResult<BitcoinSPVHandleMsg, ANSWER>;

interface Config {
    btcNetwork: networks.Network;
    confirmation: number;
}

class BitcoinSPVClient extends ContractClient<
    BitcoinSPVHandleMsg,
    QueryMsg,
    QueryAnswer
> {
    // handle
    public async addHeaders(
        tip_height: number,
        blocks: Block[],
        gasLimit?: number
    ): Promise<ExecuteResult<void>> {
        if (!gasLimit) {
            const baseFee = 3350000;
            const feePerBlock = 18000;
            gasLimit = baseFee + feePerBlock * blocks.length;
        }
        return await this.execute(
            {
                add_headers: {
                    tip_height,
                    headers: blocks.map((header) =>
                        header.toBuffer(true).toString('base64')
                    ),
                },
            },
            gasLimit,
            () => void 0
        );
    }

    // query
    public async bestHeaderHash(): Promise<Buffer> {
        return await this.query(
            {
                best_header_hash: {},
            },
            (answer) => Buffer.from(answer.best_header_hash!.hash, 'hex')
        );
    }

    public async blockHeader(height: number): Promise<Block> {
        return await this.query(
            {
                block_header: { height },
            },
            (answer) =>
                Block.fromBuffer(
                    Buffer.from(answer.block_header!.header, 'base64')
                )
        );
    }

    public async verifyMerkleProof(
        height: number,
        tx: Transaction,
        merkle_proof: MerkleProof
    ): Promise<boolean> {
        const msg = {
            verify_merkle_proof: {
                height,
                tx: tx.toBuffer().toString('base64'),
                merkle_proof: {
                    prefix: merkle_proof.prefix,
                    siblings: merkle_proof.siblings.map((sibling) => {
                        const copy = Buffer.alloc(32);
                        sibling.copy(copy);
                        return copy.reverse().toString('hex');
                    }),
                },
            },
        };
        return await this.query(
            msg,
            (answer) => answer.verify_merkle_proof!.success
        );
    }

    public async config(): Promise<Config> {
        return await this.query(
            {
                config: {},
            },
            (answer) => {
                const raw = answer.config!;

                return {
                    confirmation: raw.confirmation,
                    btcNetwork: BitcoinSPVClient.network(raw.bitcoin_network),
                } as Config;
            }
        );
    }

    private static network(network: string): networks.Network {
        switch (network) {
            case 'bitcoin':
                return networks.bitcoin;
            case 'testnet':
                return networks.testnet;
            case 'regtest':
                return networks.regtest;
            default:
                throw new Error('unknown network');
        }
    }
}

export { BitcoinSPVClient, Account, Block, Transaction, Config, networks };
