use crate::state::prefix::CONTRACT_STATUS_KEY;
use cosmwasm_std::{ReadonlyStorage, StdResult, Storage};
use secret_toolkit::storage::{TypedStore, TypedStoreMut};
use shared_types::gateway::SuspensionSwitch;

pub fn suspension_switch<S: ReadonlyStorage>(storage: &S) -> StdResult<SuspensionSwitch> {
    let store = TypedStore::attach(storage);
    store
        .may_load(CONTRACT_STATUS_KEY)
        .map(|suspension_switch| suspension_switch.unwrap_or_default())
}

pub fn set_suspension_switch<S: Storage>(
    storage: &mut S,
    status: &SuspensionSwitch,
) -> StdResult<()> {
    let mut store = TypedStoreMut::attach(storage);
    store.store(CONTRACT_STATUS_KEY, status)
}
