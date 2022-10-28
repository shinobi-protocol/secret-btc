use super::prefix::{PREFIX_RELEASE_REQUESTS, PREFIX_UTXO_QUEUE};
use super::queue_store::QueueStore;
use crate::error::Error;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::hash_types::Txid;
use bitcoin::secp256k1::SecretKey;
use bitcoin::{Network, PrivateKey};
use cosmwasm_std::{CanonicalAddr, ReadonlyStorage, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use rand::Rng;
use secret_toolkit::crypto::sha_256;
use secret_toolkit::serialization::Bincode2;
use secret_toolkit::storage::Item;
use serde::{Deserialize, Serialize};
use shared_types::gateway::RequestKey;

/// Bitcoin UTXO.
/// It has all info needed for spend a Coin, including private key.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Utxo {
    pub txid: Txid,
    pub vout: u32,
    pub key: [u8; 32],
}

impl Utxo {
    pub fn outpoint(&self) -> OutPoint {
        OutPoint {
            txid: self.txid,
            vout: self.vout,
        }
    }

    /// Returns A Bitcoin ECDSA privatekey.
    pub fn priv_key(&self, network: Network) -> Result<PrivateKey, Error> {
        Ok(PrivateKey {
            compressed: true,
            network,
            key: SecretKey::parse(&self.key)?,
        })
    }
}

/// Queue Store Of Bitcoin UTXO.
/// Each TxValue has one queue store.
pub struct UtxoQueue<S: Storage>(QueueStore<S>);

impl<'a, S: Storage> UtxoQueue<PrefixedStorage<'a, S>> {
    pub fn from_storage(storage: &'a mut S, tx_value: u64) -> Self {
        let storage =
            PrefixedStorage::multilevel(&[PREFIX_UTXO_QUEUE, &tx_value.to_be_bytes()], storage);
        Self(QueueStore::attach(storage))
    }

    pub fn enqueue(&mut self, utxo: Utxo) -> Result<(), Error> {
        self.0.enqueue(&utxo)
    }

    pub fn dequeue(&mut self) -> Result<Option<Utxo>, Error> {
        self.0.dequeue()
    }
}

/// Bitcoin withdrawal request key.
/// It is sha256 hash of 'requester address + utxo + pseudorandom bytes'.
///
/// [IMPORTANT]
/// It must be unpredictable.
/// It must not leak any information about the used pseudorandom bytes and utxo at generation process.
///
/// The request key is provided to the requester as the proof of the request, in the form of the response of the request transaction.
/// Therefore, the request key is published to the out of the contract.
/// At the claim phase, the requester send the request key to the contract so that the contract can verify the request.
pub fn gen_request_key<R: Rng>(
    requester: &CanonicalAddr,
    utxo: &Utxo,
    rng: &mut R,
) -> Result<RequestKey, Error> {
    let input_len = 68 + requester.len();
    let mut input = Vec::with_capacity(input_len);
    input.extend_from_slice(requester.as_slice());
    input.extend_from_slice(&utxo.txid);
    input.extend_from_slice(&utxo.vout.to_be_bytes());
    input.extend_from_slice(&rng.gen::<[u8; 32]>());
    Ok(RequestKey::new(sha_256(&input)))
}

/// Bitcoin UTXO which requested to spend for withdrawing.
/// It has all info needed for spend a certian value of Coin, including private key and value.
#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct RequestedUtxo {
    pub utxo: Utxo,
    pub value: u64,
}

pub fn read_release_request_utxo<S: ReadonlyStorage>(
    storage: &S,
    request_key: &RequestKey,
) -> Result<Option<RequestedUtxo>, Error> {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_RELEASE_REQUESTS, storage);
    Ok(Item::<RequestedUtxo, Bincode2>::new(request_key.as_bytes()).may_load(&storage)?)
}

pub fn write_release_request_utxo<S: Storage>(
    storage: &mut S,
    request_key: &RequestKey,
    value: u64,
    utxo: Utxo,
) -> Result<(), Error> {
    let mut storage = PrefixedStorage::new(PREFIX_RELEASE_REQUESTS, storage);
    Ok(Item::<RequestedUtxo, Bincode2>::new(request_key.as_bytes())
        .save(&mut storage, &RequestedUtxo { utxo, value })?)
}
