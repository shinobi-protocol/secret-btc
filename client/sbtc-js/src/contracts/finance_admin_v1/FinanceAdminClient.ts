/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-var-requires */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-non-null-assertion */
import {
    CustomQueryAnswer,
    CustomQueryAnswerConfig,
    HandleMsgForCustomHandleMsg,
    QueryMsgForCustomQueryMsg,
} from './types';
import { ContractClient } from '../ContractClient';
import BigNumber from 'bignumber.js';
import { UnitConverter } from '../../UnitConverter';
import { Operation } from '../treasury/types';

type QueryAnswer = Operation[] | CustomQueryAnswer;

class FinanceAdminClient extends ContractClient<
    HandleMsgForCustomHandleMsg,
    QueryMsgForCustomQueryMsg,
    QueryAnswer
> {
    public async config(): Promise<CustomQueryAnswerConfig> {
        const result = (await this.query({
            custom: { custom_msg: { config: {} } },
        })) as CustomQueryAnswer;
        return result.config!;
    }

    public async totalMintedSbtc(
        unitConverter: UnitConverter
    ): Promise<BigNumber> {
        const result = (await this.query({
            custom: { custom_msg: { total_minted_sbtc: {} } },
        })) as CustomQueryAnswer;
        return unitConverter.contractValueToUnit(result.total_minted_sbtc!);
    }

    public async mintReward(
        minter: string,
        sbtcMintAmount: BigNumber,
        sbtcTotalSupply: BigNumber,
        unitConverter: UnitConverter
    ): Promise<Operation[]> {
        const result = (await this.query({
            mint_reward: {
                minter,
                sbtc_mint_amount:
                    unitConverter.unitToContractValue(sbtcMintAmount),
                sbtc_total_supply:
                    unitConverter.unitToContractValue(sbtcTotalSupply),
            },
        })) as Operation[];
        return result;
    }

    public async releaseFee(
        releaser: string,
        sbtcReleaseAmount: BigNumber,
        sbtcTotalSupply: BigNumber,
        unitConverter: UnitConverter
    ): Promise<Operation[]> {
        const result = (await this.query({
            release_fee: {
                releaser,
                sbtc_release_amount:
                    unitConverter.unitToContractValue(sbtcReleaseAmount),
                sbtc_total_supply:
                    unitConverter.unitToContractValue(sbtcTotalSupply),
            },
        })) as Operation[];
        return result;
    }
}

export { FinanceAdminClient };
