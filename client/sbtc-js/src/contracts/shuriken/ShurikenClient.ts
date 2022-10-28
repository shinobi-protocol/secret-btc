import { Block } from 'bitcoinjs-lib';
import { Header } from 'secretjs/dist/protobuf_stuff/tendermint/types/types';
import { encodeToBase64 } from '../../proto';
import {
    ContractClient,
    ExecuteResult as GenericExecuteResult,
} from '../ContractClient';
import {
    QueryAnswerConfig,
    HandleMsg,
    QueryMsg,
    QueryAnswer,
    CommittedHashes,
} from './types';

type ExecuteResult<ANSWER> = GenericExecuteResult<HandleMsg, ANSWER>;

class ShurikenClient extends ContractClient<HandleMsg, QueryMsg, QueryAnswer> {
    // handle
    public async proxyBitcoinSPVAddHeaders(
        tip_height: number,
        blocks: Block[],
        gasLimit?: number
    ): Promise<ExecuteResult<void>> {
        if (!gasLimit) {
            const baseFee = 2000000;
            const feePerBlock = 50000;
            gasLimit = baseFee + feePerBlock * blocks.length;
        }
        return await this.execute(
            {
                bitcoin_s_p_v_add_headers: {
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

    public async proxySFPSAppendSubsequentHashes(
        committedHashes: CommittedHashes,
        gasLimit?: number
    ): Promise<ExecuteResult<void>> {
        return await this.execute(
            {
                s_f_p_s_proxy_append_subsequent_hashes: {
                    committed_hashes: committedHashes,
                },
            },
            gasLimit ||
                3000000 + 80000 * committedHashes.hashes.following_hashes.length,
            () => void 0
        );
    }

    public async config(): Promise<QueryAnswerConfig> {
        return await this.query(
            {
                config: {},
            },
            (answer) => answer.config
        );
    }
}

export { ShurikenClient };
