use cosmwasm_std::{Binary, CanonicalAddr, ReadonlyStorage, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use rand::Rng;
use schemars::JsonSchema;
use secret_toolkit::crypto::sha_256;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fmt;

pub const VIEWING_KEY_SIZE: usize = 32;
const VIEWING_KEY_PREFIX: &str = "api_key_";

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ViewingKeyHash(pub [u8; VIEWING_KEY_SIZE]);

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ViewingKey(pub String);

impl ViewingKey {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let rand_bytes: [u8; 32] = rng.gen();
        let key = sha_256(&rand_bytes);
        Self(VIEWING_KEY_PREFIX.to_string() + &Binary::from(&key).to_base64())
    }
    pub fn hash(&self) -> ViewingKeyHash {
        ViewingKeyHash(sha_256(self.as_bytes()))
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl PartialEq for ViewingKey {
    fn eq(&self, other: &Self) -> bool {
        self.hash() == other.hash()
    }
}

impl Eq for ViewingKey {}

impl fmt::Display for ViewingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct ViewingKeyHashStore<S> {
    storage: S,
}

impl<'a, S: Storage> ViewingKeyHashStore<PrefixedStorage<'a, S>> {
    pub fn from_storage(storage: &'a mut S, prefix: &[u8]) -> Self {
        Self {
            storage: PrefixedStorage::new(prefix, storage),
        }
    }
}

impl<'a, S: ReadonlyStorage> ViewingKeyHashStore<ReadonlyPrefixedStorage<'a, S>> {
    pub fn from_readonly_storage(storage: &'a S, prefix: &[u8]) -> Self {
        Self {
            storage: ReadonlyPrefixedStorage::new(prefix, storage),
        }
    }
}

impl<S: ReadonlyStorage> ViewingKeyHashStore<S> {
    pub fn read(&self, address: &CanonicalAddr) -> Option<ViewingKeyHash> {
        self.storage
            .get(address.as_slice())
            .map(|bytes| ViewingKeyHash(bytes.try_into().unwrap()))
    }
}

impl<S: Storage> ViewingKeyHashStore<S> {
    pub fn write(&mut self, address: &CanonicalAddr, key_hash: &ViewingKeyHash) {
        self.storage.set(address.as_slice(), &key_hash.0)
    }
}
