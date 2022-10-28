use crate::state::prefix::CONTRACT_STATUS_KEY;
use cosmwasm_std::{ReadonlyStorage, StdResult, Storage};
use secret_toolkit::serialization::Bincode2;
use secret_toolkit::storage::Item;
use shared_types::gateway::SuspensionSwitch;

pub fn suspension_switch<S: ReadonlyStorage>(storage: &S) -> StdResult<SuspensionSwitch> {
    Item::<SuspensionSwitch, Bincode2>::new(CONTRACT_STATUS_KEY)
        .may_load(storage)
        .map(|suspension_switch| suspension_switch.unwrap_or_default())
}

pub fn set_suspension_switch<S: Storage>(
    storage: &mut S,
    status: &SuspensionSwitch,
) -> StdResult<()> {
    Item::<SuspensionSwitch, Bincode2>::new(CONTRACT_STATUS_KEY).save(storage, status)
}
