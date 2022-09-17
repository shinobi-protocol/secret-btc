use super::{ContractReference, BLOCK_SIZE};
use cosmwasm_std::{from_binary, Binary, Querier, StdError, StdResult};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, Query};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
pub use sfps_lib;
pub use sfps_lib::cosmos_proto::tendermint::abci::ResponseDeliverTx;
pub use sfps_lib::cosmos_proto::tendermint::types::{Header, LightBlock};
pub use sfps_lib::merkle::MerkleProof;
pub use sfps_lib::response_deliver_tx_proof::ResponseDeliverTxProof;
pub use sfps_lib::subsequent_hashes::{CommittedHashes, HeaderHashWithHeight};

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct InitMsg {
    pub max_interval: u64,
    #[schemars(with = "String")]
    #[serde(with = "serde_proto_message")]
    pub initial_header: Header,
    pub entropy: Binary,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    MaxInterval { max_interval: u64 },
    CurrentHighestHeaderHash { hash: Binary, height: i64 },
    HashListLength { length: u64 },
    HashByIndex { hash: Binary, height: i64 },
    VerifyResponseDeliverTxProof { decrypted_data: Binary },
    VerifySubsequentLightBlocks { committed_hashes: CommittedHashes },
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
    VerifyResponseDeliverTxProof {
        merkle_proof: MerkleProof,
        #[schemars(with = "Vec<String>")]
        #[serde(with = "serde_proto_message_array")]
        headers: Vec<Header>,

        block_hash_index: u64,
        encryption_key: Binary,
    },
    VerifySubsequentLightBlocks {
        #[schemars(with = "String")]
        #[serde(with = "serde_proto_message")]
        current_highest_header: Header,
        #[schemars(with = "Vec<String>")]
        #[serde(with = "serde_proto_message_array")]
        light_blocks: Vec<LightBlock>,
        commit_flags: Vec<bool>,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

pub fn verify_response_deliver_tx_proof<Q: Querier, H: DeserializeOwned>(
    querier: &Q,
    sfps_reference: ContractReference,
    merkle_proof: MerkleProof,
    headers: Vec<Header>,
    block_hash_index: u64,
    encryption_key: Binary,
) -> StdResult<H> {
    let answer: QueryAnswer = (QueryMsg::VerifyResponseDeliverTxProof {
        merkle_proof,
        headers,
        block_hash_index,
        encryption_key,
    })
    .query(querier, sfps_reference.hash, sfps_reference.address)?;
    match answer {
        QueryAnswer::VerifyResponseDeliverTxProof { decrypted_data } => {
            from_binary(&decrypted_data)
        }
        _ => Err(StdError::generic_err("unexpected answer")),
    }
}

#[derive(Serialize, Deserialize, Clone, JsonSchema, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
// Workaround for exports schemas for duplicated name types on different modules.
#[schemars(rename = "SFPSHandleMsg")]
pub enum HandleMsg {
    AppendSubsequentHashes { committed_hashes: CommittedHashes },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

pub mod serde_proto_message {
    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer, Serializer};
    use sfps_lib::cosmos_proto::prost::Message;

    pub fn deserialize<'de, D, M>(deserializer: D) -> Result<M, D::Error>
    where
        D: Deserializer<'de>,
        M: Message + Default,
    {
        let string = String::deserialize(deserializer)?;
        let bin = base64::decode(&string).map_err(D::Error::custom)?;
        M::decode(bin.as_slice()).map_err(D::Error::custom)
    }

    pub fn serialize<S, M>(value: &M, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        M: Message,
    {
        let bin = value.encode_to_vec();
        let base64_string = base64::encode(bin.as_slice());
        serializer.serialize_str(&base64_string)
    }
}

pub mod serde_proto_message_array {
    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use sfps_lib::cosmos_proto::prost::Message;

    pub fn deserialize<'de, D, M>(deserializer: D) -> Result<Vec<M>, D::Error>
    where
        D: Deserializer<'de>,
        M: Message + Default,
    {
        let strings: Vec<String> = Vec::deserialize(deserializer)?;
        strings
            .iter()
            .map(|string| {
                let bin = base64::decode(&string).map_err(D::Error::custom)?;
                M::decode(bin.as_slice()).map_err(D::Error::custom)
            })
            .collect()
    }

    pub fn serialize<S, M>(values: &[M], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        M: Message,
    {
        let strings: Vec<String> = values
            .iter()
            .map(|value| {
                let bin = value.encode_to_vec();
                base64::encode(bin.as_slice())
            })
            .collect();
        strings.serialize(serializer)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_serde_proto_message_array() {
        let json = r#"
        [
            "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"
        ]"#;
        let mut deserializer = serde_json::Deserializer::from_str(&json);
        let _: Vec<Header> = serde_proto_message_array::deserialize(&mut deserializer).unwrap();
    }
}
