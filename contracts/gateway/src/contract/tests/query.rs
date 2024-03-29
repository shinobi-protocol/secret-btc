// Bitcoin Handle Tests
use super::*;
use common_macros::hash_set;
use contract_test_utils::contract_runner::ContractRunner;
use cosmwasm_std::from_binary;
use shared_types::gateway::*;
use std::string::ToString;

#[test]
fn test_query_config() {
    let mut context = init_helper();
    let query_msg = QueryMsg::Config {};
    let query_result = GatewayRunner::run_query(&mut context, query_msg).unwrap();
    match from_binary(&query_result).unwrap() {
        QueryAnswer::Config(Config {
            btc_tx_values,
            bitcoin_spv,
            sfps,
            sbtc,
            log,
            owner,
            state_proxy,
        }) => {
            assert_eq!(btc_tx_values, hash_set! {100000000, 10000000});
            assert_eq!(bitcoin_spv.address, "spv_address".into());
            assert_eq!(bitcoin_spv.hash, "spv_hash".to_string());
            assert_eq!(sfps.address, "sfps_address".into());
            assert_eq!(sfps.hash, "sfps_hash".to_string());
            assert_eq!(sbtc.address, "sbtc_address".into());
            assert_eq!(sbtc.hash, "sbtc_hash".to_string());
            assert_eq!(log.address, "log_address".into());
            assert_eq!(log.hash, "log_hash".to_string());
            assert_eq!(state_proxy.address, "state_proxy_address".into());
            assert_eq!(state_proxy.hash, "state_proxy_hash".to_string());
            assert_eq!(owner, "owner".into());
        }
        _ => panic!("Unexpected"),
    };
}

#[test]
fn test_query_suspension_switch() {
    let mut context = init_helper();
    let query_msg = QueryMsg::SuspensionSwitch {};
    match from_binary(&GatewayRunner::run_query(&mut context, query_msg.clone()).unwrap()).unwrap()
    {
        QueryAnswer::SuspensionSwitch(SuspensionSwitch {
            request_mint_address,
            verify_mint_tx,
            release_incorrect_amount_btc,
            request_release_btc,
            claim_release_btc,
        }) => {
            assert_eq!(request_mint_address, false);
            assert_eq!(verify_mint_tx, false);
            assert_eq!(release_incorrect_amount_btc, false);
            assert_eq!(request_release_btc, false);
            assert_eq!(claim_release_btc, false);
        }
        _ => panic!("Unexpected"),
    };
    GatewayRunner::run_handle(
        &mut context,
        mock_env("owner", &[]),
        HandleMsg::SetSuspensionSwitch {
            suspension_switch: SuspensionSwitch {
                request_mint_address: true,
                verify_mint_tx: false,
                release_incorrect_amount_btc: true,
                request_release_btc: false,
                claim_release_btc: true,
            },
        },
    )
    .unwrap();
    match from_binary(&GatewayRunner::run_query(&mut context, query_msg.clone()).unwrap()).unwrap()
    {
        QueryAnswer::SuspensionSwitch(SuspensionSwitch {
            request_mint_address,
            verify_mint_tx,
            release_incorrect_amount_btc,
            request_release_btc,
            claim_release_btc,
        }) => {
            assert_eq!(request_mint_address, true);
            assert_eq!(verify_mint_tx, false);
            assert_eq!(release_incorrect_amount_btc, true);
            assert_eq!(request_release_btc, false);
            assert_eq!(claim_release_btc, true);
        }
        _ => panic!("Unexpected"),
    };
}
