use cosmwasm_std::StdError;
use cosmwasm_std::{ReadonlyStorage, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use sfps_lib::light_client::{LightClientDB, ReadonlyLightClientDB};
use sfps_lib::subsequent_hashes::HeaderHashWithHeight;
use std::collections::HashMap;
use std::convert::TryInto;

pub const PREFIX_CHAIN_DB: &[u8] = b"light_client_db";
pub const PREFIX_PRNG: &[u8] = b"prng";
pub const PREFIX_BLOCK_HASH: &[u8] = b"block_hash";
pub const MAX_INTERVAL_KEY: &[u8] = b"max_interval";
pub const COMMIT_SECRET_KEY: &[u8] = b"commit_secret";

pub struct StorageLightClientDB<S: ReadonlyStorage> {
    storage: S,
    hash_by_index_cache: HashMap<usize, HeaderHashWithHeight>,
    highest_hash_cache: Option<HeaderHashWithHeight>,
    hash_list_length_cache: Option<usize>,
    max_interval_cache: Option<u64>,
}

impl<S: ReadonlyStorage> StorageLightClientDB<S> {
    fn new(storage: S) -> Self {
        Self {
            storage,
            hash_by_index_cache: HashMap::new(),
            highest_hash_cache: None,
            hash_list_length_cache: None,
            max_interval_cache: None,
        }
    }
}

impl<'a, S: ReadonlyStorage> StorageLightClientDB<ReadonlyPrefixedStorage<'a, S>> {
    pub fn from_readonly_storage(storage: &'a S) -> Self {
        Self::new(ReadonlyPrefixedStorage::new(PREFIX_CHAIN_DB, storage))
    }
}

impl<'a, S: Storage> StorageLightClientDB<PrefixedStorage<'a, S>> {
    pub fn from_storage(storage: &'a mut S) -> Self {
        Self::new(PrefixedStorage::new(PREFIX_CHAIN_DB, storage))
    }
}

impl<S: ReadonlyStorage> ReadonlyLightClientDB for StorageLightClientDB<S> {
    fn get_hash_by_index(&mut self, index: usize) -> Option<HeaderHashWithHeight> {
        match self.hash_by_index_cache.get(&index) {
            Some(hash) => Some(hash.clone()),
            None => {
                let storage = ReadonlyPrefixedStorage::new(PREFIX_BLOCK_HASH, &self.storage);
                let storage = AppendStore::attach(&storage)?.ok()?;
                let hash: HeaderHashWithHeight = storage.get_at(index as u32).ok()?;
                self.hash_by_index_cache.insert(index, hash.clone());
                Some(hash)
            }
        }
    }
    fn get_highest_hash(&mut self) -> Option<HeaderHashWithHeight> {
        match &self.highest_hash_cache {
            Some(hash) => Some(hash.clone()),
            None => {
                let storage = ReadonlyPrefixedStorage::new(PREFIX_BLOCK_HASH, &self.storage);
                let storage = AppendStore::attach(&storage)?.ok()?;
                let hash: Option<HeaderHashWithHeight> = storage.get_at(storage.len() - 1).ok();
                self.highest_hash_cache = hash.clone();
                hash
            }
        }
    }
    fn get_hash_list_length(&mut self) -> usize {
        match self.hash_list_length_cache {
            Some(length) => length,
            None => {
                let storage = ReadonlyPrefixedStorage::new(PREFIX_BLOCK_HASH, &self.storage);
                if let Some(result) =
                    AppendStore::<HeaderHashWithHeight, ReadonlyPrefixedStorage<S>>::attach(
                        &storage,
                    )
                {
                    if let Ok(storage) = result {
                        let length = storage.len() as usize;
                        self.hash_list_length_cache = Some(length);
                        return length;
                    }
                }
                0
            }
        }
    }
    fn get_max_interval(&mut self) -> u64 {
        match self.max_interval_cache {
            Some(max_interval) => max_interval,
            None => {
                let max_interval = self
                    .storage
                    .get(MAX_INTERVAL_KEY)
                    .map(|bytes| u64::from_ne_bytes(bytes.try_into().unwrap()))
                    .unwrap();
                self.max_interval_cache = Some(max_interval);
                max_interval
            }
        }
    }
    fn get_commit_secret(&mut self) -> Vec<u8> {
        self.storage.get(COMMIT_SECRET_KEY).unwrap_or_default()
    }
}

impl<S: Storage> LightClientDB for StorageLightClientDB<S> {
    type Error = StdError;
    fn append_block_hash(&mut self, block_hash: HeaderHashWithHeight) -> Result<(), Self::Error> {
        let index = self.get_hash_list_length();
        self.hash_by_index_cache.insert(index, block_hash.clone());
        self.hash_list_length_cache = Some(index + 1);
        self.highest_hash_cache = Some(block_hash.clone());
        let mut storage = PrefixedStorage::new(PREFIX_BLOCK_HASH, &mut self.storage);
        let mut storage = AppendStoreMut::attach_or_create(&mut storage)?;
        Ok(storage.push(&block_hash)?)
    }
    fn store_max_interval(&mut self, max_interval: u64) -> Result<(), Self::Error> {
        self.max_interval_cache = Some(max_interval);
        self.storage
            .set(MAX_INTERVAL_KEY, &max_interval.to_ne_bytes());
        Ok(())
    }
    fn store_commit_secret(&mut self, secret: &[u8]) -> Result<(), Self::Error> {
        self.storage.set(COMMIT_SECRET_KEY, secret);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::*;

    #[test]
    fn test_store_block_hash() {
        let mut storage = MockStorage::new();
        let mut db = StorageLightClientDB::from_storage(&mut storage);
        let hash = hex::decode("B27B2FEEA5EB3D67C2BB21B5038E145F5706A96636D367C5119A6E2E73764455")
            .unwrap();
        let header_hash_with_height = HeaderHashWithHeight {
            hash: hash.clone(),
            height: 1,
        };
        assert!(db.get_hash_by_index(0).is_none());
        assert_eq!(db.get_hash_list_length(), 0);
        db.append_block_hash(header_hash_with_height.clone())
            .unwrap();
        assert_eq!(db.get_hash_by_index(0).unwrap(), header_hash_with_height);
        assert_eq!(db.get_hash_list_length(), 1);
    }
}
