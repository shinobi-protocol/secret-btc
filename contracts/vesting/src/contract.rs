use crate::msg::HandleAnswer;
use crate::msg::HandleMsg;
use crate::msg::InitMsg;
use crate::msg::LockMsg;
use crate::msg::QueryAnswer;
use crate::msg::QueryMsg;
use crate::state::get_latest_vesting_info_id;
use crate::state::get_vesting_summary;
use crate::state::{claim, get_recipients_vesting_infos, get_vesting_infos, lock};
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HandleResult, InitResponse, InitResult, Querier,
    QueryResult, StdError, Storage,
};
use secret_toolkit::snip20::send_msg;
use shared_types::ContractReference;

pub fn init<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _: InitMsg,
) -> InitResult {
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    match msg {
        HandleMsg::Receive(receive) => {
            let lock_msg: LockMsg = receive
                .deserialize_msg()?
                .ok_or_else(|| StdError::generic_err("no receive msg"))?;
            let info = lock(
                &mut deps.storage,
                &deps.api,
                ContractReference {
                    address: env.message.sender,
                    hash: lock_msg.contract_hash,
                },
                receive.from,
                lock_msg.recipient,
                receive.amount,
                env.block.time,
                lock_msg.end_time,
            )?;
            Ok(HandleResponse {
                messages: vec![],
                log: vec![],
                data: Some(to_binary(&HandleAnswer::Receive(info))?),
            })
        }
        HandleMsg::Claim { id } => {
            let claim_info = claim(&mut deps.storage, &deps.api, id, env.block.time)?;
            let send_msg = send_msg(
                claim_info.recipient,
                claim_info.amount,
                None,
                None,
                None,
                256,
                claim_info.token.hash,
                claim_info.token.address,
            )?;
            Ok(HandleResponse {
                messages: vec![send_msg],
                log: vec![],
                data: None,
            })
        }
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::LatestID {} => {
            let latest_id = get_latest_vesting_info_id(&deps.storage, &deps.api)?;
            to_binary(&QueryAnswer::LatestID(latest_id))
        }
        QueryMsg::VestingInfos { ids } => {
            let vesting_infos = get_vesting_infos(&deps.storage, &deps.api, ids)?;
            to_binary(&QueryAnswer::VestingInfos(vesting_infos))
        }
        QueryMsg::RecipientsVestingInfos {
            recipient,
            page,
            page_size,
        } => {
            let vesting_infos = get_recipients_vesting_infos(
                &deps.storage,
                &deps.api,
                &recipient,
                page,
                page_size,
            )?;
            to_binary(&QueryAnswer::VestingInfos(vesting_infos))
        }
        QueryMsg::VestingSummary { token } => {
            let vesting_summary =
                get_vesting_summary(&deps.storage, &deps.api.canonical_address(&token)?)?;
            to_binary(&QueryAnswer::VestingSummary(vesting_summary))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::msg::Snip20ReceiveMsg;
    use crate::state::{VestingInfo, VestingSummary};
    use cosmwasm_std::from_binary;
    use cosmwasm_std::testing::*;
    use cosmwasm_std::HumanAddr;
    use cosmwasm_std::Uint128;

    #[test]
    fn test_receive_and_claim() {
        let recipient_1: HumanAddr = "recipient_1".into();
        let recipient_2: HumanAddr = "recipient_2".into();
        let spender_1: HumanAddr = "spender_1".into();
        let spender_2: HumanAddr = "spender_2".into();
        let sender: HumanAddr = "sender".into();
        let claimer: HumanAddr = "claimer".into();

        let token_1 = ContractReference {
            address: "token_address_1".into(),
            hash: "token_contract_hash_1".into(),
        };
        let token_2 = ContractReference {
            address: "token_address_2".into(),
            hash: "token_contract_hash_2".into(),
        };

        let start_time_1 = 1_100_000_000;
        let start_time_2 = 1_200_000_000;
        let start_time_3 = 1_300_000_000;
        let start_time_4 = 1_400_000_000;

        let end_time_1 = start_time_1 + 100;
        let end_time_2 = start_time_2 + 200;
        let end_time_3 = start_time_3 + 300;
        let end_time_4 = start_time_4 + 400;

        let mut deps = mock_dependencies(256, &[]);
        /*
         * 1
         * lock 500_000000 token 1 from spender_1 to recipient_1
         */
        let lock_msg_1 = LockMsg {
            recipient: recipient_1.clone(),
            contract_hash: token_1.hash.clone(),
            end_time: end_time_1,
        };
        let handle_msg_1 = HandleMsg::Receive(Snip20ReceiveMsg {
            sender: sender.clone(),
            from: spender_1.clone(),
            amount: Uint128(500_000000),
            memo: None,
            msg: Some(serde_json::to_vec(&lock_msg_1).unwrap().into()),
        });

        let handle_response_1 = handle(
            &mut deps,
            env(token_1.address.clone(), start_time_1),
            handle_msg_1,
        )
        .unwrap();
        assert_eq!(handle_response_1.messages.len(), 0);
        assert_eq!(handle_response_1.log.len(), 0);
        let mut expected_vesting_info_1 = VestingInfo {
            id: 0,
            token: token_1.clone(),
            locker: spender_1.clone(),
            recipient: recipient_1.clone(),
            start_time: start_time_1,
            end_time: end_time_1,
            locked_amount: Uint128(500_000000),
            claimed_amount: Uint128::zero(),
            remaining_amount: Uint128(500_000000),
        };
        if let HandleAnswer::Receive(vesting_info) =
            from_binary(&handle_response_1.data.unwrap()).unwrap()
        {
            assert_eq!(vesting_info, expected_vesting_info_1);
        } else {
            unreachable!()
        }
        assert_eq!(
            from_binary::<QueryAnswer>(&query(&deps, QueryMsg::LatestID {}).unwrap()).unwrap(),
            QueryAnswer::LatestID(0)
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(&deps, QueryMsg::VestingInfos { ids: vec![0] }).unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![expected_vesting_info_1.clone()])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::RecipientsVestingInfos {
                        recipient: recipient_1.clone(),
                        page: 0,
                        page_size: 10
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![expected_vesting_info_1.clone()])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_1.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(500_000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(500_000000),
            })
        );

        /*
         * 2
         * lock 200_000000 token 1 from spender_2 to recipient_1
         */
        let lock_msg_2 = LockMsg {
            recipient: recipient_1.clone(),
            contract_hash: token_1.hash.clone(),
            end_time: end_time_2,
        };
        let handle_msg_2 = HandleMsg::Receive(Snip20ReceiveMsg {
            sender: sender.clone(),
            from: spender_2.clone(),
            amount: Uint128(200_000000),
            memo: None,
            msg: Some(serde_json::to_vec(&lock_msg_2).unwrap().into()),
        });

        let handle_response_2 = handle(
            &mut deps,
            env(token_1.address.clone(), start_time_2),
            handle_msg_2,
        )
        .unwrap();
        assert_eq!(handle_response_2.messages.len(), 0);
        assert_eq!(handle_response_2.log.len(), 0);
        let mut expected_vesting_info_2 = VestingInfo {
            id: 1,
            token: token_1.clone(),
            locker: spender_2.clone(),
            recipient: recipient_1.clone(),
            start_time: start_time_2,
            end_time: end_time_2,
            locked_amount: Uint128(200_000000),
            claimed_amount: Uint128::zero(),
            remaining_amount: Uint128(200_000000),
        };
        if let HandleAnswer::Receive(vesting_info) =
            from_binary(&handle_response_2.data.unwrap()).unwrap()
        {
            assert_eq!(vesting_info, expected_vesting_info_2);
        } else {
            unreachable!()
        }
        assert_eq!(
            from_binary::<QueryAnswer>(&query(&deps, QueryMsg::LatestID {}).unwrap()).unwrap(),
            QueryAnswer::LatestID(1)
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(&deps, QueryMsg::VestingInfos { ids: vec![0, 1] }).unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![
                expected_vesting_info_1.clone(),
                expected_vesting_info_2.clone()
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::RecipientsVestingInfos {
                        recipient: recipient_1.clone(),
                        page: 0,
                        page_size: 10
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![
                expected_vesting_info_1.clone(),
                expected_vesting_info_2.clone()
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_1.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(700_000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(700_000000),
            })
        );

        /*
         * 3
         * lock 300_000000 token 2 from spender_1 to recipient_1
         */
        let lock_msg_3 = LockMsg {
            recipient: recipient_1.clone(),
            contract_hash: token_2.hash.clone(),
            end_time: end_time_3,
        };
        let handle_msg_3 = HandleMsg::Receive(Snip20ReceiveMsg {
            sender: sender.clone(),
            from: spender_1.clone(),
            amount: Uint128(300_000000),
            memo: None,
            msg: Some(serde_json::to_vec(&lock_msg_3).unwrap().into()),
        });

        let handle_response_3 = handle(
            &mut deps,
            env(token_2.address.clone(), start_time_3),
            handle_msg_3,
        )
        .unwrap();
        assert_eq!(handle_response_3.messages.len(), 0);
        assert_eq!(handle_response_3.log.len(), 0);
        let mut expected_vesting_info_3 = VestingInfo {
            id: 2,
            token: token_2.clone(),
            locker: spender_1.clone(),
            recipient: recipient_1.clone(),
            start_time: start_time_3,
            end_time: end_time_3,
            locked_amount: Uint128(300_000000),
            claimed_amount: Uint128::zero(),
            remaining_amount: Uint128(300_000000),
        };
        if let HandleAnswer::Receive(vesting_info) =
            from_binary(&handle_response_3.data.unwrap()).unwrap()
        {
            assert_eq!(vesting_info, expected_vesting_info_3);
        } else {
            unreachable!()
        }
        assert_eq!(
            from_binary::<QueryAnswer>(&query(&deps, QueryMsg::LatestID {}).unwrap()).unwrap(),
            QueryAnswer::LatestID(2)
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(&deps, QueryMsg::VestingInfos { ids: vec![0, 1, 2] }).unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![
                expected_vesting_info_1.clone(),
                expected_vesting_info_2.clone(),
                expected_vesting_info_3.clone()
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::RecipientsVestingInfos {
                        recipient: recipient_1.clone(),
                        page: 0,
                        page_size: 10
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![
                expected_vesting_info_1.clone(),
                expected_vesting_info_2.clone(),
                expected_vesting_info_3.clone(),
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_1.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(700_000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(700_000000),
            })
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_2.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(300_000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(300_000000),
            })
        );

        /*
         * 4
         * lock 400_000000 token 1 from spender_1 to recipient_2
         */
        let lock_msg_4 = LockMsg {
            recipient: recipient_2.clone(),
            contract_hash: token_1.hash.clone(),
            end_time: end_time_4,
        };
        let handle_msg_4 = HandleMsg::Receive(Snip20ReceiveMsg {
            sender: sender.clone(),
            from: spender_1.clone(),
            amount: Uint128(400_000000),
            memo: None,
            msg: Some(serde_json::to_vec(&lock_msg_4).unwrap().into()),
        });

        let handle_response_4 = handle(
            &mut deps,
            env(token_1.address.clone(), start_time_4),
            handle_msg_4,
        )
        .unwrap();
        assert_eq!(handle_response_4.messages.len(), 0);
        assert_eq!(handle_response_4.log.len(), 0);
        let mut expected_vesting_info_4 = VestingInfo {
            id: 3,
            token: token_1.clone(),
            locker: spender_1.clone(),
            recipient: recipient_2.clone(),
            start_time: start_time_4,
            end_time: end_time_4,
            locked_amount: Uint128(400_000000),
            claimed_amount: Uint128::zero(),
            remaining_amount: Uint128(400_000000),
        };
        if let HandleAnswer::Receive(vesting_info) =
            from_binary(&handle_response_4.data.unwrap()).unwrap()
        {
            assert_eq!(vesting_info, expected_vesting_info_4);
        } else {
            unreachable!()
        }
        assert_eq!(
            from_binary::<QueryAnswer>(&query(&deps, QueryMsg::LatestID {}).unwrap()).unwrap(),
            QueryAnswer::LatestID(3)
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingInfos {
                        ids: vec![0, 1, 2, 3]
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![
                expected_vesting_info_1.clone(),
                expected_vesting_info_2.clone(),
                expected_vesting_info_3.clone(),
                expected_vesting_info_4.clone()
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::RecipientsVestingInfos {
                        recipient: recipient_1.clone(),
                        page: 0,
                        page_size: 10
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![
                expected_vesting_info_1.clone(),
                expected_vesting_info_2.clone(),
                expected_vesting_info_3.clone(),
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::RecipientsVestingInfos {
                        recipient: recipient_2.clone(),
                        page: 0,
                        page_size: 10
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![expected_vesting_info_4.clone(),])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_1.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(1100_000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(1100_000000),
            })
        );

        /*
         * 5 claim 1/10 of id 0
         */
        expected_vesting_info_1.claimed_amount = Uint128(50_000000);
        expected_vesting_info_1.remaining_amount = Uint128(450_000000);
        let handle_msg_5 = HandleMsg::Claim { id: 0 };
        let handle_response_5 = handle(
            &mut deps,
            env(claimer.clone(), start_time_1 + 10),
            handle_msg_5,
        )
        .unwrap();
        assert_eq!(handle_response_5.messages.len(), 1);
        assert_eq!(
            handle_response_5.messages[0],
            send_msg(
                recipient_1.clone(),
                Uint128(50_000000),
                None,
                None,
                None,
                256,
                token_1.hash.clone(),
                token_1.address.clone()
            )
            .unwrap()
        );
        assert_eq!(handle_response_5.log.len(), 0);
        assert_eq!(handle_response_5.data, None);
        assert_eq!(
            from_binary::<QueryAnswer>(&query(&deps, QueryMsg::LatestID {}).unwrap()).unwrap(),
            QueryAnswer::LatestID(3)
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingInfos {
                        ids: vec![0, 1, 2, 3]
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![
                expected_vesting_info_1.clone(),
                expected_vesting_info_2.clone(),
                expected_vesting_info_3.clone(),
                expected_vesting_info_4.clone()
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::RecipientsVestingInfos {
                        recipient: recipient_1.clone(),
                        page: 0,
                        page_size: 10
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![
                expected_vesting_info_1.clone(),
                expected_vesting_info_2.clone(),
                expected_vesting_info_3.clone(),
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::RecipientsVestingInfos {
                        recipient: recipient_2.clone(),
                        page: 0,
                        page_size: 10
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![expected_vesting_info_4.clone(),])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_1.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(1100_000000),
                total_claimed: Uint128(50_000000),
                total_remaining: Uint128(1050_000000),
            })
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_2.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(300_000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(300_000000),
            })
        );

        /*
         * 6 claim 2/10 more (3/10 in total) of id 0
         */
        expected_vesting_info_1.claimed_amount = Uint128(150_000000);
        expected_vesting_info_1.remaining_amount = Uint128(350_000000);
        let handle_msg_6 = HandleMsg::Claim { id: 0 };
        let handle_response_6 = handle(
            &mut deps,
            env(claimer.clone(), start_time_1 + 30),
            handle_msg_6,
        )
        .unwrap();
        assert_eq!(handle_response_6.messages.len(), 1);
        assert_eq!(
            handle_response_6.messages[0],
            send_msg(
                recipient_1.clone(),
                Uint128(100_000000),
                None,
                None,
                None,
                256,
                token_1.hash.clone(),
                token_1.address.clone()
            )
            .unwrap()
        );
        assert_eq!(handle_response_6.log.len(), 0);
        assert_eq!(handle_response_6.data, None);
        assert_eq!(
            from_binary::<QueryAnswer>(&query(&deps, QueryMsg::LatestID {}).unwrap()).unwrap(),
            QueryAnswer::LatestID(3)
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingInfos {
                        ids: vec![0, 1, 2, 3]
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![
                expected_vesting_info_1.clone(),
                expected_vesting_info_2.clone(),
                expected_vesting_info_3.clone(),
                expected_vesting_info_4.clone()
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_1.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(1100_000000),
                total_claimed: Uint128(150_000000),
                total_remaining: Uint128(950_000000),
            })
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_2.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(300_000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(300_000000),
            })
        );

        /*
         * 7 claim all of id 0
         */
        expected_vesting_info_1.claimed_amount = Uint128(500_000000);
        expected_vesting_info_1.remaining_amount = Uint128(0);
        let handle_msg_7 = HandleMsg::Claim { id: 0 };
        let handle_response_7 = handle(
            &mut deps,
            env(claimer.clone(), end_time_1 + 10),
            handle_msg_7,
        )
        .unwrap();
        assert_eq!(handle_response_7.messages.len(), 1);
        assert_eq!(
            handle_response_7.messages[0],
            send_msg(
                recipient_1.clone(),
                Uint128(350_000000),
                None,
                None,
                None,
                256,
                token_1.hash.clone(),
                token_1.address.clone()
            )
            .unwrap()
        );
        assert_eq!(handle_response_7.log.len(), 0);
        assert_eq!(handle_response_7.data, None);
        assert_eq!(
            from_binary::<QueryAnswer>(&query(&deps, QueryMsg::LatestID {}).unwrap()).unwrap(),
            QueryAnswer::LatestID(3)
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingInfos {
                        ids: vec![0, 1, 2, 3]
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![
                expected_vesting_info_1.clone(),
                expected_vesting_info_2.clone(),
                expected_vesting_info_3.clone(),
                expected_vesting_info_4.clone()
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_1.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(1100_000000),
                total_claimed: Uint128(500_000000),
                total_remaining: Uint128(600_000000),
            })
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_2.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(300_000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(300_000000),
            })
        );

        /*
         * 8 claim id 0 after all claimed
         */
        let handle_msg_8 = HandleMsg::Claim { id: 0 };
        let handle_response_8 = handle(
            &mut deps,
            env(claimer.clone(), end_time_1 + 20),
            handle_msg_8,
        )
        .unwrap();
        assert_eq!(handle_response_8.messages.len(), 1);
        assert_eq!(
            handle_response_8.messages[0],
            send_msg(
                recipient_1.clone(),
                Uint128(0),
                None,
                None,
                None,
                256,
                token_1.hash.clone(),
                token_1.address.clone()
            )
            .unwrap()
        );
        assert_eq!(handle_response_8.log.len(), 0);
        assert_eq!(handle_response_8.data, None);
        assert_eq!(
            from_binary::<QueryAnswer>(&query(&deps, QueryMsg::LatestID {}).unwrap()).unwrap(),
            QueryAnswer::LatestID(3)
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingInfos {
                        ids: vec![0, 1, 2, 3]
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![
                expected_vesting_info_1.clone(),
                expected_vesting_info_2.clone(),
                expected_vesting_info_3.clone(),
                expected_vesting_info_4.clone()
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_1.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(1100_000000),
                total_claimed: Uint128(500_000000),
                total_remaining: Uint128(600_000000),
            })
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_2.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(300_000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(300_000000),
            })
        );

        /*
         * 9 claim half of id 2
         */
        expected_vesting_info_3.claimed_amount = Uint128(150_000000);
        expected_vesting_info_3.remaining_amount = Uint128(150_000000);
        let handle_msg_9 = HandleMsg::Claim { id: 2 };
        let handle_response_9 = handle(
            &mut deps,
            env(claimer.clone(), start_time_3 + 150),
            handle_msg_9,
        )
        .unwrap();
        assert_eq!(handle_response_9.messages.len(), 1);
        assert_eq!(
            handle_response_9.messages[0],
            send_msg(
                recipient_1.clone(),
                Uint128(150_000000),
                None,
                None,
                None,
                256,
                token_2.hash.clone(),
                token_2.address.clone()
            )
            .unwrap()
        );
        assert_eq!(handle_response_9.log.len(), 0);
        assert_eq!(handle_response_9.data, None);
        assert_eq!(
            from_binary::<QueryAnswer>(&query(&deps, QueryMsg::LatestID {}).unwrap()).unwrap(),
            QueryAnswer::LatestID(3)
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingInfos {
                        ids: vec![0, 1, 2, 3]
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![
                expected_vesting_info_1.clone(),
                expected_vesting_info_2.clone(),
                expected_vesting_info_3.clone(),
                expected_vesting_info_4.clone()
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_1.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(1100_000000),
                total_claimed: Uint128(500_000000),
                total_remaining: Uint128(600_000000),
            })
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_2.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(300_000000),
                total_claimed: Uint128(150_000000),
                total_remaining: Uint128(150_000000),
            })
        );

        /*
         * 10 claim half of id 3
         */
        expected_vesting_info_4.claimed_amount = Uint128(200_000000);
        expected_vesting_info_4.remaining_amount = Uint128(200_000000);
        let handle_msg_10 = HandleMsg::Claim { id: 3 };
        let handle_response_10 = handle(
            &mut deps,
            env(claimer.clone(), start_time_4 + 200),
            handle_msg_10,
        )
        .unwrap();
        assert_eq!(handle_response_10.messages.len(), 1);
        assert_eq!(
            handle_response_10.messages[0],
            send_msg(
                recipient_2.clone(),
                Uint128(200_000000),
                None,
                None,
                None,
                256,
                token_1.hash.clone(),
                token_1.address.clone()
            )
            .unwrap()
        );
        assert_eq!(handle_response_10.log.len(), 0);
        assert_eq!(handle_response_10.data, None);
        assert_eq!(
            from_binary::<QueryAnswer>(&query(&deps, QueryMsg::LatestID {}).unwrap()).unwrap(),
            QueryAnswer::LatestID(3)
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingInfos {
                        ids: vec![0, 1, 2, 3]
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![
                expected_vesting_info_1.clone(),
                expected_vesting_info_2.clone(),
                expected_vesting_info_3.clone(),
                expected_vesting_info_4.clone()
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_1.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(1100_000000),
                total_claimed: Uint128(700_000000),
                total_remaining: Uint128(400_000000),
            })
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_2.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(300_000000),
                total_claimed: Uint128(150_000000),
                total_remaining: Uint128(150_000000),
            })
        );
        /*
         * claim all of id 1
         */
        expected_vesting_info_2.claimed_amount = Uint128(200_000000);
        expected_vesting_info_2.remaining_amount = Uint128(0);
        let handle_msg_11 = HandleMsg::Claim { id: 1 };
        let handle_response_11 = handle(
            &mut deps,
            env(claimer.clone(), start_time_2 + 10000),
            handle_msg_11,
        )
        .unwrap();
        assert_eq!(handle_response_11.messages.len(), 1);
        assert_eq!(
            handle_response_11.messages[0],
            send_msg(
                recipient_1.clone(),
                Uint128(200_000000),
                None,
                None,
                None,
                256,
                token_1.hash.clone(),
                token_1.address.clone()
            )
            .unwrap()
        );
        assert_eq!(handle_response_11.log.len(), 0);
        assert_eq!(handle_response_11.data, None);
        assert_eq!(
            from_binary::<QueryAnswer>(&query(&deps, QueryMsg::LatestID {}).unwrap()).unwrap(),
            QueryAnswer::LatestID(3)
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingInfos {
                        ids: vec![0, 1, 2, 3]
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingInfos(vec![
                expected_vesting_info_1.clone(),
                expected_vesting_info_2.clone(),
                expected_vesting_info_3.clone(),
                expected_vesting_info_4.clone()
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_1.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(1100_000000),
                total_claimed: Uint128(900_000000),
                total_remaining: Uint128(200_000000),
            })
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::VestingSummary {
                        token: token_2.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::VestingSummary(VestingSummary {
                total_locked: Uint128(300_000000),
                total_claimed: Uint128(150_000000),
                total_remaining: Uint128(150_000000),
            })
        );
    }

    fn env(message_sender: HumanAddr, block_time: u64) -> Env {
        let mut env = mock_env(message_sender, &[]);
        env.block.time = block_time;
        env
    }
}
