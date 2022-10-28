use crate::gateway::RequestKey;
use cosmwasm_std::{StdError, Uint128};
use serde::{Deserialize, Serialize};

/// This enum is used as JSON schema of Query Response.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    /// tag: 0
    MintStarted(MintStartedData),
    /// tag: 1
    MintCompleted(MintCompletedData),
    /// tag: 2
    ReleaseStarted(ReleaseStartedData),
    /// tag: 3
    ReleaseRequestConfirmed(ReleaseRequestConfirmedData),
    /// tag: 4
    ReleaseCompleted(ReleaseCompletedData),
    /// tag: 5
    ReleaseIncorrectAmountBTC(ReleaseIncorrectAmountBTCData),
    /// tag: 8
    Other(String),
}

/// contracts or user to submit event.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum EventSource {
    Gateway,
    User,
    Any,
}

impl Event {
    /// Returns EventSource which is allowed to add event.
    /// Event must be submitted only by the authorized source.
    pub fn authorized_source(&self) -> EventSource {
        match self {
            Self::MintStarted(_)
            | Self::MintCompleted(_)
            | Self::ReleaseStarted(_)
            | Self::ReleaseCompleted(_)
            | Self::ReleaseIncorrectAmountBTC(_) => EventSource::Gateway,
            Self::ReleaseRequestConfirmed(_) => EventSource::User,
            Self::Other(_) => EventSource::Any,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, schemars::JsonSchema)]
pub struct MintStartedData {
    pub time: u64,
    pub address: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, schemars::JsonSchema)]
pub struct MintCompletedData {
    pub time: u64,
    pub address: String,
    pub amount: u64,
    pub txid: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, schemars::JsonSchema)]
pub struct ReceivedFromTreasuryData {
    pub time: u64,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, schemars::JsonSchema)]
pub struct ReleaseStartedData {
    pub time: u64,
    pub request_key: RequestKey,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, schemars::JsonSchema)]
pub struct ReleaseRequestConfirmedData {
    pub time: u64,
    pub request_key: RequestKey,
    pub block_height: u64,
    pub txid: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, schemars::JsonSchema)]
pub struct ReleaseCompletedData {
    pub time: u64,
    pub request_key: RequestKey,
    pub txid: String,
    pub fee_per_vb: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, schemars::JsonSchema)]
pub struct ReleaseIncorrectAmountBTCData {
    pub time: u64,
    pub amount: Uint128,
    pub release_from: String,
    pub release_to: String,
    pub txid: String,
}

/// Use Original Serialization for serialize/deserialize Event value while avoiding floating point failure.
/// It serializes inner struct of Enum value in Bincode2, and append event_type byte.
/// When deserialization, it read the last byte as event_type byte and deserialize other byte into inner struct of Enum value.
/// Using Bincode2 or Cbor for Rust Enum serialization generates some floating points in wasm code, which are forbidden on secret contract.
/// Original Compress Serialization is as small as Bincode2, and 2/3 smaller than JSON.
pub mod serde_event_on_storage {
    use super::*;
    use cosmwasm_std::StdResult;
    use secret_toolkit::serialization::{Bincode2, Serde};
    pub fn serialize(event: &Event) -> StdResult<Vec<u8>> {
        let (event_type, event_data) = match event {
            Event::MintStarted(data) => (0, Bincode2::serialize(data)?),
            Event::MintCompleted(data) => (1, Bincode2::serialize(data)?),
            Event::ReleaseStarted(data) => (2, Bincode2::serialize(data)?),
            Event::ReleaseRequestConfirmed(data) => (3, Bincode2::serialize(data)?),
            Event::ReleaseCompleted(data) => (4, Bincode2::serialize(data)?),
            Event::ReleaseIncorrectAmountBTC(data) => (5, Bincode2::serialize(data)?),
            Event::Other(data) => (6, data.as_bytes().to_vec()),
        };
        let mut serialized = event_data;
        serialized.push(event_type);
        Ok(serialized)
    }
    pub fn deserialize(mut bytes: Vec<u8>) -> StdResult<Event> {
        let event_type = bytes
            .pop()
            .ok_or_else(|| StdError::generic_err("bytes is empty"))?;
        let event_data = bytes;
        match event_type {
            0 => Ok(Event::MintStarted(Bincode2::deserialize(&event_data)?)),
            1 => Ok(Event::MintCompleted(Bincode2::deserialize(&event_data)?)),
            2 => Ok(Event::ReleaseStarted(Bincode2::deserialize(&event_data)?)),
            3 => Ok(Event::ReleaseRequestConfirmed(Bincode2::deserialize(
                &event_data,
            )?)),
            4 => Ok(Event::ReleaseCompleted(Bincode2::deserialize(&event_data)?)),
            5 => Ok(Event::ReleaseIncorrectAmountBTC(Bincode2::deserialize(
                &event_data,
            )?)),
            6 => Ok(Event::Other(Bincode2::deserialize(&event_data)?)),
            x => Err(StdError::generic_err(format!(
                "unexpected event type {}",
                x
            ))),
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        #[test]
        fn test_serde_event_on_storage() {
            let events = vec![
                Event::MintStarted(MintStartedData {
                    time: 10000,
                    address: "address_1".into(),
                }),
                Event::MintStarted(MintStartedData {
                    time: 20000,
                    address: "address_2".into(),
                }),
                Event::MintCompleted(MintCompletedData {
                    time: 10000,
                    address: "address_1".into(),
                    amount: 10,
                    txid: "txid_1".into(),
                }),
                Event::MintCompleted(MintCompletedData {
                    time: 20000,
                    address: "address_2".into(),
                    amount: 20,
                    txid: "txid_2".into(),
                }),
                Event::ReleaseStarted(ReleaseStartedData {
                    time: 10000,
                    request_key: RequestKey::new([0; 32]),
                    amount: 10,
                }),
                Event::ReleaseStarted(ReleaseStartedData {
                    time: 20000,
                    request_key: RequestKey::new([1; 32]),
                    amount: 20,
                }),
                Event::ReleaseRequestConfirmed(ReleaseRequestConfirmedData {
                    time: 10000,
                    request_key: RequestKey::new([0; 32]),
                    block_height: 1,
                    txid: "txid_1".into(),
                }),
                Event::ReleaseRequestConfirmed(ReleaseRequestConfirmedData {
                    time: 20000,
                    request_key: RequestKey::new([1; 32]),
                    block_height: 2,
                    txid: "txid_2".into(),
                }),
                Event::ReleaseCompleted(ReleaseCompletedData {
                    time: 100000,
                    request_key: RequestKey::new([0; 32]),
                    txid: "txid_1".into(),
                    fee_per_vb: 100,
                }),
                Event::ReleaseCompleted(ReleaseCompletedData {
                    time: 200000,
                    request_key: RequestKey::new([1; 32]),
                    txid: "txid_2".into(),
                    fee_per_vb: 200,
                }),
                Event::ReleaseIncorrectAmountBTC(ReleaseIncorrectAmountBTCData {
                    time: 100000,
                    amount: 10u64.into(),
                    release_from: "release_from".into(),
                    release_to: "release_to".into(),
                    txid: "txid".into(),
                }),
                Event::Other("{ \"time\": \"100000\" }".to_string()),
            ];

            for event in events {
                let serialized = serialize(&event).unwrap();
                let deserialized = deserialize(serialized).unwrap();
                assert_eq!(event, deserialized);
            }
        }
    }
}
