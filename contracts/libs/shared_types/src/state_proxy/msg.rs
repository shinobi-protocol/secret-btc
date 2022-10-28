use crate::Canonicalize;
use crate::BLOCK_SIZE;
use cosmwasm_std::{Api, Binary, CanonicalAddr, HumanAddr, StdError, StdResult};
use schemars::JsonSchema;
use secret_toolkit::crypto::secp256k1::PrivateKey;
use secret_toolkit::utils::calls::{HandleCallback, Query};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InitMsg {
    pub contract_owners: Vec<(Binary, Owner)>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum WriteAction {
    Set { value: Binary },
    Remove {},
}

pub type StateTransaction = Vec<(Binary, WriteAction)>;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Owner {
    pub address: HumanAddr,
    pub public_key: Binary,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CanonicalOwner {
    address: CanonicalAddr,
    public_key: Binary,
}

impl Canonicalize for Owner {
    type Canonicalized = CanonicalOwner;
    fn into_canonical<A: Api>(self, api: &A) -> StdResult<Self::Canonicalized> {
        Ok(Self::Canonicalized {
            address: self.address.into_canonical(api)?,
            public_key: self.public_key,
        })
    }
    fn from_canonical<A: Api>(canonical: Self::Canonicalized, api: &A) -> StdResult<Self> {
        Ok(Self {
            address: api.human_address(&canonical.address)?,
            public_key: canonical.public_key,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    InitContractState {
        contract_label: Binary,
        public_key: Binary,
    },
    WriteContractState {
        contract_label: Binary,
        transaction: StateTransaction,
    },
    ChangeOwnerByAdmin {
        contract_label: Binary,
        next_owner: Owner,
    },
    ChangeAdmin {
        next_admin: HumanAddr,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Admin {},
    Owner {
        contract_label: Binary,
    },
    ReadContractState {
        signature: ReadContractStateSignature,
        key: Binary,
    },
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Admin { admin: HumanAddr },
    Owner { owner: Option<Owner> },
    ReadContractState { value: Option<Binary> },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct ReadContractStateSignature {
    pub pub_key: Binary,
    pub contract_label: Binary,
    pub signature: Binary,
}

pub trait Secp256k1Signer {
    fn sign(&self, message: &[u8], priv_key: &[u8]) -> StdResult<Vec<u8>>;
}

pub trait Secp256k1Verifier {
    fn verify(&self, message: &[u8], signature: &[u8], pub_key: &[u8]) -> StdResult<()>;
}

impl ReadContractStateSignature {
    pub fn sign<S: Secp256k1Signer>(
        priv_key: Binary,
        contract_label: &[u8],
        secp256k1_signer: &S,
    ) -> StdResult<Self> {
        let pub_key = PrivateKey::parse(
            priv_key
                .as_slice()
                .try_into()
                .map_err(|e| StdError::generic_err(format!("priv key parse error: {}", e)))?,
        )?
        .pubkey()
        .serialize();
        let signature = secp256k1_signer.sign(contract_label, &priv_key.as_slice())?;
        Ok(Self {
            pub_key: Binary::from(pub_key.as_slice()),
            contract_label: Binary::from(contract_label),
            signature: Binary::from(signature),
        })
    }
    pub fn verify<V: Secp256k1Verifier>(&self, secp256k1_verifier: &V) -> StdResult<()> {
        secp256k1_verifier.verify(
            self.contract_label.as_slice(),
            self.signature.as_slice(),
            self.pub_key.as_slice(),
        )
    }
}
