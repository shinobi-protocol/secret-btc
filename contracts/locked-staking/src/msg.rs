use cosmwasm_std::StdResult;
use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::utils::calls::HandleCallback;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::state::{StakingInfo, StakingSummary};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct InitMsg {}

/// Snip20ReceiveMsg should be de/serialized under `Receive()` variant in a HandleMsg
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Snip20ReceiveMsg {
    pub sender: HumanAddr,
    pub from: HumanAddr,
    pub amount: Uint128,
    pub memo: Option<String>,
    pub msg: Option<Binary>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct LockMsg {
    pub recipient: HumanAddr,
    pub contract_hash: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Public(PublicHandleMsg),
    Admin(AdminHandleMsg),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PublicHandleMsg {
    Receive(Snip20ReceiveMsg),
    Unlock { ids: Vec<u32> },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AdminHandleMsg {
    ChangeAdmin { new_admin: HumanAddr },
    SetStakingEndTime { token: HumanAddr, end_time: u64 },
}

impl HandleCallback for Snip20ReceiveMsg {
    const BLOCK_SIZE: usize = 256;
}

impl Snip20ReceiveMsg {
    pub fn deserialize_msg<T: DeserializeOwned>(&self) -> StdResult<Option<T>> {
        match &self.msg {
            Some(msg) => Ok(Some(cosmwasm_std::from_slice(msg.as_slice())?)),
            None => Ok(None),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Receive(StakingInfo),
    Unlock { id: u32 },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    LatestID {},
    StakingInfos {
        ids: Vec<u32>,
    },
    RecipientsStakingInfos {
        recipient: HumanAddr,
        page: u32,
        page_size: u32,
    },
    StakingSummary {
        token: HumanAddr,
    },
    Admin {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    LatestID(u32),
    StakingInfos(Vec<StakingInfo>),
    AccountInfos(Vec<StakingInfo>),
    StakingSummary(StakingSummary),
    Admin(HumanAddr),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_json_serde_receive_msg() {
        let lock_msg = LockMsg {
            recipient: "recipient".into(),
            contract_hash: "token_contract_hash".into(),
        };
        let handle_msg = HandleMsg::Public(PublicHandleMsg::Receive(Snip20ReceiveMsg {
            sender: "token".into(),
            from: "token_sender".into(),
            amount: Uint128(500_000000),
            memo: None,
            msg: Some(serde_json::to_vec(&lock_msg).unwrap().into()),
        }));

        assert_eq!(
            serde_json::to_string(&handle_msg).unwrap(),
            r#"{"public":{"receive":{"sender":"token","from":"token_sender","amount":"500000000","memo":null,"msg":"eyJyZWNpcGllbnQiOiJyZWNpcGllbnQiLCJjb250cmFjdF9oYXNoIjoidG9rZW5fY29udHJhY3RfaGFzaCJ9"}}}"#
        );

        if let HandleMsg::Public(PublicHandleMsg::Receive(receive_msg)) = handle_msg {
            assert_eq!(
                String::from_utf8(receive_msg.msg.as_ref().unwrap().clone().into()).unwrap(),
                r#"{"recipient":"recipient","contract_hash":"token_contract_hash"}"#
            );
            let deserialized_lock_msg: LockMsg = receive_msg.deserialize_msg().unwrap().unwrap();
            assert_eq!(deserialized_lock_msg, lock_msg);
        } else {
            unreachable!();
        }
    }
}
