use cosmwasm_std::{Api, ReadonlyStorage, StdResult, Storage};
use secret_toolkit::storage::TypedStore;
use secret_toolkit::storage::TypedStoreMut;
use shared_types::treasury::Config;
use shared_types::Canonicalize;

const CONFIG_KEY: &[u8] = b"config";

pub fn write_config<S: Storage, A: Api>(storage: &mut S, api: &A, config: Config) -> StdResult<()> {
    let mut store = TypedStoreMut::attach(storage);
    store.store(CONFIG_KEY, &config.into_canonical(api)?)
}

pub fn read_config<S: ReadonlyStorage, A: Api>(storage: &S, api: &A) -> StdResult<Config> {
    let store = TypedStore::attach(storage);
    Config::from_canonical(store.load(CONFIG_KEY)?, api)
}
