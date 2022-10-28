use super::prefix::PREFIX_CHAIN_DB;
use bitcoin_header_chain::header_chain::chaindb::{ChainDB, ChainDBResult, ReadonlyChainDB};
use bitcoin_header_chain::header_chain::StoredBlockHeader;
use cosmwasm_std::{ReadonlyStorage, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::serialization::Bincode2;
use secret_toolkit::storage::Item;
use std::collections::HashMap;
use std::convert::TryInto;

/// StorageChainDB stores bitcoin chain db on Storage.
/// It has cache to save storage reading gas cost.
#[derive(Debug)]
pub struct StorageChainDB<S: ReadonlyStorage> {
    pub storage: S,
    header_cache: HashMap<u32, StoredBlockHeader>,
    tip_height: Option<u32>,
}

const TIP_HEIGHT_KEY: &[u8] = b"tip_hash";
const PREFIX_HEADERS: &[u8] = b"headers";

impl<S: ReadonlyStorage> StorageChainDB<S> {
    fn new(storage: S) -> Self {
        Self {
            storage,
            header_cache: HashMap::new(),
            tip_height: None,
        }
    }
}

impl<'a, S: ReadonlyStorage> StorageChainDB<ReadonlyPrefixedStorage<'a, S>> {
    pub fn from_readonly_storage(storage: &'a S) -> Self {
        Self::new(ReadonlyPrefixedStorage::new(PREFIX_CHAIN_DB, storage))
    }
}

impl<'a, S: Storage> StorageChainDB<PrefixedStorage<'a, S>> {
    pub fn from_storage(storage: &'a mut S) -> Self {
        Self::new(PrefixedStorage::new(PREFIX_CHAIN_DB, storage))
    }
}

/// Readable ChainDB imples ReadonlyChainDB trait.
/// methods are mutable for caching
impl<S: ReadonlyStorage> ReadonlyChainDB for StorageChainDB<S> {
    fn tip_height(&mut self) -> ChainDBResult<Option<u32>> {
        if self.tip_height.is_none() {
            if let Some(bytes) = self.storage.get(TIP_HEIGHT_KEY) {
                let height = u32::from_be_bytes(bytes.try_into().unwrap());
                self.tip_height = Some(height);
            }
        }
        Ok(self.tip_height)
    }
    fn header_at(&mut self, height: u32) -> ChainDBResult<Option<StoredBlockHeader>> {
        if let Some(cached_header) = self.header_cache.get(&height) {
            return Ok(Some(cached_header.clone()));
        }
        let storage = ReadonlyPrefixedStorage::new(PREFIX_HEADERS, &self.storage);
        if let Some(stored_header) = Item::<StoredBlockHeader, Bincode2>::new(&height.to_be_bytes())
            .may_load(&storage)
            .map_err(|e| e.to_string())?
        {
            self.header_cache.insert(height, stored_header.clone());
            Ok(Some(stored_header))
        } else {
            Ok(None)
        }
    }
}

/// Writable ChainDB impls ChainDB trait.
impl<S: Storage> ChainDB for StorageChainDB<S> {
    fn store_header(&mut self, height: u32, block_header: StoredBlockHeader) -> ChainDBResult<()> {
        let mut storage = PrefixedStorage::new(PREFIX_HEADERS, &mut self.storage);
        Item::<StoredBlockHeader, Bincode2>::new(&height.to_be_bytes())
            .save(&mut storage, &block_header)
            .map_err(|e| e.to_string())?;
        self.header_cache.insert(height, block_header);
        if self.tip_height.unwrap_or_default() <= height {
            self.storage.set(TIP_HEIGHT_KEY, &height.to_be_bytes());
            self.tip_height = Some(height);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bitcoin::blockdata::block::BlockHeader;
    use bitcoin::hash_types::TxMerkleNode;
    use bitcoin::util::uint::Uint256;
    use bitcoin_header_chain::header_chain::BlockHash;
    use cosmwasm_std::testing::*;
    use std::str::FromStr;
    #[test]
    fn test_none_tip_height() {
        let mut storage = MockStorage::new();
        let mut db = StorageChainDB::from_storage(&mut storage);

        // assert no tip
        assert!(db.tip_height().unwrap().is_none());
    }

    #[test]
    fn test_get_tip_height_from_cache() {
        let mut storage = MockStorage::new();
        let mut db = StorageChainDB::from_storage(&mut storage);
        // store hash to cache
        db.tip_height = Some(100);

        // assert tip_hash() return cached data
        assert_eq!(db.tip_height().unwrap().unwrap(), 100);
    }

    #[test]
    fn test_get_tip_height_from_storage() {
        let mut storage = MockStorage::new();
        let mut db = StorageChainDB::from_storage(&mut storage);
        // store hash to storage
        db.storage.set(TIP_HEIGHT_KEY, &100u32.to_be_bytes());

        // assert tip_hash() return storage
        assert_eq!(db.tip_height().unwrap().unwrap(), 100);
        // assert data cached
        assert_eq!(db.tip_height.unwrap(), 100);
    }

    fn block_header() -> StoredBlockHeader {
        StoredBlockHeader {
            header: BlockHeader {
                version: 1,
                prev_blockhash: BlockHash::from_str(
                    "000000007c4fc01a14af8067762fb807144b1b59cd4ec79ffc61efae3439757d",
                )
                .unwrap(),
                merkle_root: TxMerkleNode::from_str(
                    "0e3e2357e806b6cdb1f70b54c3a3a17b6714ee1f0e68bebb44a74b1efd512098",
                )
                .unwrap(),
                time: 1231006505,
                bits: 0x1d00ffff,
                nonce: 2083236893,
            },
            work: Uint256::from_u64(1000).unwrap(),
        }
    }

    #[test]
    fn test_none_block_header() {
        let mut storage = MockStorage::new();
        let mut db = StorageChainDB::from_storage(&mut storage);

        // assert no header
        assert!(db.header_at(0).unwrap().is_none());
    }

    #[test]
    fn test_store_block_header() {
        let mut storage = MockStorage::new();
        let mut db = StorageChainDB::from_storage(&mut storage);

        // store header
        let header = block_header();
        db.store_header(100, header.clone()).unwrap();

        // assert tip height cached
        assert_eq!(db.tip_height.unwrap(), 100);
        // assert tip height stored to storage
        assert_eq!(
            db.storage.get(TIP_HEIGHT_KEY).unwrap(),
            100u32.to_be_bytes()
        );
        // assert header cached
        assert_eq!(db.header_cache.get(&100).unwrap(), &header);

        // store lower height header
        db.store_header(99, header.clone()).unwrap();
        assert_eq!(db.tip_height.unwrap(), 100);
        assert_eq!(
            db.storage.get(TIP_HEIGHT_KEY).unwrap(),
            100u32.to_be_bytes()
        );

        // store higher height header
        db.store_header(101, header.clone()).unwrap();
        // assert tip height cached
        assert_eq!(db.tip_height.unwrap(), 101);
        // assert tip height stored to storage
        assert_eq!(
            db.storage.get(TIP_HEIGHT_KEY).unwrap(),
            101u32.to_be_bytes()
        );

        // assert header stored to storage
        let storage =
            ReadonlyPrefixedStorage::multilevel(&[PREFIX_CHAIN_DB, PREFIX_HEADERS], &storage);
        assert_eq!(
            Item::<StoredBlockHeader, Bincode2>::new(&100u32.to_be_bytes())
                .load(&storage)
                .unwrap(),
            header
        );
        assert_eq!(
            Item::<StoredBlockHeader, Bincode2>::new(&99u32.to_be_bytes())
                .load(&storage)
                .unwrap(),
            header
        );
        assert_eq!(
            Item::<StoredBlockHeader, Bincode2>::new(&101u32.to_be_bytes())
                .load(&storage)
                .unwrap(),
            header
        );
    }

    #[test]
    fn test_update_block_header() {
        let mut mock_storage = MockStorage::new();
        let mut header = block_header();

        // store prev data to storage manually
        let mut storage =
            PrefixedStorage::multilevel(&[PREFIX_CHAIN_DB, PREFIX_HEADERS], &mut mock_storage);
        Item::<StoredBlockHeader, Bincode2>::new(&100u32.to_be_bytes())
            .save(&mut storage, &header)
            .unwrap();

        // store prev data to cache manually
        let mut db = StorageChainDB::from_storage(&mut mock_storage);
        db.header_cache.insert(100, header.clone());

        // update header
        header.work = Uint256::default();
        db.store_header(100, header.clone()).unwrap();

        // assert new data cached
        assert_eq!(db.header_cache.get(&100).unwrap(), &header);
        // assert new data stored to storage
        let storage =
            ReadonlyPrefixedStorage::multilevel(&[PREFIX_CHAIN_DB, PREFIX_HEADERS], &mock_storage);
        assert_eq!(
            Item::<StoredBlockHeader, Bincode2>::new(&100u32.to_be_bytes())
                .load(&storage)
                .unwrap(),
            header
        );
    }

    #[test]
    fn test_get_header_from_cache() {
        let mut storage = MockStorage::new();
        let mut db = StorageChainDB::from_storage(&mut storage);
        let header = block_header();
        // store header to cache
        db.header_cache.insert(100, header.clone());

        // assert get_header() return cached data
        assert_eq!(db.header_at(100).unwrap().unwrap(), header);
    }

    #[test]
    fn test_get_header_from_storage() {
        let mut mock_storage = MockStorage::new();
        let header = block_header();

        // store data to storage manually
        let mut storage =
            PrefixedStorage::multilevel(&[PREFIX_CHAIN_DB, PREFIX_HEADERS], &mut mock_storage);
        Item::<StoredBlockHeader, Bincode2>::new(&100u32.to_be_bytes())
            .save(&mut storage, &header)
            .unwrap();

        let mut db = StorageChainDB::from_storage(&mut mock_storage);

        // assert get_header() return stored data
        assert_eq!(db.header_at(100).unwrap().unwrap(), header);

        // assert data cached
        assert_eq!(db.header_cache.get(&100).unwrap(), &header);
    }
}
