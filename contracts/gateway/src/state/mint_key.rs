use super::prefix::PREFIX_MINT_KEY;
use crate::error::Error;
use bitcoin::secp256k1::SecretKey;
use bitcoin::{Network, PrivateKey};
use cosmwasm_std::{CanonicalAddr, ReadonlyStorage, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};

pub fn write_mint_key<S: Storage>(store: &mut S, mintor: &CanonicalAddr, key: &PrivateKey) {
    let mut store = PrefixedStorage::new(PREFIX_MINT_KEY, store);
    store.set(mintor.as_slice(), &key.key.serialize())
}

pub fn read_mint_key<S: ReadonlyStorage>(
    store: &S,
    mintor: &CanonicalAddr,
    network: Network,
) -> Result<Option<PrivateKey>, Error> {
    let store = ReadonlyPrefixedStorage::new(PREFIX_MINT_KEY, store);
    match store.get(mintor.as_slice()) {
        Some(bytes) => {
            let mint_key = PrivateKey {
                key: SecretKey::parse_slice(&bytes)?,
                network,
                compressed: true,
            };
            Ok(Some(mint_key))
        }
        None => Ok(None),
    }
}

pub fn remove_mint_key<S: Storage>(store: &mut S, mintor: &CanonicalAddr) {
    let mut store = PrefixedStorage::new(PREFIX_MINT_KEY, store);
    store.remove(mintor.as_slice())
}
