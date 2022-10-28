/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-non-null-assertion */
import { Account } from 'secretjs';
import {
    MsgInstantiateContract,
    MsgExecuteContract,
} from 'secretjs/dist/protobuf_stuff/secret/compute/v1beta1/msg';
import {
    ContractClient,
    ExecuteResult as GenericExecuteResult,
} from '../ContractClient';
import { CommittedHashes } from '../shuriken/types';
import { MerkleProof } from './TendermintMerkleTree';
import {
    SFPSHandleMsg,
    QueryMsg,
    QueryAnswer,
    QueryMsgVerifyResponseDeliverTxProof,
} from './types';
import { ResponseDeliverTx } from 'secretjs/dist/protobuf_stuff/tendermint/abci/types';
import { TxResponse } from 'secretjs/dist/protobuf_stuff/cosmos/base/abci/v1beta1/abci';
import { Logger } from 'winston';
import { Tx } from 'secretjs/dist/protobuf_stuff/cosmos/tx/v1beta1/tx';
import {
    Header,
    LightBlock,
} from 'secretjs/dist/protobuf_stuff/tendermint/types/types';
import ShinobiClient from '../../ShinobiClient';
import { encodeToBase64 } from '../../proto';
import { GetTxsEventResponse } from 'secretjs/dist/protobuf_stuff/cosmos/tx/v1beta1/service';

export class ResponseDeliverTxProof {
    constructor(
        private encryptionKey: Buffer,
        private merkleProof: MerkleProof,
        private headerHashIndex: number,
        private headers: Header[]
    ) {}
    public encodeToMsg(): QueryMsgVerifyResponseDeliverTxProof {
        return {
            merkle_proof: {
                total: this.merkleProof.total,
                index: this.merkleProof.index,
                leaf: this.merkleProof.leaf.toString('hex'),
                aunts: this.merkleProof.aunts.map((aunt) =>
                    aunt.toString('hex')
                ),
            },
            headers: this.headers.map((header) =>
                encodeToBase64(Header, header)
            ),
            block_hash_index: this.headerHashIndex,
            encryption_key: this.encryptionKey.toString('base64'),
        };
    }
}

type ExecuteResult<ANSWER> = GenericExecuteResult<SFPSHandleMsg, ANSWER>;
class SFPSClient extends ContractClient<SFPSHandleMsg, QueryMsg, QueryAnswer> {
    constructor(
        contractAddress: string,
        shinobiClient: ShinobiClient,
        logger: Logger
    ) {
        super(contractAddress, shinobiClient, logger);
    }

    public async verifySubsequentLightBlocks(
        anchorHeader: Header,
        anchorHeaderIndex: number,
        followingLightBlocks: LightBlock[]
    ): Promise<CommittedHashes> {
        return await this.query(
            {
                verify_subsequent_light_blocks: {
                    anchor_header: encodeToBase64(
                        Header,
                        anchorHeader,
                    ),
                    anchor_header_index: anchorHeaderIndex,
                    following_light_blocks: followingLightBlocks.map((lb) =>
                        encodeToBase64(LightBlock, lb)
                    ),
                    commit_flags: Array(followingLightBlocks.length).fill(true) as boolean[],
                },
            },
            (answer) => {
                return answer.verify_subsequent_light_blocks!.committed_hashes;
            }
        );
    }

    // handle
    public async appendSubsequentHashes(
        committedHashes: CommittedHashes,
        gasLimit?: number
    ): Promise<ExecuteResult<void>> {
        return await this.execute(
            {
                append_subsequent_hashes: {
                    committed_hashes: committedHashes,
                },
            },
            gasLimit || 2800000,
            () => void 0
        );
    }

    public async verityTxResultProof(txHash: string): Promise<Buffer> {
        this.logger.info('verifyTxResultProof: ' + txHash);
        const proof = await this.getResponseDeliverTxProof(txHash);

        const message: QueryMsg = {
            verify_response_deliver_tx_proof: proof.encodeToMsg(),
        };
        return await this.query(message, (answer) =>
            Buffer.from(
                answer.verify_response_deliver_tx_proof!.decrypted_data,
                'base64'
            )
        );
    }

    public async getResponseDeliverTxProof(
        txHash: string
    ): Promise<ResponseDeliverTxProof> {
        return new ResponseDeliverTxProofBuilder(this).query(txHash);
    }

    public async maxInterval(): Promise<number> {
        return await this.query(
            {
                max_interval: {},
            },
            (answer) => answer.max_interval!.max_interval
        );
    }

    public async currentHighestHeaderHash(): Promise<string> {
        this.secretNetworkClient.query.txsQuery;
        return await this.query(
            {
                current_highest_header_hash: {},
            },
            (answer) =>
                Buffer.from(
                    answer.current_highest_header_hash!.hash,
                    'base64'
                ).toString('hex')
        );
    }

    public async currentHighestHeaderHeight(): Promise<number> {
        return await this.query(
            {
                current_highest_header_hash: {},
            },
            (answer) => answer.current_highest_header_hash!.height
        );
    }

    public async hashListLength(): Promise<number> {
        return await this.query(
            {
                hash_list_length: {},
            },
            (answer) => answer.hash_list_length!.length
        );
    }

    public async hashByIndex(index: number): Promise<string> {
        return await this.query(
            {
                hash_by_index: { index },
            },
            (answer) =>
                Buffer.from(answer.hash_by_index!.hash, 'base64').toString(
                    'hex'
                )
        );
    }

    public async heightByIndex(index: number): Promise<number> {
        return await this.query(
            {
                hash_by_index: { index },
            },
            (answer) => answer.hash_by_index!.height
        );
    }
}

// Query response deliver txs in a block, which are elements of last_result_hash of block header.
class ResponseDeliverTxProofBuilder {
    constructor(private sfpsClient: SFPSClient) {}

    public async query(txHash: string): Promise<ResponseDeliverTxProof> {
        const { tx, txResponse } =
            await this.sfpsClient.shinobiClient.txService.getTx({
                hash: txHash,
            });
        if (!tx || !txResponse?.height) {
            throw Error('incomplete tx returned');
        }
        const height = parseInt(txResponse.height);

        const txResponses = await this.queryTxResponses(height);
        const index = txResponses.findIndex(
            (txResponse) => txResponse.txhash == txHash
        );
        const responseDeliverTxs = txResponses.map(
            parseTxResponseToResponseDeliverTx
        );
        const { headers, headerHashIndex } = await this.queryHeaders(height);
        const encryptionKey = await this.encryptionKey(tx);
        return new ResponseDeliverTxProof(
            encryptionKey,
            MerkleProof.fromResponseDeliverTxs(responseDeliverTxs, index),
            headerHashIndex,
            headers
        );
    }

    private async queryTxResponses(txHeight: number): Promise<TxResponse[]> {
        let total = 0;
        let key = undefined;
        const txResponses = [];
        do {
            const result: GetTxsEventResponse =
                await this.sfpsClient.shinobiClient.txService.getTxsEvent({
                    events: [`tx.height=${txHeight}`],
                    pagination: { key, countTotal: true },
                });
            if (!result.pagination) {
                throw Error('no pagination response');
            }
            total = parseInt(result.pagination.total);
            key = result.pagination.nextKey;
            txResponses.push(...result.txResponses);
        } while (txResponses.length < total);
        return txResponses;
    }


    private async queryHeaders(
        txHeight: number
    ): Promise<{ headers: Header[]; headerHashIndex: number }> {
        const from = txHeight + 1;
        let to = undefined;
        let headerHashIndex = undefined;
        for (
            let index = (await this.sfpsClient.hashListLength()) - 1;
            index >= 0;
            index--
        ) {
            const height = await this.sfpsClient.heightByIndex(index);
            if (height < from) {
                break;
            }
            to = height;
            headerHashIndex = index;
        }
        if (!to || !headerHashIndex) {
            throw Error('Header is not synced yed');
        }
        const headers = [];
        for (let height = from; height <= to; height++) {
            const res =
                await this.sfpsClient.secretNetworkClient.query.tendermint.getBlockByHeight(
                    {
                        height: height.toString(),
                    }
                );
            if (!res.block?.header) {
                throw Error('No header returned');
            }
            headers.push(res.block.header);
        }
        return { headers, headerHashIndex };
    }

    private async encryptionKey(tx: Tx): Promise<Buffer> {
        const nonce = NonceExtractor.extractNoncesOfTx(tx)[0];
        if (!nonce) {
            throw Error('No nonce at tx message[0]');
        }
        return Buffer.from(
            await this.sfpsClient.shinobiClient.encryptionUtils.getTxEncryptionKey(
                nonce
            )
        );
    }
}
    const parseTxResponseToResponseDeliverTx = (
        txResponse: TxResponse
    ) => {
        return {
            code: txResponse.code,
            data: Buffer.from(txResponse.data, 'hex'),
            log: '',
            info: '',
            gasWanted: txResponse.gasWanted,
            gasUsed: txResponse.gasUsed,
            events: [],
            codespace: '',
        };
    }

interface Message {
    typeUrl: string;
    value: any;
}

class NonceExtractor {
    public static extractNoncesOfTx(tx: Tx): Array<Buffer | undefined> {
        if (!tx.body) {
            throw Error('No tx body');
        }
        const messages = tx.body.messages as Message[];
        const nonces = [];
        for (const message of messages) {
            nonces.push(this.extractNonceOfMessage(message));
        }
        return nonces;
    }

    private static extractNonceOfMessage(message: Message): Buffer | undefined {
        const decodedMessageValueMsg = this.decodeMessageValueMsg(message);
        return decodedMessageValueMsg?.slice(0, 32);
    }

    private static decodeMessageValueMsg(message: Message): Buffer | undefined {
        switch (message.typeUrl) {
            case '/secret.compute.v1beta1.MsgInstantiateContract':
                return Buffer.from(
                    MsgInstantiateContract.decode(message.value).initMsg
                );
            case '/secret.compute.v1beta1.MsgExecuteContract':
                return Buffer.from(
                    MsgExecuteContract.decode(message.value).msg
                );
            default:
                return undefined;
        }
    }
}

export { SFPSClient, Account };
