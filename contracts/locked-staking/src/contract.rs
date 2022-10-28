use crate::msg::AdminHandleMsg;
use crate::msg::HandleAnswer;
use crate::msg::HandleMsg;
use crate::msg::InitMsg;
use crate::msg::LockMsg;
use crate::msg::PublicHandleMsg;
use crate::msg::QueryAnswer;
use crate::msg::QueryMsg;
use crate::state::get_latest_staking_info_id;
use crate::state::get_staking_summary;
use crate::state::read_admin;
use crate::state::unlock;
use crate::state::write_admin;
use crate::state::write_staking_summary;
use crate::state::{get_recipients_staking_infos, get_staking_infos, lock};
use cosmwasm_std::CosmosMsg;
use cosmwasm_std::StdResult;
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HandleResult, InitResponse, InitResult, Querier,
    QueryResult, StdError, Storage,
};
use shared_types::ContractReference;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _: InitMsg,
) -> InitResult {
    write_admin(
        &mut deps.storage,
        &deps.api.canonical_address(&env.message.sender)?,
    );
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    match msg {
        HandleMsg::Admin(admin_msg) => {
            if deps.api.canonical_address(&env.message.sender)? != read_admin(&deps.storage)? {
                return Err(StdError::generic_err("message sender is not authorized"));
            }
            match admin_msg {
                AdminHandleMsg::ChangeAdmin { new_admin } => {
                    write_admin(&mut deps.storage, &deps.api.canonical_address(&new_admin)?)
                }
                AdminHandleMsg::SetStakingEndTime { token, end_time } => {
                    let token_address = deps.api.canonical_address(&token)?;
                    let mut staking_summary = get_staking_summary(&deps.storage, &token_address)?;
                    staking_summary.staking_end_time = Some(end_time);
                    write_staking_summary(&mut deps.storage, &token_address, &staking_summary)?;
                }
            }
            Ok(HandleResponse::default())
        }
        HandleMsg::Public(public_msg) => match public_msg {
            PublicHandleMsg::Receive(receive) => {
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
                )?;
                Ok(HandleResponse {
                    messages: vec![],
                    log: vec![],
                    data: Some(to_binary(&HandleAnswer::Receive(info))?),
                })
            }
            PublicHandleMsg::Unlock { ids } => {
                let messages = ids
                    .into_iter()
                    .map(|id| unlock(&mut deps.storage, &deps.api, id, env.block.time))
                    .collect::<StdResult<Vec<CosmosMsg>>>()?;
                Ok(HandleResponse {
                    messages,
                    log: vec![],
                    data: None,
                })
            }
        },
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::LatestID {} => {
            let latest_id = get_latest_staking_info_id(&deps.storage, &deps.api)?;
            to_binary(&QueryAnswer::LatestID(latest_id))
        }
        QueryMsg::StakingInfos { ids } => {
            let staking_infos = get_staking_infos(&deps.storage, &deps.api, ids)?;
            to_binary(&QueryAnswer::StakingInfos(staking_infos))
        }
        QueryMsg::RecipientsStakingInfos {
            recipient,
            page,
            page_size,
        } => {
            let staking_infos = get_recipients_staking_infos(
                &deps.storage,
                &deps.api,
                &recipient,
                page,
                page_size,
            )?;
            to_binary(&QueryAnswer::StakingInfos(staking_infos))
        }
        QueryMsg::StakingSummary { token } => {
            let staking_summary =
                get_staking_summary(&deps.storage, &deps.api.canonical_address(&token)?)?;
            to_binary(&QueryAnswer::StakingSummary(staking_summary))
        }
        QueryMsg::Admin {} => {
            let admin = read_admin(&deps.storage)?;
            to_binary(&QueryAnswer::Admin(deps.api.human_address(&admin)?))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::msg::Snip20ReceiveMsg;
    use crate::state::{StakingInfo, StakingSummary};
    use cosmwasm_std::from_binary;
    use cosmwasm_std::testing::*;
    use cosmwasm_std::HumanAddr;
    use cosmwasm_std::Uint128;
    use secret_toolkit::snip20::send_msg;

    #[test]
    fn test_receive_and_claim() {
        let recipient_1: HumanAddr = "recipient_1".into();
        let recipient_2: HumanAddr = "recipient_2".into();
        let spender_1: HumanAddr = "spender_1".into();
        let spender_2: HumanAddr = "spender_2".into();
        let sender: HumanAddr = "sender".into();
        let unlocker: HumanAddr = "unlocker".into();
        let admin: HumanAddr = "admin".into();
        let new_admin: HumanAddr = "new_admin".into();

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

        let mut deps = mock_dependencies(256, &[]);
        let expected_staking_info_1 = StakingInfo {
            id: 0,
            token: token_1.clone(),
            locker: spender_1.clone(),
            recipient: recipient_1.clone(),
            start_time: start_time_1,
            locked_amount: Uint128(500_000000),
            unlocked: false,
        };
        let expected_staking_info_2 = StakingInfo {
            id: 1,
            token: token_1.clone(),
            locker: spender_2.clone(),
            recipient: recipient_1.clone(),
            start_time: start_time_2,
            locked_amount: Uint128(200_000000),
            unlocked: false,
        };
        let expected_staking_info_3 = StakingInfo {
            id: 2,
            token: token_2.clone(),
            locker: spender_1.clone(),
            recipient: recipient_1.clone(),
            start_time: start_time_3,
            locked_amount: Uint128(300_000000),
            unlocked: false,
        };
        let expected_staking_info_4 = StakingInfo {
            id: 3,
            token: token_1.clone(),
            locker: spender_1.clone(),
            recipient: recipient_2.clone(),
            start_time: start_time_4,
            locked_amount: Uint128(400_000000),
            unlocked: false,
        };

        init(&mut deps, env(admin.clone(), 1_000_000_000), InitMsg {}).unwrap();

        /*
         * 1
         * lock 500_000000 token 1 from spender_1 to recipient_1
         */
        let lock_msg_1 = LockMsg {
            recipient: recipient_1.clone(),
            contract_hash: token_1.hash.clone(),
        };
        let handle_msg_1 = HandleMsg::Public(PublicHandleMsg::Receive(Snip20ReceiveMsg {
            sender: sender.clone(),
            from: spender_1.clone(),
            amount: Uint128(500_000000),
            memo: None,
            msg: Some(serde_json::to_vec(&lock_msg_1).unwrap().into()),
        }));

        let handle_response_1 = handle(
            &mut deps,
            env(token_1.address.clone(), start_time_1),
            handle_msg_1,
        )
        .unwrap();
        assert_eq!(handle_response_1.messages.len(), 0);
        assert_eq!(handle_response_1.log.len(), 0);
        if let HandleAnswer::Receive(staking_info) =
            from_binary(&handle_response_1.data.unwrap()).unwrap()
        {
            assert_eq!(staking_info, expected_staking_info_1);
        } else {
            unreachable!()
        }
        assert_eq!(
            from_binary::<QueryAnswer>(&query(&deps, QueryMsg::LatestID {}).unwrap()).unwrap(),
            QueryAnswer::LatestID(0)
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(&deps, QueryMsg::StakingInfos { ids: vec![0] }).unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingInfos(vec![expected_staking_info_1.clone()])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::RecipientsStakingInfos {
                        recipient: recipient_1.clone(),
                        page: 0,
                        page_size: 10
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingInfos(vec![expected_staking_info_1.clone()])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::StakingSummary {
                        token: token_1.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingSummary(StakingSummary {
                total_locked: Uint128(500_000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(500_000000),
                staking_end_time: None,
            })
        );

        /*
         * 2
         * lock 200_000000 token 1 from spender_2 to recipient_1
         */
        let lock_msg_2 = LockMsg {
            recipient: recipient_1.clone(),
            contract_hash: token_1.hash.clone(),
        };
        let handle_msg_2 = HandleMsg::Public(PublicHandleMsg::Receive(Snip20ReceiveMsg {
            sender: sender.clone(),
            from: spender_2.clone(),
            amount: Uint128(200_000000),
            memo: None,
            msg: Some(serde_json::to_vec(&lock_msg_2).unwrap().into()),
        }));

        let handle_response_2 = handle(
            &mut deps,
            env(token_1.address.clone(), start_time_2),
            handle_msg_2,
        )
        .unwrap();
        assert_eq!(handle_response_2.messages.len(), 0);
        assert_eq!(handle_response_2.log.len(), 0);
        if let HandleAnswer::Receive(staking_info) =
            from_binary(&handle_response_2.data.unwrap()).unwrap()
        {
            assert_eq!(staking_info, expected_staking_info_2);
        } else {
            unreachable!()
        }
        assert_eq!(
            from_binary::<QueryAnswer>(&query(&deps, QueryMsg::LatestID {}).unwrap()).unwrap(),
            QueryAnswer::LatestID(1)
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(&deps, QueryMsg::StakingInfos { ids: vec![0, 1] }).unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingInfos(vec![
                expected_staking_info_1.clone(),
                expected_staking_info_2.clone()
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::RecipientsStakingInfos {
                        recipient: recipient_1.clone(),
                        page: 0,
                        page_size: 10
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingInfos(vec![
                expected_staking_info_1.clone(),
                expected_staking_info_2.clone()
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::StakingSummary {
                        token: token_1.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingSummary(StakingSummary {
                total_locked: Uint128(700_000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(700_000000),
                staking_end_time: None
            })
        );

        /*
         * 3
         * lock 300_000000 token 2 from spender_1 to recipient_1
         */
        let lock_msg_3 = LockMsg {
            recipient: recipient_1.clone(),
            contract_hash: token_2.hash.clone(),
        };
        let handle_msg_3 = HandleMsg::Public(PublicHandleMsg::Receive(Snip20ReceiveMsg {
            sender: sender.clone(),
            from: spender_1.clone(),
            amount: Uint128(300_000000),
            memo: None,
            msg: Some(serde_json::to_vec(&lock_msg_3).unwrap().into()),
        }));

        let handle_response_3 = handle(
            &mut deps,
            env(token_2.address.clone(), start_time_3),
            handle_msg_3,
        )
        .unwrap();
        assert_eq!(handle_response_3.messages.len(), 0);
        assert_eq!(handle_response_3.log.len(), 0);
        if let HandleAnswer::Receive(staking_info) =
            from_binary(&handle_response_3.data.unwrap()).unwrap()
        {
            assert_eq!(staking_info, expected_staking_info_3);
        } else {
            unreachable!()
        }
        assert_eq!(
            from_binary::<QueryAnswer>(&query(&deps, QueryMsg::LatestID {}).unwrap()).unwrap(),
            QueryAnswer::LatestID(2)
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(&deps, QueryMsg::StakingInfos { ids: vec![0, 1, 2] }).unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingInfos(vec![
                expected_staking_info_1.clone(),
                expected_staking_info_2.clone(),
                expected_staking_info_3.clone()
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::RecipientsStakingInfos {
                        recipient: recipient_1.clone(),
                        page: 0,
                        page_size: 10
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingInfos(vec![
                expected_staking_info_1.clone(),
                expected_staking_info_2.clone(),
                expected_staking_info_3.clone(),
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::StakingSummary {
                        token: token_1.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingSummary(StakingSummary {
                total_locked: Uint128(700_000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(700_000000),
                staking_end_time: None
            })
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::StakingSummary {
                        token: token_2.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingSummary(StakingSummary {
                total_locked: Uint128(300_000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(300_000000),
                staking_end_time: None
            })
        );

        /*
         * 4
         * lock 400_000000 token 1 from spender_1 to recipient_2
         */
        let lock_msg_4 = LockMsg {
            recipient: recipient_2.clone(),
            contract_hash: token_1.hash.clone(),
        };
        let handle_msg_4 = HandleMsg::Public(PublicHandleMsg::Receive(Snip20ReceiveMsg {
            sender: sender.clone(),
            from: spender_1.clone(),
            amount: Uint128(400_000000),
            memo: None,
            msg: Some(serde_json::to_vec(&lock_msg_4).unwrap().into()),
        }));

        let handle_response_4 = handle(
            &mut deps,
            env(token_1.address.clone(), start_time_4),
            handle_msg_4,
        )
        .unwrap();
        assert_eq!(handle_response_4.messages.len(), 0);
        assert_eq!(handle_response_4.log.len(), 0);
        if let HandleAnswer::Receive(staking_info) =
            from_binary(&handle_response_4.data.unwrap()).unwrap()
        {
            assert_eq!(staking_info, expected_staking_info_4);
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
                    QueryMsg::StakingInfos {
                        ids: vec![0, 1, 2, 3]
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingInfos(vec![
                expected_staking_info_1.clone(),
                expected_staking_info_2.clone(),
                expected_staking_info_3.clone(),
                expected_staking_info_4.clone()
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::RecipientsStakingInfos {
                        recipient: recipient_1.clone(),
                        page: 0,
                        page_size: 10
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingInfos(vec![
                expected_staking_info_1.clone(),
                expected_staking_info_2.clone(),
                expected_staking_info_3.clone(),
            ])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::RecipientsStakingInfos {
                        recipient: recipient_2.clone(),
                        page: 0,
                        page_size: 10
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingInfos(vec![expected_staking_info_4.clone(),])
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::StakingSummary {
                        token: token_1.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingSummary(StakingSummary {
                total_locked: Uint128(1100_000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(1100_000000),
                staking_end_time: None
            })
        );

        /*
         * 5 try to set end time from not admin user
         */

        let handle_msg_5 = HandleMsg::Public(PublicHandleMsg::Unlock { ids: vec![0] });

        let handle_response_5 = handle(
            &mut deps,
            env(token_1.address.clone(), start_time_4),
            handle_msg_5,
        )
        .unwrap_err();

        assert_eq!(
            handle_response_5,
            StdError::generic_err("staking end time is not defined. staking_id:0")
        );

        /*
         * 6 set stake end time for token1
         */
        let handle_msg_6 = HandleMsg::Admin(AdminHandleMsg::SetStakingEndTime {
            token: token_1.address.clone(),
            end_time: end_time_1,
        });
        let handle_response_6 =
            handle(&mut deps, env(admin.clone(), 1_000_000_000), handle_msg_6).unwrap();
        assert_eq!(handle_response_6.messages.len(), 0);
        assert_eq!(handle_response_6.log.len(), 0);
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::StakingSummary {
                        token: token_1.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingSummary(StakingSummary {
                total_locked: Uint128(1100_000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(1100_000000),
                staking_end_time: Some(end_time_1)
            })
        );

        /*
         * 7 try to unlock at time before stake end time
         */
        let handle_msg_7 = HandleMsg::Public(PublicHandleMsg::Unlock { ids: vec![0] });
        let handle_response_7 = handle(
            &mut deps,
            env(unlocker.clone(), end_time_1 - 1),
            handle_msg_7,
        )
        .unwrap_err();
        assert_eq!(
            handle_response_7,
            StdError::generic_err("staking is not yet over. staking_id:0")
        );

        /*
         * 8 unlock 0
         */
        let handle_msg_8 = HandleMsg::Public(PublicHandleMsg::Unlock { ids: vec![0] });
        let handle_response_8 =
            handle(&mut deps, env(unlocker.clone(), end_time_1), handle_msg_8).unwrap();
        assert_eq!(handle_response_8.messages.len(), 1);
        assert_eq!(
            handle_response_8.messages[0],
            send_msg(
                recipient_1.clone(),
                Uint128(500_000000),
                None,
                None,
                None,
                256,
                token_1.hash.clone(),
                token_1.address.clone()
            )
            .unwrap()
        );
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(&deps, QueryMsg::StakingInfos { ids: vec![0] }).unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingInfos(vec![StakingInfo {
                id: 0,
                token: token_1.clone(),
                locker: spender_1.clone(),
                recipient: recipient_1.clone(),
                start_time: start_time_1,
                locked_amount: Uint128(500_000000),
                unlocked: true,
            }])
        );

        /*
         * 9 try to unlock stake end token and not end token together
         */
        let handle_msg_9 = HandleMsg::Public(PublicHandleMsg::Unlock { ids: vec![1, 2] });
        let handle_response_9 =
            handle(&mut deps, env(unlocker.clone(), end_time_1), handle_msg_9).unwrap_err();
        assert_eq!(
            handle_response_9,
            StdError::generic_err("staking end time is not defined. staking_id:2")
        );

        /*
         * 10 set stake end time for token2
         */
        let handle_msg_10 = HandleMsg::Admin(AdminHandleMsg::SetStakingEndTime {
            token: token_2.address.clone(),
            end_time: end_time_1,
        });
        let handle_response_10 =
            handle(&mut deps, env(admin.clone(), 1_000_000_000), handle_msg_10).unwrap();
        assert_eq!(handle_response_10.messages.len(), 0);
        assert_eq!(handle_response_10.log.len(), 0);
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::StakingSummary {
                        token: token_2.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingSummary(StakingSummary {
                total_locked: Uint128(300000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(300000000),
                staking_end_time: Some(end_time_1)
            })
        );

        /*
         * 11 reset stake end time for token2
         */
        let handle_msg_11 = HandleMsg::Admin(AdminHandleMsg::SetStakingEndTime {
            token: token_2.address.clone(),
            end_time: end_time_2,
        });
        let handle_response_11 =
            handle(&mut deps, env(admin.clone(), 1_000_000_000), handle_msg_11).unwrap();
        assert_eq!(handle_response_11.messages.len(), 0);
        assert_eq!(handle_response_11.log.len(), 0);
        assert_eq!(
            from_binary::<QueryAnswer>(
                &query(
                    &deps,
                    QueryMsg::StakingSummary {
                        token: token_2.address.clone()
                    }
                )
                .unwrap()
            )
            .unwrap(),
            QueryAnswer::StakingSummary(StakingSummary {
                total_locked: Uint128(300000000),
                total_claimed: Uint128::zero(),
                total_remaining: Uint128(300000000),
                staking_end_time: Some(end_time_2)
            })
        );

        /*
         * 12 unlock 2,3
         */
        let handle_msg_12 = HandleMsg::Public(PublicHandleMsg::Unlock { ids: vec![2, 3] });
        let handle_response_12 =
            handle(&mut deps, env(unlocker.clone(), end_time_2), handle_msg_12).unwrap();
        assert_eq!(handle_response_12.messages.len(), 2);
        assert_eq!(
            handle_response_12.messages[0],
            send_msg(
                recipient_1.clone(),
                Uint128(300_000000),
                None,
                None,
                None,
                256,
                token_2.hash.clone(),
                token_2.address.clone()
            )
            .unwrap()
        );
        assert_eq!(
            handle_response_12.messages[1],
            send_msg(
                recipient_2.clone(),
                Uint128(400_000000),
                None,
                None,
                None,
                256,
                token_1.hash.clone(),
                token_1.address.clone()
            )
            .unwrap()
        );

        /*
         * 13 try to unlock twice
         */
        let handle_msg_13 = HandleMsg::Public(PublicHandleMsg::Unlock { ids: vec![2] });
        let handle_response_13 =
            handle(&mut deps, env(unlocker.clone(), end_time_2), handle_msg_13).unwrap_err();

        assert_eq!(
            handle_response_13,
            StdError::generic_err("stake is already unlocked. staking_id:2")
        );

        /*
         * 14 change admin
         */
        let handle_msg_14 = HandleMsg::Admin(AdminHandleMsg::ChangeAdmin {
            new_admin: new_admin.clone(),
        });
        let handle_response_14 = handle(
            &mut deps,
            env(new_admin.clone(), end_time_2),
            handle_msg_14.clone(),
        )
        .unwrap_err();

        assert_eq!(
            handle_response_14,
            StdError::generic_err("message sender is not authorized")
        );
        let handle_response_14 = handle(
            &mut deps,
            env(admin.clone(), end_time_2),
            handle_msg_14.clone(),
        )
        .unwrap();

        assert_eq!(handle_response_14.messages.len(), 0);

        assert_eq!(
            from_binary::<QueryAnswer>(&query(&deps, QueryMsg::Admin {}).unwrap()).unwrap(),
            QueryAnswer::Admin(new_admin)
        );
    }

    fn env(message_sender: HumanAddr, block_time: u64) -> Env {
        let mut env = mock_env(message_sender, &[]);
        env.block.time = block_time;
        env
    }
}
