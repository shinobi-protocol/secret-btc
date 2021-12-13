use crate::light_block::header::Header;
use crate::merkle::{Error as MerkleProofError, MerkleProof};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    EmptyHeaders,
    NoLastHeaderHash,
    UnconnectedHeaders,
    ProstError(prost::DecodeError),
    MerkleProof(MerkleProofError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::EmptyHeaders => f.write_str("empty headers"),
            Error::NoLastHeaderHash => f.write_str("no last header hash"),
            Error::UnconnectedHeaders => f.write_str("unconnected headers"),
            Error::ProstError(e) => write!(f, "prost error: {}", e),
            Error::MerkleProof(e) => write!(f, "merkle proof error: {}", e),
        }
    }
}

impl From<prost::DecodeError> for Error {
    fn from(e: prost::DecodeError) -> Self {
        Self::ProstError(e)
    }
}

impl From<MerkleProofError> for Error {
    fn from(e: MerkleProofError) -> Self {
        Self::MerkleProof(e)
    }
}

#[derive(Message, Clone, PartialEq)]
pub struct MsgData {
    #[prost(string, tag = "1")]
    pub msg_type: String,
    #[prost(bytes, tag = "2")]
    pub data: Vec<u8>,
}

#[derive(Message, Clone, PartialEq)]
pub struct TxMsgData {
    #[prost(message, repeated, tag = "1")]
    pub data: Vec<MsgData>,
}

impl TxMsgData {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Ok(Self::decode(bytes)?)
    }
}

// leaf element of LastResultHash merkle root.
#[derive(Message, Serialize, Deserialize, schemars::JsonSchema, Clone, PartialEq)]
pub struct TxResult {
    #[prost(uint32, tag = "1")]
    pub code: u32,

    #[prost(bytes = "vec", tag = "2")]
    #[schemars(with = "String")]
    #[serde(with = "crate::serde::base64")]
    pub data: Vec<u8>,

    /// nondeterministic
    #[prost(string, tag = "3")]
    #[serde(skip)]
    pub log: String,

    /// nondeterministic
    #[prost(string, tag = "4")]
    #[serde(skip)]
    pub info: String,

    #[prost(int64, tag = "5")]
    #[schemars(with = "String")]
    #[serde(with = "crate::serde::str")]
    pub gas_wanted: i64,

    #[prost(int64, tag = "6")]
    #[schemars(with = "String")]
    #[serde(with = "crate::serde::str")]
    pub gas_used: i64,

    /// nondeterministic
    #[prost(message, repeated, tag = "7")]
    #[serde(skip)]
    pub events: Vec<Event>,

    #[prost(string, tag = "8")]
    #[serde(skip)]
    pub codespace: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct Event {
    #[prost(string, tag = "1")]
    pub r#type: String,
    #[prost(message, repeated, tag = "2")]
    pub attributes: Vec<EventAttribute>,
}

/// EventAttribute is a single key-value pair, associated with an event.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventAttribute {
    #[prost(bytes = "vec", tag = "1")]
    pub key: Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub value: Vec<u8>,
    /// nondeterministic
    #[prost(bool, tag = "3")]
    pub index: bool,
}

impl TxResult {
    pub fn encode_as_merkle_leaf(&self) -> Vec<u8> {
        self.encode_to_vec()
    }

    pub fn tx_msg_data(&self) -> Result<TxMsgData, Error> {
        TxMsgData::from_bytes(self.data.as_slice())
    }
}

// Merkle Proof for TxResult.
// This proof is valid when and only when all of 1-3 proposition are correct.
// 1. (1 < a < length(headers)) header[a] contains hash of header[a-1] as it's last_block_id.hash'
// 2. the first element of headers(= the lowest header of headers) contains merkle root of merkle proof as it's 'last_result_hash'
// 3. merkle proof includes tx_result as it's leaf
#[derive(Serialize, Deserialize, schemars::JsonSchema, Clone, Debug, PartialEq)]
pub struct TxResultProof {
    pub tx_result: TxResult,
    pub merkle_proof: MerkleProof,
    pub headers: Vec<Header>,
}

impl TxResultProof {
    // if tx result proof is correct, returns hash of the highest header of connected headers.
    pub fn verify(&self) -> Result<Vec<u8>, Error> {
        let highest_header_hash = verify_headers_connected(&self.headers)?;
        let results_hash = self.headers.first().unwrap().last_results_hash.clone();
        self.merkle_proof
            .verify(results_hash, &self.tx_result.encode_as_merkle_leaf())?;
        Ok(highest_header_hash)
    }
}

// If given headers is connected, returns hash of the highest header.
fn verify_headers_connected(headers: &[Header]) -> Result<Vec<u8>, Error> {
    match headers.len() {
        0 => Err(Error::EmptyHeaders),
        _ => {
            let mut iter = headers.iter();
            let mut last_hash = iter.next().unwrap().hash();
            for header in iter {
                if last_hash
                    != header
                        .last_block_id
                        .clone()
                        .ok_or(Error::NoLastHeaderHash)?
                        .hash
                {
                    return Err(Error::UnconnectedHeaders);
                }
                last_hash = header.hash();
            }
            Ok(last_hash)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::light_block::header::fields::*;
    use crate::merkle::simple_hash_from_byte_vectors;

    #[test]
    fn test_tx_result_encode_as_merkle_leaf() {
        let tx_result = TxResult {
            code: 0,
            data: base64::decode("CpcDCiovc2VjcmV0LmNvbXB1dGUudjFiZXRhMS5Nc2dFeGVjdXRlQ29udHJhY3QS6AJVzmyAo99HQULa7zPh5czf0jm6vUIgLaV5GqHcbtdXEEIK0ZGQvykkyg6ikiUk3o5ynhv8kWK/3I+Eg9FEESWZuBsML2e4q/a4tFJk6eogjMhfJQ2uO4SKdPg4NTjB0An4bCC1uQdryNwQOpRM//GgidK55QZCeVIfEOAaD1GdVomol5t12V2qbjuM2vk/U8OLuLdjGgV2aMxju5tmjkdfwLiw2EpJlOG4qkuF9Lef0IUiFI627ED7G7JM2bzjjeI3ihXbYEgmmBfIe6gExN6K7yrJnWjzxrSVp3wAAkDkHu968GE+nX/mAuMyW7ME3yAnRv29wbyGgQi6tOByLIcKPy/1c3j3PEkmV2PQ/003zh5giCRJ75vbBs6xKcOM2V+l6m22N0KjVZZGfpJyOdJLQQPr7DYAQYarMEO81NayHr/SBjDbfOcfZoFcosg+dr7xfzSqqegs81dZUBSwsvboa0jILtzzbOE=").unwrap(),
            log: "".to_string(),
            info: "".to_string(),
            gas_used: 23757,
            gas_wanted: 50000,
            events: vec![],
            codespace: "".to_string(),
        };
        let encoded = tx_result.encode_as_merkle_leaf();
        assert_eq!(
            hex::encode_upper(&encoded),
            "129A030A97030A2A2F7365637265742E636F6D707574652E763162657461312E4D736745786563757465436F6E747261637412E80255CE6C80A3DF474142DAEF33E1E5CCDFD239BABD42202DA5791AA1DC6ED75710420AD19190BF2924CA0EA2922524DE8E729E1BFC9162BFDC8F8483D144112599B81B0C2F67B8ABF6B8B45264E9EA208CC85F250DAE3B848A74F8383538C1D009F86C20B5B9076BC8DC103A944CFFF1A089D2B9E5064279521F10E01A0F519D5689A8979B75D95DAA6E3B8CDAF93F53C38BB8B7631A057668CC63BB9B668E475FC0B8B0D84A4994E1B8AA4B85F4B79FD08522148EB6EC40FB1BB24CD9BCE38DE2378A15DB6048269817C87BA804C4DE8AEF2AC99D68F3C6B495A77C000240E41EEF7AF0613E9D7FE602E3325BB304DF202746FDBDC1BC868108BAB4E0722C870A3F2FF57378F73C49265763D0FF4D37CE1E60882449EF9BDB06CEB129C38CD95FA5EA6DB63742A35596467E927239D24B4103EBEC36004186AB3043BCD4D6B21EBFD20630DB7CE71F66815CA2C83E76BEF17F34AAA9E82CF357595014B0B2F6E86B48C82EDCF36CE128D0860330CDB901"
        );
        let leaf_hash = simple_hash_from_byte_vectors(vec![encoded.clone()]);
        assert_eq!(
            hex::encode_upper(leaf_hash),
            "3F54943D32A473050D0C96E69E0C5778BE11A540BAEFBFE005D86883A60D6CC3"
        );
        let root = simple_hash_from_byte_vectors(vec![encoded]);
        assert_eq!(
            hex::encode_upper(root),
            "3F54943D32A473050D0C96E69E0C5778BE11A540BAEFBFE005D86883A60D6CC3"
        );
    }

    #[test]
    fn test_tx_result_tx_msg_data() {
        let tx_result = TxResult {
            code: 0,
            data: base64::decode("CpcDCiovc2VjcmV0LmNvbXB1dGUudjFiZXRhMS5Nc2dFeGVjdXRlQ29udHJhY3QS6AJVzmyAo99HQULa7zPh5czf0jm6vUIgLaV5GqHcbtdXEEIK0ZGQvykkyg6ikiUk3o5ynhv8kWK/3I+Eg9FEESWZuBsML2e4q/a4tFJk6eogjMhfJQ2uO4SKdPg4NTjB0An4bCC1uQdryNwQOpRM//GgidK55QZCeVIfEOAaD1GdVomol5t12V2qbjuM2vk/U8OLuLdjGgV2aMxju5tmjkdfwLiw2EpJlOG4qkuF9Lef0IUiFI627ED7G7JM2bzjjeI3ihXbYEgmmBfIe6gExN6K7yrJnWjzxrSVp3wAAkDkHu968GE+nX/mAuMyW7ME3yAnRv29wbyGgQi6tOByLIcKPy/1c3j3PEkmV2PQ/003zh5giCRJ75vbBs6xKcOM2V+l6m22N0KjVZZGfpJyOdJLQQPr7DYAQYarMEO81NayHr/SBjDbfOcfZoFcosg+dr7xfzSqqegs81dZUBSwsvboa0jILtzzbOE=").unwrap(),
            log: "".to_string(),
            info: "".to_string(),
            gas_used: 23757,
            gas_wanted: 50000,
            events: vec![],
            codespace: "".to_string(),
        };
        let tx_msg_data = tx_result.tx_msg_data().unwrap();
        assert_eq!(
            tx_msg_data,
            TxMsgData {
                data: vec![MsgData {
                    msg_type: "/secret.compute.v1beta1.MsgExecuteContract".into(),
                    data: vec![
                        85, 206, 108, 128, 163, 223, 71, 65, 66, 218, 239, 51, 225, 229, 204, 223,
                        210, 57, 186, 189, 66, 32, 45, 165, 121, 26, 161, 220, 110, 215, 87, 16,
                        66, 10, 209, 145, 144, 191, 41, 36, 202, 14, 162, 146, 37, 36, 222, 142,
                        114, 158, 27, 252, 145, 98, 191, 220, 143, 132, 131, 209, 68, 17, 37, 153,
                        184, 27, 12, 47, 103, 184, 171, 246, 184, 180, 82, 100, 233, 234, 32, 140,
                        200, 95, 37, 13, 174, 59, 132, 138, 116, 248, 56, 53, 56, 193, 208, 9, 248,
                        108, 32, 181, 185, 7, 107, 200, 220, 16, 58, 148, 76, 255, 241, 160, 137,
                        210, 185, 229, 6, 66, 121, 82, 31, 16, 224, 26, 15, 81, 157, 86, 137, 168,
                        151, 155, 117, 217, 93, 170, 110, 59, 140, 218, 249, 63, 83, 195, 139, 184,
                        183, 99, 26, 5, 118, 104, 204, 99, 187, 155, 102, 142, 71, 95, 192, 184,
                        176, 216, 74, 73, 148, 225, 184, 170, 75, 133, 244, 183, 159, 208, 133, 34,
                        20, 142, 182, 236, 64, 251, 27, 178, 76, 217, 188, 227, 141, 226, 55, 138,
                        21, 219, 96, 72, 38, 152, 23, 200, 123, 168, 4, 196, 222, 138, 239, 42,
                        201, 157, 104, 243, 198, 180, 149, 167, 124, 0, 2, 64, 228, 30, 239, 122,
                        240, 97, 62, 157, 127, 230, 2, 227, 50, 91, 179, 4, 223, 32, 39, 70, 253,
                        189, 193, 188, 134, 129, 8, 186, 180, 224, 114, 44, 135, 10, 63, 47, 245,
                        115, 120, 247, 60, 73, 38, 87, 99, 208, 255, 77, 55, 206, 30, 96, 136, 36,
                        73, 239, 155, 219, 6, 206, 177, 41, 195, 140, 217, 95, 165, 234, 109, 182,
                        55, 66, 163, 85, 150, 70, 126, 146, 114, 57, 210, 75, 65, 3, 235, 236, 54,
                        0, 65, 134, 171, 48, 67, 188, 212, 214, 178, 30, 191, 210, 6, 48, 219, 124,
                        231, 31, 102, 129, 92, 162, 200, 62, 118, 190, 241, 127, 52, 170, 169, 232,
                        44, 243, 87, 89, 80, 20, 176, 178, 246, 232, 107, 72, 200, 46, 220, 243,
                        108, 225
                    ],
                }]
            }
        );
    }

    #[test]
    fn test_verify_headers_connected_sanity() {
        let headers = {
            let mut prev_header_hash =
                hex::decode("AAAE99EE9C5A976417E4343837BD046CBE6E66BB2C634D0BDA699F0263C64803")
                    .unwrap();
            let mut headers = vec![];
            for i in 0..5 {
                let header = Header {
                    version: Some(Version { app: 0, block: 0 }),
                    chain_id: "chain_id".to_string(),
                    height: i + 1,
                    time: Some(Timestamp {
                        seconds: 0,
                        nanos: 0,
                    }),
                    last_block_id: Some(BlockId {
                        hash: prev_header_hash,
                        parts: Some(PartSetHeader {
                            total: 0,
                            hash: [0; 32].to_vec(),
                        }),
                    }),
                    last_commit_hash: [0; 32].to_vec(),
                    data_hash: [0; 32].to_vec(),
                    validators_hash: [0; 32].to_vec(),
                    next_validators_hash: [0; 32].to_vec(),
                    consensus_hash: [0; 32].to_vec(),
                    app_hash: [0; 32].to_vec(),
                    last_results_hash: [0; 32].to_vec(),
                    evidence_hash: [0; 32].to_vec(),
                    proposer_address: [0; 32].to_vec(),
                };
                prev_header_hash = header.hash();
                headers.push(header);
            }
            headers
        };
        assert_eq!(
            verify_headers_connected(&headers).unwrap(),
            headers.last().unwrap().hash()
        )
    }

    #[test]
    fn test_verify_headers_connected_1_header() {
        let header = Header {
            version: Some(Version { app: 0, block: 0 }),
            chain_id: "chain_id".to_string(),
            height: 1,
            time: Some(Timestamp {
                seconds: 0,
                nanos: 0,
            }),
            last_block_id: Some(BlockId {
                hash: hex::decode(
                    "AAAE99EE9C5A976417E4343837BD046CBE6E66BB2C634D0BDA699F0263C64803",
                )
                .unwrap(),
                parts: Some(PartSetHeader {
                    total: 0,
                    hash: [0; 32].to_vec(),
                }),
            }),
            last_commit_hash: [0; 32].to_vec(),
            data_hash: [0; 32].to_vec(),
            validators_hash: [0; 32].to_vec(),
            next_validators_hash: [0; 32].to_vec(),
            consensus_hash: [0; 32].to_vec(),
            app_hash: [0; 32].to_vec(),
            last_results_hash: [0; 32].to_vec(),
            evidence_hash: [0; 32].to_vec(),
            proposer_address: [0; 32].to_vec(),
        };
        let headers = vec![header.clone()];
        assert_eq!(verify_headers_connected(&headers).unwrap(), header.hash())
    }

    #[test]
    fn test_verify_headers_connected_0_header() {
        let headers = vec![];
        assert_eq!(
            verify_headers_connected(&headers).unwrap_err(),
            Error::EmptyHeaders
        )
    }

    #[test]
    fn test_verify_headers_connected_not_connected() {
        let change_hash = [0; 32].to_vec();
        for hash_change_index in 1..5 {
            let mut prev_header_hash =
                hex::decode("AAAE99EE9C5A976417E4343837BD046CBE6E66BB2C634D0BDA699F0263C64803")
                    .unwrap();
            let mut headers = vec![];
            for i in 0..5 {
                if i == hash_change_index {
                    prev_header_hash = change_hash.clone();
                }
                let header = Header {
                    version: Some(Version { app: 0, block: 0 }),
                    chain_id: "chain_id".to_string(),
                    height: i + 1,
                    time: Some(Timestamp {
                        seconds: 0,
                        nanos: 0,
                    }),
                    last_block_id: Some(BlockId {
                        hash: prev_header_hash,
                        parts: Some(PartSetHeader {
                            total: 0,
                            hash: [0; 32].to_vec(),
                        }),
                    }),
                    last_commit_hash: [0; 32].to_vec(),
                    data_hash: [0; 32].to_vec(),
                    validators_hash: [0; 32].to_vec(),
                    next_validators_hash: [0; 32].to_vec(),
                    consensus_hash: [0; 32].to_vec(),
                    app_hash: [0; 32].to_vec(),
                    last_results_hash: [0; 32].to_vec(),
                    evidence_hash: [0; 32].to_vec(),
                    proposer_address: [0; 32].to_vec(),
                };
                prev_header_hash = header.hash();
                headers.push(header);
            }
            assert_eq!(
                verify_headers_connected(&headers).unwrap_err(),
                Error::UnconnectedHeaders
            )
        }
    }
}
