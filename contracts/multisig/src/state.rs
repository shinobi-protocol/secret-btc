use cosmwasm_std::{Api, ReadonlyStorage, StdError, StdResult, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{AppendStore, AppendStoreMut, TypedStore, TypedStoreMut};
use shared_types::multisig::{CanonicalTransactionStatus, Config, TransactionStatus};
use shared_types::Canonicalize;

const CONFIG_KEY: &[u8] = b"config";
const TRANSACTION_STATUS_NAMESPACE: &[u8] = b"transaction_status";

pub fn read_config<S: ReadonlyStorage, A: Api>(storage: &S, api: &A) -> StdResult<Config> {
    let store = TypedStore::attach(storage);
    let canonical = store.load(CONFIG_KEY)?;
    Config::from_canonical(canonical, api)
}

pub fn write_config<S: Storage, A: Api>(storage: &mut S, config: Config, api: &A) -> StdResult<()> {
    let mut store = TypedStoreMut::attach(storage);
    store.store(CONFIG_KEY, &config.into_canonical(api)?)
}

pub fn read_transaction_status<S: ReadonlyStorage, A: Api>(
    storage: &S,
    id: u32,
    api: &A,
) -> StdResult<TransactionStatus> {
    let storage = ReadonlyPrefixedStorage::new(TRANSACTION_STATUS_NAMESPACE, storage);
    let store =
        AppendStore::attach(&storage).ok_or_else(|| StdError::generic_err("no append store"))??;
    let canonical = store.get_at(id)?;
    TransactionStatus::from_canonical(canonical, api)
}

pub fn transaction_count<S: ReadonlyStorage>(storage: &S) -> StdResult<u32> {
    let storage = ReadonlyPrefixedStorage::new(TRANSACTION_STATUS_NAMESPACE, storage);
    let store: Option<
        StdResult<AppendStore<CanonicalTransactionStatus, ReadonlyPrefixedStorage<S>>>,
    > = AppendStore::attach(&storage);
    match store {
        Some(store) => Ok(store?.len()),
        None => Ok(0),
    }
}

pub fn update_transaction_status<S: Storage, A: Api>(
    storage: &mut S,
    id: u32,
    status: TransactionStatus,
    api: &A,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(TRANSACTION_STATUS_NAMESPACE, storage);
    let mut store = AppendStoreMut::attach_or_create(&mut storage)?;
    store.set_at(id, &status.into_canonical(api)?)
}

pub fn append_transaction_status<S: Storage, A: Api>(
    storage: &mut S,
    status: TransactionStatus,
    api: &A,
) -> StdResult<u32> {
    let mut storage = PrefixedStorage::new(TRANSACTION_STATUS_NAMESPACE, storage);
    let mut store = AppendStoreMut::attach_or_create(&mut storage)?;
    store.push(&status.into_canonical(api)?)?;
    Ok(store.len() - 1)
}
