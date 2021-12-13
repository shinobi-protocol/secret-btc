pub mod chaindb;
use bitcoin::blockdata::constants::genesis_block;
pub use bitcoin::hash_types::BlockHash;
use bitcoin::hashes::Hash;
use bitcoin::util::uint::Uint256;
use bitcoin::BlockHeader;
use bitcoin::Network;
use chaindb::ChainDB;
use chaindb::ReadonlyChainDB;
use serde_derive::{Deserialize, Serialize};
use std::fmt;

pub const UNTRUSTED_LENGTH: u32 = 5;
pub const MAX_FUTURE_BLOCK_TIME: u32 = 2 * 60 * 60;
pub const DIFFCHANGE_TIMESPAN: u32 = 14 * 24 * 3600;

#[derive(Debug, PartialEq)]
pub enum Error {
    /// bad proof of work
    BadProofOfWork,
    /// unconnected header chain detected
    UnconnectedHeader,
    // the chain db is already initialized to genesis
    AlreadyInitialized,
    /// no chain tip found
    NoTip,
    InvalidTipHeight,
    NoHeaders,
    ReplaceTrustedHeaderNotAllowed,
    NotEnoughWork,
    MedianPastTime,
    MaxFutureTime,
    InvalidTarget,
    MustInitializeWithDiffchangeHeight,
    NoParentHeader,
    ChainDB(chaindb::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::BadProofOfWork => write!(f, "bad proof of work"),
            Error::UnconnectedHeader => write!(f, "unconnected header"),
            Error::AlreadyInitialized => write!(f, "already initialized to genesis"),
            Error::NoTip => write!(f, "no chain tip found"),
            Error::InvalidTipHeight => write!(f, "invalid tip height"),
            Error::NoHeaders => write!(f, "no headers"),
            Error::ReplaceTrustedHeaderNotAllowed => {
                write!(f, "replace trusted header not allowed")
            }
            Error::NotEnoughWork => write!(f, "not enough work"),
            Error::MedianPastTime => write!(
                f,
                "timestamp must be further forwards than the median of the last eleven blocks. "
            ),
            Error::MaxFutureTime => {
                write!(f, "timestamp cannot be more than 2 hours in the future")
            }
            Error::InvalidTarget => write!(f, "invalid target"),
            Error::MustInitializeWithDiffchangeHeight => {
                write!(f, "must initialize with diff change height")
            }
            Error::NoParentHeader => {
                write!(f, "no paretn header")
            }
            Error::ChainDB(string) => {
                write!(f, "chain db error {}", string)
            }
        }
    }
}

impl From<chaindb::Error> for Error {
    fn from(e: chaindb::Error) -> Error {
        Error::ChainDB(e)
    }
}

/// A header enriched with information about its position on the blockchain
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StoredBlockHeader {
    pub header: BlockHeader,
    pub work: Uint256,
}

// validate that work is greater than target written in the block.
fn validate_work(block_header: &BlockHeader) -> Result<(), Error> {
    let target = block_header.target();
    let hash = block_header.block_hash();
    if is_valid_proof_of_work_hash(&target, &hash) {
        Ok(())
    } else {
        Err(Error::BadProofOfWork)
    }
}

pub fn is_valid_proof_of_work_hash(target: &Uint256, hash: &BlockHash) -> bool {
    let value = &reverse_hash(hash);
    value <= target
}

fn reverse_hash(block_hash: &BlockHash) -> Uint256 {
    use byteorder::{ByteOrder, LittleEndian};
    let data: [u8; 32] = block_hash.into_inner();
    let mut ret = [0u64; 4];
    LittleEndian::read_u64_into(&data, &mut ret);
    Uint256(ret)
}

#[derive(Debug)]
pub struct HeaderChain<C: ReadonlyChainDB> {
    db: C,
    network: Network,
}

impl<C: ReadonlyChainDB> HeaderChain<C> {
    pub fn new(db: C, network: Network) -> Self {
        Self { db, network }
    }

    pub fn tip_height(&mut self) -> Result<Option<u32>, Error> {
        Ok(self.db.tip_height()?)
    }

    pub fn tip(&mut self) -> Result<Option<StoredBlockHeader>, Error> {
        if let Some(height) = self.db.tip_height()? {
            Ok(self.db.header_at(height)?)
        } else {
            Ok(None)
        }
    }

    pub fn header_at(&mut self, height: u32) -> Result<Option<StoredBlockHeader>, Error> {
        Ok(self.db.header_at(height)?)
    }
}

impl<H: ChainDB> HeaderChain<H> {
    pub fn init_to_genesis(&mut self) -> Result<(), Error> {
        let genesis = genesis_block(self.network).header;
        self.init_to_header(0, genesis, genesis.time)
    }

    pub fn init_to_header(
        &mut self,
        height: u32,
        header: BlockHeader,
        now: u32,
    ) -> Result<(), Error> {
        if height % 2016 != 0 {
            return Err(Error::MustInitializeWithDiffchangeHeight);
        }
        if self.db.tip_height()?.is_some() {
            return Err(Error::AlreadyInitialized);
        }
        let max_future = max_future(now);
        if header.time > max_future {
            return Err(Error::MaxFutureTime);
        }
        validate_work(&header)?;
        let work = header.work();
        Ok(self
            .db
            .store_header(height, StoredBlockHeader { header, work })?)
    }

    pub fn store_headers(
        &mut self,
        tip_height: u32,
        headers: Vec<BlockHeader>,
        now: u32,
    ) -> Result<Vec<StoredBlockHeader>, Error> {
        let max_future = max_future(now);
        let current_tip_height = self.db.tip_height()?.ok_or(Error::NoTip)?;
        let current_tip = self.header_at(current_tip_height)?.unwrap();
        let headers_len = headers.len() as u32;
        if headers_len == 0 {
            return Err(Error::NoHeaders);
        }
        if tip_height < current_tip_height || tip_height - current_tip_height > headers_len {
            return Err(Error::InvalidTipHeight);
        }
        let replace_length = headers_len - (tip_height - current_tip_height);
        if replace_length > UNTRUSTED_LENGTH {
            return Err(Error::ReplaceTrustedHeaderNotAllowed);
        }
        let start_height = current_tip_height - replace_length + 1;
        let mut prev_header = self
            .header_at(start_height - 1)?
            .expect("Start header must be exist.");
        let stored_headers = {
            let mut stored_headers = Vec::with_capacity(headers_len as usize);
            let mut work = prev_header.work;
            let mut height = start_height;
            for header in headers.into_iter() {
                let stored_block_header = {
                    if let Some(stored_header) = self
                        .header_at(height)?
                        .filter(|stored_header| stored_header.header == header)
                    {
                        work = stored_header.work;
                        stored_header
                    } else {
                        work = work + header.work();
                        StoredBlockHeader { header, work }
                    }
                };
                height += 1;
                stored_headers.push(stored_block_header)
            }
            stored_headers
        };
        if stored_headers.last().unwrap().work < current_tip.work {
            return Err(Error::NotEnoughWork);
        }
        let mut height = start_height;
        for header in stored_headers.clone().into_iter() {
            if prev_header.header.block_hash() != header.header.prev_blockhash {
                return Err(Error::UnconnectedHeader);
            }
            if header.header.target()
                != self.required_target(height - 1, &prev_header.header, &header.header)?
            {
                return Err(Error::InvalidTarget);
            }
            validate_work(&header.header)?;
            if self.db.mpt_at(height)?.unwrap_or_default() > header.header.time {
                return Err(Error::MedianPastTime);
            }
            if header.header.time > max_future {
                return Err(Error::MaxFutureTime);
            }
            self.db.store_header(height, header.clone())?;
            height += 1;
            prev_header = header
        }
        Ok(stored_headers)
    }

    pub fn required_target(
        &mut self,
        prev_height: u32,
        prev_header: &BlockHeader,
        next_header: &BlockHeader,
    ) -> Result<Uint256, Error> {
        Ok(
            if (prev_height + 1) % 2016 == 0 && self.network != Network::Regtest {
                let timespan = {
                    let start = self
                        .header_at(prev_height - 2015)?
                        .ok_or(Error::UnconnectedHeader)?
                        .header
                        .time;
                    let end = prev_header.time;
                    match end - start {
                        n if n < DIFFCHANGE_TIMESPAN / 4 => DIFFCHANGE_TIMESPAN / 4,
                        n if n > DIFFCHANGE_TIMESPAN * 4 => DIFFCHANGE_TIMESPAN * 4,
                        n => n,
                    }
                };
                let mut target = prev_header.target().mul_u32(timespan)
                    / Uint256::from_u64(DIFFCHANGE_TIMESPAN as u64).unwrap();
                let max = max_target();
                if target > max {
                    target = max;
                }
                satoshi_the_precision(target)
            } else if self.network == Network::Testnet
                && next_header.time > prev_header.time + 2 * 600
            {
                // Reset Difficulty
                max_target()
            } else if self.network == Network::Testnet {
                let max = max_target();
                let mut height = prev_height;
                let mut scan = prev_header.clone();
                while height % 2016 != 0 && scan.target() == max {
                    height -= 1;
                    scan = self
                        .header_at(height)?
                        .ok_or(Error::UnconnectedHeader)?
                        .header;
                }
                scan.target()
            } else {
                prev_header.target()
            },
        )
    }
}

/// taken from an early rust-bitcoin by Andrew Poelstra:
/// This function emulates the `GetCompact(SetCompact(n))` in the Satoshi code,
/// which drops the precision to something that can be encoded precisely in
/// the nBits block header field. Savour the perversity. This is in Bitcoin
/// consensus code. What. Gaah!
fn satoshi_the_precision(n: Uint256) -> Uint256 {
    use bitcoin::util::BitArray;

    // Shift by B bits right then left to turn the low bits to zero
    let bits = 8 * ((n.bits() + 7) / 8 - 3);
    let mut ret = n >> bits;
    // Oh, did I say B was that fucked up formula? I meant sometimes also + 8.
    if ret.bit(23) {
        ret = (ret >> 8) << 8;
    }
    ret << bits
}

fn max_target() -> Uint256 {
    Uint256::from_u64(0xFFFF).unwrap() << 208
}
fn max_future(now: u32) -> u32 {
    now.checked_add(MAX_FUTURE_BLOCK_TIME).unwrap()
}

#[cfg(test)]
mod test;
