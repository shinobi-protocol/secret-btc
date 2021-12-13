import { SigningCosmWasmClient } from 'secretjs';
import { Logger } from 'winston';
import {
    ContractClient,
    ExecuteResult as GenericExecuteResult,
} from '../ContractClient';
import { HandleMsg, QueryMsg, QueryAnswer, Event } from './types';

type ExecuteResult<ANSWER> = GenericExecuteResult<HandleMsg, ANSWER>;

class LogClient extends ContractClient<HandleMsg, QueryMsg, QueryAnswer> {
    public viewingKey?: string;

    constructor(
        contractAddress: string,
        signingCosmWasmClient: SigningCosmWasmClient,
        logger: Logger,
        viewingKey?: string
    ) {
        super(contractAddress, signingCosmWasmClient, logger);
        this.viewingKey = viewingKey;
    }

    // handle
    public async addReleaseRequestConfirmedEvent(
        blockHeight: number,
        requestKey: string,
        time: number,
        txId: string
    ): Promise<ExecuteResult<void>> {
        const hexToBytes = (hex: string): number[] => {
            const bytes = [];
            for (let c = 0; c < hex.length; c += 2)
                bytes.push(parseInt(hex.substr(c, 2), 16));
            return bytes;
        };
        return await this.execute(
            {
                add_events: {
                    events: [
                        [
                            this.senderAddress(),
                            {
                                release_request_confirmed: {
                                    time: time,
                                    request_key: hexToBytes(requestKey),
                                    block_height: blockHeight,
                                    txid: txId,
                                },
                            },
                        ],
                    ],
                },
            },
            50000,
            () => void 0
        );
    }

    public async setViewingKey(key: string): Promise<ExecuteResult<void>> {
        const result = await this.execute(
            { set_viewing_key: { key: key } },
            110000,
            () => void 0
        );
        this.viewingKey = key;
        return result;
    }

    public async queryLog(
        page: number,
        page_size: number,
        viewingKey = this.viewingKey
    ): Promise<Event[]> {
        if (viewingKey === undefined) {
            throw new Error('no viewing key');
        }
        const answer = await this.query({
            log: {
                address: this.senderAddress(),
                key: viewingKey,
                page,
                page_size,
            },
        });
        return answer.log!.logs;
    }
}

export { LogClient };
