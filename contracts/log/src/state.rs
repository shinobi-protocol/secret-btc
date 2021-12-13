use cosmwasm_std::{Api, CanonicalAddr, ReadonlyStorage, StdError, StdResult, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{TypedStore, TypedStoreMut};
use shared_types::log::{event::serde_event_on_storage, Config, Event};
use shared_types::Canonicalize;
use std::convert::TryInto;

pub const PRNG_KEY: &[u8] = b"prng";
pub const CONFIG_KEY: &[u8] = b"config";
pub const PREFIX_LOG: &[u8] = b"log";
pub const PREFIX_VIEWING_KEY: &[u8] = b"viewing_key";
pub const LEN_KEY: &[u8] = b"len";

pub fn read_config<S: ReadonlyStorage, A: Api>(storage: &S, api: &A) -> StdResult<Config> {
    let store = TypedStore::attach(storage);
    let canonical = store.load(CONFIG_KEY)?;
    Config::from_canonical(canonical, api)
}

pub fn write_config<S: Storage, A: Api>(storage: &mut S, config: Config, api: &A) -> StdResult<()> {
    let mut store = TypedStoreMut::attach(storage);
    store.store(CONFIG_KEY, &config.into_canonical(api)?)
}

/// Log Storage.
/// Log is list of Events about a address.
/// It serializes/deserializes Events using custom serialization.
/// Event list are stored in order of time.
pub struct LogStorage<S: ReadonlyStorage> {
    pub storage: S,
    len: u32,
}

impl<'a, S: ReadonlyStorage> LogStorage<ReadonlyPrefixedStorage<'a, S>> {
    pub fn from_readonly_storage(storage: &'a S, account: &CanonicalAddr) -> Self {
        let storage =
            ReadonlyPrefixedStorage::multilevel(&[PREFIX_LOG, account.as_slice()], storage);
        Self::new(storage)
    }
}

impl<'a, S: Storage> LogStorage<PrefixedStorage<'a, S>> {
    pub fn from_storage(storage: &'a mut S, account: &CanonicalAddr) -> Self {
        let storage = PrefixedStorage::multilevel(&[PREFIX_LOG, account.as_slice()], storage);
        Self::new(storage)
    }
}

impl<S: ReadonlyStorage> LogStorage<S> {
    fn new(storage: S) -> Self {
        let len = if let Some(bytes) = storage.get(LEN_KEY) {
            u32::from_be_bytes(bytes.try_into().unwrap())
        } else {
            0
        };
        Self { storage, len }
    }
    /// Returns Logs, which are arranged in desc order of time.
    /// It supports pagination.
    /// It skips the first `page_size * page` items and return the next `page_size` items.
    pub fn get_logs(&self, page: u32, page_size: u32) -> StdResult<Vec<Event>> {
        let max = self.len.saturating_sub(page_size.saturating_mul(page));
        let min = max.saturating_sub(page_size);
        let mut events = Vec::with_capacity(page_size as _);
        for i in (min..max).rev() {
            match self.storage.get(&i.to_be_bytes()) {
                Some(bytes) => {
                    let event = serde_event_on_storage::deserialize(bytes)?;
                    events.push(event);
                }
                None => {
                    return Err(StdError::generic_err("cannot get log"));
                }
            }
        }
        Ok(events)
    }
}

impl<S: Storage> LogStorage<S> {
    /// Append Event to log.
    /// Storage append the event to the top of the list.
    pub fn append(&mut self, event: &Event) -> StdResult<()> {
        let bytes: Vec<u8> = serde_event_on_storage::serialize(event)?;
        self.storage.set(&self.len.to_be_bytes(), &bytes);
        self.len += 1;
        self.storage.set(LEN_KEY, &(self.len).to_be_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::MockApi;
    use cosmwasm_std::testing::MockStorage;
    use cosmwasm_std::Api;
    use shared_types::log::event::*;

    #[test]
    fn test_log_storage() {
        let mut storage = MockStorage::new();
        let canonical_addr = MockApi::default()
            .canonical_address(&"lebron".into())
            .unwrap();
        let mut log_storage = LogStorage::from_storage(&mut storage, &canonical_addr);
        let time = 1000;
        let max_time = 1039;
        for i in 0..40 {
            log_storage
                .append(&Event::MintStarted(MintStartedData {
                    time: time + i,
                    address: "address".to_string(),
                }))
                .unwrap()
        }
        // page_size = 10
        for page in 0u32..4u32 {
            let logs = log_storage.get_logs(page, 10).unwrap();
            assert_eq!(logs.len(), 10);
            for (i, log) in logs.iter().enumerate() {
                assert_eq!(
                    log,
                    &Event::MintStarted(MintStartedData {
                        time: max_time - (page as u64) * 10 - i as u64,
                        address: "address".to_string()
                    })
                )
            }
        }
        assert_eq!(log_storage.get_logs(4, 10).unwrap(), vec![]);
        assert_eq!(log_storage.get_logs(5, 10).unwrap(), vec![]);
        // page_size = 20
        for page in 0u32..2u32 {
            let logs = log_storage.get_logs(page, 20).unwrap();
            assert_eq!(logs.len(), 20);
            for (i, log) in logs.iter().enumerate() {
                assert_eq!(
                    log,
                    &Event::MintStarted(MintStartedData {
                        time: max_time - (page as u64) * 20 - i as u64,
                        address: "address".to_string()
                    })
                )
            }
        }
        assert_eq!(log_storage.get_logs(2, 20).unwrap(), vec![]);
        // page_size = 30
        let logs = log_storage.get_logs(0, 30).unwrap();
        assert_eq!(logs.len(), 30);
        for (i, log) in logs.iter().enumerate() {
            assert_eq!(
                log,
                &Event::MintStarted(MintStartedData {
                    time: max_time - i as u64,
                    address: "address".to_string()
                })
            )
        }
        let logs = log_storage.get_logs(1, 30).unwrap();
        assert_eq!(logs.len(), 10);
        for (i, log) in logs.iter().enumerate() {
            assert_eq!(
                log,
                &Event::MintStarted(MintStartedData {
                    time: max_time - 30 - i as u64,
                    address: "address".to_string()
                })
            )
        }
        // page_size = 50
        let logs = log_storage.get_logs(0, 50).unwrap();
        assert_eq!(logs.len(), 40);
        for (i, log) in logs.iter().enumerate() {
            assert_eq!(
                log,
                &Event::MintStarted(MintStartedData {
                    time: max_time - i as u64,
                    address: "address".to_string()
                })
            )
        }
        assert_eq!(log_storage.get_logs(1, 50).unwrap().len(), 0);
    }
}
