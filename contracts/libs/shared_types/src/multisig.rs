use crate::Canonicalize;
use cosmwasm_std::{Api, CanonicalAddr, CosmosMsg, HumanAddr, StdError, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub struct TransactionStatus {
    pub msg: CosmosMsg,
    pub config: Config,
    pub signed_by: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CanonicalTransactionStatus {
    pub msg: CosmosMsg,
    pub config: CanonicalConfig,
    pub signed_by: Vec<usize>,
}

impl TransactionStatus {
    pub fn sign_count(&self) -> usize {
        self.signed_by.len()
    }
    pub fn is_confirmed(&self) -> bool {
        self.config.required as usize <= self.sign_count()
    }

    pub fn sign(&mut self, signer: &HumanAddr) -> StdResult<()> {
        if let Some(position) = self
            .config
            .signers
            .iter_mut()
            .position(|item| item == signer)
        {
            if self.signed_by.contains(&position) {
                return Err(StdError::generic_err("already signed"));
            }
            self.signed_by.push(position);
            Ok(())
        } else {
            Err(StdError::generic_err("not signer"))
        }
    }
}

impl Canonicalize for TransactionStatus {
    type Canonicalized = CanonicalTransactionStatus;
    fn into_canonical<A: Api>(self, api: &A) -> StdResult<CanonicalTransactionStatus> {
        Ok(CanonicalTransactionStatus {
            msg: self.msg,
            config: self.config.into_canonical(api)?,
            signed_by: self.signed_by,
        })
    }
    fn from_canonical<A: Api>(canonical: Self::Canonicalized, api: &A) -> StdResult<Self> {
        Ok(Self {
            msg: canonical.msg,
            config: Config::from_canonical(canonical.config, api)?,
            signed_by: canonical.signed_by,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub struct Config {
    pub signers: Vec<HumanAddr>,
    pub required: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CanonicalConfig {
    pub signers: Vec<CanonicalAddr>,
    pub required: u32,
}

impl Canonicalize for Config {
    type Canonicalized = CanonicalConfig;
    fn into_canonical<A: Api>(self, api: &A) -> StdResult<CanonicalConfig> {
        Ok(CanonicalConfig {
            required: self.required,
            signers: self
                .signers
                .iter()
                .map(|signer| api.canonical_address(signer))
                .collect::<StdResult<Vec<CanonicalAddr>>>()?,
        })
    }
    fn from_canonical<A: Api>(canonical: CanonicalConfig, api: &A) -> StdResult<Self> {
        Ok(Self {
            required: canonical.required,
            signers: canonical
                .signers
                .iter()
                .map(|signer| api.human_address(signer))
                .collect::<StdResult<Vec<HumanAddr>>>()?,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq)]
pub struct MultisigStatus {
    pub config: Config,
    pub transaction_count: u32,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub config: Config,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    ChangeConfig { config: Config },
    SubmitTransaction { msg: CosmosMsg },
    SignTransaction { transaction_id: u32 },
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    TransactionStatus { transaction_id: u32 },
    MultisigStatus {},
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    SubmitTransaction { transaction_id: u32 },
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    TransactionStatus(TransactionStatus),
    MultisigStatus(MultisigStatus),
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::*;
    use cosmwasm_std::*;

    fn transaction_status() -> TransactionStatus {
        TransactionStatus {
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "contract_addr".into(),

                callback_code_hash: "callback_code_hash".into(),
                msg: Binary::from(&[0, 1, 2]),
                send: vec![Coin::new(100u128, "uscrt")],
            }),
            config: Config {
                signers: vec!["signer1".into(), "signer2".into(), "signer3".into()],
                required: 2,
            },
            signed_by: vec![1],
        }
    }

    #[test]
    fn canonicalize_transaction_status() {
        let api = MockApi::default();
        let expected = transaction_status();
        let actual = TransactionStatus::from_canonical(
            transaction_status().into_canonical(&api).unwrap(),
            &api,
        )
        .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn canonical_transaction_status_sign() {
        let mut status = transaction_status();
        assert_eq!(status.sign_count(), 1);
        assert_eq!(status.is_confirmed(), false);
        assert_eq!(
            status.sign(&"signer2".into()).unwrap_err(),
            StdError::generic_err("already signed")
        );
        status.sign(&"signer1".into()).unwrap();
        assert_eq!(status.signed_by, [1, 0]);
        assert_eq!(status.sign_count(), 2);
        assert_eq!(status.is_confirmed(), true);
        assert_eq!(
            status.sign(&"signer1".into()).unwrap_err(),
            StdError::generic_err("already signed")
        );
        assert_eq!(
            status.sign(&"signer2".into()).unwrap_err(),
            StdError::generic_err("already signed")
        );
        assert_eq!(
            status.sign(&"foreigner".into()).unwrap_err(),
            StdError::generic_err("not signer")
        );
        status.sign(&"signer3".into()).unwrap();
        assert_eq!(status.signed_by, [1, 0, 2]);
        assert_eq!(status.sign_count(), 3);
        assert_eq!(status.is_confirmed(), true);
    }
}
