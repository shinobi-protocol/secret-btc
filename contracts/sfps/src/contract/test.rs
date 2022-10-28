use super::*;
use crate::contract::{init, query::decrypt_response_from_contract};
use contract_test_utils::context::Context;
use contract_test_utils::contract_runner::ContractRunner;
use cosmwasm_std::{testing::mock_env, *};
use sfps_lib::cosmos_proto::prost::Message;
use sfps_lib::header::hash_header;
use sfps_lib::subsequent_hashes::HeaderHashWithHeight;
use sfps_lib::{cosmos_proto::cosmos::base::abci::v1beta1::TxMsgData, subsequent_hashes::Commit};
use shared_types::sfps::{HandleMsg, InitMsg, LightBlock, QueryAnswer, QueryMsg};

pub struct SFPSRunner {}
impl ContractRunner for SFPSRunner {
    type InitMsg = InitMsg;

    type HandleMsg = HandleMsg;

    type QueryMsg = QueryMsg;

    fn init(
        deps: &mut contract_test_utils::context::ClientDeps,
        env: Env,
        msg: Self::InitMsg,
    ) -> StdResult<InitResponse> {
        init(deps, env, msg)
    }

    fn handle(
        deps: &mut contract_test_utils::context::ClientDeps,
        env: Env,
        msg: Self::HandleMsg,
    ) -> HandleResult {
        handle(deps, env, msg)
    }

    fn query(
        deps: &contract_test_utils::context::ClientDeps,
        msg: Self::QueryMsg,
    ) -> StdResult<Binary> {
        query(deps, msg)
    }
}

#[test]
fn test_decrypt_response_from_contract() {
    let tx_msg_data = TxMsgData::decode(Binary::from_base64("CpcDCiovc2VjcmV0LmNvbXB1dGUudjFiZXRhMS5Nc2dFeGVjdXRlQ29udHJhY3QS6AKBPee4v+Bpz0kNkY1ql6L69UewzstC5QTiNgUUusPP/Kww50gICa5ZF8VJdU4P8sz5NuylW8NInA6Oxi8K0DHGznya0GWWS3fYMueM3qL1GsGTBUsz1n5U9vsMd7jJDABh48xEtoLFsjnmAOtQWElBcpgwrwWV8UOq3cd1CNWBbUwkYt399cCMVI25fhGNphSaUIFkMDECSUDjb0dKLSPCG+s+rdCeZqRLIRsuWKudFTslz0/BKus6EHsiCrEtxc0mD0mmXLezAISyR18haN0kMk5N+cnTCvluIsYvC9LeSZgPbmFeYxZmCRLH7x+FM7J7/dxlQ96h+EFLuxK9qXPrOtpx4+YjCz/HQCEhWZnQkzZo2oHEYfe+eixCk4SSV5oYKwOzmk1NJjogpg4BhVUfURth9OwEjg/Pt9oHdxhUWmaappt2T3yS0qlmpvrhqAghrw2KVfJxtKdXv6O16odRsaKRGe2KECc=").unwrap().as_slice()).unwrap();
    let encrypted_response_from_contract = &tx_msg_data.data[0].data;
    let encryption_key =
        Binary::from_base64("c7eyjEabicAp7av5f+HN87ict5G9qSp234k13Amf0TQ=").unwrap();
    let decrypted = decrypt_response_from_contract(
        encrypted_response_from_contract.as_slice(),
        encryption_key.as_slice(),
    )
    .unwrap();
    assert_eq!(
        r#"{"request_release_btc":{"request_key":[148,207,4,220,2,94,93,52,68,131,85,214,9,210,94,242,169,36,60,84,108,145,195,180,49,200,180,228,134,27,75,102]}}                                                                                                         "#,
        std::str::from_utf8(decrypted.as_slice()).unwrap()
    );
}

fn header_hash_with_height_of_light_block(light_block: &LightBlock) -> HeaderHashWithHeight {
    let header = light_block
        .signed_header
        .as_ref()
        .unwrap()
        .header
        .as_ref()
        .unwrap();
    HeaderHashWithHeight {
        hash: hash_header(&header),
        height: header.height,
    }
}

#[test]
fn test_verify_subsequent_light_blocks() {
    let init_msg_json = r#"
        {   
            "seed": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
            "config": {
                "state_proxy": {
                    "address":"state_proxy_address",
                    "hash": "state_proxy_hash"
                }
            },
            "max_interval": 50,
            "initial_header": "CgIICxIIcHVsc2FyLTIYqo3rASILCNzO15YGEMSMzEwqSAog7HH435bYkdOJVKB7OomEJTYjp5uQgjz+phKRlKFs7mESJAgBEiC3K3aPryqmS1ltw39xiAtsPDqbJZdxNsRjpW6o9JJALDIgYzwmHJ4rhhnO8iTHcELLEiaDahrhvtajCXIDirUSBBY6IOOwxEKY/BwUmvv0yJlvuSQnrkHkZJuTTKSVmRt4UrhVQiA3z6HuyyugpHDrYFs6XMLXNrA7vKyN7YqaSOfhRa9pDkogzvkeAn9oilZZOhn02eQoAwZ98jcWJTOYZqfZgxCuPEdSIASAkbx93Cg/d7+/kdc8RNpYw9+KnLyGdAXYt/ParaIvWiBQ1sDjRRnOBFsG3hGPuqhPgNfYy75AAE1gZfzLz4FQp2IgAuS/S819bJvvdw1WS/jGVLSW35v5yFHe/yq55aoXMRBqIOOwxEKY/BwUmvv0yJlvuSQnrkHkZJuTTKSVmRt4UrhVchQxXwpT9yfpGxzEDdqCMdJ72mIlXA==",
            "entropy": "iJiTDs6+YrZHITULnyhjFWW4ciVxKWJ3+O5PYt2pBSM="
        }
        "#;
    let query_msg_json = r#"
        {
            "verify_subsequent_light_blocks": {
                "anchor_header": "CgIICxIIcHVsc2FyLTIYqo3rASILCNzO15YGEMSMzEwqSAog7HH435bYkdOJVKB7OomEJTYjp5uQgjz+phKRlKFs7mESJAgBEiC3K3aPryqmS1ltw39xiAtsPDqbJZdxNsRjpW6o9JJALDIgYzwmHJ4rhhnO8iTHcELLEiaDahrhvtajCXIDirUSBBY6IOOwxEKY/BwUmvv0yJlvuSQnrkHkZJuTTKSVmRt4UrhVQiA3z6HuyyugpHDrYFs6XMLXNrA7vKyN7YqaSOfhRa9pDkogzvkeAn9oilZZOhn02eQoAwZ98jcWJTOYZqfZgxCuPEdSIASAkbx93Cg/d7+/kdc8RNpYw9+KnLyGdAXYt/ParaIvWiBQ1sDjRRnOBFsG3hGPuqhPgNfYy75AAE1gZfzLz4FQp2IgAuS/S819bJvvdw1WS/jGVLSW35v5yFHe/yq55aoXMRBqIOOwxEKY/BwUmvv0yJlvuSQnrkHkZJuTTKSVmRt4UrhVchQxXwpT9yfpGxzEDdqCMdJ72mIlXA==",
                "anchor_header_index": 0,
                "following_light_blocks": [
                    "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",
                    "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",
                    "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",
                    "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",
                    "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"
                ],
                "commit_flags": [true, true, true, true, true],
                "p": "                                                                                                                                                                                                                 "
            }
        }"#;
    // height: [3851968, 3851990, 3852012, 3852034, 3852056] interval=22
    let init_msg: InitMsg = serde_json::from_str(&init_msg_json).unwrap();
    let query_msg: QueryMsg = serde_json::from_str(&query_msg_json).unwrap();

    let mut context = Context::new::<()>(vec![]);
    SFPSRunner::run_init(&mut context, mock_env("initializer", &[]), init_msg).unwrap();
    let committed_hashes = match from_binary(
        &SFPSRunner::run_query(&mut context, query_msg.clone()).unwrap(),
    )
    .unwrap()
    {
        QueryAnswer::VerifySubsequentLightBlocks { committed_hashes } => committed_hashes,
        _ => unreachable!(),
    };
    let (anchor_header, anchor_header_index, following_light_blocks) = match query_msg {
        QueryMsg::VerifySubsequentLightBlocks {
            anchor_header,
            anchor_header_index,
            following_light_blocks,
            ..
        } => (anchor_header, anchor_header_index, following_light_blocks),
        _ => unreachable!(),
    };
    assert_eq!(
        committed_hashes.hashes.anchor_hash,
        hash_header(&anchor_header)
    );
    for (i, hash) in committed_hashes.hashes.following_hashes.iter().enumerate() {
        assert_eq!(
            hash,
            &header_hash_with_height_of_light_block(&following_light_blocks[i])
        )
    }

    let query_msg = QueryMsg::VerifySubsequentLightBlocks {
        anchor_header: anchor_header.clone(),
        anchor_header_index: anchor_header_index,
        following_light_blocks: following_light_blocks.clone(),
        commit_flags: vec![true, false, true, false, true],
    };
    let committed_hashes = match from_binary(
        &SFPSRunner::run_query(&mut context, query_msg.clone()).unwrap(),
    )
    .unwrap()
    {
        QueryAnswer::VerifySubsequentLightBlocks { committed_hashes } => committed_hashes,
        _ => unreachable!(),
    };
    assert_eq!(
        committed_hashes.hashes.anchor_hash,
        hash_header(&anchor_header)
    );
    assert_eq!(
        committed_hashes.hashes.following_hashes,
        vec![
            header_hash_with_height_of_light_block(&following_light_blocks[0]),
            header_hash_with_height_of_light_block(&following_light_blocks[2]),
            header_hash_with_height_of_light_block(&following_light_blocks[4])
        ]
    );

    let query_msg = QueryMsg::VerifySubsequentLightBlocks {
        anchor_header: anchor_header.clone(),
        anchor_header_index: anchor_header_index,
        following_light_blocks: following_light_blocks.clone(),
        commit_flags: vec![true, false, false, true, true],
    };
    let err = SFPSRunner::run_query(&mut context, query_msg.clone()).unwrap_err();
    assert_eq!(
        err.to_string(),
        "Generic error: exceeds interval: max 50, actual 66"
    );

    let query_msg = QueryMsg::VerifySubsequentLightBlocks {
        anchor_header: {
            let mut correct = anchor_header.clone();
            correct.height += 1;
            correct
        },
        anchor_header_index: anchor_header_index,
        following_light_blocks: following_light_blocks.clone(),
        commit_flags: vec![true, true, true, true, true],
    };
    let err = SFPSRunner::run_query(&mut context, query_msg.clone()).unwrap_err();
    assert_eq!(err.to_string(), "Generic error: anchor header unmatched");

    let query_msg = QueryMsg::VerifySubsequentLightBlocks {
        anchor_header: anchor_header.clone(),
        anchor_header_index,
        following_light_blocks: {
            let mut lbs = following_light_blocks.clone();
            let mut commit = lbs[1]
                .signed_header
                .as_mut()
                .unwrap()
                .commit
                .as_mut()
                .unwrap();
            commit.height = 0;
            lbs
        },
        commit_flags: vec![true, false, true, false, true],
    };
    let err = SFPSRunner::run_query(&mut context, query_msg.clone()).unwrap_err();
    assert_eq!(
            err.to_string(),
            "Generic error: light block error: signed header error invalid height: commit 0, header 3851990"
        );

    let query_msg = QueryMsg::VerifySubsequentLightBlocks {
        anchor_header: anchor_header.clone(),
        anchor_header_index,
        following_light_blocks: {
            let mut lbs = following_light_blocks.clone();
            lbs.remove(1);
            lbs
        },
        commit_flags: vec![true, true, true, true],
    };
    let err = SFPSRunner::run_query(&mut context, query_msg.clone()).unwrap_err();
    assert_eq!(err.to_string(), "Generic error: unmatched validators hash");

    let query_msg = QueryMsg::VerifySubsequentLightBlocks {
        anchor_header: anchor_header.clone(),
        anchor_header_index: anchor_header_index + 1,
        following_light_blocks: following_light_blocks.clone(),
        commit_flags: vec![true, true, true, true, true],
    };
    let err = SFPSRunner::run_query(&mut context, query_msg.clone()).unwrap_err();
    assert_eq!(err.to_string(), "Generic error: anchor hash not found");
}

#[test]
fn test_append_subsequent_hashes() {
    let init_msg_json = r#"
        {   
            "seed": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
            "config": {
                "state_proxy": {
                    "address":"state_proxy_address",
                    "hash": "state_proxy_hash"
                }
            },
            "max_interval": 50,
            "initial_header": "CgIICxIIcHVsc2FyLTIYqo3rASILCNzO15YGEMSMzEwqSAog7HH435bYkdOJVKB7OomEJTYjp5uQgjz+phKRlKFs7mESJAgBEiC3K3aPryqmS1ltw39xiAtsPDqbJZdxNsRjpW6o9JJALDIgYzwmHJ4rhhnO8iTHcELLEiaDahrhvtajCXIDirUSBBY6IOOwxEKY/BwUmvv0yJlvuSQnrkHkZJuTTKSVmRt4UrhVQiA3z6HuyyugpHDrYFs6XMLXNrA7vKyN7YqaSOfhRa9pDkogzvkeAn9oilZZOhn02eQoAwZ98jcWJTOYZqfZgxCuPEdSIASAkbx93Cg/d7+/kdc8RNpYw9+KnLyGdAXYt/ParaIvWiBQ1sDjRRnOBFsG3hGPuqhPgNfYy75AAE1gZfzLz4FQp2IgAuS/S819bJvvdw1WS/jGVLSW35v5yFHe/yq55aoXMRBqIOOwxEKY/BwUmvv0yJlvuSQnrkHkZJuTTKSVmRt4UrhVchQxXwpT9yfpGxzEDdqCMdJ72mIlXA==",
            "entropy": "iJiTDs6+YrZHITULnyhjFWW4ciVxKWJ3+O5PYt2pBSM="
        }
        "#;
    let query_msg_json = r#"
        {
            "verify_subsequent_light_blocks": {
                "anchor_header": "CgIICxIIcHVsc2FyLTIYqo3rASILCNzO15YGEMSMzEwqSAog7HH435bYkdOJVKB7OomEJTYjp5uQgjz+phKRlKFs7mESJAgBEiC3K3aPryqmS1ltw39xiAtsPDqbJZdxNsRjpW6o9JJALDIgYzwmHJ4rhhnO8iTHcELLEiaDahrhvtajCXIDirUSBBY6IOOwxEKY/BwUmvv0yJlvuSQnrkHkZJuTTKSVmRt4UrhVQiA3z6HuyyugpHDrYFs6XMLXNrA7vKyN7YqaSOfhRa9pDkogzvkeAn9oilZZOhn02eQoAwZ98jcWJTOYZqfZgxCuPEdSIASAkbx93Cg/d7+/kdc8RNpYw9+KnLyGdAXYt/ParaIvWiBQ1sDjRRnOBFsG3hGPuqhPgNfYy75AAE1gZfzLz4FQp2IgAuS/S819bJvvdw1WS/jGVLSW35v5yFHe/yq55aoXMRBqIOOwxEKY/BwUmvv0yJlvuSQnrkHkZJuTTKSVmRt4UrhVchQxXwpT9yfpGxzEDdqCMdJ72mIlXA==",
                "anchor_header_index": 0,
                "following_light_blocks": [
                    "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",
                    "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",
                    "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",
                    "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",
                    "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"
                ],
                "commit_flags": [true, true, true, true, true],
                "p": "                                                                                                                                                                                                                 "
            }
        }"#;
    let init_msg: InitMsg = serde_json::from_str(&init_msg_json).unwrap();
    let base_query_msg: QueryMsg = serde_json::from_str(&query_msg_json).unwrap();
    let mut context = Context::new::<()>(vec![]);
    SFPSRunner::run_init(&mut context, mock_env("initializer", &[]), init_msg).unwrap();
    let (anchor_header, following_light_blocks) = match base_query_msg {
        QueryMsg::VerifySubsequentLightBlocks {
            anchor_header,
            following_light_blocks,
            ..
        } => (anchor_header, following_light_blocks),
        _ => unreachable!(),
    };
    let query_msg_1 = QueryMsg::VerifySubsequentLightBlocks {
        anchor_header: anchor_header.clone(),
        anchor_header_index: 0,
        following_light_blocks: following_light_blocks[..2].to_vec(),
        commit_flags: vec![true, false],
    };
    let committed_hashes_1 =
        match from_binary(&SFPSRunner::run_query(&mut context, query_msg_1.clone()).unwrap())
            .unwrap()
        {
            QueryAnswer::VerifySubsequentLightBlocks { committed_hashes } => committed_hashes,
            _ => unreachable!(),
        };
    let query_msg_2 = QueryMsg::VerifySubsequentLightBlocks {
        anchor_header: anchor_header.clone(),
        anchor_header_index: 0,
        following_light_blocks: following_light_blocks[..3].to_vec(),
        commit_flags: vec![true, false, true],
    };
    let committed_hashes_2 =
        match from_binary(&SFPSRunner::run_query(&mut context, query_msg_2.clone()).unwrap())
            .unwrap()
        {
            QueryAnswer::VerifySubsequentLightBlocks { committed_hashes } => committed_hashes,
            _ => unreachable!(),
        };

    SFPSRunner::run_handle(
        &mut context,
        mock_env("commtier", &[]),
        HandleMsg::AppendSubsequentHashes {
            committed_hashes: committed_hashes_1,
        },
    )
    .unwrap();

    SFPSRunner::run_handle(
        &mut context,
        mock_env("commtier", &[]),
        HandleMsg::AppendSubsequentHashes {
            committed_hashes: committed_hashes_2.clone(),
        },
    )
    .unwrap();

    SFPSRunner::run_handle(
        &mut context,
        mock_env("commtier", &[]),
        HandleMsg::AppendSubsequentHashes {
            committed_hashes: committed_hashes_2.clone(),
        },
    )
    .unwrap();
    let mut invalid_commit_hashes = committed_hashes_2.clone();
    invalid_commit_hashes.commit =
        Commit::new(&invalid_commit_hashes.hashes, b"invalid_secret").unwrap();
    let err = SFPSRunner::run_handle(
        &mut context,
        mock_env("commtier", &[]),
        HandleMsg::AppendSubsequentHashes {
            committed_hashes: invalid_commit_hashes.clone(),
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("subsequent hashes error: invalid commit")
    );

    let query_msg_3 = QueryMsg::VerifySubsequentLightBlocks {
        anchor_header: following_light_blocks[0]
            .clone()
            .signed_header
            .unwrap()
            .header
            .unwrap(),
        anchor_header_index: 1,
        following_light_blocks: [
            following_light_blocks[1].clone(),
            following_light_blocks[2].clone(),
        ]
        .to_vec(),
        commit_flags: vec![true, true],
    };
    let committed_hashes_3 =
        match from_binary(&SFPSRunner::run_query(&mut context, query_msg_3.clone()).unwrap())
            .unwrap()
        {
            QueryAnswer::VerifySubsequentLightBlocks { committed_hashes } => committed_hashes,
            _ => unreachable!(),
        };
    let query_msg_4 = QueryMsg::VerifySubsequentLightBlocks {
        anchor_header: following_light_blocks[2]
            .clone()
            .signed_header
            .unwrap()
            .header
            .unwrap(),
        anchor_header_index: 2,
        following_light_blocks: [
            following_light_blocks[3].clone(),
            following_light_blocks[4].clone(),
        ]
        .to_vec(),
        commit_flags: vec![true, true],
    };
    let committed_hashes_4 =
        match from_binary(&SFPSRunner::run_query(&mut context, query_msg_4.clone()).unwrap())
            .unwrap()
        {
            QueryAnswer::VerifySubsequentLightBlocks { committed_hashes } => committed_hashes,
            _ => unreachable!(),
        };
    SFPSRunner::run_handle(
        &mut context,
        mock_env("commtier", &[]),
        HandleMsg::AppendSubsequentHashes {
            committed_hashes: committed_hashes_3.clone(),
        },
    )
    .unwrap();
    SFPSRunner::run_handle(
        &mut context,
        mock_env("commtier", &[]),
        HandleMsg::AppendSubsequentHashes {
            committed_hashes: committed_hashes_4.clone(),
        },
    )
    .unwrap();
    match from_binary(&SFPSRunner::run_query(&mut context, QueryMsg::HashListLength {}).unwrap())
        .unwrap()
    {
        QueryAnswer::HashListLength { length } => assert_eq!(length, 5),
        _ => unreachable!(),
    };
    match from_binary(
        &SFPSRunner::run_query(&mut context, QueryMsg::HashByIndex { index: 0 }).unwrap(),
    )
    .unwrap()
    {
        QueryAnswer::HashByIndex { hash, height } => {
            assert_eq!(hash.as_slice(), hash_header(&anchor_header).as_slice());
            assert_eq!(height, 3851946);
        }
        _ => unreachable!(),
    };
    match from_binary(
        &SFPSRunner::run_query(&mut context, QueryMsg::HashByIndex { index: 1 }).unwrap(),
    )
    .unwrap()
    {
        QueryAnswer::HashByIndex { hash, height } => {
            assert_eq!(
                hash.as_slice(),
                hash_header(
                    &following_light_blocks[0]
                        .clone()
                        .signed_header
                        .unwrap()
                        .header
                        .unwrap()
                )
                .as_slice()
            );
            assert_eq!(height, 3851968);
        }
        _ => unreachable!(),
    };

    match from_binary(
        &SFPSRunner::run_query(&mut context, QueryMsg::HashByIndex { index: 2 }).unwrap(),
    )
    .unwrap()
    {
        QueryAnswer::HashByIndex { hash, height } => {
            assert_eq!(
                hash.as_slice(),
                hash_header(
                    &following_light_blocks[2]
                        .clone()
                        .signed_header
                        .unwrap()
                        .header
                        .unwrap()
                )
                .as_slice()
            );
            assert_eq!(height, 3852012);
        }
        _ => unreachable!(),
    };

    match from_binary(
        &SFPSRunner::run_query(&mut context, QueryMsg::HashByIndex { index: 3 }).unwrap(),
    )
    .unwrap()
    {
        QueryAnswer::HashByIndex { hash, height } => {
            assert_eq!(
                hash.as_slice(),
                hash_header(
                    &following_light_blocks[3]
                        .clone()
                        .signed_header
                        .unwrap()
                        .header
                        .unwrap()
                )
                .as_slice()
            );
            assert_eq!(height, 3852034);
        }
        _ => unreachable!(),
    };

    match from_binary(
        &SFPSRunner::run_query(&mut context, QueryMsg::HashByIndex { index: 4 }).unwrap(),
    )
    .unwrap()
    {
        QueryAnswer::HashByIndex { hash, height } => {
            assert_eq!(
                hash.as_slice(),
                hash_header(
                    &following_light_blocks[4]
                        .clone()
                        .signed_header
                        .unwrap()
                        .header
                        .unwrap()
                )
                .as_slice()
            );
            assert_eq!(height, 3852056);
        }
        _ => unreachable!(),
    };
}
