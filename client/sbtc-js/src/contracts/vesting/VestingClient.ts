/* eslint-disable @typescript-eslint/no-non-null-assertion */
import BigNumber from 'bignumber.js';
import {
    ContractClient,
    ExecuteResult as GenericExecuteResult,
} from '../ContractClient';
import { TokenClient } from '../token/TokenClient';
import {
    HandleMsg,
    QueryMsg,
    QueryAnswer,
    LockMsg,
    AccountInfoElement,
    VestingSummary,
} from './types';
import { HandleMsg as TokenHandleMsg } from '../token/types';
import { getUnixTime } from 'date-fns';

export default class VestingClient extends ContractClient<
    HandleMsg,
    QueryMsg,
    QueryAnswer
> {
    public async lock(
        tokenClient: TokenClient,
        amount: BigNumber,
        endTime: Date,
        recipient: string
    ): Promise<GenericExecuteResult<TokenHandleMsg, void>> {
        const lockMsg: LockMsg = {
            contract_hash: await tokenClient.codeHash,
            end_time: getUnixTime(endTime),
            recipient: recipient,
        };
        return tokenClient.send(
            amount,
            this.contractAddress,
            undefined,
            Buffer.from(JSON.stringify(lockMsg), 'utf-8'),
            await this.codeHash,
            100000
        );
    }

    public async claim(
        id: number
    ): Promise<GenericExecuteResult<HandleMsg, void>> {
        return this.execute(
            {
                claim: {
                    id,
                },
            },
            100000,
            () => void 0
        );
    }

    public async latestID(): Promise<number> {
        return this.query({ latest_i_d: {} }, (answer) => answer.latest_i_d!);
    }

    public async vestingInfos(ids: number[]): Promise<AccountInfoElement[]> {
        return this.query(
            { vesting_infos: { ids } },
            (answer) => answer.vesting_infos!
        );
    }

    public async vestingSummary(tokenAddress: string): Promise<VestingSummary> {
        return this.query(
            { vesting_summary: { token: tokenAddress } },
            (answer) => answer.vesting_summary!
        );
    }
}
