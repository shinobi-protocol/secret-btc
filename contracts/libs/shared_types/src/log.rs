pub mod event;
use crate::{viewing_key, CanonicalContractReference, Canonicalize, ContractReference, BLOCK_SIZE};
use cosmwasm_std::{Api, Binary, HumanAddr, StdResult};
pub use event::Event;
use secret_toolkit::utils::{HandleCallback, Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, schemars::JsonSchema, Debug, PartialEq)]
pub struct Config {
    pub gateway: ContractReference,
    pub treasury: ContractReference,
}

#[derive(Serialize, Deserialize)]
pub struct CanonicalConfig {
    pub gateway: CanonicalContractReference,
    pub treasury: CanonicalContractReference,
}

impl Canonicalize for Config {
    type Canonicalized = CanonicalConfig;
    fn into_canonical<A: Api>(self, api: &A) -> StdResult<Self::Canonicalized> {
        Ok(Self::Canonicalized {
            gateway: self.gateway.into_canonical(api)?,
            treasury: self.treasury.into_canonical(api)?,
        })
    }
    fn from_canonical<A: Api>(canonical: Self::Canonicalized, api: &A) -> StdResult<Self> {
        Ok(Self {
            gateway: ContractReference::from_canonical(canonical.gateway, api)?,
            treasury: ContractReference::from_canonical(canonical.treasury, api)?,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, schemars::JsonSchema, Debug)]
pub struct InitMsg {
    pub entropy: Binary,
}

#[derive(Serialize, Deserialize, Clone, schemars::JsonSchema, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Setup { config: Config },
    AddEvents { events: Vec<(HumanAddr, Event)> },
    CreateViewingKey { entropy: String },
    SetViewingKey { key: viewing_key::ViewingKey },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, schemars::JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    CreateViewingKey { key: viewing_key::ViewingKey },
}

#[derive(Serialize, Deserialize, Clone, schemars::JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Log {
        address: HumanAddr,
        key: viewing_key::ViewingKey,
        page: u32,
        page_size: u32,
    },
    Config {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, schemars::JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config(Config),
    Log { logs: Vec<Event> },
}
