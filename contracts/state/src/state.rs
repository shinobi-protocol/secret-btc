use cosmwasm_std::{CanonicalAddr, ReadonlyStorage, StdError, StdResult, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use shared_types::state::WriteMsg;

const CONTRACT_OWNER_KEY: &[u8] = b"contract_owner";
const CONTRACT_STATE_KEY: &[u8] = b"contract_state";

struct ContractOwnerStore<S: ReadonlyStorage>(S);

impl<'a, S: ReadonlyStorage> ContractOwnerStore<ReadonlyPrefixedStorage<'a, S>> {
    fn from_readonly(storage: &'a S) -> Self {
        Self(ReadonlyPrefixedStorage::new(CONTRACT_OWNER_KEY, storage))
    }
}

impl<'a, S: Storage> ContractOwnerStore<PrefixedStorage<'a, S>> {
    fn from(storage: &'a mut S) -> Self {
        Self(PrefixedStorage::new(CONTRACT_OWNER_KEY, storage))
    }
}

impl<S: ReadonlyStorage> ContractOwnerStore<S> {
    fn read(&self, contract_label: &[u8]) -> Option<CanonicalAddr> {
        self.0
            .get(contract_label)
            .map(|addr| CanonicalAddr::from(addr.as_slice()))
    }

    fn check_contract_owner(&self, contract_label: &[u8], owner: &CanonicalAddr) -> StdResult<()> {
        let loaded_owner: Option<CanonicalAddr> = self.read(contract_label);
        if let Some(loaded_owner) = loaded_owner {
            if loaded_owner != *owner {
                return Err(StdError::generic_err("wrong owner"));
            }
        }
        Ok(())
    }
}

impl<S: Storage> ContractOwnerStore<S> {
    fn write(&mut self, contract_label: &[u8], owner: &CanonicalAddr) -> () {
        self.0.set(contract_label, owner.as_slice())
    }
}

pub fn set_owner<S: Storage>(
    storage: &mut S,
    contract_label: &[u8],
    current_owner: Option<&CanonicalAddr>,
    next_owner: &CanonicalAddr,
) -> StdResult<()> {
    let mut store = ContractOwnerStore::from(storage);
    if let Some(current_owner) = current_owner {
        store.check_contract_owner(contract_label, current_owner)?;
    }
    store.write(contract_label, next_owner);
    Ok(())
}

pub fn read_owner<S: Storage>(storage: &S, contract_label: &[u8]) -> Option<CanonicalAddr> {
    let store = ContractOwnerStore::from_readonly(storage);
    store.read(contract_label)
}

pub fn write_contract_states<S: Storage>(
    storage: &mut S,
    contract_label: &[u8],
    owner: &CanonicalAddr,
    write_msgs: Vec<WriteMsg>,
) -> StdResult<()> {
    let owner_store = ContractOwnerStore::from_readonly(storage);
    owner_store.check_contract_owner(contract_label, owner)?;
    let mut contract_storage =
        PrefixedStorage::multilevel(&[CONTRACT_STATE_KEY, contract_label], storage);
    for write_msg in write_msgs {
        match write_msg {
            WriteMsg::Set { key, value } => {
                contract_storage.set(key.as_slice(), value.as_slice());
            }
            WriteMsg::Remove { key } => contract_storage.remove(key.as_slice()),
        }
    }
    Ok(())
}

pub fn read_contract_state<S: ReadonlyStorage>(
    storage: &S,
    contract_label: &[u8],
    owner: &CanonicalAddr,
    key: &[u8],
) -> StdResult<Option<Vec<u8>>> {
    let owner_store = ContractOwnerStore::from_readonly(storage);
    owner_store.check_contract_owner(contract_label, owner)?;
    let contract_storage =
        ReadonlyPrefixedStorage::multilevel(&[CONTRACT_STATE_KEY, contract_label], storage);
    Ok(contract_storage.get(&key))
}
