use super::{ContractReference, BLOCK_SIZE};
use cosmwasm_std::{from_binary, Binary, Querier, StdError, StdResult};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, Query};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
pub use sfps_lib;
pub use sfps_lib::light_block::header::Header;
pub use sfps_lib::light_block::LightBlock;
pub use sfps_lib::tx_result_proof::TxResultProof;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InitMsg {
    pub max_interval: u64,
    pub initial_header: Header,
    pub entropy: Binary,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    MaxInterval { max_interval: u64 },
    CurrentHighestHeaderHash { hash: Binary },
    HashListLength { length: u64 },
    HashByIndex { hash: Binary },
    VerifyTxResultProof { decrypted_data: Binary },
}

#[derive(Serialize, Deserialize, Clone, JsonSchema, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    MaxInterval {},
    CurrentHighestHeaderHash {},
    HashListLength {},
    HashByIndex {
        index: u64,
    },
    VerifyTxResultProof {
        tx_result_proof: TxResultProof,
        header_hash_index: u64,
        encryption_key: Binary,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

pub fn verify_tx_result_proof<Q: Querier, H: DeserializeOwned>(
    querier: &Q,
    sfps_reference: ContractReference,
    tx_result_proof: TxResultProof,
    header_hash_index: u64,
    encryption_key: Binary,
) -> StdResult<H> {
    let answer: QueryAnswer = (QueryMsg::VerifyTxResultProof {
        tx_result_proof,
        header_hash_index,
        encryption_key,
    })
    .query(querier, sfps_reference.hash, sfps_reference.address)?;
    match answer {
        QueryAnswer::VerifyTxResultProof { decrypted_data } => from_binary(&decrypted_data),
        _ => Err(StdError::generic_err("unexpected answer")),
    }
}

#[derive(Serialize, Deserialize, Clone, JsonSchema, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
// Workaround for exports schemas for duplicated name types on different modules.
#[schemars(rename = "SFPSHandleMsg")]
pub enum HandleMsg {
    AddLightBlock {
        current_highest_header: Header,
        light_block: LightBlock,
    },
    AddEntropy {
        entropy: Binary,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}
