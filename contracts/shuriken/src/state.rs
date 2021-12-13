use cosmwasm_std::{Api, ReadonlyStorage, StdResult, Storage};
use secret_toolkit::storage::{TypedStore, TypedStoreMut};
use shared_types::shuriken::{CanonicalConfig, Config};
use shared_types::Canonicalize;

const CONFIG_KEY: &[u8] = b"config";

pub fn read_config<S: ReadonlyStorage, A: Api>(storage: &S, api: &A) -> StdResult<Config> {
    let store = TypedStore::attach(storage);
    let canonical: CanonicalConfig = store.load(CONFIG_KEY)?;
    Config::from_canonical(canonical, api)
}

pub fn write_config<S: Storage, A: Api>(storage: &mut S, config: Config, api: &A) -> StdResult<()> {
    let mut store = TypedStoreMut::attach(storage);
    store.store(CONFIG_KEY, &config.into_canonical(api)?)
}
