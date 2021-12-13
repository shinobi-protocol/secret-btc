use super::BLOCK_SIZE;
use cosmwasm_std::Binary;
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
