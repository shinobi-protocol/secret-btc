use cosmwasm_std::{
    Api, CanonicalAddr, HumanAddr, ReadonlyStorage, StdError, StdResult, Storage, Uint128,
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use secret_toolkit::storage::{AppendStore, AppendStoreMut, TypedStore, TypedStoreMut};
use serde::{Deserialize, Serialize};
use shared_types::E8;
use shared_types::{CanonicalContractReference, Canonicalize, ContractReference};

#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq, Clone)]
pub struct VestingInfo {
    pub id: u32,
    pub token: ContractReference,
    pub locker: HumanAddr,
    pub recipient: HumanAddr,
    pub start_time: u64,
    pub end_time: u64,
    pub locked_amount: Uint128,
    pub claimed_amount: Uint128,
    pub remaining_amount: Uint128,
}

pub struct ClaimInfo {
    pub token: ContractReference,
    pub recipient: HumanAddr,
    pub amount: Uint128,
}

impl VestingInfo {
    pub fn claim(&mut self, now: u64) -> StdResult<ClaimInfo> {
        let claimable_amount = self.claimable_amount(now);
        self.claimed_amount += claimable_amount;
        self.remaining_amount = (self.remaining_amount - claimable_amount)?;
        Ok(ClaimInfo {
            token: self.token.clone(),
            recipient: self.recipient.clone(),
            amount: claimable_amount,
        })
    }

    pub fn unlocked_amount(&self, now: u64) -> Uint128 {
        let unlocked_late_e8 = E8
            .min((now - self.start_time) as u128 * E8 / (self.end_time - self.start_time) as u128);
        Uint128(self.locked_amount.u128() * unlocked_late_e8 / E8)
    }

    pub fn claimable_amount(&self, now: u64) -> Uint128 {
        Uint128(self.unlocked_amount(now).u128() - self.claimed_amount.u128())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CanonicalVestingInfo {
    id: u32,
    token: CanonicalContractReference,
    locker: CanonicalAddr,
    recipient: CanonicalAddr,
    start_time: u64,
    end_time: u64,
    locked_amount: Uint128,
    claimed_amount: Uint128,
    remaining_amount: Uint128,
}

impl Canonicalize for VestingInfo {
    type Canonicalized = CanonicalVestingInfo;
    fn into_canonical<A>(self, api: &A) -> StdResult<Self::Canonicalized>
    where
        A: Api,
    {
        Ok(CanonicalVestingInfo {
            id: self.id,
            token: self.token.into_canonical(api)?,
            locker: api.canonical_address(&self.locker)?,
            recipient: api.canonical_address(&self.recipient)?,
            start_time: self.start_time,
            end_time: self.end_time,
            locked_amount: self.locked_amount,
            claimed_amount: self.claimed_amount,
            remaining_amount: self.remaining_amount,
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
            end_time: canonicalized.end_time,
            locked_amount: canonicalized.locked_amount,
            claimed_amount: canonicalized.claimed_amount,
            remaining_amount: canonicalized.remaining_amount,
        })
    }
}

#[derive(JsonSchema, Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct VestingSummary {
    pub total_locked: Uint128,
    pub total_claimed: Uint128,
    pub total_remaining: Uint128,
}

const PREFIX_VESTING_INFO: &[u8] = b"vesting_info";
const PREFIX_RECEIVERS_ID: &[u8] = b"recipients_id";
const PREFIX_VESTING_SUMMARY: &[u8] = b"vesting_summary";

pub fn get_latest_vesting_info_id<S: ReadonlyStorage, A: Api>(
    storage: &S,
    api: &A,
) -> StdResult<u32> {
    let store = VestingInfoListStore::from_readonly_storage(storage, api);
    store.get_latest_id()
}

pub fn get_vesting_infos<S: ReadonlyStorage, A: Api>(
    storage: &S,
    api: &A,
    ids: Vec<u32>,
) -> StdResult<Vec<VestingInfo>> {
    let store = VestingInfoListStore::from_readonly_storage(storage, api);
    store.get_vesting_infos(&ids)
}

pub fn get_recipients_vesting_infos<S: ReadonlyStorage, A: Api>(
    storage: &S,
    api: &A,
    recipient: &HumanAddr,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<VestingInfo>> {
    let recipients_ids =
        RecipientsIDStore::from_readonly_storage(storage, &api.canonical_address(recipient)?)
            .get_ids(page, page_size)?;
    let store = VestingInfoListStore::from_readonly_storage(storage, api);
    store.get_vesting_infos(&recipients_ids)
}

pub fn claim<S: Storage, A: Api>(
    storage: &mut S,
    api: &A,
    id: u32,
    now: u64,
) -> StdResult<ClaimInfo> {
    let mut store = VestingInfoListStore::from_storage(storage, api);
    let mut info = store.get_vesting_info(id)?;
    let claim_info = info.claim(now)?;
    let token_address = api.canonical_address(&info.token.address)?;
    store.set_at(id, info)?;
    let mut vesting_summary = get_vesting_summary(storage, &token_address)?;
    vesting_summary.total_claimed += claim_info.amount;
    vesting_summary.total_remaining = (vesting_summary.total_remaining - claim_info.amount)?;
    write_vesting_summary(storage, &token_address, &vesting_summary)?;
    Ok(claim_info)
}

pub fn lock<S: Storage, A: Api>(
    storage: &mut S,
    api: &A,
    token: ContractReference,
    locker: HumanAddr,
    recipient: HumanAddr,
    locked_amount: Uint128,
    start_time: u64,
    end_time: u64,
) -> StdResult<VestingInfo> {
    let mut store = VestingInfoListStore::from_storage(storage, api);
    let info = store.push(
        token,
        locker,
        recipient.clone(),
        locked_amount,
        start_time,
        end_time,
    )?;
    let mut store = RecipientsIDStore::from_storage(storage, &api.canonical_address(&recipient)?);
    store.push(info.id)?;
    let token_address = api.canonical_address(&info.token.address)?;
    let mut vesting_summary = get_vesting_summary(storage, &token_address)?;
    vesting_summary.total_locked += locked_amount;
    vesting_summary.total_remaining += locked_amount;
    write_vesting_summary(storage, &token_address, &vesting_summary)?;
    Ok(info)
}

struct VestingInfoListStore<'a, S: ReadonlyStorage, A: Api> {
    storage: S,
    api: &'a A,
}

impl<'a, 'b, S: ReadonlyStorage, A: Api>
    VestingInfoListStore<'b, ReadonlyPrefixedStorage<'a, S>, A>
{
    fn from_readonly_storage(storage: &'a S, api: &'b A) -> Self {
        Self {
            storage: ReadonlyPrefixedStorage::new(PREFIX_VESTING_INFO, storage),
            api,
        }
    }
}

impl<'a, 'b, S: Storage, A: Api> VestingInfoListStore<'b, PrefixedStorage<'a, S>, A> {
    fn from_storage(storage: &'a mut S, api: &'b A) -> Self {
        Self {
            storage: PrefixedStorage::new(PREFIX_VESTING_INFO, storage),
            api,
        }
    }
}

impl<'a, S: ReadonlyStorage, A: Api> VestingInfoListStore<'a, S, A> {
    fn get_vesting_info(&self, id: u32) -> StdResult<VestingInfo> {
        let store = self.append_store()?;
        self.get_at(&store, id)
    }
    fn get_vesting_infos(&self, ids: &[u32]) -> StdResult<Vec<VestingInfo>> {
        let store = self.append_store()?;
        ids.iter().map(|id| self.get_at(&store, *id)).collect()
    }
    fn get_latest_id(&self) -> StdResult<u32> {
        let store = self.append_store()?;
        Ok(store.len() - 1)
    }
    fn append_store(&self) -> StdResult<AppendStore<'_, CanonicalVestingInfo, S>> {
        match AppendStore::attach(&self.storage) {
            Some(store) => store,
            None => Err(StdError::generic_err("no vesting info append store")),
        }
    }
    fn get_at(
        &self,
        append_store: &AppendStore<'_, CanonicalVestingInfo, S>,
        id: u32,
    ) -> StdResult<VestingInfo> {
        append_store
            .get_at(id)
            .map(|canonical| VestingInfo::from_canonical(canonical, self.api))?
    }
}

impl<'a, S: Storage, A: Api> VestingInfoListStore<'a, S, A> {
    fn push(
        &mut self,
        token: ContractReference,
        locker: HumanAddr,
        recipient: HumanAddr,
        locked_amount: Uint128,
        start_time: u64,
        end_time: u64,
    ) -> StdResult<VestingInfo> {
        let mut store = AppendStoreMut::attach_or_create(&mut self.storage)?;
        let id = store.len();
        let info = VestingInfo {
            id,
            token,
            locker,
            recipient,
            locked_amount,
            start_time,
            end_time,
            claimed_amount: Uint128::zero(),
            remaining_amount: locked_amount,
        };
        store.push(&info.clone().into_canonical(self.api)?)?;
        Ok(info)
    }

    fn set_at(&mut self, id: u32, vesting_info: VestingInfo) -> StdResult<()> {
        let mut store = AppendStoreMut::attach_or_create(&mut self.storage)?;
        store.set_at(id, &vesting_info.into_canonical(self.api)?)
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

pub fn get_vesting_summary<S: ReadonlyStorage>(
    storage: &S,
    token_address: &CanonicalAddr,
) -> StdResult<VestingSummary> {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_VESTING_SUMMARY, storage);
    let store = TypedStore::attach(&storage);
    Ok(store
        .may_load(token_address.as_slice())?
        .unwrap_or_default())
}

fn write_vesting_summary<S: Storage>(
    storage: &mut S,
    token_address: &CanonicalAddr,
    vesting_summary: &VestingSummary,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(PREFIX_VESTING_SUMMARY, storage);
    let mut store = TypedStoreMut::attach(&mut storage);
    store.store(token_address.as_slice(), vesting_summary)
}
