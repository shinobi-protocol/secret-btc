use crate::light_block::header::Header;
use crate::light_block::Ed25519Verifier;
use crate::light_block::{Error as LightBlockError, LightBlock};
use crate::tx_result_proof::{Error as TxResultProofError, TxResultProof};
use std::convert::TryInto;
use std::fmt;

pub trait ReadonlyChainDB {
    fn get_hash_by_index(&mut self, index: usize) -> Option<Vec<u8>>;
    fn get_highest_hash(&mut self) -> Option<Vec<u8>>;
    fn get_hash_list_length(&mut self) -> usize;
    fn get_max_interval(&mut self) -> u64;
}

pub trait ChainDB: ReadonlyChainDB {
    type Error: std::fmt::Display;
    fn append_header_hash(&mut self, hash: Vec<u8>) -> Result<(), Self::Error>;
    fn store_max_interval(&mut self, max_interval: u64) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    AlreadyInitialized,
    ChainDB(String),
    InvalidHighestHash,
    ExceedsInterval { max: u64, actual: u64 },
    NoHighestHeaderHash,
    InvalidCurrentHighestHeader,
    UnmatchedValidatorsHash,
    LightBlock(LightBlockError),
    TxResultProof(TxResultProofError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::AlreadyInitialized => f.write_str("already initialized"),
            Error::ChainDB(e) => write!(f, "chain db error: {}", e),
            Error::InvalidHighestHash => f.write_str("invalid highest hash"),
            Error::ExceedsInterval { max, actual } => {
                write!(f, "exceeds interval: max {}, actual {}", max, actual)
            }
            Error::NoHighestHeaderHash => f.write_str("no highest header hash"),
            Error::InvalidCurrentHighestHeader => f.write_str("invalid current highest header"),
            Error::UnmatchedValidatorsHash => f.write_str("unmatched validators hash"),
            Error::LightBlock(e) => write!(f, "light block error: {}", e),
            Error::TxResultProof(e) => write!(f, "tx result proof error: {}", e),
        }
    }
}

impl From<LightBlockError> for Error {
    fn from(e: LightBlockError) -> Self {
        Self::LightBlock(e)
    }
}

impl From<TxResultProofError> for Error {
    fn from(e: TxResultProofError) -> Self {
        Self::TxResultProof(e)
    }
}

pub struct HeaderChain<C: ReadonlyChainDB> {
    chain_db: C,
}

impl<C: ReadonlyChainDB> HeaderChain<C> {
    pub fn new(chain_db: C) -> Self {
        Self { chain_db }
    }
    pub fn verify_tx_result_proof(
        &mut self,
        tx_result_proof: &TxResultProof,
        header_hash_index: usize,
    ) -> Result<(), Error> {
        let highest_header_hash = tx_result_proof.verify()?;
        if let Some(stored_hash) = self.chain_db.get_hash_by_index(header_hash_index) {
            if stored_hash == highest_header_hash {
                return Ok(());
            }
        }
        Err(Error::InvalidHighestHash)
    }
}

impl<C: ChainDB> HeaderChain<C> {
    pub fn init(&mut self, header: Header, max_interval: u64) -> Result<(), Error> {
        if self.chain_db.get_hash_list_length() > 0 {
            return Err(Error::AlreadyInitialized);
        }
        self.append_header(header)?;
        self.chain_db
            .store_max_interval(max_interval)
            .map_err(|e| Error::ChainDB(format!("{}", e)))
    }

    pub fn add_block_to_highest<E: Ed25519Verifier>(
        &mut self,
        current_highest_header: &Header,
        light_block: LightBlock,
        ed25519_verifier: &mut E,
    ) -> Result<(), Error> {
        {
            let actual: u64 = light_block
                .signed_header
                .header
                .height
                .checked_sub(current_highest_header.height)
                .unwrap()
                .try_into()
                .unwrap();
            let max = self.chain_db.get_max_interval();
            if actual > max {
                return Err(Error::ExceedsInterval { max, actual });
            }
        }
        let current_highest_hash = self
            .chain_db
            .get_highest_hash()
            .ok_or(Error::NoHighestHeaderHash)?;
        if current_highest_header.hash() != current_highest_hash {
            return Err(Error::InvalidCurrentHighestHeader);
        }
        self.verify_block(
            &current_highest_header.next_validators_hash,
            &light_block,
            ed25519_verifier,
        )?;
        self.append_header(light_block.signed_header.header)
    }

    fn verify_block<E: Ed25519Verifier>(
        &self,
        validators_hash: &[u8],
        light_block: &LightBlock,
        ed25519_verifier: &mut E,
    ) -> Result<(), Error> {
        if validators_hash != light_block.signed_header.header.validators_hash {
            return Err(Error::UnmatchedValidatorsHash);
        }
        light_block.verify(ed25519_verifier)?;
        Ok(())
    }

    fn append_header(&mut self, header: Header) -> Result<(), Error> {
        let hash = header.hash();
        self.chain_db
            .append_header_hash(hash)
            .map_err(|e| Error::ChainDB(format!("{}", e)))
    }
}
