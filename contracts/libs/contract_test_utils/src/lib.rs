use cosmwasm_std::{
    from_binary, to_binary, to_vec, Binary, CosmosMsg, Empty, Querier, QuerierResult, QueryRequest,
    StdError, StdResult, WasmMsg, WasmQuery,
};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

pub struct MockQuerier {
    cases: HashMap<Vec<u8>, Result<Binary, String>>,
}

impl MockQuerier {
    pub fn new() -> Self {
        Self {
            cases: HashMap::default(),
        }
    }

    pub fn add_case<R: Serialize>(&mut self, request: WasmQuery, result: R) {
        self.cases.insert(
            to_vec(&QueryRequest::<Empty>::Wasm(request)).unwrap(),
            Ok(to_binary(&result).unwrap()),
        );
    }

    pub fn add_error_case(&mut self, request: WasmQuery, error: String) {
        self.cases.insert(
            to_vec(&QueryRequest::<Empty>::Wasm(request)).unwrap(),
            Err(error),
        );
    }

    pub fn reset_cases(&mut self) {
        self.cases = HashMap::default();
    }

    fn mock_query(&self, bin_request: &[u8]) -> StdResult<Binary> {
        match self.cases.get(bin_request) {
            Some(result) => result.clone().map_err(|e| StdError::generic_err(e)),
            None => Err(StdError::generic_err(format!(
                "request is not in expected case: {}",
                String::from_utf8(bin_request.to_vec()).unwrap()
            ))),
        }
    }
}

impl Querier for MockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        Ok(self.mock_query(bin_request))
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
