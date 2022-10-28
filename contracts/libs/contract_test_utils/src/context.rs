use crate::mock_api;
use crate::querier::{MockQuerier, QueryCases};
use crate::storage::{MutableStorageWrapper, NamedMockStorage};
use cosmwasm_std::{from_binary, testing::*, CosmosMsg, HumanAddr, Storage, WasmMsg};
use cosmwasm_std::{Extern, WasmQuery};
use serde::Serialize;
use shared_types::{state_proxy::server::Secp256k1ApiVerifier, ContractReference};

pub type ClientDepsStorage<'a> = MutableStorageWrapper<'a>;
pub type ClientDepsApi = MockApi;
pub type ClientDepsQuerier<'a> =
    MockQuerier<'a, 'a, MockApi, NamedMockStorage, Secp256k1ApiVerifier<'a, MockApi>>;

pub type ClientDeps<'a> = Extern<ClientDepsStorage<'a>, ClientDepsApi, ClientDepsQuerier<'a>>;

pub struct Context {
    pub contract_storage: NamedMockStorage,
    pub state_proxy_server_storage: NamedMockStorage,
    pub query_cases: QueryCases,
    pub mock_api: MockApi,
}

pub const STATE_PROXY_CONTRACT_ADDRESS: &str = "state_proxy_address";
pub const STATE_PROXY_CONTRACT_HASH: &str = "state_proxy_hash";
pub const CLIENT_CONTRACT_ADDRESS: &str = "client_contract";

impl Context {
    pub fn new<R: Serialize>(cases: Vec<(WasmQuery, R)>) -> Self {
        Self {
            contract_storage: NamedMockStorage::new("contract"),
            state_proxy_server_storage: NamedMockStorage::new("proxy"),
            query_cases: QueryCases::from(cases),
            mock_api: mock_api(),
        }
    }
    pub fn client_deps(&mut self) -> ClientDeps {
        let querier = MockQuerier::new(
            ContractReference {
                address: STATE_PROXY_CONTRACT_ADDRESS.into(),
                hash: STATE_PROXY_CONTRACT_HASH.into(),
            },
            &self.state_proxy_server_storage,
            Secp256k1ApiVerifier::new(&self.mock_api),
            &self.mock_api,
            self.query_cases.clone(),
        );
        let storage = MutableStorageWrapper {
            storage: &mut self.contract_storage,
        };
        let deps = Extern {
            storage,
            api: mock_api(),
            querier,
        };
        deps
    }

    pub fn server_deps(
        &mut self,
    ) -> Extern<MutableStorageWrapper, MockApi, cosmwasm_std::testing::MockQuerier> {
        let querier = cosmwasm_std::testing::MockQuerier::new(&[]);
        let storage = MutableStorageWrapper {
            storage: &mut self.state_proxy_server_storage,
        };
        let deps = Extern {
            storage,
            api: mock_api(),
            querier,
        };
        deps
    }

    pub fn exec_state_contract_messages(&mut self, messages: &[CosmosMsg]) {
        for message in messages {
            if let CosmosMsg::Wasm(WasmMsg::Execute { msg, .. }) = message {
                if let Ok(handle_msg) = from_binary(&msg) {
                    let mut deps = self.server_deps();
                    shared_types::state_proxy::server::handle(
                        &mut deps,
                        mock_env(CLIENT_CONTRACT_ADDRESS, &[]),
                        handle_msg,
                    )
                    .unwrap();
                }
            }
        }
    }
}

/*
pub fn write_to_server_storage<S: Storage>(storage: &mut S, deps: &ContextDeps) {
    if let CosmosMsg::Wasm(WasmMsg::Execute { msg, .. }) = cosmos_msg {
        if let shared_types::state_proxy::msg::HandleMsg::WriteContractState {
            contract_label,
            transaction,
        } = from_binary(&msg).unwrap()
        {
            commit_transaction_to_storage(
                storage,
                contract_label.as_slice(),
                &HumanAddr::from(MOCK_CONTRACT_ADDR),
                &transaction,
                &mock_api(),
            )
            .unwrap();
            return;
        } else {
            unreachable!()
        }
    }
    unreachable!()
}
*/
