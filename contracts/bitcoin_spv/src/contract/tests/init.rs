// Init Tests
use super::*;
use crate::state::config::read_config;
use contract_test_utils::contract_runner::ContractRunner;
use cosmwasm_std::from_binary;
use shared_types::bitcoin_spv::{QueryAnswer, QueryMsg};

#[test]
fn test_init_sanity() {
    let mut context = init_helper();

    let config =
        match from_binary(&BitcoinSPVRunner::run_query(&mut context, QueryMsg::Config {}).unwrap())
            .unwrap()
        {
            QueryAnswer::Config(config) => config,
            _ => unreachable!(),
        };

    assert_eq!(config.bitcoin_network, "regtest");
    assert_eq!(config.confirmation, 6);
}
