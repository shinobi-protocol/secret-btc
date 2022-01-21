use crate::state::{
    append_transaction_status, read_config, read_transaction_status, transaction_count,
    update_transaction_status, write_config,
};
use cosmwasm_std::{
    to_binary, Api, CosmosMsg, Env, Extern, HandleResponse, HandleResult, InitResponse, InitResult,
    Querier, QueryResult, StdError, StdResult, Storage,
};
use shared_types::multisig::{
    Config, HandleAnswer, HandleMsg, InitMsg, MultisigStatus, QueryAnswer, QueryMsg,
    TransactionStatus,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> InitResult {
    set_config(deps, msg.config)?;
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    match msg {
        HandleMsg::ChangeConfig { config } => {
            only_multisig(env)?;
            set_config(deps, config)?;
            Ok(HandleResponse::default())
        }
        HandleMsg::SubmitTransaction { transaction } => {
            let config = read_config(&deps.storage, &deps.api)?;
            let mut transaction_status = TransactionStatus {
                transaction,
                config,
                signed_by: vec![],
            };
            transaction_status.sign(&env.message.sender)?;
            let msg = extract_confirmed_msg(&transaction_status);
            let id = append_transaction_status(&mut deps.storage, transaction_status, &deps.api)?;
            Ok(HandleResponse {
                data: Some(to_binary(&HandleAnswer::SubmitTransaction {
                    transaction_id: id,
                })?),
                log: vec![],
                messages: msg,
            })
        }
        HandleMsg::SignTransaction { transaction_id } => {
            let mut status = read_transaction_status(&deps.storage, transaction_id, &deps.api)?;
            status.sign(&env.message.sender)?;
            let msg = extract_confirmed_msg(&status);
            update_transaction_status(&mut deps.storage, transaction_id, status, &deps.api)?;
            Ok(HandleResponse {
                data: None,
                log: vec![],
                messages: msg,
            })
        }
    }
}

fn extract_confirmed_msg(status: &TransactionStatus) -> Vec<CosmosMsg> {
    if status.is_confirmed() {
        vec![status.transaction.clone().into()]
    } else {
        vec![]
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::TransactionStatus { transaction_id } => {
            let status = read_transaction_status(&deps.storage, transaction_id, &deps.api)?;
            to_binary(&QueryAnswer::TransactionStatus(status))
        }
        QueryMsg::MultisigStatus {} => {
            let config = read_config(&deps.storage, &deps.api)?;
            let transaction_count = transaction_count(&deps.storage)?;
            let multisig_status = MultisigStatus {
                config,
                transaction_count,
            };
            to_binary(&QueryAnswer::MultisigStatus(multisig_status))
        }
    }
}

fn only_multisig(env: Env) -> StdResult<()> {
    if env.message.sender == env.contract.address {
        Ok(())
    } else {
        Err(StdError::generic_err("sender is not multisig"))
    }
}

fn set_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    config: Config,
) -> StdResult<()> {
    if config.signers.len() < config.required as usize {
        return Err(StdError::generic_err("invalid required parameter"));
    }
    write_config(&mut deps.storage, config, &deps.api)
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::*;
    use cosmwasm_std::*;
    use shared_types::multisig::Transaction;

    fn mock_config() -> Config {
        Config {
            signers: vec!["signer1".into(), "signer2".into(), "signer3".into()],
            required: 2,
        }
    }

    fn mock_deps_with_config() -> Extern<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies(20, &[]);
        write_config(&mut deps.storage, mock_config(), &deps.api).unwrap();
        deps
    }

    #[test]
    fn test_init() {
        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env("instantiator", &[]);
        let init_msg = InitMsg {
            config: mock_config(),
        };

        init(&mut deps, env, init_msg).unwrap();

        let config = read_config(&deps.storage, &deps.api).unwrap();
        assert_eq!(config, mock_config());
    }

    #[test]
    fn test_change_config_sanity() {
        let mut deps = mock_deps_with_config();
        let env = mock_env(MOCK_CONTRACT_ADDR, &[]);
        let handle_msg = HandleMsg::ChangeConfig {
            config: Config {
                signers: vec![
                    "new_signer1".into(),
                    "new_signer2".into(),
                    "new_signer3".into(),
                ],
                required: 3,
            },
        };

        handle(&mut deps, env, handle_msg).unwrap();

        let config = read_config(&deps.storage, &deps.api).unwrap();
        assert_eq!(
            config,
            Config {
                signers: vec![
                    "new_signer1".into(),
                    "new_signer2".into(),
                    "new_signer3".into(),
                ],
                required: 3,
            }
        );
    }

    #[test]
    fn test_change_config_from_signer() {
        let mut deps = mock_deps_with_config();
        let env = mock_env("signer_1", &[]);
        let handle_msg = HandleMsg::ChangeConfig {
            config: Config {
                signers: vec![
                    "new_signer1".into(),
                    "new_signer2".into(),
                    "new_signer3".into(),
                ],
                required: 3,
            },
        };

        let err = handle(&mut deps, env, handle_msg).unwrap_err();
        assert_eq!(err, StdError::generic_err("sender is not multisig"))
    }

    #[test]
    fn test_submit_transaction_sanity() {
        let mut deps = mock_deps_with_config();
        let env = mock_env("signer1", &[]);
        let transaction = Transaction {
            contract_addr: "contract_addr".into(),
            callback_code_hash: "callback_code_hash".into(),
            msg: Binary::from(&[0, 1, 2]),
            send: vec![Coin::new(100u128, "uscrt")],
        };
        let handle_msg = HandleMsg::SubmitTransaction {
            transaction: transaction.clone(),
        };
        let response = handle(&mut deps, env, handle_msg).unwrap();
        match from_binary(&response.data.unwrap()).unwrap() {
            HandleAnswer::SubmitTransaction { transaction_id } => {
                assert_eq!(transaction_id, 0)
            }
        }
        assert_eq!(response.messages, vec![]);
        let status = read_transaction_status(&deps.storage, 0, &deps.api).unwrap();
        assert_eq!(
            status,
            TransactionStatus {
                transaction: transaction,
                config: mock_config(),
                signed_by: vec![0]
            }
        )
    }

    #[test]
    fn test_submit_transaction_from_foreigner() {
        let mut deps = mock_deps_with_config();
        let env = mock_env("foreigner", &[]);
        let transaction = Transaction {
            contract_addr: "contract_addr".into(),
            callback_code_hash: "callback_code_hash".into(),
            msg: Binary::from(&[0, 1, 2]),
            send: vec![Coin::new(100u128, "uscrt")],
        };
        let handle_msg = HandleMsg::SubmitTransaction {
            transaction: transaction.clone(),
        };
        let err = handle(&mut deps, env, handle_msg).unwrap_err();
        assert_eq!(err, StdError::generic_err("not signer"))
    }

    #[test]
    fn test_change_config_and_submit_transaction() {
        let mut deps = mock_deps_with_config();

        // Submit Transaction From Signers
        let env = mock_env("signer1", &[]);
        let transaction = Transaction {
            contract_addr: "contract_addr".into(),
            callback_code_hash: "callback_code_hash".into(),
            msg: Binary::from(&[0, 1, 2]),
            send: vec![Coin::new(100u128, "uscrt")],
        };
        let handle_msg = HandleMsg::SubmitTransaction {
            transaction: transaction.clone(),
        };
        handle(&mut deps, env, handle_msg).unwrap();

        // Change Signers
        let env = mock_env(MOCK_CONTRACT_ADDR, &[]);
        let handle_msg = HandleMsg::ChangeConfig {
            config: Config {
                signers: vec![
                    "new_signer1".into(),
                    "new_signer2".into(),
                    "new_signer3".into(),
                ],
                required: 3,
            },
        };
        handle(&mut deps, env, handle_msg).unwrap();

        // Submit Transaction From New Signers
        let env = mock_env("new_signer1", &[]);
        let handle_msg = HandleMsg::SubmitTransaction {
            transaction: transaction.clone(),
        };
        handle(&mut deps, env, handle_msg).unwrap();

        // Assert Transation Stautus
        let status = read_transaction_status(&deps.storage, 0, &deps.api).unwrap();
        assert_eq!(
            status,
            TransactionStatus {
                transaction: transaction.clone(),
                config: mock_config(),
                signed_by: vec![0]
            }
        );

        let status = read_transaction_status(&deps.storage, 1, &deps.api).unwrap();
        assert_eq!(
            status,
            TransactionStatus {
                transaction: transaction,
                config: Config {
                    signers: vec![
                        "new_signer1".into(),
                        "new_signer2".into(),
                        "new_signer3".into(),
                    ],
                    required: 3,
                },
                signed_by: vec![0]
            }
        )
    }

    #[test]
    fn test_submit_transaction_and_exec() {
        let mut deps = mock_dependencies(20, &[]);
        write_config(
            &mut deps.storage,
            Config {
                signers: vec!["signer".into()],
                required: 1,
            },
            &deps.api,
        )
        .unwrap();
        let env = mock_env("signer", &[]);
        let transaction = Transaction {
            contract_addr: "contract_addr".into(),
            callback_code_hash: "callback_code_hash".into(),
            msg: Binary::from(&[0, 1, 2]),
            send: vec![Coin::new(100u128, "uscrt")],
        };
        let handle_msg = HandleMsg::SubmitTransaction {
            transaction: transaction.clone(),
        };
        let response = handle(&mut deps, env, handle_msg).unwrap();
        assert_eq!(response.messages, vec![transaction.into()]);
    }

    #[test]
    fn test_sign_transaction() {
        let mut deps = mock_dependencies(20, &[]);
        let transaction = Transaction {
            contract_addr: "contract_addr".into(),
            callback_code_hash: "callback_code_hash".into(),
            msg: Binary::from(&[0, 1, 2]),
            send: vec![Coin::new(100u128, "uscrt")],
        };
        append_transaction_status(
            &mut deps.storage,
            TransactionStatus {
                transaction: transaction.clone(),
                config: Config {
                    signers: vec!["signer1".into(), "signer2".into(), "signer3".into()],
                    required: 3,
                },
                signed_by: vec![0],
            },
            &deps.api,
        )
        .unwrap();

        let env = mock_env("foreigner", &[]);
        let handle_msg = HandleMsg::SignTransaction { transaction_id: 0 };
        let err = handle(&mut deps, env, handle_msg).unwrap_err();
        assert_eq!(err, StdError::generic_err("not signer"));

        let env = mock_env("signer1", &[]);
        let handle_msg = HandleMsg::SignTransaction { transaction_id: 0 };
        let err = handle(&mut deps, env, handle_msg).unwrap_err();
        assert_eq!(err, StdError::generic_err("already signed"));

        let env = mock_env("signer2", &[]);
        let handle_msg = HandleMsg::SignTransaction { transaction_id: 0 };
        let response = handle(&mut deps, env, handle_msg).unwrap();
        assert_eq!(response.messages, vec![]);

        let env = mock_env("signer3", &[]);
        let handle_msg = HandleMsg::SignTransaction { transaction_id: 0 };
        let response = handle(&mut deps, env, handle_msg).unwrap();
        assert_eq!(response.messages, vec![transaction.into()]);
    }

    #[test]
    fn test_query_transaction_status() {
        let mut deps = mock_dependencies(20, &[]);
        let transaction = Transaction {
            contract_addr: "contract_addr".into(),
            callback_code_hash: "callback_code_hash".into(),
            msg: Binary::from(&[0, 1, 2]),
            send: vec![Coin::new(100u128, "uscrt")],
        };
        append_transaction_status(
            &mut deps.storage,
            TransactionStatus {
                transaction: transaction.clone(),
                config: Config {
                    signers: vec!["signer1".into(), "signer2".into(), "signer3".into()],
                    required: 3,
                },
                signed_by: vec![0],
            },
            &deps.api,
        )
        .unwrap();
        let response = query(&deps, QueryMsg::TransactionStatus { transaction_id: 0 }).unwrap();
        match from_binary(&response).unwrap() {
            QueryAnswer::TransactionStatus(status) => {
                assert_eq!(
                    status,
                    TransactionStatus {
                        transaction: transaction.clone(),
                        config: (Config {
                            signers: vec!["signer1".into(), "signer2".into(), "signer3".into()],
                            required: 3,
                        }),
                        signed_by: vec![0],
                    }
                )
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_query_multisig_status() {
        let mut deps = mock_deps_with_config();
        let response = query(&deps, QueryMsg::MultisigStatus {}).unwrap();
        match from_binary(&response).unwrap() {
            QueryAnswer::MultisigStatus(status) => {
                assert_eq!(
                    status,
                    MultisigStatus {
                        config: (mock_config()),
                        transaction_count: 0,
                    }
                )
            }
            _ => unreachable!(),
        }
        let transaction = Transaction {
            contract_addr: "contract_addr".into(),
            callback_code_hash: "callback_code_hash".into(),
            msg: Binary::from(&[0, 1, 2]),
            send: vec![Coin::new(100u128, "uscrt")],
        };
        append_transaction_status(
            &mut deps.storage,
            TransactionStatus {
                transaction: transaction.clone(),
                config: mock_config(),
                signed_by: vec![0],
            },
            &deps.api,
        )
        .unwrap();
        let response = query(&deps, QueryMsg::MultisigStatus {}).unwrap();
        match from_binary(&response).unwrap() {
            QueryAnswer::MultisigStatus(status) => {
                assert_eq!(
                    status,
                    MultisigStatus {
                        config: (mock_config()),
                        transaction_count: 1,
                    }
                )
            }
            _ => unreachable!(),
        }
    }
}
