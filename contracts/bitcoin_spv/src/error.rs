use bitcoin::consensus::encode;
use bitcoin::hashes;
use bitcoin_header_chain::{header_chain, merkle_proof};

#[derive(Debug)]
pub enum Error {
    BitcoinEncode(encode::Error),
    HashHexError(hashes::hex::Error),
    HeaderChain(header_chain::Error),
    MerkleProof(merkle_proof::Error),
    StdIoError(std::io::Error),
    Cosmwasm(cosmwasm_std::StdError),
    Contract(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::BitcoinEncode(ref e) => write!(f, "bitcoin encode error {}", e),
            Error::HashHexError(ref e) => write!(f, "hash hex error {}", e),
            Error::HeaderChain(ref e) => write!(f, "header chain error {}", e),
            Error::MerkleProof(ref e) => write!(f, "merkle path error {}", e),
            Error::StdIoError(ref e) => write!(f, "std io error {}", e),
            Error::Cosmwasm(ref e) => write!(f, "cosmwasm std error {}", e),
            Error::Contract(ref msg) => write!(f, "contract error {}", msg),
        }
    }
}

impl From<encode::Error> for Error {
    fn from(e: encode::Error) -> Error {
        Error::BitcoinEncode(e)
    }
}
impl From<hashes::hex::Error> for Error {
    fn from(e: hashes::hex::Error) -> Error {
        Error::HashHexError(e)
    }
}
impl From<header_chain::Error> for Error {
    fn from(e: header_chain::Error) -> Error {
        Error::HeaderChain(e)
    }
}
impl From<merkle_proof::Error> for Error {
    fn from(e: merkle_proof::Error) -> Error {
        Error::MerkleProof(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::StdIoError(e)
    }
}

impl From<cosmwasm_std::StdError> for Error {
    fn from(e: cosmwasm_std::StdError) -> Error {
        Error::Cosmwasm(e)
    }
}

impl From<Error> for cosmwasm_std::StdError {
    fn from(e: Error) -> cosmwasm_std::StdError {
        if let Error::Cosmwasm(err) = e {
            err
        } else {
            cosmwasm_std::StdError::generic_err(e.to_string())
        }
    }
}

impl Error {
    pub fn contract_err<S: Into<String>>(msg: S) -> Self {
        Error::Contract(msg.into())
    }
}
