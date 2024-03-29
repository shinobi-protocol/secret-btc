use super::*;
use crate::state::bitcoin_utxo::gen_request_key;
use crate::state::bitcoin_utxo::{
    read_release_request_utxo, write_release_request_utxo, RequestedUtxo, Utxo, UtxoQueue,
};
use crate::state::config::read_config;
use crate::state::mint_key::{read_mint_key, write_mint_key};
use crate::state::suspension_switch::set_suspension_switch;
use crate::state::suspension_switch::suspension_switch;
use bitcoin::consensus::encode::{deserialize, serialize};
use bitcoin::hash_types::Txid;
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::SecretKey;
use bitcoin::{Address, Network, PrivateKey, Transaction, TxOut};
use contract_test_utils::contract_runner::ContractRunner;
use contract_test_utils::mock_timestamp;
use cosmwasm_std::{from_binary, to_binary, Api, Binary, StdError, WasmQuery};
use rand::{thread_rng, Rng};
use secret_toolkit::{snip20, utils::HandleCallback};
use shared_types::gateway::*;
use shared_types::state_proxy::client::{Secp256k1ApiSigner, StateProxyDeps};
use shared_types::{bitcoin_spv, log, sfps, BLOCK_SIZE};
use std::string::ToString;

/// wrapper to serialize/deserialize snip20 TokenInfo response
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TokenInfoResponse {
    token_info: snip20::TokenInfo,
}

//let handle_wrapper =

#[test]
fn test_request_mint_address_sanity() {
    let mut context = init_helper();
    //  handle
    let handle_msg = HandleMsg::RequestMintAddress {
        entropy: Binary::from(b"entropy"),
    };

    // assert response
    let handle_response = GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("bob", &[]),
        handle_msg,
    )
    .unwrap();
    let mint_address: String = match from_binary(&handle_response.data.unwrap()).unwrap() {
        HandleAnswer::RequestMintAddress { mint_address } => mint_address,
        _ => panic!("Unexpected"),
    };
    assert_eq!(mint_address, "bcrt1q0r489mvjxujmd2ufss7av3ch2p3x0y856yt3y7");
    assert_eq!(handle_response.messages.len(), 2);
    assert_eq!(
        handle_response.messages[1],
        log::HandleMsg::AddEvents {
            events: vec![(
                "bob".into(),
                log::Event::MintStarted(log::event::MintStartedData {
                    time: contract_test_utils::mock_timestamp() as u64,
                    address: mint_address.to_string(),
                }),
            )],
        }
        .to_cosmos_msg("log_hash".into(), "log_address".into(), None)
        .unwrap()
    );

    let canonical_addr = context.mock_api.canonical_address(&"bob".into()).unwrap();
    // assert states
    let deps = context.client_deps();
    let deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )
    .unwrap();
    let mint_key = read_mint_key(&deps.storage, &canonical_addr, Network::Regtest)
        .unwrap()
        .unwrap();
    assert_eq!(
        format!("{:x}", mint_key.key),
        "11e9035b2dd043fca629844208d7b0fba5a2beccfdcaa43607fc8853c5e06e3b"
    );
    let address = Address::p2wpkh(&mint_key.public_key(), mint_key.network).unwrap();
    assert_eq!(address.to_string(), mint_address);
}

#[test]
fn test_suspend_request_mint_address() {
    let mut context = init_helper();
    GatewayRunner::run_handle(
        &mut context,
        mock_env("owner", &[]),
        HandleMsg::SetSuspensionSwitch {
            suspension_switch: SuspensionSwitch {
                request_mint_address: true,
                verify_mint_tx: false,
                release_incorrect_amount_btc: false,
                request_release_btc: false,
                claim_release_btc: false,
            },
        },
    )
    .unwrap();
    let err = GatewayRunner::run_handle(
        &mut context,
        mock_env("bob", &[]),
        HandleMsg::RequestMintAddress {
            entropy: Binary::from(b"entropy"),
        },
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Generic error: contract error request mint address is being suspended"
    );
}

#[test]
fn test_request_mint_address_twice_with_same_entropy() {
    let mut context = init_helper();

    // request 1
    let handle_msg = HandleMsg::RequestMintAddress {
        entropy: Binary::from(b"entropy"),
    };
    let handle_result = GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("bob", &[]),
        handle_msg,
    );
    let mint_address: String = match from_binary(&handle_result.unwrap().data.unwrap()).unwrap() {
        HandleAnswer::RequestMintAddress { mint_address } => mint_address,
        _ => panic!("Unexpected"),
    };
    assert_eq!(mint_address, "bcrt1q0r489mvjxujmd2ufss7av3ch2p3x0y856yt3y7");

    // request 2
    let handle_msg = HandleMsg::RequestMintAddress {
        entropy: Binary::from(b"entropy"),
    };
    let handle_result = GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("bob", &[]),
        handle_msg,
    );
    let mint_address: String = match from_binary(&handle_result.unwrap().data.unwrap()).unwrap() {
        HandleAnswer::RequestMintAddress { mint_address } => mint_address,
        _ => panic!("Unexpected"),
    };
    assert_eq!(mint_address, "bcrt1qstdvzcekutkmy8qtzlt3e7xxh60v6p3fy29f9z");
}

#[test]
fn test_request_mint_address_to_same_state_from_different_account() {
    let mut bob_context = init_helper();
    let mut lebron_context = init_helper();

    let entropy = Binary::from(b"entropy");

    // from bob
    let handle_msg = HandleMsg::RequestMintAddress {
        entropy: entropy.clone(),
    };
    let handle_result = GatewayRunner::run_handle(
        &mut bob_context,
        contract_test_utils::mock_env("bob", &[]),
        handle_msg,
    );
    let mint_address: String = match from_binary(&handle_result.unwrap().data.unwrap()).unwrap() {
        HandleAnswer::RequestMintAddress { mint_address } => mint_address,
        _ => panic!("Unexpected"),
    };
    assert_eq!(mint_address, "bcrt1q0r489mvjxujmd2ufss7av3ch2p3x0y856yt3y7");

    // from lebron
    let handle_msg = HandleMsg::RequestMintAddress { entropy };
    let handle_result = GatewayRunner::run_handle(
        &mut lebron_context,
        contract_test_utils::mock_env("lebron", &[]),
        handle_msg,
    );
    let mint_address: String = match from_binary(&handle_result.unwrap().data.unwrap()).unwrap() {
        HandleAnswer::RequestMintAddress { mint_address } => mint_address,
        _ => panic!("Unexpected"),
    };
    assert_eq!(mint_address, "bcrt1qrp420qleq0pvvk7pal4v3hm63tnjj7upqqdjuw");
}

#[test]
fn test_verify_mint_tx_sanity() {
    for tx_value in [100000000, 10000000] {
        let mut context = init_helper();
        let config = match from_binary(
            &GatewayRunner::run_query(&mut context, QueryMsg::Config {}).unwrap(),
        )
        .unwrap()
        {
            QueryAnswer::Config(config) => config,
            _ => unreachable!(),
        };
        let canonical_minter = contract_test_utils::mock_api()
            .canonical_address(&"minter".into())
            .unwrap();

        // create random mint key
        let mint_key = PrivateKey {
            compressed: true,
            network: Network::Regtest,
            key: SecretKey::random(&mut thread_rng()),
        };
        let mint_address = Address::p2wpkh(&mint_key.public_key(), mint_key.network).unwrap();

        let deps = context.client_deps();
        let mut proxy_deps = StateProxyDeps::restore(
            &deps.storage,
            &deps.api,
            &deps.querier,
            CONTRACT_LABEL,
            &Secp256k1ApiSigner::new(&deps.api),
        )
        .unwrap();
        // set mint key to storage
        write_mint_key(&mut proxy_deps.storage, &canonical_minter, &mint_key);
        let msg = proxy_deps.storage.cosmos_msgs().unwrap();
        context.exec_state_contract_messages(&msg);
        let mint_tx = Transaction {
            version: 1,
            lock_time: 0,
            input: vec![],
            output: vec![TxOut {
                value: tx_value,
                script_pubkey: mint_address.script_pubkey(),
            }],
        };
        let bin_mint_tx = Binary::from(serialize(&mint_tx));

        context.query_cases.add_case(
            WasmQuery::Smart {
                msg: to_padded_binary(&snip20::QueryMsg::TokenInfo {}).unwrap(),
                contract_addr: config.sbtc.address,
                callback_code_hash: config.sbtc.hash,
            },
            TokenInfoResponse {
                token_info: snip20::TokenInfo {
                    name: "sbtc".into(),
                    symbol: "SBTC".into(),
                    decimals: 8,
                    total_supply: Some(500000000u64.into()),
                },
            },
        );
        context.query_cases.add_case(
            WasmQuery::Smart {
                msg: to_padded_binary(&bitcoin_spv::QueryMsg::VerifyMerkleProof {
                    height: 1,
                    tx: bin_mint_tx,
                    merkle_proof: bitcoin_spv::MerkleProofMsg::default(),
                })
                .unwrap(),
                contract_addr: config.bitcoin_spv.address,
                callback_code_hash: config.bitcoin_spv.hash,
            },
            bitcoin_spv::QueryAnswer::VerifyMerkleProof { success: true },
        );

        // handle verify mint tx
        let msg = HandleMsg::VerifyMintTx {
            height: 1,
            tx: Binary::from(serialize(&mint_tx)),
            merkle_proof: bitcoin_spv::MerkleProofMsg::default(),
        };
        let handle_response = GatewayRunner::run_handle(
            &mut context,
            contract_test_utils::mock_env("minter", &[]),
            msg,
        )
        .unwrap();
        assert_eq!(handle_response.messages.len(), 3);
        assert_eq!(
            handle_response.messages[1],
            snip20::mint_msg(
                "minter".into(),
                tx_value.into(),
                None,
                None,
                BLOCK_SIZE,
                "sbtc_hash".into(),
                "sbtc_address".into()
            )
            .unwrap()
        );
        assert_eq!(
            handle_response.messages[2],
            log::HandleMsg::AddEvents {
                events: vec![(
                    "minter".into(),
                    log::Event::MintCompleted(log::event::MintCompletedData {
                        time: contract_test_utils::mock_timestamp() as u64,
                        address: mint_address.to_string(),
                        amount: tx_value.into(),
                        txid: mint_tx.txid().to_string()
                    }),
                )],
            }
            .to_cosmos_msg("log_hash".into(), "log_address".into(), None)
            .unwrap()
        );

        // assert mint key was removed
        let deps = context.client_deps();
        let mut proxy_deps = StateProxyDeps::restore(
            &deps.storage,
            &deps.api,
            &deps.querier,
            CONTRACT_LABEL,
            &Secp256k1ApiSigner::new(&deps.api),
        )
        .unwrap();
        assert!(
            read_mint_key(&proxy_deps.storage, &canonical_minter, Network::Regtest)
                .unwrap()
                .is_none()
        );

        // assert utxo stack
        let utxo = UtxoQueue::from_storage(&mut proxy_deps.storage, tx_value)
            .dequeue()
            .unwrap()
            .unwrap();
        assert_eq!(
            utxo,
            Utxo {
                txid: mint_tx.txid(),
                vout: 0,
                key: mint_key.key.serialize(),
            }
        );
    }
}

#[test]
fn test_suspend_verify_mint_tx() {
    let mut context = init_helper();
    //  handle
    let handle_msg = HandleMsg::VerifyMintTx {
        height: 0,
        tx: Binary::from(&[]),
        merkle_proof: bitcoin_spv::MerkleProofMsg::default(),
    };
    GatewayRunner::run_handle(
        &mut context,
        mock_env("owner", &[]),
        HandleMsg::SetSuspensionSwitch {
            suspension_switch: SuspensionSwitch {
                request_mint_address: false,
                verify_mint_tx: true,
                release_incorrect_amount_btc: false,
                request_release_btc: false,
                claim_release_btc: false,
            },
        },
    )
    .unwrap();
    let err = GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("bob", &[]),
        handle_msg,
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Generic error: contract error verify mint tx is being suspended"
    );
}

#[test]
fn test_verify_mint_tx_merkle_proof_verification_failure() {
    let mut context = init_helper();
    let canonical_minter = contract_test_utils::mock_api()
        .canonical_address(&"minter".into())
        .unwrap();
    let config =
        match from_binary(&GatewayRunner::run_query(&mut context, QueryMsg::Config {}).unwrap())
            .unwrap()
        {
            QueryAnswer::Config(config) => config,
            _ => unreachable!(),
        };
    // create random mint key
    let mint_key = PrivateKey {
        compressed: true,
        network: Network::Regtest,
        key: SecretKey::random(&mut thread_rng()),
    };
    let mint_address = Address::p2wpkh(&mint_key.public_key(), mint_key.network).unwrap();

    // set mint key to storage
    let deps = context.client_deps();
    let mut proxy_deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )
    .unwrap();
    // set mint key to storage
    write_mint_key(&mut proxy_deps.storage, &canonical_minter, &mint_key);
    let msg = proxy_deps.storage.cosmos_msgs().unwrap();
    context.exec_state_contract_messages(&msg);

    // mint tx sample
    let mint_tx = Transaction {
        version: 1,
        lock_time: 0,
        input: vec![],
        output: vec![TxOut {
            value: 100000000,
            script_pubkey: mint_address.script_pubkey(),
        }],
    };
    let bin_mint_tx = Binary::from(serialize(&mint_tx));

    context.query_cases.add_case(
        WasmQuery::Smart {
            msg: to_padded_binary(&bitcoin_spv::QueryMsg::VerifyMerkleProof {
                height: 1,
                tx: bin_mint_tx.clone(),
                merkle_proof: bitcoin_spv::MerkleProofMsg::default(),
            })
            .unwrap(),
            contract_addr: config.bitcoin_spv.address.clone(),
            callback_code_hash: config.bitcoin_spv.hash.clone(),
        },
        bitcoin_spv::QueryAnswer::VerifyMerkleProof { success: false },
    );

    context.query_cases.add_error_case(
        WasmQuery::Smart {
            msg: to_padded_binary(&bitcoin_spv::QueryMsg::VerifyMerkleProof {
                height: 2,
                tx: bin_mint_tx.clone(),
                merkle_proof: bitcoin_spv::MerkleProofMsg::default(),
            })
            .unwrap(),
            contract_addr: config.bitcoin_spv.address,
            callback_code_hash: config.bitcoin_spv.hash,
        },
        "bitcoin spv error".into(),
    );

    //
    // Execute tests
    //

    // CASE: spv query returns false
    // handle verify mint tx
    let msg = HandleMsg::VerifyMintTx {
        height: 1,
        tx: bin_mint_tx.clone(),
        merkle_proof: bitcoin_spv::MerkleProofMsg::default(),
    };
    let err = GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("minter", &[]),
        msg,
    )
    .unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("contract error merkle proof verification failed")
    );

    // CASE: spv query returns error
    let msg = HandleMsg::VerifyMintTx {
        height: 2,
        tx: bin_mint_tx,
        merkle_proof: bitcoin_spv::MerkleProofMsg::default(),
    };
    let err = GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("minter", &[]),
        msg,
    )
    .unwrap_err();
    assert_eq!(err, StdError::generic_err("bitcoin spv error"));
}

#[test]
fn test_verify_mint_tx_no_output() {
    let mut context = init_helper();
    let canonical_minter = contract_test_utils::mock_api()
        .canonical_address(&"minter".into())
        .unwrap();
    let config =
        match from_binary(&GatewayRunner::run_query(&mut context, QueryMsg::Config {}).unwrap())
            .unwrap()
        {
            QueryAnswer::Config(config) => config,
            _ => unreachable!(),
        };

    // create random mint key
    let mint_key = PrivateKey {
        compressed: true,
        network: Network::Regtest,
        key: SecretKey::random(&mut thread_rng()),
    };

    let deps = context.client_deps();
    let mut proxy_deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )
    .unwrap();
    // set mint key to storage
    write_mint_key(&mut proxy_deps.storage, &canonical_minter, &mint_key);
    let msg = proxy_deps.storage.cosmos_msgs().unwrap();
    context.exec_state_contract_messages(&msg);

    let mint_tx  =
        // mint tx
        Transaction {
            version: 1,
            lock_time: 0,
            input: vec![],
            output: vec![], // no output
        };
    let bin_mint_tx = Binary::from(serialize(&mint_tx));

    context.query_cases.add_case(
        WasmQuery::Smart {
            msg: to_padded_binary(&bitcoin_spv::QueryMsg::VerifyMerkleProof {
                height: 1,
                tx: bin_mint_tx.clone(),
                merkle_proof: bitcoin_spv::MerkleProofMsg::default(),
            })
            .unwrap(),
            contract_addr: config.bitcoin_spv.address,
            callback_code_hash: config.bitcoin_spv.hash,
        },
        bitcoin_spv::QueryAnswer::VerifyMerkleProof { success: true },
    );
    // handle verify mint tx
    let msg = HandleMsg::VerifyMintTx {
        height: 1,
        tx: bin_mint_tx,
        merkle_proof: bitcoin_spv::MerkleProofMsg::default(),
    };
    let err = GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("minter", &[]),
        msg,
    )
    .unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("contract error no valid tx output")
    );
}

#[test]
fn test_verify_mint_tx_invalid_mint_address() {
    let mut context = init_helper();
    let canonical_minter = contract_test_utils::mock_api()
        .canonical_address(&"minter".into())
        .unwrap();
    let config =
        match from_binary(&GatewayRunner::run_query(&mut context, QueryMsg::Config {}).unwrap())
            .unwrap()
        {
            QueryAnswer::Config(config) => config,
            _ => unreachable!(),
        };

    // create random mint key
    let mint_key = PrivateKey {
        compressed: true,
        network: Network::Regtest,
        key: SecretKey::random(&mut thread_rng()),
    };

    let deps = context.client_deps();
    let mut proxy_deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )
    .unwrap();

    // set mint key to storage
    write_mint_key(&mut proxy_deps.storage, &canonical_minter, &mint_key);
    let msg = proxy_deps.storage.cosmos_msgs().unwrap();
    context.exec_state_contract_messages(&msg);

    let invalid_mint_key = PrivateKey {
        compressed: true,
        network: Network::Regtest,
        key: SecretKey::random(&mut thread_rng()),
    };
    let invalid_mint_address =
        Address::p2wpkh(&invalid_mint_key.public_key(), invalid_mint_key.network).unwrap();

    let mint_tx = Transaction {
        version: 1,
        lock_time: 0,
        input: vec![],
        output: vec![TxOut {
            value: 100000000,
            script_pubkey: invalid_mint_address.script_pubkey(), // invalid mint address
        }],
    };

    let bin_mint_tx = Binary::from(serialize(&mint_tx));

    context.query_cases.add_case(
        WasmQuery::Smart {
            msg: to_padded_binary(&bitcoin_spv::QueryMsg::VerifyMerkleProof {
                height: 1,
                tx: bin_mint_tx.clone(),
                merkle_proof: bitcoin_spv::MerkleProofMsg::default(),
            })
            .unwrap(),
            contract_addr: config.bitcoin_spv.address,
            callback_code_hash: config.bitcoin_spv.hash,
        },
        bitcoin_spv::QueryAnswer::VerifyMerkleProof { success: true },
    );
    // handle verify mint tx
    let msg = HandleMsg::VerifyMintTx {
        height: 1,
        tx: bin_mint_tx,
        merkle_proof: bitcoin_spv::MerkleProofMsg::default(),
    };
    let err = GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("minter", &[]),
        msg,
    )
    .unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("contract error no valid tx output")
    );
}

#[test]
fn test_verify_mint_tx_invalid_tx_value() {
    let mut context = init_helper();
    let canonical_minter = contract_test_utils::mock_api()
        .canonical_address(&"minter".into())
        .unwrap();
    let config =
        match from_binary(&GatewayRunner::run_query(&mut context, QueryMsg::Config {}).unwrap())
            .unwrap()
        {
            QueryAnswer::Config(config) => config,
            _ => unreachable!(),
        };

    // create random mint key
    let mint_key = PrivateKey {
        compressed: true,
        network: Network::Regtest,
        key: SecretKey::random(&mut thread_rng()),
    };

    let deps = context.client_deps();
    let mut proxy_deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )
    .unwrap();
    // set mint key to storage
    write_mint_key(&mut proxy_deps.storage, &canonical_minter, &mint_key);
    let msg = proxy_deps.storage.cosmos_msgs().unwrap();
    context.exec_state_contract_messages(&msg);

    let invalid_mint_key = PrivateKey {
        compressed: true,
        network: Network::Regtest,
        key: SecretKey::random(&mut thread_rng()),
    };
    let invalid_mint_address =
        Address::p2wpkh(&invalid_mint_key.public_key(), invalid_mint_key.network).unwrap();

    let mint_tx = Transaction {
        version: 1,
        lock_time: 0,
        input: vec![],
        output: vec![TxOut {
            value: 100000000 - 1,
            script_pubkey: invalid_mint_address.script_pubkey(), // invalid mint address
        }],
    };

    let bin_mint_tx = Binary::from(serialize(&mint_tx));

    context.query_cases.add_case(
        WasmQuery::Smart {
            msg: to_padded_binary(&bitcoin_spv::QueryMsg::VerifyMerkleProof {
                height: 1,
                tx: bin_mint_tx.clone(),
                merkle_proof: bitcoin_spv::MerkleProofMsg::default(),
            })
            .unwrap(),
            contract_addr: config.bitcoin_spv.address,
            callback_code_hash: config.bitcoin_spv.hash,
        },
        bitcoin_spv::QueryAnswer::VerifyMerkleProof { success: true },
    );
    // handle verify mint tx
    let msg = HandleMsg::VerifyMintTx {
        height: 1,
        tx: bin_mint_tx,
        merkle_proof: bitcoin_spv::MerkleProofMsg::default(),
    };
    let err = GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("minter", &[]),
        msg,
    )
    .unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("contract error no valid tx output")
    );
}

#[test]
fn test_release_incorrect_amount_btc() {
    let mut context = init_helper();
    let canonical_minter = contract_test_utils::mock_api()
        .canonical_address(&"minter".into())
        .unwrap();
    let config =
        match from_binary(&GatewayRunner::run_query(&mut context, QueryMsg::Config {}).unwrap())
            .unwrap()
        {
            QueryAnswer::Config(config) => config,
            _ => unreachable!(),
        };

    // create random mint key
    let mint_key = PrivateKey {
        compressed: true,
        network: Network::Regtest,
        key: SecretKey::random(&mut thread_rng()),
    };
    let mint_address = Address::p2wpkh(&mint_key.public_key(), mint_key.network).unwrap();

    let deps = context.client_deps();
    let mut proxy_deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )
    .unwrap();
    // set mint key to storage
    write_mint_key(&mut proxy_deps.storage, &canonical_minter, &mint_key);
    let msg = proxy_deps.storage.cosmos_msgs().unwrap();
    context.exec_state_contract_messages(&msg);

    let mint_tx = Transaction {
        version: 1,
        lock_time: 0,
        input: vec![],
        output: vec![TxOut {
            value: 100000000 - 1, // invalid value
            script_pubkey: mint_address.script_pubkey(),
        }],
    };

    let bin_mint_tx = Binary::from(serialize(&mint_tx));

    context.query_cases.add_case(
        WasmQuery::Smart {
            msg: to_padded_binary(&bitcoin_spv::QueryMsg::VerifyMerkleProof {
                height: 1,
                tx: bin_mint_tx.clone(),
                merkle_proof: bitcoin_spv::MerkleProofMsg::default(),
            })
            .unwrap(),
            contract_addr: config.bitcoin_spv.address,
            callback_code_hash: config.bitcoin_spv.hash,
        },
        bitcoin_spv::QueryAnswer::VerifyMerkleProof { success: true },
    );

    // release incorrect amount btc
    let recipient_address = {
        let recipient_priv_key = PrivateKey {
            compressed: true,
            network: Network::Regtest,
            key: bitcoin::secp256k1::SecretKey::random(&mut thread_rng()),
        };
        Address::p2wpkh(&recipient_priv_key.public_key(), recipient_priv_key.network).unwrap()
    };
    let msg = HandleMsg::ReleaseIncorrectAmountBTC {
        height: 1,
        tx: bin_mint_tx.clone(),
        merkle_proof: bitcoin_spv::MerkleProofMsg::default(),
        recipient_address: recipient_address.to_string(),
        fee_per_vb: 200,
    };
    let response = GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("minter", &[]),
        msg,
    )
    .unwrap();

    let tx: Transaction = match from_binary(&response.data.unwrap()).unwrap() {
        HandleAnswer::ReleaseIncorrectAmountBTC { tx } => deserialize(tx.as_slice()).unwrap(),
        _ => panic!("unexpected"),
    };
    // assert signature is valid
    // mint_tx.output[0].script_pubkey.verify(0, 100000000, &bitcoin::consensus::encode::serialize(&tx)).unwrap();
    assert_eq!(tx.output.len(), 1);
    assert_eq!(
        tx.output[0].script_pubkey,
        recipient_address.script_pubkey()
    );
    assert_eq!(tx.output[0].value, 100000000 - 1 - 200 * 110);

    contract_test_utils::assert_handle_response_message(
        &response.messages[1],
        "log_address",
        "log_hash",
        &log::HandleMsg::AddEvents {
            events: vec![(
                "minter".into(),
                log::Event::ReleaseIncorrectAmountBTC(log::event::ReleaseIncorrectAmountBTCData {
                    time: mock_timestamp().into(),
                    amount: 99999999u64.into(),
                    release_from: mint_address.to_string(),
                    release_to: recipient_address.to_string(),
                    txid: tx.txid().to_string(),
                }),
            )],
        },
    )
}

#[test]
fn test_suspend_release_incorrect_amount_btc() {
    let mut context = init_helper();

    //  handle
    let handle_msg = HandleMsg::ReleaseIncorrectAmountBTC {
        height: 0,
        tx: Binary::from(&[]),
        merkle_proof: bitcoin_spv::MerkleProofMsg::default(),
        recipient_address: String::default(),
        fee_per_vb: 0,
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
                claim_release_btc: false,
            },
        },
    )
    .unwrap();
    let err = GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("bob", &[]),
        handle_msg,
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Generic error: contract error release incorrect amount btc is being suspended"
    );
}

#[test]
fn test_request_release_btc_sanity() {
    for tx_value in [100000000u64, 10000000u64] {
        let mut context = init_helper();
        let config = match from_binary(
            &GatewayRunner::run_query(&mut context, QueryMsg::Config {}).unwrap(),
        )
        .unwrap()
        {
            QueryAnswer::Config(config) => config,
            _ => unreachable!(),
        };
        let mut thread_rng = thread_rng();
        let txid = Txid::from_inner(thread_rng.gen());
        let key = thread_rng.gen();

        let deps = context.client_deps();
        let mut proxy_deps = StateProxyDeps::restore(
            &deps.storage,
            &deps.api,
            &deps.querier,
            CONTRACT_LABEL,
            &Secp256k1ApiSigner::new(&deps.api),
        )
        .unwrap();
        let mut utxo_queue = UtxoQueue::from_storage(&mut proxy_deps.storage, tx_value);
        let utxo = Utxo { txid, vout: 0, key };
        utxo_queue.enqueue(utxo.clone()).unwrap();
        let msg = proxy_deps.storage.cosmos_msgs().unwrap();
        context.exec_state_contract_messages(&msg);

        // Execute Handle
        let msg = HandleMsg::RequestReleaseBtc {
            entropy: Binary::from(b"entropy"),
            amount: tx_value,
        };
        context.query_cases.add_case(
            WasmQuery::Smart {
                msg: to_padded_binary(&snip20::QueryMsg::TokenInfo {}).unwrap(),
                contract_addr: config.sbtc.address,
                callback_code_hash: config.sbtc.hash,
            },
            TokenInfoResponse {
                token_info: snip20::TokenInfo {
                    name: "sbtc".into(),
                    symbol: "SBTC".into(),
                    decimals: 8,
                    total_supply: Some(500000000u64.into()),
                },
            },
        );
        let response = GatewayRunner::run_handle(
            &mut context,
            contract_test_utils::mock_env("releaser", &[]),
            msg,
        )
        .unwrap();
        let request_key = match from_binary(&response.data.unwrap()).unwrap() {
            HandleAnswer::RequestReleaseBtc { request_key } => request_key,
            _ => panic!("unexpected"),
        };
        assert_eq!(response.messages.len(), 3);
        assert_eq!(
            response.messages[1],
            snip20::burn_from_msg(
                "releaser".into(),
                tx_value.into(),
                None,
                None,
                BLOCK_SIZE,
                "sbtc_hash".into(),
                "sbtc_address".into()
            )
            .unwrap()
        );
        assert_eq!(
            response.messages[2],
            log::HandleMsg::AddEvents {
                events: vec![(
                    "releaser".into(),
                    log::Event::ReleaseStarted(log::event::ReleaseStartedData {
                        time: mock_timestamp() as u64,
                        request_key: request_key.clone(),
                        amount: tx_value.into()
                    })
                )]
            }
            .to_cosmos_msg("log_hash".into(), "log_address".into(), None)
            .unwrap()
        );

        //
        // Assertion
        //

        // Assert utxo_queue
        let deps = context.client_deps();
        let mut proxy_deps = StateProxyDeps::restore(
            &deps.storage,
            &deps.api,
            &deps.querier,
            CONTRACT_LABEL,
            &Secp256k1ApiSigner::new(&deps.api),
        )
        .unwrap();
        let mut utxo_queue = UtxoQueue::from_storage(&mut proxy_deps.storage, tx_value);
        assert!(utxo_queue.dequeue().unwrap().is_none());

        // Assert request stored
        let requested_utxo = read_release_request_utxo(&proxy_deps.storage, &request_key)
            .unwrap()
            .unwrap();
        assert_eq!(
            requested_utxo,
            RequestedUtxo {
                value: tx_value,
                utxo
            }
        );
    }
}

#[test]
fn test_suspend_request_release_btc() {
    let mut context = init_helper();
    GatewayRunner::run_handle(
        &mut context,
        mock_env("owner", &[]),
        HandleMsg::SetSuspensionSwitch {
            suspension_switch: SuspensionSwitch {
                request_mint_address: false,
                verify_mint_tx: false,
                release_incorrect_amount_btc: false,
                request_release_btc: true,
                claim_release_btc: false,
            },
        },
    )
    .unwrap();
    //  handle
    let handle_msg = HandleMsg::RequestReleaseBtc {
        entropy: Binary::from(&[]),
        amount: 0,
    };
    let err = GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("bob", &[]),
        handle_msg,
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Generic error: contract error request release btc is being suspended"
    );
}

#[test]
fn test_claim_release_btc_sanity() {
    let mut context = init_helper();
    let mut thread_rng = thread_rng();
    let config =
        match from_binary(&GatewayRunner::run_query(&mut context, QueryMsg::Config {}).unwrap())
            .unwrap()
        {
            QueryAnswer::Config(config) => config,
            _ => unreachable!(),
        };
    // create random mint key
    let sign_key = PrivateKey {
        compressed: true,
        network: Network::Regtest,
        key: SecretKey::random(&mut thread_rng),
    };
    let mint_address = Address::p2wpkh(&sign_key.public_key(), sign_key.network).unwrap();

    let mint_tx =
        // mint tx
        Transaction {
            version: 2,
            lock_time: 0,
            input: vec![],
            output: vec![TxOut {
                value: 100000000,
                script_pubkey: mint_address.script_pubkey(),
            }],
        };

    let recipient_address = {
        let recipient_priv_key = PrivateKey {
            compressed: true,
            network: Network::Regtest,
            key: bitcoin::secp256k1::SecretKey::random(&mut thread_rng),
        };
        Address::p2wpkh(&recipient_priv_key.public_key(), recipient_priv_key.network).unwrap()
    };

    let canonical_releaser = contract_test_utils::mock_api()
        .canonical_address(&"releaser".into())
        .unwrap();

    // set release request
    let utxo = Utxo {
        txid: mint_tx.txid(),
        vout: 0,
        key: sign_key.key.serialize(),
    };
    let request_key = gen_request_key(&canonical_releaser, &utxo, &mut thread_rng).unwrap();
    let deps = context.client_deps();
    let mut proxy_deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )
    .unwrap();
    write_release_request_utxo(&mut proxy_deps.storage, &request_key, 100000000, utxo).unwrap();
    let msg = proxy_deps.storage.cosmos_msgs().unwrap();
    context.exec_state_contract_messages(&msg);

    // create merkle proof
    let merkle_proof = sfps::sfps_lib::merkle::MerkleProof {
        total: 1,
        index: 0,
        leaf: vec![],
        aunts: vec![],
    };

    context.query_cases.add_case(
        WasmQuery::Smart {
            msg: to_padded_binary(&sfps::QueryMsg::VerifyResponseDeliverTxProof {
                merkle_proof: merkle_proof.clone(),
                headers: vec![],
                block_hash_index: 1,
                encryption_key: Binary::from(b"encryption_key"),
            })
            .unwrap(),
            contract_addr: config.sfps.address,
            callback_code_hash: config.sfps.hash,
        },
        sfps::QueryAnswer::VerifyResponseDeliverTxProof {
            decrypted_data: to_binary(&HandleAnswer::RequestReleaseBtc { request_key }).unwrap(),
        },
    );
    // Claim Release Tx
    let msg = HandleMsg::ClaimReleasedBtc {
        merkle_proof,
        headers: vec![],
        block_hash_index: 1,
        encryption_key: Binary::from(b"encryption_key"),
        recipient_address: recipient_address.to_string(),
        fee_per_vb: 200,
    };
    let response = GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("releaser", &[]),
        msg,
    )
    .unwrap();
    let tx: Transaction = match from_binary(&response.data.unwrap()).unwrap() {
        HandleAnswer::ClaimReleasedBtc { tx } => deserialize(tx.as_slice()).unwrap(),
        _ => panic!("unexpected"),
    };
    // assert signature is valid
    #[cfg(feature = "bitcoinconsensus")]
    mint_tx.output[0]
        .script_pubkey
        .verify(
            0,
            bitcoin::Amount::from_sat(100000000),
            &bitcoin::consensus::encode::serialize(&tx),
        )
        .unwrap();
    assert_eq!(tx.output.len(), 1);
    assert_eq!(
        tx.output[0].script_pubkey,
        recipient_address.script_pubkey()
    );
    assert_eq!(tx.output[0].value, 100000000 - 200 * 110);

    assert_eq!(response.messages.len(), 2);
    assert_eq!(
        response.messages[1],
        log::HandleMsg::AddEvents {
            events: vec![(
                "releaser".into(),
                log::Event::ReleaseCompleted(log::event::ReleaseCompletedData {
                    time: contract_test_utils::mock_timestamp() as u64,
                    request_key: request_key,
                    txid: tx.txid().to_string(),
                    fee_per_vb: 200,
                })
            )]
        }
        .to_cosmos_msg("log_hash".into(), "log_address".into(), None)
        .unwrap()
    );
}

#[test]
fn test_suspend_claim_release_btc() {
    let mut context = init_helper();
    GatewayRunner::run_handle(
        &mut context,
        mock_env("owner", &[]),
        HandleMsg::SetSuspensionSwitch {
            suspension_switch: SuspensionSwitch {
                request_mint_address: false,
                verify_mint_tx: false,
                release_incorrect_amount_btc: false,
                request_release_btc: false,
                claim_release_btc: true,
            },
        },
    )
    .unwrap();
    //  handle
    let handle_msg = HandleMsg::ClaimReleasedBtc {
        merkle_proof: sfps::MerkleProof::default(),
        headers: vec![],
        block_hash_index: 0,
        encryption_key: Binary::from(&[]),
        recipient_address: String::default(),
        fee_per_vb: 0,
    };
    let err = GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("bob", &[]),
        handle_msg,
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Generic error: contract error claim release btc is being suspended"
    );
}

#[test]
fn test_change_owner() {
    let mut context = init_helper();
    let msg = HandleMsg::ChangeOwner {
        new_owner: "new_owner".into(),
    };
    let err = GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("not_owner", &[]),
        msg.clone(),
    )
    .unwrap_err();
    assert_eq!(err.to_string(), "Generic error: contract error not owner");
    GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("owner", &[]),
        msg,
    )
    .unwrap();
    let config =
        match from_binary(&GatewayRunner::run_query(&mut context, QueryMsg::Config {}).unwrap())
            .unwrap()
        {
            QueryAnswer::Config(config) => config,
            _ => unreachable!(),
        };
    assert_eq!(config.owner, "new_owner".into());
}

#[test]
fn test_suspension_switch() {
    let mut context = init_helper();
    let mut switch = match from_binary(
        &GatewayRunner::run_query(&mut context, QueryMsg::SuspensionSwitch {}).unwrap(),
    )
    .unwrap()
    {
        QueryAnswer::SuspensionSwitch(suspension_switch) => suspension_switch,
        _ => unreachable!(),
    };
    assert_eq!(
        switch,
        SuspensionSwitch {
            request_mint_address: false,
            verify_mint_tx: false,
            release_incorrect_amount_btc: false,
            request_release_btc: false,
            claim_release_btc: false,
        }
    );
    switch.verify_mint_tx = true;
    GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("owner", &[]),
        HandleMsg::SetSuspensionSwitch {
            suspension_switch: switch.clone(),
        },
    )
    .unwrap();
    let updated_switch = match from_binary(
        &GatewayRunner::run_query(&mut context, QueryMsg::SuspensionSwitch {}).unwrap(),
    )
    .unwrap()
    {
        QueryAnswer::SuspensionSwitch(suspension_switch) => suspension_switch,
        _ => unreachable!(),
    };
    assert_eq!(updated_switch, switch);
}

#[test]
fn test_release_btc_by_owner() {
    let mut context = init_helper();
    let mut thread_rng = thread_rng();
    let recipient_address = {
        let recipient_priv_key = PrivateKey {
            compressed: true,
            network: Network::Regtest,
            key: bitcoin::secp256k1::SecretKey::random(&mut thread_rng),
        };
        Address::p2wpkh(&recipient_priv_key.public_key(), recipient_priv_key.network).unwrap()
    };

    let deps = context.client_deps();
    let mut proxy_deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )
    .unwrap();
    let mut mint_txs = Vec::with_capacity(10);
    let mut queue = UtxoQueue::from_storage(&mut proxy_deps.storage, 100000000);
    for _ in 0..10 {
        // create random mint key
        let sign_key = PrivateKey {
            compressed: true,
            network: Network::Regtest,
            key: SecretKey::random(&mut thread_rng),
        };
        let mint_address = Address::p2wpkh(&sign_key.public_key(), sign_key.network).unwrap();

        mint_txs.push(
            // mint tx
            Transaction {
                version: 2,
                lock_time: 0,
                input: vec![],
                output: vec![TxOut {
                    value: 100000000,
                    script_pubkey: mint_address.script_pubkey(),
                }],
            },
        );

        // set release request
        let utxo = Utxo {
            txid: mint_txs.last().unwrap().txid(),
            vout: 0,
            key: sign_key.key.serialize(),
        };
        queue.enqueue(utxo).unwrap();
    }
    let msg = proxy_deps.storage.cosmos_msgs().unwrap();
    context.exec_state_contract_messages(&msg);

    let response = GatewayRunner::run_handle(
        &mut context,
        contract_test_utils::mock_env("owner", &[]),
        HandleMsg::ReleaseBtcByOwner {
            tx_value: 100000000,
            max_input_length: 100,
            recipient_address: recipient_address.to_string(),
            fee_per_vb: 200,
        },
    )
    .unwrap();
    let tx: Transaction = match from_binary(&response.data.unwrap()).unwrap() {
        HandleAnswer::ReleaseBtcByOwner { tx } => deserialize(tx.as_slice()).unwrap(),
        _ => panic!("unexpected"),
    };
    #[cfg(feature = "bitcoinconsensus")]
    for i in 0..10 {
        // assert signature is valid
        mint_txs[i].output[0]
            .script_pubkey
            .verify(
                i,
                bitcoin::Amount::from_sat(100000000),
                &bitcoin::consensus::encode::serialize(&tx),
            )
            .unwrap();
    }
    assert_eq!(tx.output.len(), 1);
    assert_eq!(
        tx.output[0].script_pubkey,
        recipient_address.script_pubkey()
    );
    // the tx vsize caluclated by the contract is higher than the actual one.
    let small_length_signature_input_count = tx
        .input
        .iter()
        .filter(|input| input.witness[0].len() == 71)
        .count();
    let calculated_vsize = ((tx.get_weight() + small_length_signature_input_count + 3) / 4) as u64;
    assert_eq!(tx.output[0].value, 100000000 * 10 - 200 * calculated_vsize);
}
