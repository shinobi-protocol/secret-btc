use crate::config::Config;
use cosmwasm_std::{Api, ReadonlyStorage, StdResult, Storage, Uint128};
use secret_toolkit::storage::{TypedStore, TypedStoreMut};
use serde::{Deserialize, Serialize};
use shared_types::Canonicalize;
use std::convert::TryInto;

const CONFIG_KEY: &[u8] = b"config";
const TOTAL_MINTED_KEY: &[u8] = b"total_minted";

pub fn write_config<S: Storage, A: Api>(storage: &mut S, api: &A, config: Config) -> StdResult<()> {
    let mut store = TypedStoreMut::attach(storage);
    store.store(CONFIG_KEY, &config.into_canonical(api)?)
}

pub fn read_config<S: ReadonlyStorage, A: Api>(storage: &S, api: &A) -> StdResult<Config> {
    let store = TypedStore::attach(storage);
    let canonicalized = store.load(CONFIG_KEY)?;
    Config::from_canonical(canonicalized, api)
}

pub fn write_total_minted<S: Storage>(storage: &mut S, total_minted: Uint128) {
    storage.set(TOTAL_MINTED_KEY, &total_minted.0.to_be_bytes())
}

pub fn read_total_minted<S: ReadonlyStorage>(storage: &S) -> Uint128 {
    match storage.get(TOTAL_MINTED_KEY) {
        Some(bytes) => u128::from_be_bytes(bytes.try_into().unwrap()).into(),
        None => Uint128::zero(),
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct ShurikenRewardTracker {
    pub block_height: u64,
    pub base_reward: u128,
    pub rewarded_count: u8,
}

impl ShurikenRewardTracker {
    pub fn reward(&mut self, best_height: u64, over_time: u128) -> u128 {
        if best_height != self.block_height {
            self.block_height = best_height;
            self.rewarded_count = 0;
            self.base_reward = base_reward(self.base_reward, over_time);
        }
        let reward = self.base_reward / 2u128.pow(self.rewarded_count.into());
        self.rewarded_count = self.rewarded_count.saturating_add(1);
        reward
    }
}

pub enum RewardType {
    BitcoinSPV,
    SFPS,
}

impl RewardType {
    fn storage_key(&self) -> Vec<u8> {
        match self {
            RewardType::BitcoinSPV => b"bitcoin_spv_reward".to_vec(),
            RewardType::SFPS => b"sfps_reward".to_vec(),
        }
    }
}

/*
* header base reward increase rate 'r' calculation
* 1) 0 < over_time < 60:      r = 0.99 + 0.01/60 * over_time
* 2) 60 <= over_time < 3660: r = 1 + 0.05/3600 * (over_time - 60)
* 3) 3660 <= over_time:        r = 1.05
*/
fn base_reward(prev: u128, over_time: u128) -> u128 {
    let over_time = over_time as u128;
    if over_time < 60u128 {
        (prev * 99 + prev * over_time / 60u128) / 100
    } else if over_time < 3660 {
        prev + prev * (over_time - 60) / 72000u128
    } else {
        prev * 105u128 / 100u128
    }
}

pub fn read_reward_tracker<S: ReadonlyStorage>(
    storage: &S,
    reward_type: RewardType,
) -> StdResult<ShurikenRewardTracker> {
    let store = TypedStore::attach(storage);
    store.load(&reward_type.storage_key())
}

pub fn write_reward_tracker<S: Storage>(
    storage: &mut S,
    reward_tracker: &ShurikenRewardTracker,
    reward_type: RewardType,
) -> StdResult<()> {
    let mut store = TypedStoreMut::attach(storage);
    store.store(&reward_type.storage_key(), reward_tracker)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_base_reward() {
        let reward = base_reward(1000, 0);
        assert_eq!(reward, 990);
        let reward = base_reward(1000, 5);
        assert_eq!(reward, 990);
        let reward = base_reward(1000, 6);
        assert_eq!(reward, 991);
        let reward = base_reward(1000, 54);
        assert_eq!(reward, 999);
        let reward = base_reward(1000, 60);
        assert_eq!(reward, 1000);
        let reward = base_reward(1000, 1860);
        assert_eq!(reward, 1025);
        let reward = base_reward(1000, 3659);
        assert_eq!(reward, 1049);
        let reward = base_reward(1000, 3660);
        assert_eq!(reward, 1050);
        let reward = base_reward(1000, 7320);
        assert_eq!(reward, 1050);
    }

    #[test]
    fn test_shuriken_reward_tracker() {
        let mut tracker = ShurikenRewardTracker {
            block_height: 1,
            base_reward: 1000,
            rewarded_count: 1,
        };
        assert_eq!(tracker.reward(1, 60), 500);
        assert_eq!(tracker.rewarded_count, 2);
        assert_eq!(tracker.reward(1, 100), 250);
        assert_eq!(tracker.rewarded_count, 3);
        assert_eq!(tracker.reward(2, 60), 1000);
        assert_eq!(tracker.block_height, 2);
        assert_eq!(tracker.rewarded_count, 1);
        assert_eq!(tracker.reward(2, 60), 500);
        assert_eq!(tracker.reward(4, 1860), 1025);
        assert_eq!(tracker.base_reward, 1025);
    }
}
