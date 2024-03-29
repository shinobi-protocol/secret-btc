use super::*;
use crate::state::chaindb::StorageChainDB;
use bitcoin::consensus::encode::{deserialize, serialize};
use bitcoin::hash_types::TxMerkleNode;
use bitcoin::util::hash::bitcoin_merkle_root;
use bitcoin::{Address, BlockHeader, Network, Transaction, TxOut};
use bitcoin_header_chain::header_chain::HeaderChain;
use contract_test_utils::contract_runner::ContractRunner;
use cosmwasm_std::{from_binary, Binary, StdError};
use shared_types::bitcoin_spv::{Config, MerkleProofMsg, QueryAnswer, QueryMsg};
use shared_types::state_proxy::client::Secp256k1ApiSigner;
use shared_types::state_proxy::client::StateProxyDeps;
use std::str::FromStr;
use std::string::ToString;

#[test]
fn test_query_tip() {
    let mut context = init_helper();

    for i in 1u32..101u32 {
        let header = regtest_block_header(i as _);
        let deps = context.client_deps();
        let mut deps = StateProxyDeps::restore(
            &deps.storage,
            &deps.api,
            &deps.querier,
            CONTRACT_LABEL,
            &Secp256k1ApiSigner::new(&deps.api),
        )
        .unwrap();
        let chaindb = StorageChainDB::from_storage(&mut deps.storage);
        let mut header_chain = HeaderChain::new(chaindb, Network::Bitcoin);
        let deserialized_header = deserialize(&header).unwrap();
        header_chain
            .store_headers(i, vec![deserialized_header], deserialized_header.time)
            .unwrap();
        let msgs = &deps.storage.cosmos_msgs().unwrap();
        context.exec_state_contract_messages(&msgs);
        let query_msg = QueryMsg::BestHeaderHash {};
        let query_result = BitcoinSPVRunner::run_query(&mut context, query_msg);
        let tip_block_hash: String = match from_binary(&query_result.unwrap()).unwrap() {
            QueryAnswer::BestHeaderHash { hash } => hash,
            _ => panic!("Unexpected"),
        };
        assert_eq!(
            tip_block_hash,
            deserialize::<BlockHeader>(&header)
                .unwrap()
                .block_hash()
                .to_string()
        );
    }
}

#[test]
fn test_query_block() {
    let mut context = init_helper();
    let mut headers: Vec<BlockHeader> = vec![];
    for i in 1..101 {
        headers.push(deserialize(&regtest_block_header(i)).unwrap());
    }
    let deps = context.client_deps();
    let mut deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )
    .unwrap();
    let chaindb = StorageChainDB::from_storage(&mut deps.storage);
    let mut header_chain = HeaderChain::new(chaindb, Network::Bitcoin);
    let time = headers.last().unwrap().time;
    header_chain.store_headers(100, headers, time).unwrap();
    let msgs = &deps.storage.cosmos_msgs().unwrap();
    context.exec_state_contract_messages(&msgs);
    for i in 1..101 {
        let header = Binary::from(regtest_block_header(i));
        let query_msg = QueryMsg::BlockHeader { height: i as u32 };
        let query_result = BitcoinSPVRunner::run_query(&mut context, query_msg);
        let result_header: Binary = match from_binary(&query_result.unwrap()).unwrap() {
            QueryAnswer::BlockHeader { header } => header,
            _ => panic!("Unexpected"),
        };
        assert_eq!(header, result_header);
    }
}

#[test]
fn test_query_config() {
    let mut context = init_helper();
    let query_msg = QueryMsg::Config {};
    let query_result = BitcoinSPVRunner::run_query(&mut context, query_msg);
    match from_binary(&query_result.unwrap()).unwrap() {
        QueryAnswer::Config(Config {
            confirmation,
            bitcoin_network,
            state_proxy,
        }) => {
            assert_eq!(confirmation, 6);
            assert_eq!(bitcoin_network, "regtest");
            assert_eq!(
                state_proxy.address.to_string(),
                "state_proxy_address".to_string()
            );
            assert_eq!(state_proxy.hash, "state_proxy_hash".to_string());
        }
        _ => panic!("Unexpected"),
    };
}

#[test]
fn test_verify_merkle_proof() {
    let mut context = init_helper();
    let address = Address::from_str("mqkhEMH6NCeYjFybv7pvFC22MFeaNT9AQC").unwrap();

    let txdata = vec![
        // sibling
        Transaction {
            version: 1,
            lock_time: 0,
            input: vec![],
            output: vec![],
        },
        // mint tx
        Transaction {
            version: 1,
            lock_time: 0,
            input: vec![],
            output: vec![TxOut {
                value: 1000,
                script_pubkey: address.script_pubkey(),
            }],
        },
    ];
    let merkle_root = TxMerkleNode::from_hash(bitcoin_merkle_root(
        txdata.iter().map(|tx| tx.txid().as_hash()),
    ));

    // set block headers to header chain
    let deps = context.client_deps();
    let mut deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )
    .unwrap();
    let chain_db = StorageChainDB::from_storage(&mut deps.storage);
    let mut header_chain = HeaderChain::new(chain_db, Network::Regtest);
    let tip = header_chain.tip().unwrap().unwrap();
    let tip_height = header_chain.tip_height().unwrap().unwrap();
    // confirm 6 times
    // mint tx confirmed block header
    let confirmed_header = gen_block_header(
        tip.header.block_hash(),
        contract_test_utils::mock_timestamp(),
        tip.header.target(),
        merkle_root,
    );
    let mut headers = vec![confirmed_header.clone()];
    let mut prev_header = confirmed_header;
    for _ in 0..5 {
        let header = gen_block_header(
            prev_header.block_hash(),
            prev_header.time + 600,
            prev_header.target(),
            random_merkle_root(),
        );
        headers.push(header.clone());
        prev_header = header;
    }
    header_chain
        .store_headers(
            tip_height + 6,
            headers,
            contract_test_utils::mock_timestamp(),
        )
        .unwrap();

    let msgs = &deps.storage.cosmos_msgs().unwrap();
    context.exec_state_contract_messages(&msgs);
    // query verify merkle proof
    let msg = QueryMsg::VerifyMerkleProof {
        height: tip_height + 1,
        tx: Binary::from(serialize(&txdata[1])),
        merkle_proof: MerkleProofMsg {
            prefix: vec![true],
            siblings: vec![txdata[1].txid().to_string(), txdata[0].txid().to_string()],
        },
    };
    let response = BitcoinSPVRunner::run_query(&mut context, msg).unwrap();
    match from_binary(&response).unwrap() {
        QueryAnswer::VerifyMerkleProof { success } => {
            assert_eq!(success, true)
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_verify_merkle_proof_invalid_merkle_proof() {
    let mut context = init_helper();
    let address = Address::from_str("mqkhEMH6NCeYjFybv7pvFC22MFeaNT9AQC").unwrap();

    let txdata = vec![
        // sibling
        Transaction {
            version: 1,
            lock_time: 0,
            input: vec![],
            output: vec![],
        },
        // mint tx
        Transaction {
            version: 1,
            lock_time: 0,
            input: vec![],
            output: vec![TxOut {
                value: 1000,
                script_pubkey: address.script_pubkey(),
            }],
        },
    ];
    let merkle_root = TxMerkleNode::from_hash(bitcoin_merkle_root(
        txdata.iter().map(|tx| tx.txid().as_hash()),
    ));

    // set block headers to header chain

    let deps = context.client_deps();
    let mut deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )
    .unwrap();
    let chain_db = StorageChainDB::from_storage(&mut deps.storage);
    let mut header_chain = HeaderChain::new(chain_db, Network::Regtest);
    let tip = header_chain.tip().unwrap().unwrap();
    let tip_height = header_chain.tip_height().unwrap().unwrap();
    // confirm 6 times
    // mint tx confirmed block header
    let confirmed_header = gen_block_header(
        tip.header.block_hash(),
        contract_test_utils::mock_timestamp(),
        tip.header.target(),
        merkle_root,
    );
    let mut headers = vec![confirmed_header.clone()];
    let mut prev_header = confirmed_header;
    for _ in 0..5 {
        let header = gen_block_header(
            prev_header.block_hash(),
            prev_header.time + 600,
            prev_header.target(),
            random_merkle_root(),
        );
        headers.push(header.clone());
        prev_header = header;
    }
    header_chain
        .store_headers(
            tip_height + 6,
            headers,
            contract_test_utils::mock_timestamp(),
        )
        .unwrap();
    let msgs = &deps.storage.cosmos_msgs().unwrap();
    context.exec_state_contract_messages(&msgs);

    // CASE: no sibling
    let msg = QueryMsg::VerifyMerkleProof {
        height: tip_height + 1,
        tx: Binary::from(serialize(&txdata[1])),
        merkle_proof: MerkleProofMsg {
            prefix: vec![],
            siblings: vec![],
        },
    };
    let err = BitcoinSPVRunner::run_query(&mut context, msg).unwrap_err();
    assert_eq!(err, StdError::generic_err("merkle path error no sibling"));

    // CASE: merkle path and tx does not match
    let msg = QueryMsg::VerifyMerkleProof {
        height: tip_height + 1,
        tx: Binary::from(serialize(&txdata[1])),
        merkle_proof: MerkleProofMsg {
            prefix: vec![false],
            siblings: vec![txdata[0].txid().to_string(), txdata[1].txid().to_string()],
        },
    };
    let err = BitcoinSPVRunner::run_query(&mut context, msg).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("contract error merkle path and tx does not match")
    );

    // CASE: merkle root does not match
    let msg = QueryMsg::VerifyMerkleProof {
        height: tip_height,
        tx: Binary::from(serialize(&txdata[1])),
        merkle_proof: MerkleProofMsg {
            prefix: vec![true],
            siblings: vec![txdata[1].txid().to_string(), txdata[0].txid().to_string()],
        },
    };
    let err = BitcoinSPVRunner::run_query(&mut context, msg).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("contract error invalid merkle root")
    );
}

#[test]
fn test_verify_mint_tx_not_confirmed() {
    let mut context = init_helper();
    let address = Address::from_str("mqkhEMH6NCeYjFybv7pvFC22MFeaNT9AQC").unwrap();

    let txdata = vec![
        // sibling
        Transaction {
            version: 1,
            lock_time: 0,
            input: vec![],
            output: vec![],
        },
        // mint tx
        Transaction {
            version: 1,
            lock_time: 0,
            input: vec![],
            output: vec![TxOut {
                value: 1000,
                script_pubkey: address.script_pubkey(),
            }],
        },
    ];
    let merkle_root = TxMerkleNode::from_hash(bitcoin_merkle_root(
        txdata.iter().map(|tx| tx.txid().as_hash()),
    ));

    // set block headers to header chain
    let deps = context.client_deps();
    let mut deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )
    .unwrap();
    let chain_db = StorageChainDB::from_storage(&mut deps.storage);
    let mut header_chain = HeaderChain::new(chain_db, Network::Regtest);
    let tip = header_chain.tip().unwrap().unwrap();
    let tip_height = header_chain.tip_height().unwrap().unwrap();
    // mint tx confirmed block header
    let confirmed_header = gen_block_header(
        tip.header.block_hash(),
        contract_test_utils::mock_env("minter", &[]).block.time as u32 - 9,
        tip.header.target(),
        merkle_root,
    );
    // CASE: confirm only 5 times
    let mut headers = vec![confirmed_header.clone()];
    let mut prev_header = confirmed_header;
    for _ in 0..4 {
        let header = gen_block_header(
            prev_header.block_hash(),
            prev_header.time + 600,
            prev_header.target(),
            random_merkle_root(),
        );
        headers.push(header.clone());
        prev_header = header;
    }
    header_chain
        .store_headers(
            tip_height + 5,
            headers,
            contract_test_utils::mock_timestamp(),
        )
        .unwrap();
    let msgs = &deps.storage.cosmos_msgs().unwrap();
    context.exec_state_contract_messages(&msgs);
    // handle verify mint tx
    let msg = QueryMsg::VerifyMerkleProof {
        height: tip_height + 1,
        tx: Binary::from(serialize(&txdata[1])),
        merkle_proof: MerkleProofMsg {
            prefix: vec![true],
            siblings: vec![txdata[1].txid().to_string(), txdata[0].txid().to_string()],
        },
    };
    let err = BitcoinSPVRunner::run_query(&mut context, msg).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("contract error not confirmed yet")
    );
}
