use cosmwasm_std::{Api, ReadonlyStorage, StdResult, Storage};
use secret_toolkit::serialization::Bincode2;
use secret_toolkit::storage::Item;
use shared_types::shuriken::{CanonicalConfig, Config};
use shared_types::Canonicalize;

const CONFIG_KEY: &[u8] = b"config";

pub fn read_config<S: ReadonlyStorage, A: Api>(storage: &S, api: &A) -> StdResult<Config> {
    let store = Item::<CanonicalConfig, Bincode2>::new(CONFIG_KEY);
    let canonical: CanonicalConfig = store.load(storage)?;
    Config::from_canonical(canonical, api)
}

pub fn write_config<S: Storage, A: Api>(storage: &mut S, config: Config, api: &A) -> StdResult<()> {
    let store = Item::<CanonicalConfig, Bincode2>::new(CONFIG_KEY);
    store.save(storage, &config.into_canonical(api)?)
}
