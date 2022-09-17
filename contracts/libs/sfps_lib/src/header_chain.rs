use crate::header::hash_header;
use crate::light_block::Ed25519Verifier;
use crate::light_block::Error as LightBlockError;
use crate::light_block::{validate_light_block, verify_light_block};
use crate::response_deliver_tx_proof::{
    Error as ResponseDeliverTxProofError, ResponseDeliverTxProof,
};
use cosmos_proto::tendermint::types::{Header, LightBlock};
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
    fn append_block_hash(&mut self, hash: Vec<u8>) -> Result<(), Self::Error>;
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
    ResponseDeliverTxProof(ResponseDeliverTxProofError),
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
            Error::ResponseDeliverTxProof(e) => write!(f, "response deliver proof error: {}", e),
        }
    }
}

impl From<LightBlockError> for Error {
    fn from(e: LightBlockError) -> Self {
        Self::LightBlock(e)
    }
}

impl From<ResponseDeliverTxProofError> for Error {
    fn from(e: ResponseDeliverTxProofError) -> Self {
        Self::ResponseDeliverTxProof(e)
    }
}

pub struct HeaderChain<C: ReadonlyChainDB> {
    chain_db: C,
}

impl<C: ReadonlyChainDB> HeaderChain<C> {
    pub fn new(chain_db: C) -> Self {
        Self { chain_db }
    }
    pub fn verify_response_deliver_tx_proof(
        &mut self,
        response_deliver_tx_proof: &ResponseDeliverTxProof,
        block_hash_index: usize,
    ) -> Result<(), Error> {
        let highest_block_hash = response_deliver_tx_proof.verify()?;
        if let Some(stored_hash) = self.chain_db.get_hash_by_index(block_hash_index) {
            if stored_hash == highest_block_hash {
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

    pub fn add_block_to_highest<S: ToString, E: Ed25519Verifier<S>>(
        &mut self,
        current_highest_header: &Header,
        light_block: LightBlock,
        ed25519_verifier: &mut E,
    ) -> Result<(), Error> {
        validate_light_block(&light_block)?;
        {
            let actual: u64 = light_block
                .signed_header
                .as_ref()
                .unwrap()
                .header
                .as_ref()
                .unwrap()
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
        if hash_header(current_highest_header) != current_highest_hash {
            return Err(Error::InvalidCurrentHighestHeader);
        }
        self.verify_block(
            &current_highest_header.next_validators_hash,
            &light_block,
            ed25519_verifier,
        )?;
        self.append_header(light_block.signed_header.unwrap().header.unwrap())
    }

    fn verify_block<S: ToString, E: Ed25519Verifier<S>>(
        &self,
        validators_hash: &[u8],
        light_block: &LightBlock,
        ed25519_verifier: &mut E,
    ) -> Result<(), Error> {
        if validators_hash
            != light_block
                .signed_header
                .as_ref()
                .unwrap()
                .header
                .as_ref()
                .unwrap()
                .validators_hash
        {
            return Err(Error::UnmatchedValidatorsHash);
        }
        verify_light_block(light_block, ed25519_verifier)?;
        Ok(())
    }

    fn append_header(&mut self, header: Header) -> Result<(), Error> {
        let hash = hash_header(&header);
        self.chain_db
            .append_block_hash(hash)
            .map_err(|e| Error::ChainDB(format!("{}", e)))
    }
}
