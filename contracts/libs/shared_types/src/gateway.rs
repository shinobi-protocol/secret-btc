use crate::{
    bitcoin_spv, sfps, viewing_key, CanonicalContractReference, Canonicalize, ContractReference,
    BLOCK_SIZE,
};
use cosmwasm_std::{Api, Binary, CanonicalAddr, HumanAddr, StdResult};
use schemars::JsonSchema;
use secret_toolkit::utils::HandleCallback;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InitMsg {
    pub entropy: Binary,
    pub config: Config,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct Config {
    /// [Bitcoin]
    /// Unit of utxo value that the contrat accepts
    pub btc_tx_values: HashSet<u64>,

    /// [Contract References]
    pub bitcoin_spv: ContractReference,
    pub sfps: ContractReference,
    pub sbtc: ContractReference,
    pub finance_admin: ContractReference,
    pub log: ContractReference,

    /// [Owner]
    pub owner: HumanAddr,
}

#[derive(Serialize, Deserialize)]
pub struct CanonicalConfig {
    pub btc_tx_values: HashSet<u64>,
    pub bitcoin_spv: CanonicalContractReference,
    pub sfps: CanonicalContractReference,
    pub sbtc: CanonicalContractReference,
    pub finance_admin: CanonicalContractReference,
    pub log: CanonicalContractReference,
    pub owner: CanonicalAddr,
}

impl Canonicalize for Config {
    type Canonicalized = CanonicalConfig;

    fn into_canonical<A: Api>(self, api: &A) -> StdResult<Self::Canonicalized> {
        Ok(Self::Canonicalized {
            btc_tx_values: self.btc_tx_values,
            bitcoin_spv: self.bitcoin_spv.into_canonical(api)?,
            sfps: self.sfps.into_canonical(api)?,
            sbtc: self.sbtc.into_canonical(api)?,
            finance_admin: self.finance_admin.into_canonical(api)?,
            log: self.log.into_canonical(api)?,
            owner: self.owner.into_canonical(api)?,
        })
    }

    fn from_canonical<A: Api>(canonical: Self::Canonicalized, api: &A) -> StdResult<Self> {
        Ok(Self {
            btc_tx_values: canonical.btc_tx_values,
            bitcoin_spv: ContractReference::from_canonical(canonical.bitcoin_spv, api)?,
            sfps: ContractReference::from_canonical(canonical.sfps, api)?,
            sbtc: ContractReference::from_canonical(canonical.sbtc, api)?,
            finance_admin: ContractReference::from_canonical(canonical.finance_admin, api)?,
            log: ContractReference::from_canonical(canonical.log, api)?,
            owner: HumanAddr::from_canonical(canonical.owner, api)?,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    CreateViewingKey {
        entropy: String,
    },
    SetViewingKey {
        key: viewing_key::ViewingKey,
    },

    RequestMintAddress {
        entropy: Binary,
    },
    VerifyMintTx {
        height: u32,
        tx: Binary,
        merkle_proof: bitcoin_spv::MerkleProofMsg,
    },
    ReleaseIncorrectAmountBTC {
        height: u32,
        tx: Binary,
        merkle_proof: bitcoin_spv::MerkleProofMsg,
        recipient_address: String,
        fee_per_vb: u64,
    },
    RequestReleaseBtc {
        entropy: Binary,
        amount: u64,
    },
    ClaimReleasedBtc {
        tx_result_proof: sfps::TxResultProof,
        header_hash_index: u64,
        encryption_key: Binary,
        recipient_address: String,
        fee_per_vb: u64,
    },
    ChangeFinanceAdmin {
        new_finance_admin: ContractReference,
    },
    ChangeOwner {
        new_owner: HumanAddr,
    },
    SetSuspensionSwitch {
        suspension_switch: SuspensionSwitch,
    },
    ReleaseBtcByOwner {
        tx_value: u64,
        max_input_length: u64,
        recipient_address: String,
        fee_per_vb: u64,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    CreateViewingKey { key: viewing_key::ViewingKey },
    RequestMintAddress { mint_address: String },
    ReleaseIncorrectAmountBTC { tx: Binary },
    ClaimReleasedBtc { tx: Binary },
    RequestReleaseBtc { request_key: RequestKey },
    ReleaseBtcByOwner { tx: Binary },
}

#[derive(Serialize, Deserialize, Clone, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    MintAddress {
        address: HumanAddr,
        key: viewing_key::ViewingKey,
    },
    SuspensionSwitch {},
    Config {},
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    MintAddress { address: Option<String> },
    Config(Config),
    SuspensionSwitch(SuspensionSwitch),
    ViewingKeyError { msg: String },
}

/// Bitcoin withdrawal request key.
/// It is sha256 hash of 'requester address + utxo + pseudorandom bytes'.
///
/// [IMPORTANT]
/// It must be unpredictable.
/// It must not leak any information about the used pseudorandom bytes and utxo at generation process.
///
/// The request key is provided to the requseter as the proof of the request, in the form of the response of the request transaction.
/// Therefore, the request key is published to the out of the contract.
/// At the claim phase, the requester send the request key to the contract so that the contract can verify the request.
#[derive(Debug, PartialEq, Eq, Copy, Clone, JsonSchema, Serialize, Deserialize)]
pub struct RequestKey([u8; 32]);

impl RequestKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
// true => supsend
pub struct SuspensionSwitch {
    pub request_mint_address: bool,
    pub verify_mint_tx: bool,
    pub release_incorrect_amount_btc: bool,
    pub request_release_btc: bool,
    pub claim_release_btc: bool,
}
