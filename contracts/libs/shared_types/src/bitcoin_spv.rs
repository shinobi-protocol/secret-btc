use super::BLOCK_SIZE;
use crate::state_proxy::client::Seed;
use crate::CanonicalContractReference;
use crate::Canonicalize;
use crate::ContractReference;
use cosmwasm_std::Api;
use cosmwasm_std::Binary;
use cosmwasm_std::StdResult;
use schemars::JsonSchema;
use secret_toolkit::utils::calls::{HandleCallback, Query};
use serde::{Deserialize, Serialize};

/// Contract Config set at contrat init.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct Config {
    /// "bitcoin" | "testnet" | 'regtest"
    pub bitcoin_network: String,
    /// minimum block needed for tx confirmed
    pub confirmation: u8,

    pub state_proxy: ContractReference,
}

/// Contract Config set at contrat init.
#[derive(Serialize, Deserialize, Debug)]
pub struct CanonicalConfig {
    /// "bitcoin" | "testnet" | 'regtest"
    pub bitcoin_network: String,
    /// minimum block needed for tx confirmed
    pub confirmation: u8,
    pub state_proxy: CanonicalContractReference,
}

impl Canonicalize for Config {
    type Canonicalized = CanonicalConfig;
    fn into_canonical<A: Api>(self, api: &A) -> StdResult<Self::Canonicalized> {
        Ok(CanonicalConfig {
            bitcoin_network: self.bitcoin_network,
            confirmation: self.confirmation,
            state_proxy: self.state_proxy.into_canonical(api)?,
        })
    }
    fn from_canonical<A: Api>(canonical: Self::Canonicalized, api: &A) -> StdResult<Self> {
        Ok(Self {
            bitcoin_network: canonical.bitcoin_network,
            confirmation: canonical.confirmation,
            state_proxy: ContractReference::from_canonical(canonical.state_proxy, api)?,
        })
    }
}

// Bitcoin Merkle Proof Std Message
#[derive(JsonSchema, Serialize, Deserialize, Clone, Debug, Default)]
pub struct MerkleProofMsg {
    pub prefix: Vec<bool>,
    pub siblings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct InitialHeader {
    pub height: u32,
    pub header: Binary,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InitMsg {
    pub bitcoin_network: String,
    pub confirmation: u8,
    pub initial_header: Option<InitialHeader>,
    pub state_proxy: ContractReference,
    pub seed: Seed,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
// Workaround for exports schemas for duplicated name types on different modules.
#[schemars(rename = "BitcoinSPVHandleMsg")]
pub enum HandleMsg {
    AddHeaders {
        tip_height: u32,
        headers: Vec<Binary>,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, Clone, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    BlockHeader {
        height: u32,
    },
    BestHeaderHash {},
    VerifyMerkleProof {
        height: u32,
        tx: Binary,
        merkle_proof: MerkleProofMsg,
    },
    Config {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    BlockHeader { header: Binary },
    BestHeaderHash { hash: String },
    Config(Config),
    VerifyMerkleProof { success: bool },
}
