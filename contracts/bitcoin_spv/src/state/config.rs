use super::prefix::CONFIG_KEY;
use cosmwasm_std::{ReadonlyStorage, StdResult, Storage};
use secret_toolkit::storage::{TypedStore, TypedStoreMut};
use shared_types::bitcoin_spv::Config;

pub fn read_config<S: ReadonlyStorage>(storage: &S) -> StdResult<Config> {
    let store = TypedStore::attach(storage);
    store.load(CONFIG_KEY)
}

pub fn write_config<S: Storage>(storage: &mut S, config: &Config) -> StdResult<()> {
    let mut store = TypedStoreMut::attach(storage);
    store.store(CONFIG_KEY, config)
}
