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
    QueryAnswer,
    FluffyContractReference,
} from './types';
import { ContractClient } from '../ContractClient';
import BigNumber from 'bignumber.js';
import { UnitConverter } from '../../UnitConverter';
import { Operation } from '../treasury/types';

class FinanceAdminClient extends ContractClient<
    HandleMsgForCustomHandleMsg,
    QueryMsgForCustomQueryMsg,
    QueryAnswer
> {
    public async migrate(
        newFinanceAdmin: FluffyContractReference
    ): Promise<void> {
        await this.execute(
            {
                migrate: {
                    new_finance_admin: newFinanceAdmin,
                },
            },
            1000000,
            () => void 0
        );
    }

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
        const result = await this.query({
            mint_reward: {
                minter,
                sbtc_mint_amount:
                    unitConverter.unitToContractValue(sbtcMintAmount),
                sbtc_total_supply:
                    unitConverter.unitToContractValue(sbtcTotalSupply),
            },
        });
        return result.mint_reward!;
    }

    public async releaseFee(
        releaser: string,
        sbtcReleaseAmount: BigNumber,
        sbtcTotalSupply: BigNumber,
        unitConverter: UnitConverter
    ): Promise<Operation[]> {
        const result = await this.query({
            release_fee: {
                releaser,
                sbtc_release_amount:
                    unitConverter.unitToContractValue(sbtcReleaseAmount),
                sbtc_total_supply:
                    unitConverter.unitToContractValue(sbtcTotalSupply),
            },
        });
        return result.release_fee!;
    }
  
    public async latestBitcoinSPVReward(
        unitConverter: UnitConverter
    ): Promise<BigNumber> {
        const result = await this.query({
            latest_bitcoin_s_p_v_reward: {},
        });
        return unitConverter.contractValueToUnit(
            result.latest_bitcoin_s_p_v_reward!
        );
    }

    public async latestSFPSReward(
        unitConverter: UnitConverter
    ): Promise<BigNumber> {
        const result = await this.query({
            latest_s_f_p_s_reward: {},
        });
        return unitConverter.contractValueToUnit(result.latest_s_f_p_s_reward!);
    }
}

export { FinanceAdminClient };
