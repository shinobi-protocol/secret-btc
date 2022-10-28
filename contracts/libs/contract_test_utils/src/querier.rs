use cosmwasm_std::Api;
use cosmwasm_std::Storage;
use cosmwasm_std::{
    from_binary, from_slice, to_binary, to_vec, Binary, Empty, Querier, QuerierResult,
    QueryRequest, StdError, StdResult, WasmQuery,
};
use serde::Serialize;
use shared_types::state_proxy::msg::{QueryAnswer, QueryMsg, Secp256k1Verifier};
use shared_types::state_proxy::server::read_contract_state;
use shared_types::ContractReference;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct QueryCases(HashMap<Vec<u8>, Result<Binary, String>>);

impl QueryCases {
    pub fn from<R: Serialize>(cases: Vec<(WasmQuery, R)>) -> Self {
        let mut query_cases = Self(HashMap::new());
        for case in cases {
            query_cases.add_case(case.0, case.1)
        }
        query_cases
    }

    pub fn add_case<R: Serialize>(&mut self, request: WasmQuery, result: R) {
        self.0.insert(
            to_vec(&QueryRequest::<Empty>::Wasm(request)).unwrap(),
            Ok(to_binary(&result).unwrap()),
        );
    }

    pub fn add_error_case(&mut self, request: WasmQuery, error: String) {
        self.0.insert(
            to_vec(&QueryRequest::<Empty>::Wasm(request)).unwrap(),
            Err(error),
        );
    }
}

#[derive(Debug)]
pub struct MockQuerier<'a, 'b, A: Api, S: Storage, V: Secp256k1Verifier> {
    state_proxy_server_reference: ContractReference,
    state_proxy_server_storage: &'a S,
    secp256k1_verifier: V,
    api: &'b A,
    cases: QueryCases,
}

impl<'a, 'b, A: Api, S: Storage, V: Secp256k1Verifier> MockQuerier<'a, 'b, A, S, V> {
    pub fn new(
        state_proxy_server_reference: ContractReference,
        state_proxy_server_storage: &'a S,
        secp256k1_verifier: V,
        api: &'b A,
        cases: QueryCases,
    ) -> Self {
        Self {
            state_proxy_server_reference,
            state_proxy_server_storage,
            secp256k1_verifier,
            api,
            cases,
        }
    }

    pub fn add_case<R: Serialize>(&mut self, request: WasmQuery, result: R) {
        self.cases.add_case(request, result);
    }

    pub fn reset_cases(&mut self) {
        self.cases = QueryCases::from::<()>(vec![]);
    }

    fn mock_query(&self, bin_request: &[u8]) -> StdResult<Binary> {
        if let QueryRequest::<Empty>::Wasm(WasmQuery::Smart {
            contract_addr,
            callback_code_hash,
            msg,
        }) = from_slice(bin_request)?
        {
            if contract_addr == self.state_proxy_server_reference.address
                && callback_code_hash == self.state_proxy_server_reference.hash
            {
                if let QueryMsg::ReadContractState { signature, key } = from_binary(&msg)? {
                    let value = read_contract_state(
                        self.state_proxy_server_storage,
                        signature,
                        key.as_slice(),
                        &self.secp256k1_verifier,
                        self.api,
                    )?;
                    return to_binary(&QueryAnswer::ReadContractState {
                        value: value.map(|v| Binary::from(v)),
                    });
                }
            }
        }
        match self.cases.0.get(bin_request) {
            Some(result) => result.clone().map_err(|e| StdError::generic_err(e)),
            None => Err(StdError::generic_err(format!(
                "request is not in expected case: {}",
                String::from_utf8(bin_request.to_vec()).unwrap()
            ))),
        }
    }
}

impl<'a, 'b, A: Api, S: Storage, V: Secp256k1Verifier> Querier for MockQuerier<'a, 'b, A, S, V> {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        Ok(self.mock_query(bin_request))
    }
}
