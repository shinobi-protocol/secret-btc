use crate::state_proxy::msg::{
    HandleMsg, Owner, QueryAnswer, QueryMsg, ReadContractStateSignature, Secp256k1Signer,
    WriteAction,
};
use crate::{Canonicalize, ContractReference};
use cosmwasm_std::HumanAddr;
use cosmwasm_std::{
    Api, Binary, CosmosMsg, Querier, ReadonlyStorage, StdError, StdResult, Storage,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use schemars::JsonSchema;
use secret_toolkit::serialization::{Bincode2, Serde};
use secret_toolkit::utils::calls::{HandleCallback, Query};
use serde::de::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;

#[derive(JsonSchema, Debug, Clone, Default)]
pub struct Seed([u8; 32]);

impl AsRef<[u8]> for Seed {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl Serialize for Seed {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
    where
        S: serde::Serializer,
    {
        Binary::from(self.0.as_slice()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Seed {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match Binary::deserialize(deserializer) {
            Ok(binary) => {
                let array: [u8; 32] = binary
                    .0
                    .try_into()
                    .map_err(|e| D::Error::custom(format!("{:?}", e)))?;
                Ok(Seed(array))
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug)]
pub struct StateProxyStorage<'a, Q: Querier> {
    querier: &'a Q,
    read_contract_state_signature: ReadContractStateSignature,
    state_contract_reference: ContractReference,
    init_flag: bool,
    transaction: StateTransaction,
}

pub type StateTransaction = HashMap<Vec<u8>, WriteAction>;

const STATE_CONTRACT_OWNER_KEY: &[u8] = b"state_contract_owner_key";
const STATE_CONTRACT_REFERENCE: &[u8] = b"state_contract_reference";

impl<'a, Q: Querier> StateProxyStorage<'a, Q> {
    pub fn cosmos_msgs(&self) -> StdResult<Vec<CosmosMsg>> {
        let mut msgs = vec![];
        if self.init_flag {
            msgs.push(
                HandleMsg::InitContractState {
                    contract_label: self.read_contract_state_signature.contract_label.clone(),
                    public_key: self.read_contract_state_signature.pub_key.clone(),
                }
                .to_cosmos_msg(
                    self.state_contract_reference.hash.clone(),
                    self.state_contract_reference.address.clone(),
                    None,
                )?,
            )
        }
        msgs.push(
            HandleMsg::WriteContractState {
                contract_label: self.read_contract_state_signature.contract_label.clone(),
                transaction: self
                    .transaction
                    .iter()
                    .map(|(key, action)| (Binary::from(key.as_slice()), action.clone()))
                    .collect(),
            }
            .to_cosmos_msg(
                self.state_contract_reference.hash.clone(),
                self.state_contract_reference.address.clone(),
                None,
            )?,
        );
        Ok(msgs)
    }

    pub fn add_messages_to_state_proxy_msg(
        &self,
        messages: Vec<CosmosMsg>,
    ) -> StdResult<Vec<CosmosMsg>> {
        let mut new_messages = Vec::with_capacity(2 + messages.len());
        new_messages.extend(self.cosmos_msgs()?);
        new_messages.extend(messages);
        Ok(new_messages)
    }
}

impl<'a, Q: Querier> ReadonlyStorage for StateProxyStorage<'a, Q> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        if let Some((_, WriteAction::Set { value })) =
            self.transaction.iter().find(|(k, _)| k.as_slice() == key)
        {
            return Some(value.clone().0);
        }
        let query_response: StdResult<QueryAnswer> = QueryMsg::ReadContractState {
            signature: self.read_contract_state_signature.clone(),
            key: Binary::from(key),
        }
        .query(
            self.querier,
            self.state_contract_reference.hash.clone(),
            self.state_contract_reference.address.clone(),
        );
        if let Ok(QueryAnswer::ReadContractState { value }) = query_response {
            {
                return value.map(|value| value.0);
            }
        };
        return None;
    }
}

impl<'a, Q: Querier> Storage for StateProxyStorage<'a, Q> {
    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.transaction.insert(
            key.to_vec(),
            WriteAction::Set {
                value: Binary::from(value),
            },
        );
    }
    fn remove(&mut self, key: &[u8]) {
        self.transaction
            .insert(key.to_vec(), WriteAction::Remove {});
    }
}

#[derive(Debug)]
pub struct StateProxyDeps<'a, A: Api, Q: Querier> {
    pub storage: StateProxyStorage<'a, Q>,
    pub api: A,
    pub querier: &'a Q,
}

impl<'a, A: Api, Q: Querier> StateProxyDeps<'a, A, Q> {
    pub fn init<S: Secp256k1Signer, ST: Storage>(
        storage: &'a mut ST,
        api: &'a A,
        querier: &'a Q,
        contract_label: &[u8],
        state_contract_key_seed: Seed,
        state_contract_reference: ContractReference,
        signer: &S,
    ) -> StdResult<Self> {
        let mut rng = StdRng::from_seed(state_contract_key_seed.0);
        let priv_key = Binary::from(secp256k1::SecretKey::random(&mut rng).serialize());
        storage.set(STATE_CONTRACT_OWNER_KEY, priv_key.as_slice());
        storage.set(
            STATE_CONTRACT_REFERENCE,
            Bincode2::serialize(&state_contract_reference.clone().into_canonical(api)?)?.as_slice(),
        );
        let signature = ReadContractStateSignature::sign(priv_key, contract_label, signer)?;
        Ok(Self {
            storage: StateProxyStorage {
                querier: &querier,
                read_contract_state_signature: signature,
                state_contract_reference,
                init_flag: true,
                transaction: StateTransaction::new(),
            },
            api: api.clone(),
            querier: &querier,
        })
    }
    pub fn restore<S: Secp256k1Signer, ST: Storage>(
        storage: &'a ST,
        api: &'a A,
        querier: &'a Q,
        contract_label: &[u8],
        signer: &S,
    ) -> StdResult<Self> {
        let state_contract_key = Binary::from(
            storage
                .get(STATE_CONTRACT_OWNER_KEY)
                .ok_or_else(|| StdError::generic_err("state contract key not found"))?,
        );
        let state_contract_reference = ContractReference::from_canonical(
            Bincode2::deserialize(
                storage
                    .get(STATE_CONTRACT_REFERENCE)
                    .ok_or_else(|| StdError::generic_err("state contract reference not found"))?
                    .as_slice(),
            )?,
            api,
        )?;
        let signature =
            ReadContractStateSignature::sign(state_contract_key, contract_label, signer)?;
        Ok(Self {
            storage: StateProxyStorage {
                querier: &querier,
                read_contract_state_signature: signature,
                state_contract_reference,
                init_flag: false,
                transaction: StateTransaction::new(),
            },
            api: api.clone(),
            querier: &querier,
        })
    }
}

pub struct Secp256k1ApiSigner<'a, A: Api> {
    api: &'a A,
}

impl<'a, A: Api> Secp256k1ApiSigner<'a, A> {
    pub fn new(api: &'a A) -> Self {
        Self { api }
    }
}

impl<'a, A: Api> Secp256k1Signer for Secp256k1ApiSigner<'a, A> {
    fn sign(&self, message: &[u8], priv_key: &[u8]) -> StdResult<Vec<u8>> {
        self.api
            .secp256k1_sign(message, priv_key)
            .map_err(|e| StdError::generic_err(format!("api secp256k1 sign error: {}", e)))
    }
}
