use cosmwasm_std::{Api, ReadonlyStorage, StdError, StdResult, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::serialization::Bincode2;
use secret_toolkit::storage::{AppendStore, Item};
use shared_types::multisig::{
    CanonicalConfig, CanonicalTransactionStatus, Config, TransactionStatus,
};
use shared_types::Canonicalize;

const CONFIG_KEY: &[u8] = b"config";
const TRANSACTION_STATUS_NAMESPACE: &[u8] = b"transaction_status";

pub fn read_config<S: ReadonlyStorage, A: Api>(storage: &S, api: &A) -> StdResult<Config> {
    let store = Item::<CanonicalConfig, Bincode2>::new(CONFIG_KEY);
    let canonical = store.load(storage)?;
    Config::from_canonical(canonical, api)
}

pub fn write_config<S: Storage, A: Api>(storage: &mut S, config: Config, api: &A) -> StdResult<()> {
    let store = Item::<CanonicalConfig, Bincode2>::new(CONFIG_KEY);
    store.save(storage, &config.into_canonical(api)?)
}

pub fn read_transaction_status<S: ReadonlyStorage, A: Api>(
    storage: &S,
    id: u32,
    api: &A,
) -> StdResult<TransactionStatus> {
    let store = AppendStore::<CanonicalTransactionStatus>::new(TRANSACTION_STATUS_NAMESPACE);
    let canonical = store.get_at(storage, id)?;
    TransactionStatus::from_canonical(canonical, api)
}

pub fn transaction_count<S: ReadonlyStorage>(storage: &S) -> StdResult<u32> {
    let store = AppendStore::<CanonicalTransactionStatus>::new(TRANSACTION_STATUS_NAMESPACE);
    store.get_len(storage)
}

pub fn update_transaction_status<S: Storage, A: Api>(
    storage: &mut S,
    id: u32,
    status: TransactionStatus,
    api: &A,
) -> StdResult<()> {
    let store = AppendStore::<CanonicalTransactionStatus>::new(TRANSACTION_STATUS_NAMESPACE);
    store.set_at(storage, id, &status.into_canonical(api)?)
}

pub fn append_transaction_status<S: Storage, A: Api>(
    storage: &mut S,
    status: TransactionStatus,
    api: &A,
) -> StdResult<u32> {
    let store = AppendStore::<CanonicalTransactionStatus>::new(TRANSACTION_STATUS_NAMESPACE);
    store.push(storage, &status.into_canonical(api)?)?;
    Ok(store.get_len(storage)? - 1)
}
