use crate::config::Config;
use crate::ibm::{fee, reward, split_reward};
use crate::msg::{CustomHandleMsg, CustomQueryAnswer, CustomQueryMsg, InitMsg};
use crate::state::{read_config, read_total_minted, write_config, write_total_minted};
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HandleResult, InitResponse, InitResult, Querier,
    QueryResult, ReadonlyStorage, StdError, StdResult, Storage, Uint128,
};
use secret_toolkit::snip20;
use secret_toolkit::utils::{pad_handle_result, HandleCallback};
use shared_types::{finance_admin, gateway, shuriken, treasury, BLOCK_SIZE};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> InitResult {
    write_config(&mut deps.storage, &deps.api, msg.config)?;
    Ok(InitResponse::default())
}

fn check_sender_is_owner(config: &Config, env: &Env) -> StdResult<()> {
    if env.message.sender == config.owner {
        Ok(())
    } else {
        Err(StdError::generic_err("message sender is not owner"))
    }
}

fn check_sender_is_gateway(config: &Config, env: &Env) -> StdResult<()> {
    if env.message.sender == config.gateway.address {
        Ok(())
    } else {
        Err(StdError::generic_err("message sender is not gateway"))
    }
}

fn check_sender_is_shuriken(config: &Config, env: &Env) -> StdResult<()> {
    if env.message.sender == config.shuriken.address {
        Ok(())
    } else {
        Err(StdError::generic_err("message sender is not shuriken"))
    }
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: finance_admin::HandleMsg<CustomHandleMsg>,
) -> HandleResult {
    let response =
        match msg {
            finance_admin::HandleMsg::Custom { custom_msg } => match custom_msg {
                CustomHandleMsg::TransferOwnership { owner } => {
                    let mut config = read_config(&deps.storage, &deps.api)?;
                    check_sender_is_owner(&config, &env)?;
                    config.owner = owner;
                    write_config(&mut deps.storage, &deps.api, config)?;
                    Ok(HandleResponse::default())
                }
            },
            finance_admin::HandleMsg::Migrate { new_finance_admin } => {
                let config = read_config(&deps.storage, &deps.api)?;
                check_sender_is_owner(&config, &env)?;
                let messages =
                    vec![
                        treasury::HandleMsg::TransferOwnership {
                            owner: new_finance_admin.address.clone(),
                        }
                        .to_cosmos_msg(
                            config.treasury.hash,
                            config.treasury.address,
                            None,
                        )?,
                        gateway::HandleMsg::ChangeFinanceAdmin {
                            new_finance_admin: new_finance_admin.clone(),
                        }
                        .to_cosmos_msg(
                            config.gateway.hash,
                            config.gateway.address,
                            None,
                        )?,
                        shuriken::HandleMsg::ChangeFinanceAdmin { new_finance_admin }
                            .to_cosmos_msg(config.shuriken.hash, config.shuriken.address, None)?,
                    ];
                Ok(HandleResponse {
                    messages,
                    log: vec![],
                    data: None,
                })
            }
            finance_admin::HandleMsg::SendMintReward {
                minter,
                sbtc_mint_amount,
                sbtc_total_supply: _,
            } => {
                let config = read_config(&deps.storage, &deps.api)?;
                check_sender_is_gateway(&config, &env)?;
                let (minter_reward, developer_reward, total_minted) =
                    calc_mint_rewards(&deps.storage, sbtc_mint_amount);
                write_total_minted(&mut deps.storage, total_minted);
                let operations = vec![
                    treasury::Operation::Send {
                        to: minter,
                        amount: minter_reward,
                    },
                    treasury::Operation::Send {
                        to: config.developer_address,
                        amount: developer_reward,
                    },
                ];
                let message = treasury::HandleMsg::Operate { operations }.to_cosmos_msg(
                    config.treasury.hash,
                    config.treasury.address,
                    None,
                )?;
                Ok(HandleResponse {
                    messages: vec![message],
                    log: vec![],
                    data: None,
                })
            }
            finance_admin::HandleMsg::ReceiveReleaseFee {
                releaser,
                sbtc_release_amount,
                sbtc_total_supply: _,
            } => {
                let config = read_config(&deps.storage, &deps.api)?;
                check_sender_is_gateway(&config, &env)?;
                let fee = fee(sbtc_release_amount.into());
                let operations = vec![treasury::Operation::ReceiveFrom {
                    from: releaser,
                    amount: fee.into(),
                }];
                let message = treasury::HandleMsg::Operate { operations }.to_cosmos_msg(
                    config.treasury.hash,
                    config.treasury.address,
                    None,
                )?;
                Ok(HandleResponse {
                    messages: vec![message],
                    log: vec![],
                    data: None,
                })
            }
            finance_admin::HandleMsg::MintBitcoinSPVReward {
                executer,
                best_height,
            } => {
                // TODO implement reward calculation
                let config = read_config(&deps.storage, &deps.api)?;
                check_sender_is_shuriken(&config, &env)?;
                let message = snip20::mint_msg(
                    executer,
                    (best_height as u64).into(),
                    None,
                    BLOCK_SIZE,
                    config.snb.hash,
                    config.snb.address,
                )?;
                Ok(HandleResponse {
                    messages: vec![message],
                    log: vec![],
                    data: None,
                })
            }
            finance_admin::HandleMsg::MintSFPSReward {
                executer,
                best_height,
            } => {
                // TODO implement reward calculation
                let config = read_config(&deps.storage, &deps.api)?;
                check_sender_is_shuriken(&config, &env)?;
                let message = snip20::mint_msg(
                    executer,
                    best_height.into(),
                    None,
                    BLOCK_SIZE,
                    config.snb.hash,
                    config.snb.address,
                )?;
                Ok(HandleResponse {
                    messages: vec![message],
                    log: vec![],
                    data: None,
                })
            }
        };
    pad_handle_result(response, BLOCK_SIZE)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: finance_admin::QueryMsg<CustomQueryMsg>,
) -> QueryResult {
    match msg {
        finance_admin::QueryMsg::Custom { custom_msg } => match custom_msg {
            CustomQueryMsg::Config {} => {
                let config = read_config(&deps.storage, &deps.api)?;
                to_binary(&CustomQueryAnswer::Config(config))
            }
            CustomQueryMsg::TotalMintedSbtc {} => {
                let total_minted = read_total_minted(&deps.storage);
                to_binary(&CustomQueryAnswer::TotalMintedSbtc(total_minted))
            }
        },
        finance_admin::QueryMsg::MintReward {
            minter,
            sbtc_mint_amount,
            sbtc_total_supply: _,
        } => {
            let config = read_config(&deps.storage, &deps.api)?;
            let (minter_reward, developer_reward, _) =
                calc_mint_rewards(&deps.storage, sbtc_mint_amount);
            let answer: finance_admin::QueryAnswer = vec![
                treasury::Operation::Send {
                    to: minter,
                    amount: minter_reward,
                },
                treasury::Operation::Send {
                    to: config.developer_address,
                    amount: developer_reward,
                },
            ];
            to_binary(&answer)
        }
        finance_admin::QueryMsg::ReleaseFee {
            releaser,
            sbtc_release_amount,
            sbtc_total_supply: _,
        } => {
            let fee = fee(sbtc_release_amount.into());
            let answer: finance_admin::QueryAnswer = vec![treasury::Operation::ReceiveFrom {
                from: releaser,
                amount: fee.into(),
            }];
            to_binary(&answer)
        }
    }
}

// returns (mint reward, developer reward, total mint amount)
fn calc_mint_rewards<S: ReadonlyStorage>(
    storage: &S,
    mint_amount: Uint128,
) -> (Uint128, Uint128, Uint128) {
    let total_minted = read_total_minted(storage);
    let (minter_reward, developer_reward) =
        split_reward(reward(mint_amount.into(), total_minted.into()));
    (
        minter_reward.into(),
        developer_reward.into(),
        total_minted + mint_amount,
    )
}

#[cfg(test)]

mod test {
    use super::*;
    use contract_test_utils::assert_handle_response_message;
    use cosmwasm_std::testing::*;
    use shared_types::ContractReference;

    fn init_helper() -> Extern<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies(20, &[]);
        init(
            &mut deps,
            mock_env("initializer", &[]),
            InitMsg {
                config: Config {
                    owner: "owner".into(),
                    gateway: ContractReference {
                        address: "gateway_address".into(),
                        hash: "gateway_hash".into(),
                    },
                    treasury: ContractReference {
                        address: "treasury_address".into(),
                        hash: "treasury_hash".into(),
                    },
                    shuriken: ContractReference {
                        address: "shuriken_address".into(),
                        hash: "shuriken_hash".into(),
                    },
                    snb: ContractReference {
                        address: "snb_address".into(),
                        hash: "snb_hash".into(),
                    },
                    developer_address: "developer".into(),
                },
            },
        )
        .unwrap();
        deps
    }

    #[test]
    fn test_transfer_ownership() {
        let mut deps = init_helper();
        let handle_msg = finance_admin::HandleMsg::<CustomHandleMsg>::Custom {
            custom_msg: CustomHandleMsg::TransferOwnership {
                owner: "new_owner".into(),
            },
        };
        assert_eq!(
            handle(&mut deps, mock_env("not_owner", &[]), handle_msg.clone()).unwrap_err(),
            StdError::generic_err("message sender is not owner")
        );
        handle(&mut deps, mock_env("owner", &[]), handle_msg.clone()).unwrap();
        assert_eq!(
            read_config(&deps.storage, &deps.api).unwrap().owner,
            "new_owner".into()
        );
        assert_eq!(
            handle(&mut deps, mock_env("owner", &[]), handle_msg.clone()).unwrap_err(),
            StdError::generic_err("message sender is not owner")
        );
    }

    #[test]
    fn test_migrate() {
        let mut deps = init_helper();
        let new_finance_admin = ContractReference {
            address: "new_address".into(),
            hash: "new_hash".into(),
        };
        let handle_msg = finance_admin::HandleMsg::<CustomHandleMsg>::Migrate {
            new_finance_admin: new_finance_admin.clone(),
        };
        assert_eq!(
            handle(&mut deps, mock_env("not_owner", &[]), handle_msg.clone()).unwrap_err(),
            StdError::generic_err("message sender is not owner")
        );
        let response = handle(&mut deps, mock_env("owner", &[]), handle_msg.clone()).unwrap();
        assert_eq!(
            response.messages,
            vec![
                treasury::HandleMsg::TransferOwnership {
                    owner: new_finance_admin.address.clone(),
                }
                .to_cosmos_msg("treasury_hash".into(), "treasury_address".into(), None,)
                .unwrap(),
                gateway::HandleMsg::ChangeFinanceAdmin {
                    new_finance_admin: new_finance_admin.clone(),
                }
                .to_cosmos_msg("gateway_hash".into(), "gateway_address".into(), None,)
                .unwrap(),
                shuriken::HandleMsg::ChangeFinanceAdmin { new_finance_admin }
                    .to_cosmos_msg("shuriken_hash".into(), "shuriken_address".into(), None,)
                    .unwrap(),
            ]
        );
    }

    #[test]
    fn test_calc_mint_rewards() {
        let mut deps = init_helper();
        assert_eq!(
            calc_mint_rewards(&deps.storage, Uint128::from(100_000_000u128)),
            (
                Uint128::from(33_000_000_000_000u128 * 8 / 10),
                Uint128::from(33_000_000_000_000u128 * 2 / 10),
                Uint128::from(100_000_000u128)
            )
        );
        write_total_minted(&mut deps.storage, Uint128::from(7_000_000_000u128));
        assert_eq!(
            calc_mint_rewards(&deps.storage, Uint128::from(100_000_000u128)),
            (
                Uint128::from(26_400_000_000_000u128 * 8 / 10),
                Uint128::from(26_400_000_000_000u128 * 2 / 10),
                Uint128::from(7_100_000_000u128)
            )
        );
    }

    #[test]
    fn test_send_mint_reward() {
        let handle_msg = finance_admin::HandleMsg::<CustomHandleMsg>::SendMintReward {
            minter: "minter".into(),
            sbtc_mint_amount: Uint128::from(100_000_000u128),
            sbtc_total_supply: Uint128::from(1_000_000_000u128),
        };
        let mut deps = init_helper();
        assert_eq!(
            handle(
                &mut deps,
                mock_env("not_gateway_address", &[]),
                handle_msg.clone()
            )
            .unwrap_err(),
            StdError::generic_err("message sender is not gateway")
        );

        let response = handle(
            &mut deps,
            mock_env("gateway_address", &[]),
            handle_msg.clone(),
        )
        .unwrap();
        assert_eq!(response.messages.len(), 1);
        assert_handle_response_message(
            &response.messages[0],
            "treasury_address",
            "treasury_hash",
            &treasury::HandleMsg::Operate {
                operations: vec![
                    treasury::Operation::Send {
                        to: "minter".into(),
                        amount: Uint128::from(33_000_000_000_000u128 * 8 / 10),
                    },
                    treasury::Operation::Send {
                        to: "developer".into(),
                        amount: Uint128::from(33_000_000_000_000u128 * 2 / 10),
                    },
                ],
            },
        );
        assert_eq!(
            read_total_minted(&deps.storage),
            Uint128::from(100_000_000u128)
        );

        write_total_minted(&mut deps.storage, Uint128::from(7_000_000_000u128));
        let response = handle(
            &mut deps,
            mock_env("gateway_address", &[]),
            handle_msg.clone(),
        )
        .unwrap();
        assert_eq!(response.messages.len(), 1);
        assert_handle_response_message(
            &response.messages[0],
            "treasury_address",
            "treasury_hash",
            &treasury::HandleMsg::Operate {
                operations: vec![
                    treasury::Operation::Send {
                        to: "minter".into(),
                        amount: Uint128::from(26_400_000_000_000u128 * 8 / 10),
                    },
                    treasury::Operation::Send {
                        to: "developer".into(),
                        amount: Uint128::from(26_400_000_000_000u128 * 2 / 10),
                    },
                ],
            },
        );
        assert_eq!(
            read_total_minted(&deps.storage),
            Uint128::from(7_100_000_000u128)
        );
    }

    #[test]
    fn test_receive_release_fee() {
        let mut deps = init_helper();
        let handle_msg = finance_admin::HandleMsg::<CustomHandleMsg>::ReceiveReleaseFee {
            releaser: "releaser".into(),
            sbtc_release_amount: Uint128::from(100_000_000u128),
            sbtc_total_supply: Uint128::from(1_000_000_000u128),
        };
        assert_eq!(
            handle(
                &mut deps,
                mock_env("not_gateway_address", &[]),
                handle_msg.clone()
            )
            .unwrap_err(),
            StdError::generic_err("message sender is not gateway")
        );
        let response = handle(&mut deps, mock_env("gateway_address", &[]), handle_msg).unwrap();
        assert_eq!(response.messages.len(), 1);
        assert_handle_response_message(
            &response.messages[0],
            "treasury_address",
            "treasury_hash",
            &treasury::HandleMsg::Operate {
                operations: vec![treasury::Operation::ReceiveFrom {
                    from: "releaser".into(),
                    amount: Uint128::from(33_000_000_000_000u128),
                }],
            },
        );
    }

    #[test]
    fn test_mint_bitcoin_spv_reward() {
        let mut deps = init_helper();
        let handle_msg = finance_admin::HandleMsg::<CustomHandleMsg>::MintBitcoinSPVReward {
            executer: "executer".into(),
            best_height: 1000,
        };
        assert_eq!(
            handle(
                &mut deps,
                mock_env("not_shuriken_address", &[]),
                handle_msg.clone()
            )
            .unwrap_err(),
            StdError::generic_err("message sender is not shuriken")
        );
        let response = handle(&mut deps, mock_env("shuriken_address", &[]), handle_msg).unwrap();
        assert_eq!(response.messages.len(), 1);
        assert_eq!(
            response.messages[0],
            snip20::mint_msg(
                "executer".into(),
                1000u64.into(),
                None,
                BLOCK_SIZE,
                "snb_hash".into(),
                "snb_address".into(),
            )
            .unwrap(),
        );
    }

    #[test]
    fn test_mint_sfps_reward() {
        let mut deps = init_helper();
        let handle_msg = finance_admin::HandleMsg::<CustomHandleMsg>::MintSFPSReward {
            executer: "executer".into(),
            best_height: 1000,
        };
        assert_eq!(
            handle(
                &mut deps,
                mock_env("not_shuriken_address", &[]),
                handle_msg.clone()
            )
            .unwrap_err(),
            StdError::generic_err("message sender is not shuriken")
        );
        let response = handle(&mut deps, mock_env("shuriken_address", &[]), handle_msg).unwrap();
        assert_eq!(response.messages.len(), 1);
        assert_eq!(
            response.messages[0],
            snip20::mint_msg(
                "executer".into(),
                1000u64.into(),
                None,
                BLOCK_SIZE,
                "snb_hash".into(),
                "snb_address".into(),
            )
            .unwrap(),
        );
    }
}
