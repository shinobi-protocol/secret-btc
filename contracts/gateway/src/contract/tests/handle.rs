use super::*;
use crate::state::bitcoin_utxo::gen_request_key;
use crate::state::bitcoin_utxo::{
    read_release_request_utxo, write_release_request_utxo, RequestedUtxo, Utxo, UtxoQueue,
};
use crate::state::config::read_config;
use crate::state::mint_key::{read_mint_key, write_mint_key};
use bitcoin::consensus::encode::{deserialize, serialize};
use bitcoin::hash_types::Txid;
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::SecretKey;
use bitcoin::{Address, Network, PrivateKey, Transaction, TxOut};
use cosmwasm_std::{from_binary, to_binary, Api, Binary, StdError, WasmQuery};
use rand::{thread_rng, Rng};
use secret_toolkit::{snip20, utils::HandleCallback};
use shared_types::gateway::*;
use shared_types::{bitcoin_spv, finance_admin, log, sfps, BLOCK_SIZE};
use std::string::ToString;

/// wrapper to serialize/deserialize snip20 TokenInfo response
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TokenInfoResponse {
    token_info: snip20::TokenInfo,
}

#[test]
fn test_request_mint_address_sanity() {
    let mut deps = init_helper();
    // execute handle
    let handle_msg = HandleMsg::RequestMintAddress {
        entropy: Binary::from(b"entropy"),
    };

    // assert response
    let handle_response = handle(&mut deps, helper::mock_env("bob", &[]), handle_msg).unwrap();
    let mint_address: String = match from_binary(&handle_response.data.unwrap()).unwrap() {
        HandleAnswer::RequestMintAddress { mint_address } => mint_address,
        _ => panic!("Unexpected"),
    };
    assert_eq!(mint_address, "bcrt1q7wn8f7qt9sllujdlukazyhh87uyva0xq0s5rmt");
    assert_eq!(handle_response.messages.len(), 1);
    assert_eq!(
        handle_response.messages[0],
        log::HandleMsg::AddEvents {
            events: vec![(
                "bob".into(),
                log::Event::MintStarted(log::event::MintStartedData {
                    time: helper::mock_timestamp() as u64,
                    address: mint_address.to_string(),
                }),
            )],
        }
        .to_cosmos_msg("log_hash".into(), "log_address".into(), None)
        .unwrap()
    );

    let canonical_addr = deps.api.canonical_address(&"bob".into()).unwrap();
    // assert states
    let mint_key = read_mint_key(&deps.storage, &canonical_addr, Network::Regtest)
        .unwrap()
        .unwrap();
    assert_eq!(
        format!("{:x}", mint_key.key),
        "5f23067487496469d03dfd56c8c041ad2855977cbdb5b9c8ef6ff383cac21f63"
    );
    let address = Address::p2wpkh(&mint_key.public_key(), mint_key.network).unwrap();
    assert_eq!(address.to_string(), mint_address);
}

#[test]
fn test_request_mint_address_twice_with_same_entropy() {
    let mut deps = init_helper();

    // request 1
    let handle_msg = HandleMsg::RequestMintAddress {
        entropy: Binary::from(b"entropy"),
    };
    let handle_result = handle(&mut deps, helper::mock_env("bob", &[]), handle_msg);
    let mint_address: String = match from_binary(&handle_result.unwrap().data.unwrap()).unwrap() {
        HandleAnswer::RequestMintAddress { mint_address } => mint_address,
        _ => panic!("Unexpected"),
    };
    assert_eq!(mint_address, "bcrt1q7wn8f7qt9sllujdlukazyhh87uyva0xq0s5rmt");

    // request 2
    let handle_msg = HandleMsg::RequestMintAddress {
        entropy: Binary::from(b"entropy"),
    };
    let handle_result = handle(&mut deps, helper::mock_env("bob", &[]), handle_msg);
    let mint_address: String = match from_binary(&handle_result.unwrap().data.unwrap()).unwrap() {
        HandleAnswer::RequestMintAddress { mint_address } => mint_address,
        _ => panic!("Unexpected"),
    };
    assert_eq!(mint_address, "bcrt1q85q7xyyy86uw2zm2s2ld6j307p60hmfx4eamx9");
}

#[test]
fn test_request_mint_address_to_same_state_from_different_account() {
    let mut deps_bob = init_helper();
    let mut deps_lebron = init_helper();

    let entropy = Binary::from(b"entropy");

    // from bob
    let handle_msg = HandleMsg::RequestMintAddress {
        entropy: entropy.clone(),
    };
    let handle_result = handle(&mut deps_bob, helper::mock_env("bob", &[]), handle_msg);
    let mint_address: String = match from_binary(&handle_result.unwrap().data.unwrap()).unwrap() {
        HandleAnswer::RequestMintAddress { mint_address } => mint_address,
        _ => panic!("Unexpected"),
    };
    assert_eq!(mint_address, "bcrt1q7wn8f7qt9sllujdlukazyhh87uyva0xq0s5rmt");

    // from lebron
    let handle_msg = HandleMsg::RequestMintAddress { entropy };
    let handle_result = handle(
        &mut deps_lebron,
        helper::mock_env("lebron", &[]),
        handle_msg,
    );
    let mint_address: String = match from_binary(&handle_result.unwrap().data.unwrap()).unwrap() {
        HandleAnswer::RequestMintAddress { mint_address } => mint_address,
        _ => panic!("Unexpected"),
    };
    assert_eq!(mint_address, "bcrt1qcqwd80unkc0gqjzwa5vn5rxhfn6ugck7k82av4");
}

#[test]
fn test_verify_mint_tx_sanity() {
    for tx_value in [100000000, 10000000] {
        let mut deps = init_helper();
        let config = read_config(&deps.storage, &deps.api).unwrap();
        let canonical_minter = deps.api.canonical_address(&"minter".into()).unwrap();

        // create random mint key
        let mint_key = PrivateKey {
            compressed: true,
            network: Network::Regtest,
            key: SecretKey::random(&mut thread_rng()),
        };
        let mint_address = Address::p2wpkh(&mint_key.public_key(), mint_key.network).unwrap();

        // set mint key to storage
        write_mint_key(&mut deps.storage, &canonical_minter, &mint_key);

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

        deps.querier.add_case(
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
        deps.querier.add_case(
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
        let handle_response = handle(&mut deps, helper::mock_env("minter", &[]), msg).unwrap();
        assert_eq!(handle_response.messages.len(), 3);
        assert_eq!(
            handle_response.messages[0],
            snip20::mint_msg(
                "minter".into(),
                tx_value.into(),
                None,
                BLOCK_SIZE,
                "sbtc_hash".into(),
                "sbtc_address".into()
            )
            .unwrap()
        );
        assert_eq!(
            handle_response.messages[1],
            finance_admin::CommonHandleMsg::SendMintReward {
                minter: "minter".into(),
                sbtc_mint_amount: tx_value.into(),
                sbtc_total_supply: 500000000u64.into(),
            }
            .to_cosmos_msg("finance_admin_hash".into(), "finance_a_addr".into(), None)
            .unwrap()
        );
        assert_eq!(
            handle_response.messages[2],
            log::HandleMsg::AddEvents {
                events: vec![(
                    "minter".into(),
                    log::Event::MintCompleted(log::event::MintCompletedData {
                        time: helper::mock_timestamp() as u64,
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
        assert!(
            read_mint_key(&deps.storage, &canonical_minter, Network::Regtest)
                .unwrap()
                .is_none()
        );

        // assert utxo stack
        let utxo = UtxoQueue::from_storage(&mut deps.storage, tx_value)
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
fn test_verify_mint_tx_merkle_proof_verification_failure() {
    let mut deps = init_helper();
    let canonical_minter = deps.api.canonical_address(&"minter".into()).unwrap();
    let config = read_config(&deps.storage, &deps.api).unwrap();
    // create random mint key
    let mint_key = PrivateKey {
        compressed: true,
        network: Network::Regtest,
        key: SecretKey::random(&mut thread_rng()),
    };
    let mint_address = Address::p2wpkh(&mint_key.public_key(), mint_key.network).unwrap();

    // set mint key to storage
    write_mint_key(&mut deps.storage, &canonical_minter, &mint_key);

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

    deps.querier.add_case(
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

    deps.querier.add_error_case(
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
    let err = handle(&mut deps, helper::mock_env("minter", &[]), msg).unwrap_err();
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
    let err = handle(&mut deps, helper::mock_env("minter", &[]), msg).unwrap_err();
    assert_eq!(err, StdError::generic_err("bitcoin spv error"));
}

#[test]
fn test_verify_mint_tx_no_output() {
    let mut deps = init_helper();
    let canonical_minter = deps.api.canonical_address(&"minter".into()).unwrap();
    let config = read_config(&deps.storage, &deps.api).unwrap();

    // create random mint key
    let mint_key = PrivateKey {
        compressed: true,
        network: Network::Regtest,
        key: SecretKey::random(&mut thread_rng()),
    };

    // set mint key to storage
    write_mint_key(&mut deps.storage, &canonical_minter, &mint_key);

    let mint_tx  =
        // mint tx
        Transaction {
            version: 1,
            lock_time: 0,
            input: vec![],
            output: vec![], // no output
        };
    let bin_mint_tx = Binary::from(serialize(&mint_tx));

    deps.querier.add_case(
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
    let err = handle(&mut deps, helper::mock_env("minter", &[]), msg).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("contract error no valid tx output")
    );
}

#[test]
fn test_verify_mint_tx_invalid_mint_address() {
    let mut deps = init_helper();
    let canonical_minter = deps.api.canonical_address(&"minter".into()).unwrap();
    let config = read_config(&deps.storage, &deps.api).unwrap();

    // create random mint key
    let mint_key = PrivateKey {
        compressed: true,
        network: Network::Regtest,
        key: SecretKey::random(&mut thread_rng()),
    };

    // set mint key to storage
    write_mint_key(&mut deps.storage, &canonical_minter, &mint_key);

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

    deps.querier.add_case(
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
    let err = handle(&mut deps, helper::mock_env("minter", &[]), msg).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("contract error no valid tx output")
    );
}

#[test]
fn test_verify_mint_tx_invalid_tx_value() {
    let mut deps = init_helper();
    let canonical_minter = deps.api.canonical_address(&"minter".into()).unwrap();
    let config = read_config(&deps.storage, &deps.api).unwrap();

    // create random mint key
    let mint_key = PrivateKey {
        compressed: true,
        network: Network::Regtest,
        key: SecretKey::random(&mut thread_rng()),
    };

    // set mint key to storage
    write_mint_key(&mut deps.storage, &canonical_minter, &mint_key);

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

    deps.querier.add_case(
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
    let err = handle(&mut deps, helper::mock_env("minter", &[]), msg).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("contract error no valid tx output")
    );
}

#[test]
fn test_release_incorret_amount_btc() {
    let mut deps = init_helper();
    let canonical_minter = deps.api.canonical_address(&"minter".into()).unwrap();
    let config = read_config(&deps.storage, &deps.api).unwrap();

    // create random mint key
    let mint_key = PrivateKey {
        compressed: true,
        network: Network::Regtest,
        key: SecretKey::random(&mut thread_rng()),
    };
    let mint_address = Address::p2wpkh(&mint_key.public_key(), mint_key.network).unwrap();

    // set mint key to storage
    write_mint_key(&mut deps.storage, &canonical_minter, &mint_key);

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

    deps.querier.add_case(
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
    let response = handle(&mut deps, helper::mock_env("minter", &[]), msg).unwrap();

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
}

#[test]
fn test_request_release_btc_sanity() {
    for tx_value in [100000000u64, 10000000u64] {
        let mut deps = init_helper();
        let config = read_config(&deps.storage, &deps.api).unwrap();
        let mut thread_rng = thread_rng();
        let txid = Txid::from_inner(thread_rng.gen());
        let key = thread_rng.gen();

        let mut utxo_queue = UtxoQueue::from_storage(&mut deps.storage, tx_value);
        let utxo = Utxo { txid, vout: 0, key };
        utxo_queue.enqueue(utxo.clone()).unwrap();

        // Execute Handle
        let msg = HandleMsg::RequestReleaseBtc {
            entropy: Binary::from(b"entropy"),
            amount: tx_value,
        };
        deps.querier.add_case(
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
        let response = handle(&mut deps, helper::mock_env("releaser", &[]), msg).unwrap();
        let request_key = match from_binary(&response.data.unwrap()).unwrap() {
            HandleAnswer::RequestReleaseBtc { request_key } => request_key,
            _ => panic!("unexpected"),
        };
        assert_eq!(response.messages.len(), 3);
        assert_eq!(
            response.messages[0],
            snip20::burn_from_msg(
                "releaser".into(),
                tx_value.into(),
                None,
                BLOCK_SIZE,
                "sbtc_hash".into(),
                "sbtc_address".into()
            )
            .unwrap()
        );
        assert_eq!(
            response.messages[1],
            finance_admin::CommonHandleMsg::ReceiveReleaseFee {
                releaser: "releaser".into(),
                sbtc_release_amount: tx_value.into(),
                sbtc_total_supply: 500000000u64.into(),
            }
            .to_cosmos_msg("finance_admin_hash".into(), "finance_a_addr".into(), None)
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
        let mut utxo_queue = UtxoQueue::from_storage(&mut deps.storage, tx_value);
        assert!(utxo_queue.dequeue().unwrap().is_none());

        // Assert request stored
        let requested_utxo = read_release_request_utxo(&deps.storage, &request_key)
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
fn test_execute_release_btc_sanity() {
    let mut deps = init_helper();
    let mut thread_rng = thread_rng();
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

    let canonical_releaser = deps.api.canonical_address(&"releaser".into()).unwrap();

    // set release request
    let utxo = Utxo {
        txid: mint_tx.txid(),
        vout: 0,
        key: sign_key.key.serialize(),
    };
    let request_key = gen_request_key(&canonical_releaser, &utxo, &mut thread_rng).unwrap();
    write_release_request_utxo(&mut deps.storage, &request_key, 100000000, utxo).unwrap();

    let tx_result = sfps::sfps_lib::tx_result_proof::TxResult {
        code: 0,
        data: vec![],
        gas_used: 0,
        gas_wanted: 0,
        log: "".into(),
        info: "".into(),
        events: vec![],
        codespace: "".into(),
    };

    let config = read_config(&deps.storage, &deps.api).unwrap();

    // create merkle proof
    let merkle_proof = sfps::sfps_lib::merkle::MerkleProof {
        total: 1,
        index: 0,
        leaf_hash: vec![],
        aunts: vec![],
    };
    let tx_result_proof = sfps::TxResultProof {
        merkle_proof,
        tx_result,
        headers: vec![],
    };

    deps.querier.add_case(
        WasmQuery::Smart {
            msg: to_padded_binary(&sfps::QueryMsg::VerifyTxResultProof {
                tx_result_proof: tx_result_proof.clone(),
                header_hash_index: 1,
                encryption_key: Binary::from(b"encryption_key"),
            })
            .unwrap(),
            contract_addr: config.sfps.address,
            callback_code_hash: config.sfps.hash,
        },
        sfps::QueryAnswer::VerifyTxResultProof {
            decrypted_data: to_binary(&HandleAnswer::RequestReleaseBtc { request_key }).unwrap(),
        },
    );
    // Claim Release Tx
    let msg = HandleMsg::ClaimReleasedBtc {
        tx_result_proof,
        header_hash_index: 1,
        encryption_key: Binary::from(b"encryption_key"),
        recipient_address: recipient_address.to_string(),
        fee_per_vb: 200,
    };
    let response = handle(&mut deps, helper::mock_env("releaser", &[]), msg).unwrap();
    let tx: Transaction = match from_binary(&response.data.unwrap()).unwrap() {
        HandleAnswer::ClaimReleasedBtc { tx } => deserialize(tx.as_slice()).unwrap(),
        _ => panic!("unexpected"),
    };
    // assert signature is valid
    // mint_tx.output[0].script_pubkey.verify(0, 100000000, &bitcoin::consensus::encode::serialize(&tx)).unwrap();
    assert_eq!(tx.output.len(), 1);
    assert_eq!(
        tx.output[0].script_pubkey,
        recipient_address.script_pubkey()
    );
    assert_eq!(tx.get_vsize(), 110);
    assert_eq!(tx.output[0].value, 100000000 - 200 * 110);

    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0],
        log::HandleMsg::AddEvents {
            events: vec![(
                "releaser".into(),
                log::Event::ReleaseCompleted(log::event::ReleaseCompletedData {
                    time: helper::mock_timestamp() as u64,
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
