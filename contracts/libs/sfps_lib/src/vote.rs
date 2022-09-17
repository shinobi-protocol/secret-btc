use cosmos_proto::prost::Message;
use cosmos_proto::prost_types::Timestamp;
use cosmos_proto::tendermint::types::{
    BlockId, CanonicalBlockId, CanonicalPartSetHeader, CanonicalVote, CommitSig,
};
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    DecodeSignature,
    NoTimestamp,
    InvalidBlockIdFlag(i32),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::DecodeSignature => write!(f, "failed to decode signature error"),
            Error::NoTimestamp => f.write_str("no timestamp"),
            Error::InvalidBlockIdFlag(value) => {
                write!(f, "invalid block id flag {}", value)
            }
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Vote {
    Absent,
    Commit(CommitVote),
    Nil(NilVote),
}

#[derive(PartialEq, Debug, Clone)]
pub struct CommitVote {
    pub timestamp: Timestamp,
    pub signature: Vec<u8>,
}

impl CommitVote {
    pub fn signature_message(
        &self,
        height: i64,
        round: i64,
        block_id: BlockId,
        chain_id: String,
    ) -> Vec<u8> {
        signature_message(
            2,
            height,
            round,
            Some(block_id),
            Some(self.timestamp.clone()),
            chain_id,
        )
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct NilVote {
    pub timestamp: Timestamp,
    pub signature: Vec<u8>,
}

impl NilVote {
    pub fn signature_message(&self, height: i64, round: i64, chain_id: String) -> Vec<u8> {
        signature_message(
            2,
            height,
            round,
            None,
            Some(self.timestamp.clone()),
            chain_id,
        )
    }
}

fn signature_message(
    r#type: i32,
    height: i64,
    round: i64,
    block_id: Option<BlockId>,
    timestamp: Option<Timestamp>,
    chain_id: String,
) -> Vec<u8> {
    let vote = CanonicalVote {
        r#type,
        height,
        round,
        block_id: block_id.map(|block_id| CanonicalBlockId {
            hash: block_id.hash,
            part_set_header: block_id.part_set_header.map(|part_set_header| {
                CanonicalPartSetHeader {
                    total: part_set_header.total,
                    hash: part_set_header.hash,
                }
            }),
        }),
        timestamp,
        chain_id,
    };
    let mut bytes = Vec::with_capacity(vote.encoded_len());
    vote.encode_length_delimited(&mut bytes).unwrap();
    bytes
}

impl TryFrom<&CommitSig> for Vote {
    type Error = Error;
    fn try_from(raw: &CommitSig) -> Result<Self, Self::Error> {
        let block_id_flag = BlockIdFlag::try_from(raw.block_id_flag)?;
        if block_id_flag == BlockIdFlag::Absent {
            return Ok(Self::Absent);
        }
        if block_id_flag == BlockIdFlag::Commit {
            Ok(Self::Commit(CommitVote {
                timestamp: raw.timestamp.clone().ok_or_else(|| Error::NoTimestamp)?,
                signature: raw.signature.clone(),
            }))
        } else {
            Ok(Self::Nil(NilVote {
                timestamp: raw.timestamp.clone().ok_or_else(|| Error::NoTimestamp)?,
                signature: raw.signature.clone(),
            }))
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum BlockIdFlag {
    Absent = 1,
    Commit = 2,
    Nil = 3,
}

impl TryFrom<i32> for BlockIdFlag {
    type Error = Error;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(BlockIdFlag::Absent),
            2 => Ok(BlockIdFlag::Commit),
            3 => Ok(BlockIdFlag::Nil),
            _ => Err(Error::InvalidBlockIdFlag(value)),
        }
    }
}
