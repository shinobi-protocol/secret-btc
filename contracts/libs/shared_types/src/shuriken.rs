use crate::{sfps, CanonicalContractReference, Canonicalize, ContractReference, BLOCK_SIZE};
use cosmwasm_std::{Api, Binary, StdResult};
use schemars::JsonSchema;
use secret_toolkit::utils::calls::{HandleCallback, Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Config {
    pub finance_admin: ContractReference,
    pub bitcoin_spv: ContractReference,
    pub sfps: ContractReference,
}

#[derive(Serialize, Deserialize)]
pub struct CanonicalConfig {
    pub finance_admin: CanonicalContractReference,
    pub bitcoin_spv: CanonicalContractReference,
    pub sfps: CanonicalContractReference,
}

impl Canonicalize for Config {
    type Canonicalized = CanonicalConfig;
    fn into_canonical<A: Api>(self, api: &A) -> StdResult<Self::Canonicalized> {
        Ok(Self::Canonicalized {
            finance_admin: self.finance_admin.into_canonical(api)?,
            bitcoin_spv: self.bitcoin_spv.into_canonical(api)?,
            sfps: self.sfps.into_canonical(api)?,
        })
    }
    fn from_canonical<A: Api>(canonical: Self::Canonicalized, api: &A) -> StdResult<Self> {
        Ok(Self {
            finance_admin: ContractReference::from_canonical(canonical.finance_admin, api)?,
            bitcoin_spv: ContractReference::from_canonical(canonical.bitcoin_spv, api)?,
            sfps: ContractReference::from_canonical(canonical.sfps, api)?,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct InitMsg {
    #[schemars(inline)]
    pub config: Config,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    ChangeFinanceAdmin {
        new_finance_admin: ContractReference,
    },
    BitcoinSPVAddHeaders {
        tip_height: u32,
        headers: Vec<Binary>,
    },
    SFPSProxyAppendSubsequentHashes {
        committed_hashes: sfps::CommittedHashes,
        last_header: sfps::Header,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, Clone, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config(Config),
}
