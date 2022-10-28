use schemars::JsonSchema;
use sha2::{Digest, Sha256};
use std::fmt;
use std::string::ToString;

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    Bincode(String),
    InvalidCommit,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Bincode(e) => write!(f, "bincode error: {}", e),
            Error::InvalidCommit => f.write_str("invalid commit"),
        }
    }
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Self::Bincode(e.to_string())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HeaderHashWithHeight {
    pub hash: Vec<u8>,
    pub height: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Hashes {
    pub anchor_hash: Vec<u8>,
    pub anchor_index: u64,
    pub following_hashes: Vec<HeaderHashWithHeight>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug, JsonSchema)]
pub struct Commit(Vec<u8>);

impl Commit {
    pub fn new(hashes: &Hashes, secret: &[u8]) -> Result<Self, Error> {
        let hashes_binary = bincode::serialize(&hashes)?;
        let mut digest = Sha256::default();
        digest.update(hashes_binary);
        digest.update(secret);
        Ok(Self(digest.finalize().as_slice().into()))
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CommittedHashes {
    pub hashes: Hashes,
    pub commit: Commit,
}

impl CommittedHashes {
    pub fn new(hashes: Hashes, secret: &[u8]) -> Result<Self, Error> {
        let commit = Commit::new(&hashes, secret)?;
        Ok(Self {
            hashes: hashes,
            commit: commit,
        })
    }

    pub fn verify(&self, secret: &[u8]) -> Result<(), Error> {
        if self.commit == Commit::new(&self.hashes, secret)? {
            Ok(())
        } else {
            Err(Error::InvalidCommit)
        }
    }
}
