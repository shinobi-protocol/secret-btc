use crate::config::Config;
use cosmwasm_std::{HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct InitMsg {
    pub config: Config,
    pub bitcoin_spv_base_reward: Uint128,
    pub sfps_base_reward: Uint128,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CustomHandleMsg {
    TransferOwnership { owner: HumanAddr },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CustomQueryMsg {
    Config {},
    TotalMintedSbtc {},
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CustomQueryAnswer {
    Config(Config),
    TotalMintedSbtc(Uint128),
}
