use super::signature_message::signature_message;
use super::Error;
use crate::light_block::header::fields::BlockId;
use crate::light_block::header::fields::Timestamp;
use serde::Serializer;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use std::convert::{TryFrom, TryInto};

#[derive(PartialEq, Debug, Clone)]
pub enum Vote {
    Absent,
    Commit(CommitVote),
    Nil(NilVote),
}

pub const SIGNATURE_LENGTH: usize = 64;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Signature([u8; SIGNATURE_LENGTH]);

impl Signature {
    pub fn new(bytes: [u8; SIGNATURE_LENGTH]) -> Self {
        Self(bytes)
    }

    pub fn to_bytes(&self) -> [u8; SIGNATURE_LENGTH] {
        self.0
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct CommitVote {
    pub validator_address: String,
    pub timestamp: Timestamp,
    pub signature: Signature,
}

impl CommitVote {
    pub fn signature_message(
        &self,
        height: i64,
        round: i32,
        block_id: BlockId,
        chain_id: String,
    ) -> Vec<u8> {
        signature_message(
            2,
            height,
            round,
            Some(block_id),
            self.timestamp.clone(),
            chain_id,
        )
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct NilVote {
    pub validator_address: String,
    pub timestamp: Timestamp,
    pub signature: Signature,
}

impl NilVote {
    pub fn signature_message(&self, height: i64, round: i32, chain_id: String) -> Vec<u8> {
        signature_message(2, height, round, None, self.timestamp.clone(), chain_id)
    }
}

impl TryFrom<CommitSig> for Vote {
    type Error = Error;
    fn try_from(raw: CommitSig) -> Result<Self, Self::Error> {
        let block_id_flag = BlockIdFlag::try_from(raw.block_id_flag)?;
        if block_id_flag == BlockIdFlag::Absent {
            return Ok(Self::Absent);
        }
        let signature: Signature = Signature::new(
            base64::decode(raw.signature.ok_or(Error::NoSignature)?)?
                .try_into()
                .map_err(|_| Error::DecodeSignature())?,
        );
        if block_id_flag == BlockIdFlag::Commit {
            Ok(Self::Commit(CommitVote {
                validator_address: raw.validator_address,
                timestamp: raw.timestamp,
                signature,
            }))
        } else {
            Ok(Self::Nil(NilVote {
                validator_address: raw.validator_address,
                timestamp: raw.timestamp,
                signature,
            }))
        }
    }
}

impl<'de> Deserialize<'de> for Vote {
    fn deserialize<D>(deserializer: D) -> Result<Vote, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = CommitSig::deserialize(deserializer)?;
        raw.try_into().map_err(D::Error::custom)
    }
}

impl Serialize for Vote {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let raw: CommitSig = self.into();
        raw.serialize(serializer)
    }
}

/*
   example data
   {
       block_id_flag: 1,
       validator_address: "",
       timestamp: "0001-01-01T00:00:00Z",
       signature: null
   },
   {
       block_id_flag: 2,
       validator_address: "2DD098C8ECAF04DFE31BBC59799C786AC09BF53F",
       timestamp: "2021-03-19T13:14:19.2585042Z",
       signature: "V6rRUf3GHNWYveEl2iKsIYAJVmJpjVg3pDj5DRdZ/1L9Vv0bD4k52vnrnHlShdumhRbbWDjQaCg+xGnn48D+Bw=="
   },
*/
#[derive(Clone, PartialEq, Deserialize, Serialize, Debug, schemars::JsonSchema)]
pub struct CommitSig {
    pub block_id_flag: u64,
    pub validator_address: String,
    #[schemars(with = "String")]
    pub timestamp: Timestamp,
    pub signature: Option<String>,
}

impl From<&Vote> for CommitSig {
    fn from(commit_sig: &Vote) -> Self {
        match commit_sig {
            Vote::Absent => Self {
                block_id_flag: 1,
                validator_address: "".into(),
                // time stamp of 0001-01-01T00:00:00Z
                timestamp: Timestamp {
                    seconds: -62135596800,
                    nanos: 0,
                },
                signature: None,
            },
            Vote::Commit(vote) => Self {
                block_id_flag: 2,
                validator_address: vote.validator_address.to_string(),
                timestamp: vote.timestamp.clone(),
                signature: Some(base64::encode(vote.signature.to_bytes())),
            },
            Vote::Nil(vote) => Self {
                block_id_flag: 3,
                validator_address: vote.validator_address.to_string(),
                timestamp: vote.timestamp.clone(),
                signature: Some(base64::encode(vote.signature.to_bytes())),
            },
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum BlockIdFlag {
    Absent = 1,
    Commit = 2,
    Nil = 3,
}

impl TryFrom<u64> for BlockIdFlag {
    type Error = Error;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(BlockIdFlag::Absent),
            2 => Ok(BlockIdFlag::Commit),
            3 => Ok(BlockIdFlag::Nil),
            _ => Err(Error::InvalidBlockIdFlag(value)),
        }
    }
}

impl<'de> Deserialize<'de> for BlockIdFlag {
    fn deserialize<D>(deserializer: D) -> Result<BlockIdFlag, D::Error>
    where
        D: Deserializer<'de>,
    {
        let flag: u8 = u8::deserialize(deserializer)?;
        match flag {
            1 => Ok(BlockIdFlag::Absent),
            2 => Ok(BlockIdFlag::Commit),
            3 => Ok(BlockIdFlag::Nil),
            _ => Err(D::Error::custom(format!("invalid block id flag: {}", flag))),
        }
    }
}

impl Serialize for BlockIdFlag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let flag: u8 = match self {
            BlockIdFlag::Absent => 1,
            BlockIdFlag::Commit => 2,
            BlockIdFlag::Nil => 3,
        };
        flag.serialize(serializer)
    }
}
