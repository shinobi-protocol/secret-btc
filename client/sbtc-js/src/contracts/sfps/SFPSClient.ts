/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-non-null-assertion */
import { Account } from 'secretjs';
import { TxResult } from '../../TendermintRPCClient';
import {
    ContractClient,
    ExecuteResult as GenericExecuteResult,
} from '../ContractClient';
import { MerkleProof } from './TendermintMerkleTree';
import {
    SFPSHandleMsg,
    QueryMsg,
    QueryAnswer,
    LightBlock,
    CurrentHighestHeaderObject,
    HeaderElement,
} from './types';

type ExecuteResult<ANSWER> = GenericExecuteResult<SFPSHandleMsg, ANSWER>;
class SFPSClient extends ContractClient<SFPSHandleMsg, QueryMsg, QueryAnswer> {
    // handle
    public async addLightBlock(
        current_highest_header: CurrentHighestHeaderObject,
        light_block: LightBlock,
        gasLimit?: number
    ): Promise<ExecuteResult<void>> {
        return await this.execute(
            {
                add_light_block: {
                    current_highest_header,
                    light_block,
                },
            },
            gasLimit || 2800000,
            () => void 0
        );
    }

    public async verityTxResultProof(
        headers: HeaderElement[],
        encryption_key: string,
        merkle_proof: MerkleProof,
        tx_result: TxResult,
        header_hash_index: number
    ): Promise<Buffer> {
        const message: QueryMsg = {
            verify_tx_result_proof: {
                encryption_key,
                header_hash_index,
                tx_result_proof: {
                    headers: headers,
                    merkle_proof: {
                        total: merkle_proof.total,
                        index: merkle_proof.index,
                        leaf_hash: merkle_proof.leafHash.toString('hex'),
                        aunts: merkle_proof.aunts.map((aunt) =>
                            aunt.toString('hex')
                        ),
                    },
                    tx_result: {
                        code: tx_result.code,
                        data: tx_result.data,
                        gas_used: tx_result.gas_used,
                        gas_wanted: tx_result.gas_wanted,
                    },
                },
            },
        };
        const answer = await this.query(message);
        return Buffer.from(
            answer.verify_tx_result_proof!.decrypted_data,
            'base64'
        );
    }

    public async maxInterval(): Promise<number> {
        const answer = await this.query({
            max_interval: {},
        });
        return answer.max_interval!.max_interval;
    }

    public async currentHighestHeaderHash(): Promise<string> {
        const answer = await this.query({
            current_highest_header_hash: {},
        });
        return Buffer.from(
            answer.current_highest_header_hash!.hash,
            'base64'
        ).toString('hex');
    }

    public async hashListLength(): Promise<number> {
        const answer = await this.query({
            hash_list_length: {},
        });
        return answer.hash_list_length!.length;
    }

    public async hashByIndex(index: number): Promise<string> {
        const answer = await this.query({
            hash_by_index: { index },
        });
        return Buffer.from(answer.hash_by_index!.hash, 'base64').toString(
            'hex'
        );
    }
}

export { SFPSClient, Account };
