use cosmwasm_std::{Api, CanonicalAddr, HumanAddr, StdResult};
use serde::{Deserialize, Serialize};

#[cfg(feature = "bitcoin_spv")]
pub mod bitcoin_spv;
#[cfg(feature = "gateway")]
pub mod gateway;
#[cfg(feature = "log")]
pub mod log;
#[cfg(feature = "multisig")]
pub mod multisig;
#[cfg(feature = "prng")]
pub mod prng;
#[cfg(feature = "sfps")]
pub mod sfps;
#[cfg(feature = "shuriken")]
pub mod shuriken;
#[cfg(feature = "viewing_key")]
pub mod viewing_key;

pub const BLOCK_SIZE: usize = 256;
pub const E8: u128 = 100000000;

pub trait Canonicalize: Sized {
    type Canonicalized;
    fn into_canonical<A: Api>(self, api: &A) -> StdResult<Self::Canonicalized>;
    fn from_canonical<A: Api>(canonical: Self::Canonicalized, api: &A) -> StdResult<Self>;
}

impl Canonicalize for HumanAddr {
    type Canonicalized = CanonicalAddr;
    fn into_canonical<A: Api>(self, api: &A) -> StdResult<Self::Canonicalized> {
        api.canonical_address(&self)
    }
    fn from_canonical<A: Api>(canonical: Self::Canonicalized, api: &A) -> StdResult<Self> {
        api.human_address(&canonical)
    }
}

#[derive(Serialize, Deserialize, schemars::JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct ContractReference {
    pub address: HumanAddr,
    pub hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct CanonicalContractReference {
    pub address: CanonicalAddr,
    pub hash: String,
}

impl Canonicalize for ContractReference {
    type Canonicalized = CanonicalContractReference;

    fn into_canonical<A: Api>(self, api: &A) -> StdResult<Self::Canonicalized> {
        Ok(Self::Canonicalized {
            address: self.address.into_canonical(api)?,
            hash: self.hash,
        })
    }
    fn from_canonical<A: Api>(canonical: Self::Canonicalized, api: &A) -> StdResult<Self> {
        Ok(Self {
            address: HumanAddr::from_canonical(canonical.address, api)?,
            hash: canonical.hash,
        })
    }
}
