use bitcoin::consensus::encode;
use bitcoin::secp256k1;
use bitcoin::util::address;
use bitcoin::util::sighash;

#[derive(Debug)]
pub enum Error {
    BitcoinEncode(encode::Error),
    SigHash(sighash::Error),
    Secp256k1(secp256k1::Error),
    TryFromInt(std::num::TryFromIntError),
    BitcoinAddressError(address::Error),
    StdIoError(std::io::Error),
    Cosmwasm(cosmwasm_std::StdError),
    Contract(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::BitcoinEncode(ref e) => write!(f, "bitcoin encode error {}", e),
            Error::SigHash(ref e) => write!(f, "sighash error {}", e),
            Error::Secp256k1(ref e) => write!(f, "secp256k1 error {}", e),
            Error::TryFromInt(ref e) => write!(f, "try from int error {}", e),
            Error::BitcoinAddressError(ref e) => write!(f, "bitcoin address error {}", e),
            Error::StdIoError(ref e) => write!(f, "std io  error {}", e),
            Error::Cosmwasm(ref e) => write!(f, "cosmwasm std error {}", e),
            Error::Contract(ref msg) => write!(f, "contract error {}", msg),
        }
    }
}

impl From<sighash::Error> for Error {
    fn from(e: sighash::Error) -> Error {
        Error::SigHash(e)
    }
}

impl From<encode::Error> for Error {
    fn from(e: encode::Error) -> Error {
        Error::BitcoinEncode(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Error {
        Error::Secp256k1(e)
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(e: std::num::TryFromIntError) -> Error {
        Error::TryFromInt(e)
    }
}

impl From<address::Error> for Error {
    fn from(e: address::Error) -> Error {
        Error::BitcoinAddressError(e)
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
