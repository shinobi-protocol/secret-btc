use cosmwasm_std::{ReadonlyStorage, Storage};
use secret_toolkit::serialization::Bincode2;
use std::collections::HashMap;
use std::string::ToString;

#[derive(Debug)]
pub struct NamedMockStorage {
    pub name: String,
    pub data: HashMap<Vec<u8>, Vec<u8>>,
}

impl NamedMockStorage {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            data: HashMap::new(),
        }
    }
}

impl Storage for NamedMockStorage {
    fn set(&mut self, key: &[u8], value: &[u8]) {
        println!(
            "set to [{}] KEY:{:?}, VALUE:{:?}",
            self.name,
            String::from_utf8(key.to_vec()).unwrap_or(hex::encode(key)),
            value
        );
        self.data.insert(key.to_vec(), value.to_vec());
    }
    fn remove(&mut self, key: &[u8]) {
        println!("remove from [{}] KEY:{:?}", self.name, key);
        self.data.remove(key);
    }
}

impl ReadonlyStorage for NamedMockStorage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let value = self.data.get(key).map(|value| value.clone());
        println!(
            "get from [{}] KEY:{:?}, VALUE:{:?}",
            self.name,
            String::from_utf8(key.to_vec()).unwrap_or(hex::encode(key)),
            value
        );
        value
    }
}

pub struct MutableStorageWrapper<'a> {
    pub storage: &'a mut NamedMockStorage,
}

pub struct ReadonlyStorageWrapper<'a> {
    pub storage: &'a NamedMockStorage,
}

impl<'a> Storage for MutableStorageWrapper<'a> {
    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.storage.set(key, value);
    }
    fn remove(&mut self, key: &[u8]) {
        self.storage.remove(key);
    }
}

impl<'a> ReadonlyStorage for MutableStorageWrapper<'a> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.storage.get(key)
    }
}

impl<'a> ReadonlyStorage for ReadonlyStorageWrapper<'a> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.storage.get(key)
    }
}
