use crate::error::Error;
use cosmwasm_std::Storage;
use secret_toolkit::serialization::Bincode2;
use secret_toolkit::serialization::Serde;
use serde::{de::DeserializeOwned, Serialize};
use std::convert::TryInto;

/// Circular Queue Store
/// Max data number of the queue is u64::max - 1
/// It serialize data element in Bincode2.
#[derive(Debug)]
pub struct QueueStore<S>
where
    S: Storage,
{
    pub storage: S,
    front: u64,
    rear: u64,
}

const FRONT_KEY: &[u8] = b"front";
const REAR_KEY: &[u8] = b"rear";

impl<S> QueueStore<S>
where
    S: Storage,
{
    /// Attach a storage as QueueStore.
    pub fn attach(storage: S) -> Self {
        let front = if let Some(bytes) = storage.get(FRONT_KEY) {
            u64::from_be_bytes(bytes.try_into().unwrap())
        } else {
            0
        };
        let rear = if let Some(bytes) = storage.get(REAR_KEY) {
            u64::from_be_bytes(bytes.try_into().unwrap())
        } else {
            0
        };
        Self {
            storage,
            front,
            rear,
        }
    }

    /// Enqueue data.
    /// Returns error when the queue is full.
    pub fn enqueue<T: Serialize>(&mut self, value: &T) -> Result<(), Error> {
        if self.rear.wrapping_add(1) == self.front {
            return Err(Error::contract_err("queue store is full"));
        }
        self.storage
            .set(&self.rear.to_be_bytes(), &Bincode2::serialize(value)?);
        self.increment_rear();
        Ok(())
    }

    /// Dequeue data.
    /// Returns None when the queue is empty.
    pub fn dequeue<T: DeserializeOwned>(&mut self) -> Result<Option<T>, Error> {
        if self.front == self.rear {
            return Ok(None);
        }
        let bytes = self
            .storage
            .get(&self.front.to_be_bytes())
            .ok_or_else(|| Error::contract_err("failed to read queue item"))?;
        self.increment_front();
        let item: T = Bincode2::deserialize(&bytes)?;
        Ok(Some(item))
    }

    /// Increments front.
    /// if front is at end of the queue, return to start of the queue.
    fn increment_front(&mut self) {
        self.front = self.front.wrapping_add(1);
        self.storage.set(FRONT_KEY, &self.front.to_be_bytes());
    }

    /// Increments rear.
    /// if rear is at end of the queue, return to start of the queue.
    fn increment_rear(&mut self) {
        self.rear = self.rear.wrapping_add(1);
        self.storage.set(REAR_KEY, &self.rear.to_be_bytes());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::MockStorage;

    #[test]
    fn test_new_first() {
        let storage = MockStorage::new();
        let queue_store = QueueStore::attach(storage);
        assert_eq!(queue_store.front, 0);
        assert_eq!(queue_store.rear, 0);
    }

    #[test]
    fn test_new_restore() {
        let mut storage = MockStorage::new();
        storage.set(FRONT_KEY, &1u64.to_be_bytes());
        storage.set(REAR_KEY, &2u64.to_be_bytes());
        let queue_store = QueueStore::attach(storage);
        assert_eq!(queue_store.front, 1);
        assert_eq!(queue_store.rear, 2);
    }

    #[test]
    fn test_enqueue_dequeue() {
        let storage = MockStorage::new();

        let mut queue_store = QueueStore::attach(storage);
        queue_store.enqueue(&1000u32).unwrap();
        assert_eq!(queue_store.front, 0);
        assert_eq!(queue_store.rear, 1);

        let mut queue_store = QueueStore::attach(queue_store.storage);
        assert_eq!(queue_store.front, 0);
        assert_eq!(queue_store.rear, 1);
        queue_store.enqueue(&2000u32).unwrap();
        assert_eq!(queue_store.front, 0);
        assert_eq!(queue_store.rear, 2);

        assert_eq!(queue_store.dequeue::<u32>().unwrap().unwrap(), 1000);
        assert_eq!(queue_store.front, 1);
        assert_eq!(queue_store.rear, 2);
        assert_eq!(queue_store.dequeue::<u32>().unwrap().unwrap(), 2000);
        assert_eq!(queue_store.front, 2);
        assert_eq!(queue_store.rear, 2);
        assert!(queue_store.dequeue::<u32>().unwrap().is_none());
        let mut queue_store = QueueStore::attach(queue_store.storage);
        assert_eq!(queue_store.front, 2);
        assert_eq!(queue_store.rear, 2);
        queue_store.enqueue(&3000u32).unwrap();
        assert_eq!(queue_store.front, 2);
        assert_eq!(queue_store.rear, 3);
    }

    #[test]
    fn test_enqueue_dequeue_circulate() {
        let mut storage = MockStorage::new();
        storage.set(FRONT_KEY, &u64::MAX.to_be_bytes());
        storage.set(REAR_KEY, &u64::MAX.to_be_bytes());
        let mut queue_store = QueueStore::attach(storage);
        queue_store.enqueue(&3000u32).unwrap();
        assert_eq!(queue_store.front, u64::MAX);
        assert_eq!(queue_store.rear, 0);
        let mut queue_store = QueueStore::attach(queue_store.storage);
        assert_eq!(queue_store.front, u64::MAX);
        assert_eq!(queue_store.rear, 0);
        assert_eq!(queue_store.dequeue::<u32>().unwrap().unwrap(), 3000u32);
        assert_eq!(queue_store.front, 0);
        assert_eq!(queue_store.rear, 0);
        let queue_store = QueueStore::attach(queue_store.storage);
        assert_eq!(queue_store.front, 0);
        assert_eq!(queue_store.rear, 0);
    }

    #[test]
    fn test_queue_limit() {
        let mut storage = MockStorage::new();
        storage.set(FRONT_KEY, &1u64.to_be_bytes());
        storage.set(REAR_KEY, &0u64.to_be_bytes());
        let mut queue_store = QueueStore::attach(storage);
        queue_store.enqueue(&3000u32).unwrap_err();
        let mut storage = MockStorage::new();
        storage.set(FRONT_KEY, &0u64.to_be_bytes());
        storage.set(REAR_KEY, &u64::MAX.to_be_bytes());
        let mut queue_store = QueueStore::attach(storage);
        queue_store.enqueue(&3000u32).unwrap_err();
    }
}
