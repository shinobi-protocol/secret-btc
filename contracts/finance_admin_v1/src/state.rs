use crate::config::Config;
use cosmwasm_std::{Api, ReadonlyStorage, StdResult, Storage, Uint128};
use secret_toolkit::storage::{TypedStore, TypedStoreMut};
use shared_types::Canonicalize;
use std::convert::TryInto;

const CONFIG_KEY: &[u8] = b"config";
const TOTAL_MINTED_KEY: &[u8] = b"total_minted";

pub fn write_config<S: Storage, A: Api>(storage: &mut S, api: &A, config: Config) -> StdResult<()> {
    let mut store = TypedStoreMut::attach(storage);
    store.store(CONFIG_KEY, &config.into_canonical(api)?)
}

pub fn read_config<S: ReadonlyStorage, A: Api>(storage: &S, api: &A) -> StdResult<Config> {
    let store = TypedStore::attach(storage);
    let canonicalized = store.load(CONFIG_KEY)?;
    Config::from_canonical(canonicalized, api)
}

pub fn write_total_minted<S: Storage>(storage: &mut S, total_minted: Uint128) {
    storage.set(TOTAL_MINTED_KEY, &total_minted.0.to_be_bytes())
}

pub fn read_total_minted<S: ReadonlyStorage>(storage: &S) -> Uint128 {
    match storage.get(TOTAL_MINTED_KEY) {
        Some(bytes) => u128::from_be_bytes(bytes.try_into().unwrap()).into(),
        None => Uint128::zero(),
    }
}
