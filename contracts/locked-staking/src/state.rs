use cosmwasm_std::{
    Api, CanonicalAddr, CosmosMsg, HumanAddr, ReadonlyStorage, StdError, StdResult, Storage,
    Uint128,
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use secret_toolkit::{
    snip20::send_msg,
    storage::{AppendStore, AppendStoreMut, TypedStore, TypedStoreMut},
};
use serde::{Deserialize, Serialize};
use shared_types::{CanonicalContractReference, Canonicalize, ContractReference};

const PREFIX_STAKING_INFO: &[u8] = b"staking_info";
const PREFIX_RECEIVERS_ID: &[u8] = b"recipients_id";
const PREFIX_STAKING_SUMMARY: &[u8] = b"staking_summary";
const ADMIN_KEY: &[u8] = b"admin";

pub fn write_admin<S: Storage>(storage: &mut S, admin: &CanonicalAddr) {
    storage.set(ADMIN_KEY, admin.as_slice())
}

pub fn read_admin<S: ReadonlyStorage>(storage: &S) -> StdResult<CanonicalAddr> {
    match storage.get(ADMIN_KEY) {
        Some(bin) => Ok(CanonicalAddr::from(bin)),
        None => Err(StdError::generic_err("no admin stored")),
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq, Eq, Clone)]
pub struct StakingInfo {
    pub id: u32,
    pub token: ContractReference,
    pub locker: HumanAddr,
    pub recipient: HumanAddr,
    pub start_time: u64,
    pub locked_amount: Uint128,
    pub unlocked: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CanonicalStakingInfo {
    id: u32,
    token: CanonicalContractReference,
    locker: CanonicalAddr,
    recipient: CanonicalAddr,
    start_time: u64,
    locked_amount: Uint128,
    unlocked: bool,
}

impl Canonicalize for StakingInfo {
    type Canonicalized = CanonicalStakingInfo;
    fn into_canonical<A>(self, api: &A) -> StdResult<Self::Canonicalized>
    where
        A: Api,
    {
        Ok(CanonicalStakingInfo {
            id: self.id,
            token: self.token.into_canonical(api)?,
            locker: api.canonical_address(&self.locker)?,
            recipient: api.canonical_address(&self.recipient)?,
            start_time: self.start_time,
            locked_amount: self.locked_amount,
            unlocked: self.unlocked,
        })
    }

    fn from_canonical<A>(canonicalized: Self::Canonicalized, api: &A) -> StdResult<Self>
    where
        A: Api,
    {
        Ok(Self {
            id: canonicalized.id,
            token: ContractReference::from_canonical(canonicalized.token, api)?,
            locker: api.human_address(&canonicalized.locker)?,
            recipient: api.human_address(&canonicalized.recipient)?,
            start_time: canonicalized.start_time,
            locked_amount: canonicalized.locked_amount,
            unlocked: canonicalized.unlocked,
        })
    }
}

#[derive(JsonSchema, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct StakingSummary {
    pub total_locked: Uint128,
    pub staking_end_time: Option<u64>,
    pub total_claimed: Uint128,
    pub total_remaining: Uint128,
}

pub fn get_latest_staking_info_id<S: ReadonlyStorage, A: Api>(
    storage: &S,
    api: &A,
) -> StdResult<u32> {
    let store = StakingInfoListStore::from_readonly_storage(storage, api);
    store.get_latest_id()
}

pub fn get_staking_infos<S: ReadonlyStorage, A: Api>(
    storage: &S,
    api: &A,
    ids: Vec<u32>,
) -> StdResult<Vec<StakingInfo>> {
    let store = StakingInfoListStore::from_readonly_storage(storage, api);
    store.get_staking_infos(&ids)
}

pub fn get_recipients_staking_infos<S: ReadonlyStorage, A: Api>(
    storage: &S,
    api: &A,
    recipient: &HumanAddr,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<StakingInfo>> {
    let recipients_ids =
        RecipientsIDStore::from_readonly_storage(storage, &api.canonical_address(recipient)?)
            .get_ids(page, page_size)?;
    let store = StakingInfoListStore::from_readonly_storage(storage, api);
    store.get_staking_infos(&recipients_ids)
}

pub fn lock<S: Storage, A: Api>(
    storage: &mut S,
    api: &A,
    token: ContractReference,
    locker: HumanAddr,
    recipient: HumanAddr,
    locked_amount: Uint128,
    start_time: u64,
) -> StdResult<StakingInfo> {
    let mut store = StakingInfoListStore::from_storage(storage, api);
    let info = store.push(token, locker, recipient.clone(), locked_amount, start_time)?;
    let mut store = RecipientsIDStore::from_storage(storage, &api.canonical_address(&recipient)?);
    store.push(info.id)?;
    let token_address = api.canonical_address(&info.token.address)?;
    let mut staking_summary = get_staking_summary(storage, &token_address)?;
    staking_summary.total_locked += locked_amount;
    staking_summary.total_remaining += locked_amount;
    write_staking_summary(storage, &token_address, &staking_summary)?;
    Ok(info)
}

pub fn unlock<S: Storage, A: Api>(
    storage: &mut S,
    api: &A,
    id: u32,
    block_time: u64,
) -> StdResult<CosmosMsg> {
    let store = StakingInfoListStore::from_readonly_storage(storage, api);
    let mut staking_info = store.get_staking_info(id)?;
    if staking_info.unlocked {
        return Err(StdError::generic_err(format!(
            "stake is already unlocked. staking_id:{}",
            id
        )));
    }
    let token_address = api.canonical_address(&staking_info.token.address)?;
    let mut staking_summary = get_staking_summary(storage, &token_address)?;
    if let Some(staking_end_time) = staking_summary.staking_end_time {
        if staking_end_time <= block_time {
            staking_info.unlocked = true;
            let mut store = StakingInfoListStore::from_storage(storage, api);
            store.set_at(id, staking_info.clone())?;
            staking_summary.total_claimed += staking_info.locked_amount;
            staking_summary.total_remaining =
                (staking_summary.total_remaining - staking_info.locked_amount)?;
            write_staking_summary(storage, &token_address, &staking_summary)?;
            send_msg(
                staking_info.recipient,
                staking_info.locked_amount,
                None,
                None,
                None,
                256,
                staking_info.token.hash,
                staking_info.token.address,
            )
        } else {
            Err(StdError::generic_err(format!(
                "staking is not yet over. staking_id:{}",
                id,
            )))
        }
    } else {
        Err(StdError::generic_err(format!(
            "staking end time is not defined. staking_id:{}",
            id,
        )))
    }
}

pub struct StakingInfoListStore<'a, S: ReadonlyStorage, A: Api> {
    storage: S,
    api: &'a A,
}

impl<'a, 'b, S: ReadonlyStorage, A: Api>
    StakingInfoListStore<'b, ReadonlyPrefixedStorage<'a, S>, A>
{
    pub fn from_readonly_storage(storage: &'a S, api: &'b A) -> Self {
        Self {
            storage: ReadonlyPrefixedStorage::new(PREFIX_STAKING_INFO, storage),
            api,
        }
    }
}

impl<'a, 'b, S: Storage, A: Api> StakingInfoListStore<'b, PrefixedStorage<'a, S>, A> {
    pub fn from_storage(storage: &'a mut S, api: &'b A) -> Self {
        Self {
            storage: PrefixedStorage::new(PREFIX_STAKING_INFO, storage),
            api,
        }
    }
}

impl<'a, S: ReadonlyStorage, A: Api> StakingInfoListStore<'a, S, A> {
    pub fn get_staking_info(&self, id: u32) -> StdResult<StakingInfo> {
        let store = self.append_store()?;
        self.get_at(&store, id)
    }
    fn get_staking_infos(&self, ids: &[u32]) -> StdResult<Vec<StakingInfo>> {
        let store = self.append_store()?;
        ids.iter().map(|id| self.get_at(&store, *id)).collect()
    }
    fn get_latest_id(&self) -> StdResult<u32> {
        let store = self.append_store()?;
        Ok(store.len() - 1)
    }
    fn append_store(&self) -> StdResult<AppendStore<'_, CanonicalStakingInfo, S>> {
        match AppendStore::attach(&self.storage) {
            Some(store) => store,
            None => Err(StdError::generic_err("no staking info append store")),
        }
    }
    fn get_at(
        &self,
        append_store: &AppendStore<'_, CanonicalStakingInfo, S>,
        id: u32,
    ) -> StdResult<StakingInfo> {
        append_store
            .get_at(id)
            .map(|canonical| StakingInfo::from_canonical(canonical, self.api))?
    }
}

impl<'a, S: Storage, A: Api> StakingInfoListStore<'a, S, A> {
    fn push(
        &mut self,
        token: ContractReference,
        locker: HumanAddr,
        recipient: HumanAddr,
        locked_amount: Uint128,
        start_time: u64,
    ) -> StdResult<StakingInfo> {
        let mut store = AppendStoreMut::attach_or_create(&mut self.storage)?;
        let id = store.len();
        let info = StakingInfo {
            id,
            token,
            locker,
            recipient,
            locked_amount,
            start_time,
            unlocked: false,
        };
        store.push(&info.clone().into_canonical(self.api)?)?;
        Ok(info)
    }

    fn set_at(&mut self, id: u32, staking_info: StakingInfo) -> StdResult<()> {
        let mut store = AppendStoreMut::attach_or_create(&mut self.storage)?;
        store.set_at(id, &staking_info.into_canonical(self.api)?)
    }
}

struct RecipientsIDStore<S: ReadonlyStorage> {
    storage: S,
}

impl<'a, S: ReadonlyStorage> RecipientsIDStore<ReadonlyPrefixedStorage<'a, S>> {
    fn from_readonly_storage(storage: &'a S, recipient: &CanonicalAddr) -> Self {
        Self {
            storage: ReadonlyPrefixedStorage::multilevel(
                &[PREFIX_RECEIVERS_ID, recipient.as_slice()],
                storage,
            ),
        }
    }
}

impl<S: ReadonlyStorage> RecipientsIDStore<S> {
    fn get_ids(&self, page: u32, page_size: u32) -> StdResult<Vec<u32>> {
        let store = match AppendStore::attach(&self.storage) {
            Some(store) => store,
            None => Err(StdError::generic_err("recipients id store")),
        }?;
        store
            .iter()
            .skip((page * page_size) as usize)
            .take(page_size as usize)
            .collect()
    }
}

impl<'a, S: Storage> RecipientsIDStore<PrefixedStorage<'a, S>> {
    fn from_storage(storage: &'a mut S, recipient: &CanonicalAddr) -> Self {
        Self {
            storage: PrefixedStorage::multilevel(
                &[PREFIX_RECEIVERS_ID, recipient.as_slice()],
                storage,
            ),
        }
    }
}

impl<S: Storage> RecipientsIDStore<S> {
    fn push(&mut self, id: u32) -> StdResult<()> {
        let mut store = AppendStoreMut::attach_or_create(&mut self.storage)?;
        store.push(&id)
    }
}

pub fn get_staking_summary<S: ReadonlyStorage>(
    storage: &S,
    token_address: &CanonicalAddr,
) -> StdResult<StakingSummary> {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_STAKING_SUMMARY, storage);
    let store = TypedStore::attach(&storage);
    Ok(store
        .may_load(token_address.as_slice())?
        .unwrap_or_default())
}

pub fn write_staking_summary<S: Storage>(
    storage: &mut S,
    token_address: &CanonicalAddr,
    staking_summary: &StakingSummary,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(PREFIX_STAKING_SUMMARY, storage);
    let mut store = TypedStoreMut::attach(&mut storage);
    store.store(token_address.as_slice(), staking_summary)
}
