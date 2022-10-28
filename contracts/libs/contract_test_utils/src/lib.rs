use cosmwasm_std::{
    from_binary, testing::MockApi, BlockInfo, Coin, ContractInfo, CosmosMsg, Env, HumanAddr,
    MessageInfo, WasmMsg,
};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

pub mod context;
pub mod contract_runner;
pub mod querier;
pub mod storage;

pub fn mock_api() -> MockApi {
    MockApi::new(20)
}

pub fn mock_timestamp() -> u32 {
    1_610_794_295
}

pub fn mock_env<U: Into<HumanAddr>>(sender: U, sent: &[Coin]) -> Env {
    Env {
        block: BlockInfo {
            height: 12_345,
            // change time
            time: mock_timestamp() as u64,
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        message: MessageInfo {
            sender: sender.into(),
            sent_funds: sent.to_vec(),
        },
        contract: ContractInfo {
            address: HumanAddr::from(cosmwasm_std::testing::MOCK_CONTRACT_ADDR),
        },
        contract_key: Some("".to_string()),
        contract_code_hash: "".to_string(),
    }
}

pub fn assert_handle_response_message<H>(
    response_message: &CosmosMsg,
    expected_contract_addr: &str,
    expected_callback_code_hash: &str,
    expected_handle_msg: &H,
) where
    H: DeserializeOwned + PartialEq + Debug,
{
    match response_message {
        CosmosMsg::Wasm(msg) => match msg {
            WasmMsg::Execute {
                contract_addr,
                callback_code_hash,
                msg,
                ..
            } => {
                assert_eq!(contract_addr.as_str(), expected_contract_addr);
                assert_eq!(callback_code_hash.as_str(), expected_callback_code_hash);
                let handle_msg: H = from_binary(&msg).unwrap();
                assert_eq!(&handle_msg, expected_handle_msg);
            }
            _ => unreachable!("WasmMsg is not WasmMsg::Execute"),
        },
        _ => unreachable!("CosmosMsg is not CosmosMsg::Wasm"),
    }
}
