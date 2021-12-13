// Init Tests
use super::*;
use crate::state::config::read_config;
use shared_types::bitcoin_spv::InitMsg;

#[test]
fn test_init_sanity() {
    let mut deps = mock_dependencies(20, &[]);
    let env = helper::mock_env("instantiator", &[]);

    let init_msg = InitMsg {
        bitcoin_network: "regtest".to_string(),
        initial_header: None,
        confirmation: 6,
    };
    init(&mut deps, env, init_msg).unwrap();
    let config = read_config(&deps.storage).unwrap();

    assert_eq!(config.bitcoin_network, "regtest");
    assert_eq!(config.confirmation, 6);
}
