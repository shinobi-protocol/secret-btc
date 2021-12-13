use cosmwasm_std::{Api, CanonicalAddr, HumanAddr, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared_types::{CanonicalContractReference, Canonicalize, ContractReference};

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub owner: HumanAddr,
    pub gateway: ContractReference,
    pub treasury: ContractReference,
    pub shuriken: ContractReference,
    pub snb: ContractReference,
    pub developer_address: HumanAddr,
}

#[derive(Serialize, Deserialize)]
pub struct CanonicalConfig {
    pub owner: CanonicalAddr,
    pub gateway: CanonicalContractReference,
    pub treasury: CanonicalContractReference,
    pub shuriken: CanonicalContractReference,
    pub snb: CanonicalContractReference,
    pub developer_address: CanonicalAddr,
}

impl Canonicalize for Config {
    type Canonicalized = CanonicalConfig;
    fn into_canonical<A: Api>(self, api: &A) -> StdResult<Self::Canonicalized> {
        Ok(Self::Canonicalized {
            owner: self.owner.into_canonical(api)?,
            gateway: self.gateway.into_canonical(api)?,
            treasury: self.treasury.into_canonical(api)?,
            shuriken: self.shuriken.into_canonical(api)?,
            snb: self.snb.into_canonical(api)?,
            developer_address: self.developer_address.into_canonical(api)?,
        })
    }
    fn from_canonical<A: Api>(canonical: Self::Canonicalized, api: &A) -> StdResult<Self> {
        Ok(Self {
            owner: HumanAddr::from_canonical(canonical.owner, api)?,
            gateway: ContractReference::from_canonical(canonical.gateway, api)?,
            treasury: ContractReference::from_canonical(canonical.treasury, api)?,
            shuriken: ContractReference::from_canonical(canonical.shuriken, api)?,
            snb: ContractReference::from_canonical(canonical.snb, api)?,
            developer_address: HumanAddr::from_canonical(canonical.developer_address, api)?,
        })
    }
}
