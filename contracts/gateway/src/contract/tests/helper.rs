use super::*;
use common_macros::hash_set;
use contract_test_utils::context::{
    ClientDeps, Context, STATE_PROXY_CONTRACT_ADDRESS, STATE_PROXY_CONTRACT_HASH,
};
use contract_test_utils::contract_runner::ContractRunner;
use contract_test_utils::mock_env;
use cosmwasm_std::{to_binary, Binary, Env, HandleResult, InitResponse, StdResult, WasmQuery};
use secret_toolkit::utils::space_pad;
use serde::Serialize;
use shared_types::gateway::HandleMsg;
use shared_types::gateway::QueryMsg;
use shared_types::gateway::{Config, InitMsg};
use shared_types::state_proxy::client::Seed;
use shared_types::{bitcoin_spv, ContractReference};

pub fn to_padded_binary<S: Serialize>(item: &S) -> StdResult<Binary> {
    let mut bin = to_binary(item)?;
    space_pad(&mut bin.0, 256);
    Ok(bin)
}

pub fn init_helper() -> Context {
    let mut context = Context::new(vec![(
        WasmQuery::Smart {
            msg: to_padded_binary(&bitcoin_spv::QueryMsg::Config {}).unwrap(),
            contract_addr: "spv_address".into(),
            callback_code_hash: "spv_hash".into(),
        },
        bitcoin_spv::QueryAnswer::Config(bitcoin_spv::Config {
            bitcoin_network: "regtest".into(),
            confirmation: 6,
            state_proxy: ContractReference {
                address: STATE_PROXY_CONTRACT_ADDRESS.into(),
                hash: STATE_PROXY_CONTRACT_HASH.into(),
            },
        }),
    )]);

    let env = mock_env("instantiator", &[]);

    let init_msg = InitMsg {
        seed: Seed::default(),
        config: Config {
            btc_tx_values: hash_set! {100000000, 10000000}, //1BTC, 0.1BTC
            bitcoin_spv: ContractReference {
                address: "spv_address".into(),
                hash: "spv_hash".into(),
            },
            sfps: ContractReference {
                address: "sfps_address".into(),
                hash: "sfps_hash".into(),
            },
            sbtc: ContractReference {
                address: "sbtc_address".into(),
                hash: "sbtc_hash".into(),
            },
            log: ContractReference {
                address: "log_address".into(),
                hash: "log_hash".into(),
            },
            state_proxy: ContractReference {
                address: STATE_PROXY_CONTRACT_ADDRESS.into(),
                hash: STATE_PROXY_CONTRACT_HASH.into(),
            },
            owner: "owner".into(),
        },
    };
    GatewayRunner::run_init(&mut context, env, init_msg).unwrap();
    context
}

pub struct GatewayRunner {}

impl contract_test_utils::contract_runner::ContractRunner for GatewayRunner {
    type InitMsg = InitMsg;
    type HandleMsg = HandleMsg;
    type QueryMsg = QueryMsg;

    fn init(deps: &mut ClientDeps, env: Env, msg: InitMsg) -> StdResult<InitResponse> {
        init(deps, env, msg)
    }

    fn handle(deps: &mut ClientDeps, env: Env, msg: HandleMsg) -> HandleResult {
        handle(deps, env, msg)
    }

    fn query(deps: &ClientDeps, msg: QueryMsg) -> StdResult<Binary> {
        query(deps, msg)
    }
}
