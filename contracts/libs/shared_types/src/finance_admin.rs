use super::BLOCK_SIZE;
use crate::treasury::Operation;
use crate::ContractReference;
use cosmwasm_std::{HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::utils::calls::{HandleCallback, Query};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

pub type CommonHandleMsg = HandleMsg<()>;

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg<C>
where
    C: Serialize + DeserializeOwned + JsonSchema + Clone + Sized + Debug + PartialEq + Eq,
{
    Migrate {
        new_finance_admin: ContractReference,
    },
    SendMintReward {
        minter: HumanAddr,
        sbtc_mint_amount: Uint128,
        sbtc_total_supply: Uint128,
    },
    ReceiveReleaseFee {
        releaser: HumanAddr,
        sbtc_release_amount: Uint128,
        sbtc_total_supply: Uint128,
    },
    MintBitcoinSPVReward {
        executer: HumanAddr,
        best_height: u32,
        best_block_time: u32,
    },
    MintSFPSReward {
        executer: HumanAddr,
        best_height: u64,
        best_block_time: u64,
    },
    Custom {
        #[serde(bound = "")]
        custom_msg: C,
    },
}

impl<C> HandleCallback for HandleMsg<C>
where
    C: Serialize + DeserializeOwned + JsonSchema + Clone + Sized + Debug + PartialEq + Eq,
{
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg<C>
where
    C: Serialize + DeserializeOwned + JsonSchema + Clone + Sized + Debug + PartialEq + Eq,
{
    MintReward {
        minter: HumanAddr,
        sbtc_mint_amount: Uint128,
        sbtc_total_supply: Uint128,
    },
    ReleaseFee {
        releaser: HumanAddr,
        sbtc_release_amount: Uint128,
        sbtc_total_supply: Uint128,
    },
    LatestBitcoinSPVReward {},
    LatestSFPSReward {},
    Custom {
        #[serde(bound = "")]
        custom_msg: C,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    MintReward(Vec<Operation>),
    ReleaseFee(Vec<Operation>),
    LatestBitcoinSPVReward(Uint128),
    LatestSFPSReward(Uint128),
}

impl<C> Query for QueryMsg<C>
where
    C: Serialize + DeserializeOwned + JsonSchema + Clone + Sized + Debug + PartialEq + Eq,
{
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_serde_handle_msg() {
        let msg1_json = r#"
            {
                "custom":  {
                    "custom_msg": {
                        "msg1": {
                            "a": 1
                        }
                    },
                    "p": "   "
                }
            }
        "#;

        #[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq, Eq)]
        #[serde(rename_all = "snake_case")]
        enum CustomMsg {
            Msg1 { a: u8 },
            Msg2 { b: u8 },
        }
        let handle_msg: HandleMsg<CustomMsg> =
            cosmwasm_std::from_slice(msg1_json.as_bytes()).unwrap();
        assert_eq!(
            handle_msg,
            HandleMsg::Custom {
                custom_msg: CustomMsg::Msg1 { a: 1 }
            }
        );
    }
}
