use crate::{CanonicalContractReference, Canonicalize, ContractReference, BLOCK_SIZE};
use cosmwasm_std::{Api, CanonicalAddr, HumanAddr, StdResult, Uint128};
use schemars::JsonSchema;
use secret_toolkit::utils::calls::{HandleCallback, Query};
use serde::{Deserialize, Serialize};

// Token viewing key of Treasury contract is constant and public to make public token balance
pub const TREASURY_VIEWING_KEY: &str = "TREASURY_VIEWING_KEY";

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct Config {
    pub owner: HumanAddr,
    pub snb: ContractReference,
    pub log: ContractReference,
}

#[derive(Serialize, Deserialize)]
pub struct CanonicalConfig {
    pub owner: CanonicalAddr,
    pub snb: CanonicalContractReference,
    pub log: CanonicalContractReference,
}

impl Canonicalize for Config {
    type Canonicalized = CanonicalConfig;
    fn into_canonical<A: Api>(self, api: &A) -> StdResult<Self::Canonicalized> {
        Ok(Self::Canonicalized {
            owner: self.owner.into_canonical(api)?,
            snb: self.snb.into_canonical(api)?,
            log: self.log.into_canonical(api)?,
        })
    }
    fn from_canonical<A: Api>(canonical: Self::Canonicalized, api: &A) -> StdResult<Self> {
        Ok(Self {
            owner: HumanAddr::from_canonical(canonical.owner, api)?,
            snb: ContractReference::from_canonical(canonical.snb, api)?,
            log: ContractReference::from_canonical(canonical.log, api)?,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct InitMsg {
    pub config: Config,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Send { to: HumanAddr, amount: Uint128 },
    ReceiveFrom { from: HumanAddr, amount: Uint128 },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Operate { operations: Vec<Operation> },
    TransferOwnership { owner: HumanAddr },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config(Config),
}
