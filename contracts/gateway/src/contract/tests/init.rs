// Init Tests
use super::*;
use crate::state::config::read_config;
use crate::state::prefix::PREFIX_PRNG;
use common_macros::hash_set;
use cosmwasm_std::{Binary, ReadonlyStorage};
use shared_types::gateway::{Config, InitMsg};
use shared_types::ContractReference;

#[test]
fn test_init_sanity() {
    let mut deps = mock_dependencies(20, &[]);
    let env = helper::mock_env("instantiator", &[]);

    let init_msg = InitMsg {
        entropy: Binary::from("lolz fun yay".as_bytes()),
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
                address: "log_addr".into(),
                hash: "log_hash".into(),
            },
            owner: "owner".into(),
        },
    };
    init(&mut deps, env, init_msg).unwrap();
    let config = read_config(&deps.storage, &deps.api).unwrap();

    assert_eq!(config.btc_tx_values, hash_set! {100000000, 10000000});
    let seed = &deps.storage.get(PREFIX_PRNG).unwrap();
    assert_eq!(
        seed.as_slice(),
        [
            250, 77, 51, 7, 154, 121, 199, 140, 180, 209, 102, 1, 252, 205, 187, 38, 103, 33, 184,
            233, 83, 94, 220, 114, 125, 169, 83, 48, 170, 86, 75, 247
        ]
    );
}
