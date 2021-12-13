import { Block } from 'bitcoinjs-lib';
import {
    ContractClient,
    ExecuteResult as GenericExecuteResult,
} from '../ContractClient';
import { CurrentHighestHeaderObject, LightBlock } from '../sfps/types';
import { QueryAnswerConfig, HandleMsg, QueryMsg, QueryAnswer } from './types';

type ExecuteResult<ANSWER> = GenericExecuteResult<HandleMsg, ANSWER>;

class ShurikenClient extends ContractClient<HandleMsg, QueryMsg, QueryAnswer> {
    // handle
    public async proxyBitcoinSPVAddHeaders(
        tip_height: number,
        blocks: Block[],
        gasLimit?: number
    ): Promise<ExecuteResult<void>> {
        if (!gasLimit) {
            const baseFee = 800000;
            const feePerBlock = 25000;
            gasLimit = baseFee + feePerBlock * blocks.length;
        }
        return await this.execute(
            {
                bitcoin_s_p_v_proxy: {
                    msg: {
                        add_headers: {
                            tip_height,
                            headers: blocks.map((header) =>
                                header.toBuffer(true).toString('base64')
                            ),
                        },
                    },
                },
            },
            gasLimit,
            () => void 0
        );
    }

    public async proxySFPSAddLightBlock(
        current_highest_header: CurrentHighestHeaderObject,
        light_block: LightBlock,
        gasLimit?: number
    ): Promise<ExecuteResult<void>> {
        return await this.execute(
            {
                s_f_p_s_proxy: {
                    msg: {
                        add_light_block: {
                            current_highest_header,
                            light_block,
                        },
                    },
                },
            },
            gasLimit || 300000,
            () => void 0
        );
    }

    public async config(): Promise<QueryAnswerConfig> {
        const result = await this.query({
            config: {},
        });
        return result.config;
    }
}

export { ShurikenClient };
