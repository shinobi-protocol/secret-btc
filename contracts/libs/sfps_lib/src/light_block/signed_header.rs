pub mod commit;

pub use commit::vote::Vote;
pub use commit::Commit;

use super::header::Header;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidHeaderHash { commit: Vec<u8>, header: Vec<u8> },
    InvalidHeight { commit: i64, header: i64 },
    InvalidBlockIdFlag(u64),
    NoSignature,
    Base64(base64::DecodeError),
    DecodeSignature(),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidHeaderHash { commit, header } => write!(
                f,
                "invalid block hash: commit {}, header {}",
                hex::encode(&commit),
                hex::encode(&header)
            ),
            Error::InvalidHeight { commit, header } => {
                write!(f, "invalid height: commit {}, header {}", commit, header)
            }
            Error::InvalidBlockIdFlag(value) => {
                write!(f, "invalid block id flag {}", value)
            }
            Error::NoSignature => f.write_str("no signature"),
            Error::Base64(e) => write!(f, "base64 error: {}", e),
            Error::DecodeSignature() => write!(f, "failed to decode signature error"),
        }
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Self::Base64(e)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
pub struct SignedHeader {
    pub header: Header,
    pub commit: Commit,
}

impl SignedHeader {
    pub fn validate_basic(&self) -> Result<(), Error> {
        self.validate_header_hash()?;
        self.validate_height()
    }

    fn validate_header_hash(&self) -> Result<(), Error> {
        let headers_hash = self.header.hash().into();
        if self.commit.block_id.hash != headers_hash {
            return Err(Error::InvalidHeaderHash {
                commit: self.commit.block_id.hash.clone(),
                header: headers_hash,
            });
        }
        Ok(())
    }

    fn validate_height(&self) -> Result<(), Error> {
        let headers_height = self.header.height;
        if self.commit.height != headers_height {
            return Err(Error::InvalidHeight {
                commit: self.commit.height,
                header: headers_height,
            });
        }
        Ok(())
    }
}
