use super::PREFIX_CHAIN_DB;
use cosmwasm_std::StdError;
use cosmwasm_std::{ReadonlyStorage, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use sfps_lib::header_chain::{ChainDB, ReadonlyChainDB};
use std::convert::TryInto;

pub struct StorageChainDB<S: ReadonlyStorage> {
    storage: S,
}

impl<S: ReadonlyStorage> StorageChainDB<S> {
    fn new(storage: S) -> Self {
        Self { storage }
    }
}

const PREFIX_HEADER_HASH: &[u8] = b"header_hash";
const MAX_INTERVAL_KEY: &[u8] = b"max_interval_key";

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

impl<S: ReadonlyStorage> ReadonlyChainDB for StorageChainDB<S> {
    fn get_hash_by_index(&self, index: usize) -> Option<Vec<u8>> {
        let storage = ReadonlyPrefixedStorage::new(PREFIX_HEADER_HASH, &self.storage);
        let storage = AppendStore::attach(&storage)?.ok()?;
        storage.get_at(index as u32).ok()
    }
    fn get_highest_hash(&self) -> Option<Vec<u8>> {
        let storage = ReadonlyPrefixedStorage::new(PREFIX_HEADER_HASH, &self.storage);
        let storage = AppendStore::attach(&storage)?.ok()?;
        storage.iter().last()?.ok()
    }
    fn get_hash_list_length(&self) -> usize {
        let storage = ReadonlyPrefixedStorage::new(PREFIX_HEADER_HASH, &self.storage);
        if let Some(result) = AppendStore::<Vec<u8>, ReadonlyPrefixedStorage<S>>::attach(&storage) {
            if let Ok(storage) = result {
                return storage.len() as usize;
            }
        }
        0
    }
    fn get_max_interval(&self) -> u64 {
        self.storage
            .get(MAX_INTERVAL_KEY)
            .map(|bytes| u64::from_ne_bytes(bytes.try_into().unwrap()))
            .unwrap()
    }
}

impl<S: Storage> ChainDB for StorageChainDB<S> {
    type Error = StdError;
    fn append_header_hash(&mut self, header_hash: Vec<u8>) -> Result<(), Self::Error> {
        let mut storage = PrefixedStorage::new(PREFIX_HEADER_HASH, &mut self.storage);
        let mut storage = AppendStoreMut::attach_or_create(&mut storage)?;
        Ok(storage.push(&header_hash)?)
    }
    fn store_max_interval(&mut self, max_interval: u64) -> Result<(), Self::Error> {
        self.storage
            .set(MAX_INTERVAL_KEY, &max_interval.to_ne_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::*;

    #[test]
    fn test_store_header_hash() {
        let mut storage = MockStorage::new();
        let mut db = StorageChainDB::from_storage(&mut storage);
        let hash = hex::decode("B27B2FEEA5EB3D67C2BB21B5038E145F5706A96636D367C5119A6E2E73764455")
            .unwrap();
        assert!(db.get_hash_by_index(0).is_none());
        assert_eq!(db.get_hash_list_length(), 0);
        db.append_header_hash(hash.clone()).unwrap();
        assert_eq!(db.get_hash_by_index(0).unwrap(), hash);
        assert_eq!(db.get_hash_list_length(), 1);
    }
}
