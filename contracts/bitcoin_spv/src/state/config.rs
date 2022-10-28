use super::prefix::CONFIG_KEY;
use cosmwasm_std::Api;
use cosmwasm_std::{ReadonlyStorage, StdResult, Storage};
use secret_toolkit::serialization::Bincode2;
use secret_toolkit::storage::Item;
use shared_types::bitcoin_spv::{CanonicalConfig, Config};
use shared_types::Canonicalize;

pub fn read_config<S: ReadonlyStorage, A: Api>(storage: &S, api: &A) -> StdResult<Config> {
    Config::from_canonical(
        Item::<CanonicalConfig, Bincode2>::new(CONFIG_KEY).load(storage)?,
        api,
    )
}

pub fn write_config<S: Storage, A: Api>(storage: &mut S, config: Config, api: &A) -> StdResult<()> {
    Item::<CanonicalConfig, Bincode2>::new(CONFIG_KEY).save(storage, &config.into_canonical(api)?)
}
