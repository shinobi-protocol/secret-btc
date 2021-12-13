use super::prefix::CONFIG_KEY;
use cosmwasm_std::{Api, ReadonlyStorage, StdResult, Storage};
use secret_toolkit::storage::{TypedStore, TypedStoreMut};
use shared_types::gateway::Config;
use shared_types::Canonicalize;

pub fn read_config<S: ReadonlyStorage, A: Api>(storage: &S, api: &A) -> StdResult<Config> {
    let store = TypedStore::attach(storage);
    let canonicalized = store.load(CONFIG_KEY)?;
    Config::from_canonical(canonicalized, api)
}

pub fn write_config<S: Storage, A: Api>(storage: &mut S, config: Config, api: &A) -> StdResult<()> {
    let mut store = TypedStoreMut::attach(storage);
    store.store(CONFIG_KEY, &config.into_canonical(api)?)
}
