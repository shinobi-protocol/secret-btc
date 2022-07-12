mod signature_message;
pub mod vote;

use super::Error;
use crate::light_block::header::fields::BlockId;
use serde::{Deserialize, Serialize};
use vote::{CommitSig, Vote};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Commit {
    /// Block height
    #[schemars(with = "String")]
    #[serde(with = "crate::serde::str")]
    pub height: i64,

    /// Round
    pub round: i32,

    /// Block ID
    pub block_id: BlockId,

    /// Votes
    #[schemars(with = "Vec<CommitSig>")]
    pub signatures: Vec<Vote>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::light_block::header::fields::PartSetHeader;
    use crate::light_block::signed_header::commit::vote::CommitVote;
    use std::convert::TryInto;
    use vote::Signature;

    #[test]
    fn test_deserialize_commit() {
        let json_str = r#"
            {
                "height": "1000001",
                "round": 0,
                "block_id": {
                    "hash": "275AFF29FB91FCFC3E6581B3522502205F03758864CD1030D2A5E2212AA4FBE2",
                    "parts": {
                        "total": 1,
                        "hash": "8C36952ADE6331D2F711AF167879C60340F44DEDEF9E1ADB26DF9DC7549D9B06"
                    }
                },
                "signatures": [
                    {
                        "block_id_flag": 2,
                        "validator_address": "2E76AE6E453395F35D6C0728D44FB6147CE5B5A0",
                        "timestamp": "2021-11-23T16:07:43.581599777Z",
                        "signature": "rIt3m7ehMrIVRzzd3q6Ty6x3JGVutjKyEepb+VLVHmqzB76QgtbtHLRPm4Z5axTcUHf06hh8H2gCCiTN/jRYBg=="
                    }
                ]
            }
        "#;
        let commit: Commit = serde_json::from_str(json_str).unwrap();
        assert_eq!(commit.height, 1000001);
        assert_eq!(commit.round, 0);
        assert_eq!(
            commit.block_id,
            BlockId {
                hash: hex::decode(
                    "275AFF29FB91FCFC3E6581B3522502205F03758864CD1030D2A5E2212AA4FBE2"
                )
                .unwrap(),
                parts: Some(PartSetHeader {
                    total: 1,
                    hash: hex::decode(
                        "8C36952ADE6331D2F711AF167879C60340F44DEDEF9E1ADB26DF9DC7549D9B06"
                    )
                    .unwrap(),
                }),
            }
        );
        assert_eq!(
            commit.signatures,
            vec![Vote::Commit(CommitVote {
                validator_address: "2E76AE6E453395F35D6C0728D44FB6147CE5B5A0".into(),
                timestamp: serde_json::from_str(r#""2021-11-23T16:07:43.581599777Z""#).unwrap(),
                signature: Signature::new(base64::decode("rIt3m7ehMrIVRzzd3q6Ty6x3JGVutjKyEepb+VLVHmqzB76QgtbtHLRPm4Z5axTcUHf06hh8H2gCCiTN/jRYBg==").unwrap().as_slice().try_into().unwrap())
            })]
        );
    }
}
