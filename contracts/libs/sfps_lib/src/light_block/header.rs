pub mod fields;

use crate::merkle::simple_hash_from_byte_vectors;
use fields::{BlockId, Timestamp, Version};
use prost::Message;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, schemars::JsonSchema, Message)]
pub struct Header {
    /// Header version
    #[prost(message, optional, tag = "1")]
    pub version: Option<Version>,

    /// Chain ID
    #[prost(string, tag = "2")]
    pub chain_id: String,

    /// Current block height
    #[prost(int64, tag = "3")]
    #[schemars(with = "String")]
    #[serde(with = "crate::serde::str")]
    pub height: i64,

    /// Current timestamp
    #[prost(message, optional, tag = "4")]
    #[schemars(with = "String")]
    #[serde(with = "crate::serde::default_to_none")]
    pub time: Option<Timestamp>,

    /// Previous block info
    #[prost(message, optional, tag = "5")]
    #[schemars(with = "BlockId")]
    #[serde(with = "crate::serde::default_to_none")]
    pub last_block_id: Option<BlockId>,

    /// Commit from validators from the last block
    #[prost(bytes, tag = "6")]
    #[schemars(with = "String")]
    #[serde(serialize_with = "hex::serde::serialize_upper")]
    #[serde(deserialize_with = "hex::serde::deserialize")]
    pub last_commit_hash: Vec<u8>,

    /// Merkle root of transaction hashes
    #[prost(bytes, tag = "7")]
    #[schemars(with = "String")]
    #[serde(serialize_with = "hex::serde::serialize_upper")]
    #[serde(deserialize_with = "hex::serde::deserialize")]
    pub data_hash: Vec<u8>,

    /// Validators for the current block
    #[prost(bytes, tag = "8")]
    #[schemars(with = "String")]
    #[serde(serialize_with = "hex::serde::serialize_upper")]
    #[serde(deserialize_with = "hex::serde::deserialize")]
    pub validators_hash: Vec<u8>,

    /// Validators for the next block
    #[prost(bytes, tag = "9")]
    #[schemars(with = "String")]
    #[serde(serialize_with = "hex::serde::serialize_upper")]
    #[serde(deserialize_with = "hex::serde::deserialize")]
    pub next_validators_hash: Vec<u8>,

    /// Consensus params for the current block
    #[prost(bytes, tag = "10")]
    #[schemars(with = "String")]
    #[serde(serialize_with = "hex::serde::serialize_upper")]
    #[serde(deserialize_with = "hex::serde::deserialize")]
    pub consensus_hash: Vec<u8>,

    /// State after txs from the previous block
    #[prost(bytes, tag = "11")]
    #[schemars(with = "String")]
    #[serde(serialize_with = "hex::serde::serialize_upper")]
    #[serde(deserialize_with = "hex::serde::deserialize")]
    pub app_hash: Vec<u8>,

    /// Root hash of all results from the txs from the previous block
    #[prost(bytes, tag = "12")]
    #[schemars(with = "String")]
    #[serde(serialize_with = "hex::serde::serialize_upper")]
    #[serde(deserialize_with = "hex::serde::deserialize")]
    pub last_results_hash: Vec<u8>,

    /// Hash of evidence included in the block
    #[prost(bytes, tag = "13")]
    #[schemars(with = "String")]
    #[serde(serialize_with = "hex::serde::serialize_upper")]
    #[serde(deserialize_with = "hex::serde::deserialize")]
    pub evidence_hash: Vec<u8>,

    /// Original proposer of the block
    #[prost(bytes, tag = "14")]
    #[schemars(with = "String")]
    #[serde(serialize_with = "hex::serde::serialize_upper")]
    #[serde(deserialize_with = "hex::serde::deserialize")]
    pub proposer_address: Vec<u8>,
}

const FIELDS_NUM: usize = 14;

impl Header {
    pub fn hash(&self) -> Vec<u8> {
        let bytes = self.hash_bytes();
        simple_hash_from_byte_vectors(bytes.to_vec())
    }

    fn hash_bytes(&self) -> [Vec<u8>; FIELDS_NUM] {
        [
            self.version.clone().unwrap().encode_to_vec(),
            self.chain_id.encode_to_vec(),
            self.height.encode_to_vec(),
            self.time.clone().unwrap().encode_to_vec(),
            self.last_block_id.clone().unwrap().encode_to_vec(),
            self.last_commit_hash.encode_to_vec(),
            self.data_hash.encode_to_vec(),
            self.validators_hash.encode_to_vec(),
            self.next_validators_hash.encode_to_vec(),
            self.consensus_hash.encode_to_vec(),
            self.app_hash.encode_to_vec(),
            self.last_results_hash.encode_to_vec(),
            self.evidence_hash.encode_to_vec(),
            self.proposer_address.encode_to_vec(),
        ]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::light_block::header::fields::PartSetHeader;
    #[test]
    fn test_deserialize_header() {
        let header: Header = serde_json::de::from_str(r#"
            {
                "version": {
                    "block": "11"
                },
                "chain_id": "secret-4",
                "height": "1000001",
                "time": "2021-11-23T16:07:37.575128904Z",
                "last_block_id": {
                    "hash": "EDFFB7C581AEDADA9A41E355BEED063E15BB1B2C281957AE864E0392B5DEDE2F",
                    "parts": {
                        "total": 1,
                        "hash": "01932DD4FA96010353DA4A60AC99EED3865C28D8AEADB67DFD9949A9A82D1940"
                    }
                },
                "last_commit_hash": "7FE08F2D155922661247762A68A16C0439E894336FE3E87C07ED3ED0FDECE418",
                "data_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
                "validators_hash": "9EFBBA1CEA6B4CAE8F27C7F16E830FBBFEED6AB8D35245DE263D63FE4F7211B0",
                "next_validators_hash": "9EFBBA1CEA6B4CAE8F27C7F16E830FBBFEED6AB8D35245DE263D63FE4F7211B0",
                "consensus_hash": "717BE5422EFEECF5C48B9B5EA4AFD9C4C2E002A676CA2E9512CAE7802DA37D92",
                "app_hash": "EE32DBF923F4B63F0CCCBB7CABE3BFE58AD33B6749A2D2DC7130E683ED41FE91",
                "last_results_hash": "D0C1BDF3B1811F72A1DA190266D06CE950B465DE1681436AF826619D7DC92A79",
                "evidence_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
                "proposer_address": "B89CDCF017D80A946FBFFC41A2583C03190E8613"
            }
        "#).unwrap();
        assert_eq!(header.version, Some(Version { block: 11, app: 0 }));
        assert_eq!(header.chain_id, "secret-4".to_string());
        assert_eq!(header.height, 1000001);
        assert_eq!(
            serde_json::to_string(&header.time).unwrap(),
            "\"2021-11-23T16:07:37.575128904Z\""
        );
        assert_eq!(
            header.last_block_id.unwrap(),
            BlockId {
                hash: hex::decode(
                    "EDFFB7C581AEDADA9A41E355BEED063E15BB1B2C281957AE864E0392B5DEDE2F"
                )
                .unwrap(),
                parts: Some(PartSetHeader {
                    total: 1,
                    hash: hex::decode(
                        "01932DD4FA96010353DA4A60AC99EED3865C28D8AEADB67DFD9949A9A82D1940"
                    )
                    .unwrap(),
                })
            }
        );
        assert_eq!(
            hex::encode_upper(&header.last_commit_hash),
            "7FE08F2D155922661247762A68A16C0439E894336FE3E87C07ED3ED0FDECE418"
        );
        assert_eq!(
            hex::encode_upper(&header.data_hash),
            "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"
        );
        assert_eq!(
            hex::encode_upper(&header.validators_hash),
            "9EFBBA1CEA6B4CAE8F27C7F16E830FBBFEED6AB8D35245DE263D63FE4F7211B0"
        );
        assert_eq!(
            hex::encode_upper(&header.next_validators_hash),
            "9EFBBA1CEA6B4CAE8F27C7F16E830FBBFEED6AB8D35245DE263D63FE4F7211B0"
        );
        assert_eq!(
            hex::encode_upper(&header.consensus_hash),
            "717BE5422EFEECF5C48B9B5EA4AFD9C4C2E002A676CA2E9512CAE7802DA37D92"
        );
        assert_eq!(
            hex::encode_upper(&header.app_hash),
            "EE32DBF923F4B63F0CCCBB7CABE3BFE58AD33B6749A2D2DC7130E683ED41FE91"
        );
        assert_eq!(
            hex::encode_upper(&header.last_results_hash),
            "D0C1BDF3B1811F72A1DA190266D06CE950B465DE1681436AF826619D7DC92A79"
        );
        assert_eq!(
            hex::encode_upper(&header.evidence_hash),
            "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"
        );
        assert_eq!(
            hex::encode_upper(&header.proposer_address),
            "B89CDCF017D80A946FBFFC41A2583C03190E8613"
        );
    }

    #[test]
    fn test_hash_header() {
        // genesis header
        let header: Header = serde_json::de::from_str(r#"
            {
                "version": {
                    "block": "11"
                },
                "chain_id": "secret-4",
                "height": "1000001",
                "time": "2021-11-23T16:07:37.575128904Z",
                "last_block_id": {
                    "hash": "EDFFB7C581AEDADA9A41E355BEED063E15BB1B2C281957AE864E0392B5DEDE2F",
                    "parts": {
                        "total": 1,
                        "hash": "01932DD4FA96010353DA4A60AC99EED3865C28D8AEADB67DFD9949A9A82D1940"
                    }
                },
                "last_commit_hash": "7FE08F2D155922661247762A68A16C0439E894336FE3E87C07ED3ED0FDECE418",
                "data_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
                "validators_hash": "9EFBBA1CEA6B4CAE8F27C7F16E830FBBFEED6AB8D35245DE263D63FE4F7211B0",
                "next_validators_hash": "9EFBBA1CEA6B4CAE8F27C7F16E830FBBFEED6AB8D35245DE263D63FE4F7211B0",
                "consensus_hash": "717BE5422EFEECF5C48B9B5EA4AFD9C4C2E002A676CA2E9512CAE7802DA37D92",
                "app_hash": "EE32DBF923F4B63F0CCCBB7CABE3BFE58AD33B6749A2D2DC7130E683ED41FE91",
                "last_results_hash": "D0C1BDF3B1811F72A1DA190266D06CE950B465DE1681436AF826619D7DC92A79",
                "evidence_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
                "proposer_address": "B89CDCF017D80A946FBFFC41A2583C03190E8613"
            }
        "#).unwrap();
        assert_eq!(
            hex::encode_upper(&header.hash()),
            "275AFF29FB91FCFC3E6581B3522502205F03758864CD1030D2A5E2212AA4FBE2",
        );
    }
}
