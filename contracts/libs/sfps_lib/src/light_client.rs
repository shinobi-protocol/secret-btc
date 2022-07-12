use crate::light_block::header::Header;
use crate::light_block::Ed25519Verifier;
use crate::light_block::{Error as LightBlockError, LightBlock};
use crate::subsequent_hashes::{CommittedHashes, Error as SubsequentHashesError, Hashes};
use crate::tx_result_proof::{Error as TxResultProofError, TxResultProof};
use std::convert::TryInto;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    AlreadyInitialized,
    LightClientDB(String),
    InvalidHighestHash,
    ExceedsInterval { max: u64, actual: u64 },
    NoHighestHeaderHash,
    InvalidCurrentHighestHeader,
    UnmatchedValidatorsHash,
    LightBlock(LightBlockError),
    TxResultProof(TxResultProofError),
    SubsequentHashes(SubsequentHashesError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::AlreadyInitialized => f.write_str("already initialized"),
            Error::LightClientDB(e) => write!(f, "chain db error: {}", e),
            Error::InvalidHighestHash => f.write_str("invalid highest hash"),
            Error::ExceedsInterval { max, actual } => {
                write!(f, "exceeds interval: max {}, actual {}", max, actual)
            }
            Error::NoHighestHeaderHash => f.write_str("no highest header hash"),
            Error::InvalidCurrentHighestHeader => f.write_str("invalid current highest header"),
            Error::UnmatchedValidatorsHash => f.write_str("unmatched validators hash"),
            Error::LightBlock(e) => write!(f, "light block error: {}", e),
            Error::TxResultProof(e) => write!(f, "tx result proof error: {}", e),
            Error::SubsequentHashes(e) => write!(f, "subsequent hashes error: {}", e),
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

impl From<SubsequentHashesError> for Error {
    fn from(e: SubsequentHashesError) -> Self {
        Self::SubsequentHashes(e)
    }
}

pub trait ReadonlyLightClientDB {
    fn get_hash_by_index(&mut self, index: usize) -> Option<Vec<u8>>;
    fn get_highest_hash(&mut self) -> Option<Vec<u8>>;
    fn get_hash_list_length(&mut self) -> usize;
    fn get_max_interval(&mut self) -> u64;
    fn get_commit_secret(&mut self) -> Vec<u8>;
}

pub trait LightClientDB: ReadonlyLightClientDB {
    type Error: std::fmt::Display;
    fn append_header_hash(&mut self, hash: Vec<u8>) -> Result<(), Self::Error>;
    fn store_max_interval(&mut self, max_interval: u64) -> Result<(), Self::Error>;
    fn store_commit_secret(&mut self, secret: &[u8]) -> Result<(), Self::Error>;
}

pub struct LightClient<C: ReadonlyLightClientDB> {
    db: C,
}

impl<C: ReadonlyLightClientDB> LightClient<C> {
    pub fn new(db: C) -> Self {
        Self { db }
    }

    pub fn verify_tx_result_proof(
        &mut self,
        tx_result_proof: &TxResultProof,
        header_hash_index: usize,
    ) -> Result<(), Error> {
        let highest_header_hash = tx_result_proof.verify()?;
        if let Some(stored_hash) = self.db.get_hash_by_index(header_hash_index) {
            if stored_hash == highest_header_hash {
                return Ok(());
            }
        }
        Err(Error::InvalidHighestHash)
    }

    pub fn verify_subsequent_light_blocks<E: Ed25519Verifier>(
        &mut self,
        mut current_highest_header: Header,
        light_blocks: Vec<LightBlock>,
        ed25519_verifier: &mut E,
    ) -> Result<CommittedHashes, Error> {
        let first_hash = current_highest_header.hash();
        let mut following_hashes = Vec::new();
        let current_highest_hash = self
            .db
            .get_highest_hash()
            .ok_or(Error::NoHighestHeaderHash)?;
        if first_hash != current_highest_hash {
            return Err(Error::InvalidCurrentHighestHeader);
        }
        let max_interval = self.db.get_max_interval();
        for light_block in light_blocks {
            let actual_interval: u64 = light_block
                .signed_header
                .header
                .height
                .checked_sub(current_highest_header.height)
                .unwrap()
                .try_into()
                .unwrap();
            if actual_interval > max_interval {
                return Err(Error::ExceedsInterval {
                    max: max_interval,
                    actual: actual_interval,
                });
            }
            self.verify_block(
                &current_highest_header.next_validators_hash,
                &light_block,
                ed25519_verifier,
            )?;
            following_hashes.push(light_block.signed_header.header.hash());
            current_highest_header = light_block.signed_header.header.clone();
        }
        Ok(CommittedHashes::new(
            Hashes {
                first_hash,
                following_hashes,
            },
            &self.db.get_commit_secret(),
        )?)
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
}

impl<C: LightClientDB> LightClient<C> {
    pub fn init(&mut self, header: Header, max_interval: u64) -> Result<(), Error> {
        if self.db.get_hash_list_length() > 0 {
            return Err(Error::AlreadyInitialized);
        }
        let hash = header.hash();
        self.db
            .append_header_hash(hash)
            .map_err(|e| Error::LightClientDB(e.to_string()))?;
        self.db
            .store_max_interval(max_interval)
            .map_err(|e| Error::LightClientDB(e.to_string()))
    }

    pub fn append_subsequent_hashes(
        &mut self,
        committed_hashes: CommittedHashes,
    ) -> Result<(), Error> {
        committed_hashes.verify(&self.db.get_commit_secret())?;
        let db_highest_hash = self
            .db
            .get_highest_hash()
            .ok_or(Error::NoHighestHeaderHash)?;
        if committed_hashes.hashes.first_hash != db_highest_hash {
            return Err(Error::InvalidHighestHash);
        }
        for hash in committed_hashes.hashes.following_hashes {
            self.db
                .append_header_hash(hash)
                .map_err(|e| Error::LightClientDB(e.to_string()))?;
        }
        Ok(())
    }
}
