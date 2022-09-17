use crate::header::hash_header;
use crate::light_block::validate_light_block;
use crate::light_block::verify_light_block;
use crate::light_block::Ed25519Verifier;
use crate::light_block::Error as LightBlockError;
use crate::response_deliver_tx_proof::{
    Error as ResponseDeliverTxProofError, ResponseDeliverTxProof,
};
use crate::subsequent_hashes::HeaderHashWithHeight;
use crate::subsequent_hashes::{CommittedHashes, Error as SubsequentHashesError, Hashes};
use cosmos_proto::tendermint::types::Header;
use cosmos_proto::tendermint::types::LightBlock;
use std::convert::TryInto;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    AlreadyInitialized,
    LightClientDB(String),
    InvalidHighestHash,
    InvalidCommitFlagsLength,
    ExceedsInterval { max: u64, actual: u64 },
    NoHighestHeaderHash,
    InvalidCurrentHighestHeader,
    UnmatchedValidatorsHash,
    LightBlock(LightBlockError),
    ResponseDeliverTxProof(ResponseDeliverTxProofError),
    SubsequentHashes(SubsequentHashesError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::AlreadyInitialized => f.write_str("already initialized"),
            Error::LightClientDB(e) => write!(f, "chain db error: {}", e),
            Error::InvalidHighestHash => f.write_str("invalid highest hash"),
            Error::InvalidCommitFlagsLength => f.write_str("invalid commit flags length"),
            Error::ExceedsInterval { max, actual } => {
                write!(f, "exceeds interval: max {}, actual {}", max, actual)
            }
            Error::NoHighestHeaderHash => f.write_str("no highest header hash"),
            Error::InvalidCurrentHighestHeader => f.write_str("invalid current highest header"),
            Error::UnmatchedValidatorsHash => f.write_str("unmatched validators hash"),
            Error::LightBlock(e) => write!(f, "light block error: {}", e),
            Error::ResponseDeliverTxProof(e) => write!(f, "tx result proof error: {}", e),
            Error::SubsequentHashes(e) => write!(f, "subsequent hashes error: {}", e),
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

impl From<SubsequentHashesError> for Error {
    fn from(e: SubsequentHashesError) -> Self {
        Self::SubsequentHashes(e)
    }
}

pub trait ReadonlyLightClientDB {
    fn get_hash_by_index(&mut self, index: usize) -> Option<HeaderHashWithHeight>;
    fn get_highest_hash(&mut self) -> Option<HeaderHashWithHeight>;
    fn get_hash_list_length(&mut self) -> usize;
    fn get_max_interval(&mut self) -> u64;
    fn get_commit_secret(&mut self) -> Vec<u8>;
}

pub trait LightClientDB: ReadonlyLightClientDB {
    type Error: std::fmt::Display;
    fn append_block_hash(&mut self, hash: HeaderHashWithHeight) -> Result<(), Self::Error>;
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

    pub fn verify_response_deliver_tx_proof(
        &mut self,
        response_deliver_tx_proof: &ResponseDeliverTxProof,
        block_hash_index: usize,
    ) -> Result<(), Error> {
        let highest_block_hash = response_deliver_tx_proof.verify()?;
        if let Some(stored_hash) = self.db.get_hash_by_index(block_hash_index) {
            if stored_hash.hash == highest_block_hash {
                return Ok(());
            }
        }
        Err(Error::InvalidHighestHash)
    }

    pub fn verify_subsequent_light_blocks<S: ToString, E: Ed25519Verifier<S>>(
        &mut self,
        mut current_highest_header: Header,
        light_blocks: Vec<LightBlock>,
        commit_flags: Vec<bool>,
        ed25519_verifier: &mut E,
    ) -> Result<CommittedHashes, Error> {
        let first_hash = hash_header(&current_highest_header);
        let current_highest_hash = self
            .db
            .get_highest_hash()
            .ok_or(Error::NoHighestHeaderHash)?;
        if first_hash != current_highest_hash.hash {
            return Err(Error::InvalidCurrentHighestHeader);
        }
        if light_blocks.len() != commit_flags.len() {
            return Err(Error::InvalidCommitFlagsLength);
        }
        let mut validators_hash = current_highest_header.next_validators_hash;
        let mut following_hashes = Vec::new();
        let max_interval = self.db.get_max_interval();
        for (i, light_block) in light_blocks.iter().enumerate() {
            self.verify_block(&validators_hash, &light_block, ed25519_verifier)?;
            let header = light_block
                .signed_header
                .as_ref()
                .unwrap()
                .header
                .as_ref()
                .unwrap();
            let actual_interval: u64 = header
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

            if commit_flags[i] {
                following_hashes.push(HeaderHashWithHeight {
                    hash: hash_header(header),
                    height: header.height,
                });
                current_highest_header = header.clone();
            }
            validators_hash = header.next_validators_hash.clone();
        }
        Ok(CommittedHashes::new(
            Hashes {
                first_hash,
                following_hashes,
            },
            &self.db.get_commit_secret(),
        )?)
    }

    fn verify_block<S: ToString, E: Ed25519Verifier<S>>(
        &self,
        validators_hash: &[u8],
        light_block: &LightBlock,
        ed25519_verifier: &mut E,
    ) -> Result<(), Error> {
        validate_light_block(&light_block)?;
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
}

impl<C: LightClientDB> LightClient<C> {
    pub fn init(&mut self, header: Header, max_interval: u64) -> Result<(), Error> {
        if self.db.get_hash_list_length() > 0 {
            return Err(Error::AlreadyInitialized);
        }
        self.db
            .append_block_hash(HeaderHashWithHeight {
                hash: hash_header(&header),
                height: header.height,
            })
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
        if committed_hashes.hashes.first_hash != db_highest_hash.hash {
            return Err(Error::InvalidHighestHash);
        }
        for hash in committed_hashes.hashes.following_hashes {
            self.db
                .append_block_hash(hash)
                .map_err(|e| Error::LightClientDB(e.to_string()))?;
        }
        Ok(())
    }
}
