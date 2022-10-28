use super::prefix::CONFIG_KEY;
use cosmwasm_std::{Api, ReadonlyStorage, StdResult, Storage};
use secret_toolkit::serialization::Bincode2;
use secret_toolkit::storage::Item;
use shared_types::gateway::{CanonicalConfig, Config};
use shared_types::Canonicalize;

pub fn read_config<S: ReadonlyStorage, A: Api>(storage: &S, api: &A) -> StdResult<Config> {
    let canonicalized = Item::<CanonicalConfig, Bincode2>::new(CONFIG_KEY).load(storage)?;
    Config::from_canonical(canonicalized, api)
}

pub fn write_config<S: Storage, A: Api>(storage: &mut S, config: Config, api: &A) -> StdResult<()> {
    let canonicalized = &config.into_canonical(api)?;
    Item::<CanonicalConfig, Bincode2>::new(CONFIG_KEY).save(storage, &canonicalized)
}
