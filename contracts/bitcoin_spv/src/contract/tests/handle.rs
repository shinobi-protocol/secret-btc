use super::*;
use crate::state::chaindb::StorageChainDB;
use bitcoin::consensus::encode::deserialize;
use bitcoin::{BlockHeader, Network};
use bitcoin_header_chain::header_chain::{
    chaindb::ReadonlyChainDB, HeaderChain, StoredBlockHeader,
};
use cosmwasm_std::{Binary, StdError};
use shared_types::bitcoin_spv::HandleMsg;

#[test]
fn test_add_headers_sanity() {
    let mut deps = init_helper();
    // add headers
    let env = helper::mock_env("lebron", &[]);
    for i in 1..101 {
        let header = regtest_block_header(i);
        let handle_msg = HandleMsg::AddHeaders {
            tip_height: i as u32,
            headers: vec![Binary::from(header.clone())],
        };
        handle(&mut deps, env.clone(), handle_msg).unwrap();
        let mut chaindb = StorageChainDB::from_readonly_storage(&deps.storage);
        assert_eq!(chaindb.tip_height().unwrap().unwrap(), i as u32);
        let mut header_chain = HeaderChain::new(chaindb, Network::Regtest);
        let tip: StoredBlockHeader = header_chain.tip().unwrap().unwrap();
        assert_eq!(
            tip.header.block_hash(),
            deserialize::<BlockHeader>(&regtest_block_header(i))
                .unwrap()
                .block_hash()
        );
    }
}

#[test]
fn test_add_headers_sanity_twice() {
    let mut deps = init_helper();
    // add headers
    let env = helper::mock_env("lebron", &[]);
    for i in 1..101 {
        let handle_msg = HandleMsg::AddHeaders {
            tip_height: i as u32,
            headers: vec![Binary::from(regtest_block_header(i))],
        };
        handle(&mut deps, env.clone(), handle_msg.clone()).unwrap();
        handle(&mut deps, env.clone(), handle_msg.clone()).unwrap();
        let mut chaindb = StorageChainDB::from_readonly_storage(&deps.storage);
        assert_eq!(chaindb.tip_height().unwrap().unwrap(), i as u32);
        let mut header_chain = HeaderChain::new(chaindb, Network::Regtest);
        let tip: StoredBlockHeader = header_chain.tip().unwrap().unwrap();
        assert_eq!(
            tip.header.block_hash(),
            deserialize::<BlockHeader>(&regtest_block_header(i))
                .unwrap()
                .block_hash()
        );
    }
}

#[test]
fn test_add_headers_bulk() {
    let mut deps = init_helper();
    let mut headers = vec![];
    for i in 1..101 {
        headers.push(Binary::from(regtest_block_header(i)));
    }
    // add headers
    let handle_msg = HandleMsg::AddHeaders {
        tip_height: 100,
        headers,
    };
    handle(&mut deps, helper::mock_env("lebron", &[]), handle_msg).unwrap();
    let mut chaindb = StorageChainDB::from_readonly_storage(&deps.storage);
    assert_eq!(chaindb.tip_height().unwrap().unwrap(), 100);
    let mut header_chain = HeaderChain::new(chaindb, Network::Regtest);
    let tip: StoredBlockHeader = header_chain.tip().unwrap().unwrap();
    assert_eq!(
        tip.header.block_hash(),
        deserialize::<BlockHeader>(&regtest_block_header(100))
            .unwrap()
            .block_hash()
    );
}

#[test]
fn test_add_headers_empty() {
    let mut deps = init_helper();

    // add headers
    let handle_msg = HandleMsg::AddHeaders {
        tip_height: 1,
        headers: vec![],
    };
    let err = handle(&mut deps, helper::mock_env("lebron", &[]), handle_msg).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("contract error no header in msg")
    )
}
