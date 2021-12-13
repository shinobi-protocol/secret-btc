// Bitcoin Handle Tests
use super::*;
use common_macros::hash_set;
use cosmwasm_std::from_binary;
use shared_types::gateway::*;
use std::string::ToString;

#[test]
fn test_query_config() {
    let deps = init_helper();
    let query_msg = QueryMsg::Config {};
    let query_result = query(&deps, query_msg);
    match from_binary(&query_result.unwrap()).unwrap() {
        QueryAnswer::Config(Config {
            btc_tx_values,
            bitcoin_spv,
            sfps,
            sbtc,
            finance_admin,
            log,
        }) => {
            assert_eq!(btc_tx_values, hash_set! {100000000, 10000000});
            assert_eq!(bitcoin_spv.address, "spv_address".into());
            assert_eq!(bitcoin_spv.hash, "spv_hash".to_string());
            assert_eq!(sfps.address, "sfps_address".into());
            assert_eq!(sfps.hash, "sfps_hash".to_string());
            assert_eq!(sbtc.address, "sbtc_address".into());
            assert_eq!(sbtc.hash, "sbtc_hash".to_string());
            assert_eq!(finance_admin.address, "finance_a_addr".into());
            assert_eq!(finance_admin.hash, "finance_admin_hash".to_string());
            assert_eq!(log.address, "log_address".into());
            assert_eq!(log.hash, "log_hash".to_string());
        }
        _ => panic!("Unexpected"),
    };
}
