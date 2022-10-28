use crate::state::VestingInfo;
use crate::state::VestingSummary;
use cosmwasm_std::StdResult;
use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::utils::calls::HandleCallback;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
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

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct LockMsg {
    pub recipient: HumanAddr,
    pub contract_hash: String,
    pub end_time: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive(Snip20ReceiveMsg),
    Claim { id: u32 },
}

impl HandleCallback for Snip20ReceiveMsg {
    const BLOCK_SIZE: usize = 256;
}

impl Snip20ReceiveMsg {
    pub fn deserialize_msg<'a, T: DeserializeOwned>(&'a self) -> StdResult<Option<T>> {
        match &self.msg {
            Some(msg) => Ok(Some(cosmwasm_std::from_slice(msg.as_slice())?)),
            None => Ok(None),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Receive(VestingInfo),
    Unlock { id: u32 },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    LatestID {},
    VestingInfos {
        ids: Vec<u32>,
    },
    RecipientsVestingInfos {
        recipient: HumanAddr,
        page: u32,
        page_size: u32,
    },
    VestingSummary {
        token: HumanAddr,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    LatestID(u32),
    VestingInfos(Vec<VestingInfo>),
    AccountInfos(Vec<VestingInfo>),
    VestingSummary(VestingSummary),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_json_serde_receive_msg() {
        let lock_msg = LockMsg {
            recipient: "recipient".into(),
            contract_hash: "token_contract_hash".into(),
            end_time: 1660581394,
        };
        let handle_msg = HandleMsg::Receive(Snip20ReceiveMsg {
            sender: "token".into(),
            from: "token_sender".into(),
            amount: Uint128(500_000000),
            memo: None,
            msg: Some(serde_json::to_vec(&lock_msg).unwrap().into()),
        });

        assert_eq!(
            serde_json::to_string(&handle_msg).unwrap(),
            r#"{"receive":{"sender":"token","from":"token_sender","amount":"500000000","memo":null,"msg":"eyJyZWNpcGllbnQiOiJyZWNpcGllbnQiLCJjb250cmFjdF9oYXNoIjoidG9rZW5fY29udHJhY3RfaGFzaCIsImVuZF90aW1lIjoxNjYwNTgxMzk0fQ=="}}"#
        );

        if let HandleMsg::Receive(receive_msg) = handle_msg {
            assert_eq!(
                String::from_utf8(receive_msg.msg.as_ref().unwrap().clone().into()).unwrap(),
                r#"{"recipient":"recipient","contract_hash":"token_contract_hash","end_time":1660581394}"#
            );
            let deserialized_lock_msg: LockMsg = receive_msg.deserialize_msg().unwrap().unwrap();
            assert_eq!(deserialized_lock_msg, lock_msg);
        } else {
            unreachable!();
        }
    }
}
