use crate::header::hash_header;
use cosmos_proto::tendermint::types::SignedHeader;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    NoHeader,
    NoCommit,
    InvalidCommit,
    InvalidHeaderHash { commit: Vec<u8>, header: Vec<u8> },
    InvalidHeight { commit: i64, header: i64 },
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::NoHeader => f.write_str("no header"),
            Error::NoCommit => f.write_str("no commit"),
            Error::InvalidCommit => f.write_str("invalid commit"),
            Error::InvalidHeaderHash { commit, header } => write!(
                f,
                "invalid block hash: commit {}, header {}",
                hex::encode(&commit),
                hex::encode(&header)
            ),
            Error::InvalidHeight { commit, header } => {
                write!(f, "invalid height: commit {}, header {}", commit, header)
            }
        }
    }
}

pub fn validate_signed_header(signed_header: &SignedHeader) -> Result<(), Error> {
    let header = signed_header
        .header
        .as_ref()
        .ok_or_else(|| Error::NoHeader)?;
    let commit = signed_header
        .commit
        .as_ref()
        .ok_or_else(|| Error::NoCommit)?;
    let block_id = commit
        .block_id
        .as_ref()
        .ok_or_else(|| Error::InvalidCommit)?;
    let headers_hash = hash_header(header).into();
    if block_id.hash != headers_hash {
        return Err(Error::InvalidHeaderHash {
            commit: block_id.hash.clone(),
            header: headers_hash,
        });
    }
    let headers_height = header.height;
    if commit.height != headers_height {
        return Err(Error::InvalidHeight {
            commit: commit.height,
            header: headers_height,
        });
    }
    Ok(())
}
