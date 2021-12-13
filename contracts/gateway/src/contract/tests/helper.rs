use super::*;
use common_macros::hash_set;
use contract_test_utils::MockQuerier;
use cosmwasm_std::{
    to_binary, Binary, BlockInfo, Coin, ContractInfo, Env, Extern, HumanAddr, MessageInfo,
    StdResult, WasmQuery,
};
use secret_toolkit::utils::space_pad;
use serde::Serialize;
use shared_types::gateway::{Config, InitMsg};
use shared_types::{bitcoin_spv, ContractReference};
use std::string::ToString;

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
            address: HumanAddr::from(MOCK_CONTRACT_ADDR),
        },
        contract_key: Some("".to_string()),
        contract_code_hash: "".to_string(),
    }
}

pub fn mock_timestamp() -> u32 {
    1_610_794_295
}

pub fn init_helper() -> Extern<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies(20, &[]);
    let env = mock_env("instantiator", &[]);

    let init_msg = InitMsg {
        prng_seed: Binary::from("lolz fun yay".as_bytes()),
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
            finance_admin: ContractReference {
                address: "finance_a_addr".into(),
                hash: "finance_admin_hash".into(),
            },
            log: ContractReference {
                address: "log_address".into(),
                hash: "log_hash".into(),
            },
        },
    };

    init(&mut deps, env, init_msg).unwrap();

    let mut querier = MockQuerier::new();
    querier.add_case(
        WasmQuery::Smart {
            msg: to_padded_binary(&bitcoin_spv::QueryMsg::Config {}).unwrap(),
            contract_addr: "spv_address".into(),
            callback_code_hash: "spv_hash".into(),
        },
        bitcoin_spv::QueryAnswer::Config(bitcoin_spv::Config {
            bitcoin_network: "regtest".into(),
            confirmation: 6,
        }),
    );

    Extern {
        storage: deps.storage,
        api: deps.api,
        querier: querier,
    }
}

pub fn to_padded_binary<S: Serialize>(item: &S) -> StdResult<Binary> {
    let mut bin = to_binary(item)?;
    space_pad(&mut bin.0, 256);
    Ok(bin)
}
