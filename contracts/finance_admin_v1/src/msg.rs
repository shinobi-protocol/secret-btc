use crate::config::Config;
use cosmwasm_std::{HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct InitMsg {
    pub config: Config,
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
