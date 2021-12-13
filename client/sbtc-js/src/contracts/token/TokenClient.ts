/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-non-null-assertion */
import BigNumber from 'bignumber.js';
import { Account, SigningCosmWasmClient } from 'secretjs';
import {
    ContractClient,
    ExecuteResult as GenericExecuteResult,
} from '../ContractClient';
import { HandleMsg, QueryMsg, QueryAnswer, Convert, TokenInfo } from './types';
import { Logger } from 'winston';
import { UnitConverter } from '../../UnitConverter';

type ExecuteResult<ANSWER> = GenericExecuteResult<HandleMsg, ANSWER>;

// max value of contract value is u128::max
const MAX_CONTRACT_VALUE = new BigNumber(2).pow(128).minus(1).toString();

class TokenClient extends ContractClient<HandleMsg, QueryMsg, QueryAnswer> {
    public viewingKey?: string;

    private unitConverterCache?: UnitConverter;

    constructor(
        contractAddress: string,
        signingCosmWasmClient: SigningCosmWasmClient,
        logger: Logger,
        viewingKey?: string
    ) {
        super(contractAddress, signingCosmWasmClient, logger);
        this.viewingKey = viewingKey;
    }

    public async unitConverter(): Promise<UnitConverter> {
        if (this.unitConverterCache == undefined) {
            const tokenInfo = await this.tokenInfo();
            this.unitConverterCache = new UnitConverter(tokenInfo.decimals);
        }
        return this.unitConverterCache;
    }

    // returns new allowance
    public async increaseAllowance(
        spender: string,
        amount: BigNumber,
        gasLimit?: number
    ): Promise<ExecuteResult<BigNumber>> {
        return await this.execute(
            {
                increase_allowance: {
                    spender,
                    amount: (
                        await this.unitConverter()
                    ).unitToContractValue(amount),
                },
            },
            gasLimit || 120000,
            (answerJson) =>
                new BigNumber(
                    Convert.toHandleAnswer(
                        answerJson
                    ).increase_allowance!.allowance
                )
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

    public async tokenInfo(): Promise<TokenInfo> {
        const answer = await this.query({
            token_info: {},
        });
        return answer.token_info!;
    }

    public async getBalance(viewingKey = this.viewingKey): Promise<BigNumber> {
        if (viewingKey === undefined) {
            throw new Error('no viewing key');
        }
        const answer = await this.query({
            balance: {
                address: this.signingCosmWasmClient.senderAddress,
                key: viewingKey,
            },
        });
        return (await this.unitConverter()).contractValueToUnit(
            answer.balance!.amount
        );
    }

    public async getAllowance(
        spender: string,
        viewingKey = this.viewingKey
    ): Promise<BigNumber> {
        if (viewingKey === undefined) {
            throw new Error('no viewing key');
        }
        const answer = await this.query({
            allowance: {
                owner: this.signingCosmWasmClient.senderAddress,
                spender: spender,
                key: viewingKey,
            },
        });
        return (await this.unitConverter()).contractValueToUnit(
            answer.allowance!.allowance
        );
    }

    public async maxValue(): Promise<BigNumber> {
        return (await this.unitConverter()).contractValueToUnit(
            MAX_CONTRACT_VALUE
        );
    }
}

export { TokenClient, Account, TokenInfo };
