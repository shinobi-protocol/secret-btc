use crate::state_proxy::msg::{
    CanonicalOwner, HandleMsg, Owner, QueryAnswer, QueryMsg, ReadContractStateSignature,
    Secp256k1Verifier, StateTransaction, WriteAction,
};
use crate::Canonicalize;
use cosmwasm_std::{
    to_binary, Api, Binary, CanonicalAddr, Env, Extern, HandleResponse, HandleResult, HumanAddr,
    Querier, QueryResult, ReadonlyStorage, StdError, StdResult, Storage,
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::crypto::sha_256;
use secret_toolkit::serialization::Bincode2;
use secret_toolkit::storage::Item;

const CONTRACTS_OWNER_KEY: &[u8] = b"contracts_owner";
const CONTRACTS_STATE_KEY: &[u8] = b"contracts_state";
const ADMIN_KEY: &[u8] = b"admin";

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    match msg {
        HandleMsg::InitContractState {
            contract_label,
            public_key,
        } => {
            let mut owner_store = ContractsOwnerStore::from(&mut deps.storage);
            if owner_store
                .read(contract_label.as_slice(), &deps.api)?
                .is_some()
            {
                return Err(StdError::generic_err(
                    "contract state is already instantiated",
                ));
            }
            owner_store.write(
                contract_label.as_slice(),
                Owner {
                    address: env.message.sender,
                    public_key,
                },
                &deps.api,
            )?;
            Ok(HandleResponse::default())
        }
        HandleMsg::WriteContractState {
            contract_label,
            transaction,
        } => {
            commit_transaction_to_storage(
                &mut deps.storage,
                contract_label.as_slice(),
                &env.message.sender,
                &transaction,
                &deps.api,
            )?;
            Ok(HandleResponse::default())
        }
        HandleMsg::ChangeOwnerByAdmin {
            contract_label,
            next_owner,
        } => {
            let current_admin = read_admin(&deps.storage);
            if deps.api.canonical_address(&env.message.sender)? != current_admin {
                return Err(StdError::generic_err("message sender is not current admin"));
            }
            let mut owner_store = ContractsOwnerStore::from(&mut deps.storage);
            owner_store.write(contract_label.as_slice(), next_owner, &deps.api)?;
            Ok(HandleResponse {
                data: None,
                log: vec![],
                messages: vec![],
            })
        }
        HandleMsg::ChangeAdmin { next_admin } => {
            let current_admin = read_admin(&deps.storage);
            if deps.api.canonical_address(&env.message.sender)? != current_admin {
                return Err(StdError::generic_err("message sender is not current admin"));
            }
            set_admin(&mut deps.storage, &deps.api.canonical_address(&next_admin)?);
            Ok(HandleResponse {
                data: None,
                log: vec![],
                messages: vec![],
            })
        }
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::Owner { contract_label } => {
            let owner_store = ContractsOwnerStore::from_readonly(&deps.storage);
            let owner = owner_store.read(contract_label.as_slice(), &deps.api)?;
            to_binary(&QueryAnswer::Owner { owner })
        }
        QueryMsg::ReadContractState { signature, key } => {
            let state = read_contract_state(
                &deps.storage,
                signature,
                key.as_slice(),
                &Secp256k1ApiVerifier::new(&deps.api),
                &deps.api,
            )?;
            to_binary(&QueryAnswer::ReadContractState {
                value: state.map(|state| Binary::from(state)),
            })
        }
        QueryMsg::Admin {} => {
            let admin = read_admin(&deps.storage);
            to_binary(&QueryAnswer::Admin {
                admin: deps.api.human_address(&admin)?,
            })
        }
    }
}

pub fn set_admin<S: Storage>(storage: &mut S, admin: &CanonicalAddr) -> () {
    storage.set(ADMIN_KEY, admin.as_slice())
}

pub fn read_admin<S: ReadonlyStorage>(storage: &S) -> CanonicalAddr {
    CanonicalAddr::from(storage.get(ADMIN_KEY).unwrap().as_slice())
}

pub struct ContractsOwnerStore<S: ReadonlyStorage>(S);

impl<'a, S: ReadonlyStorage> ContractsOwnerStore<ReadonlyPrefixedStorage<'a, S>> {
    pub fn from_readonly(storage: &'a S) -> Self {
        Self(ReadonlyPrefixedStorage::new(CONTRACTS_OWNER_KEY, storage))
    }
}

impl<'a, S: Storage> ContractsOwnerStore<PrefixedStorage<'a, S>> {
    pub fn from(storage: &'a mut S) -> Self {
        Self(PrefixedStorage::new(CONTRACTS_OWNER_KEY, storage))
    }
}

impl<S: ReadonlyStorage> ContractsOwnerStore<S> {
    pub fn read<A: Api>(&self, contract_label: &[u8], api: &A) -> StdResult<Option<Owner>> {
        match Item::<CanonicalOwner, Bincode2>::new(contract_label).may_load(&self.0)? {
            Some(canonical) => Ok(Some(Owner::from_canonical(canonical, api)?)),
            None => Ok(None),
        }
    }

    pub fn check_contract_owner_by_address<A: Api>(
        &self,
        contract_label: &[u8],
        owner_address: &HumanAddr,
        api: &A,
    ) -> StdResult<()> {
        let loaded_owner: Option<Owner> = self.read(contract_label, api)?;
        if let Some(loaded_owner) = loaded_owner {
            if loaded_owner.address != *owner_address {
                return Err(StdError::generic_err("wrong owner"));
            }
        }
        Ok(())
    }

    pub fn check_contract_owner_by_pubkey<A: Api>(
        &self,
        contract_label: &[u8],
        owner_pubkey: &[u8],
        api: &A,
    ) -> StdResult<()> {
        let loaded_owner: Option<Owner> = self.read(contract_label, api)?;
        if let Some(loaded_owner) = loaded_owner {
            if loaded_owner.public_key.as_slice() != owner_pubkey {
                return Err(StdError::generic_err("wrong owner"));
            }
        }
        Ok(())
    }
}

impl<S: Storage> ContractsOwnerStore<S> {
    pub fn write<A: Api>(&mut self, contract_label: &[u8], owner: Owner, api: &A) -> StdResult<()> {
        Item::<CanonicalOwner, Bincode2>::new(contract_label)
            .save(&mut self.0, &owner.into_canonical(api)?)?;
        Ok(())
    }
}

pub fn commit_transaction_to_storage<A: Api, S: Storage>(
    storage: &mut S,
    contract_label: &[u8],
    committer: &HumanAddr,
    transaction: &StateTransaction,
    api: &A,
) -> StdResult<()> {
    let owner_store = ContractsOwnerStore::from(storage);
    match owner_store.read(contract_label, api)? {
        Some(owner) => {
            if owner.address != *committer {
                return Err(StdError::generic_err("wrong owner"));
            }
        }
        None => {
            return Err(StdError::generic_err("contract state is not initialized"));
        }
    }
    let mut contract_storage =
        PrefixedStorage::multilevel(&[CONTRACTS_STATE_KEY, contract_label], storage);
    for (key, write_action) in transaction {
        match write_action {
            WriteAction::Set { value } => contract_storage.set(key.as_slice(), value.as_slice()),
            WriteAction::Remove {} => contract_storage.remove(key.as_slice()),
        }
    }
    Ok(())
}

pub fn read_contract_state<A: Api, S: ReadonlyStorage, V: Secp256k1Verifier>(
    storage: &S,
    signature: ReadContractStateSignature,
    key: &[u8],
    verifier: &V,
    api: &A,
) -> StdResult<Option<Vec<u8>>> {
    signature.verify(verifier)?;
    let owner_store = ContractsOwnerStore::from_readonly(storage);
    let loaded_owner: Option<Owner> = owner_store.read(signature.contract_label.as_slice(), api)?;
    if let Some(loaded_owner) = loaded_owner {
        if loaded_owner.public_key != signature.pub_key {
            return Err(StdError::generic_err("wrong owner"));
        }
        let contract_storage = ReadonlyPrefixedStorage::multilevel(
            &[CONTRACTS_STATE_KEY, signature.contract_label.as_slice()],
            storage,
        );
        Ok(contract_storage.get(&key))
    } else {
        Err(StdError::generic_err("no owner authorized"))
    }
}

pub struct Secp256k1ApiVerifier<'a, A: Api> {
    api: &'a A,
}

impl<'a, A: Api> Secp256k1ApiVerifier<'a, A> {
    pub fn new(api: &'a A) -> Self {
        Self { api }
    }
}

impl<'a, A: Api> Secp256k1Verifier for Secp256k1ApiVerifier<'a, A> {
    fn verify(&self, message: &[u8], signature: &[u8], pub_key: &[u8]) -> StdResult<()> {
        if self
            .api
            .secp256k1_verify(&sha_256(message), signature, pub_key)
            .map_err(|e| StdError::generic_err(format!("api secp256k1 verify error: {}", e)))?
        {
            Ok(())
        } else {
            Err(StdError::generic_err(format!(
                "api secp246k1 returned 'false'"
            )))
        }
    }
}
