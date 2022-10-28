use cosmwasm_std::{
    Api, Env, Extern, HandleResult, InitResponse, InitResult, Querier, QueryResult, Storage,
};
use shared_types::state_proxy::msg::{HandleMsg, InitMsg, QueryMsg};

use shared_types::state_proxy::server::{set_admin, ContractsOwnerStore};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> InitResult {
    set_admin(
        &mut deps.storage,
        &deps.api.canonical_address(&env.message.sender)?,
    );
    let mut owner_store = ContractsOwnerStore::from(&mut deps.storage);
    for (contract_label, owner) in msg.contract_owners.into_iter() {
        owner_store.write(contract_label.as_slice(), owner, &deps.api)?;
    }
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    shared_types::state_proxy::server::handle(deps, env, msg)
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    shared_types::state_proxy::server::query(deps, msg)
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::*;
    use cosmwasm_std::*;
    use cosmwasm_storage::PrefixedStorage;
    use rand::thread_rng;
    use shared_types::state_proxy::client::Secp256k1ApiSigner;
    use shared_types::state_proxy::msg::ReadContractStateSignature;

    use shared_types::state_proxy::msg::{InitMsg, Owner, QueryAnswer, QueryMsg, WriteAction};
    use shared_types::state_proxy::server::read_admin;

    const CONTRACTS_STATE_KEY: &[u8] = b"contracts_state";

    fn mock_deps() -> Extern<MockStorage, MockApi, MockQuerier> {
        mock_dependencies(20, &[])
    }

    #[test]
    fn test_init() {
        let mut deps = mock_deps();
        let resp = init(
            &mut deps,
            mock_env("admin", &[]),
            InitMsg {
                contract_owners: vec![
                    (
                        Binary::from(b"contract_1"),
                        Owner {
                            address: "owner_1".into(),
                            public_key: Binary::from(b"pubkey_1"),
                        },
                    ),
                    (
                        Binary::from(b"contract_2"),
                        Owner {
                            address: "owner_2".into(),
                            public_key: Binary::from(b"pubkey_2"),
                        },
                    ),
                ],
            },
        )
        .unwrap();

        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);

        let admin = read_admin(&deps.storage);
        let canonical_admin = deps
            .api
            .canonical_address(&HumanAddr::from("admin"))
            .unwrap();
        assert_eq!(canonical_admin, admin);

        let owner_store = ContractsOwnerStore::from_readonly(&deps.storage);

        let owner1 = owner_store
            .read(Binary::from(b"contract_1").as_slice(), &deps.api)
            .unwrap()
            .unwrap();

        assert_eq!(
            Owner {
                address: "owner_1".into(),
                public_key: Binary::from(b"pubkey_1"),
            },
            owner1
        );

        let owner2 = owner_store
            .read(Binary::from(b"contract_2").as_slice(), &deps.api)
            .unwrap()
            .unwrap();

        assert_eq!(
            Owner {
                address: "owner_2".into(),
                public_key: Binary::from(b"pubkey_2"),
            },
            owner2
        );
    }

    #[test]
    fn test_init_contract_state_ok() {
        let mut deps = mock_deps();
        let env = mock_env("alice", &[]);

        init(
            &mut deps,
            mock_env("admin", &[]),
            InitMsg {
                contract_owners: vec![(
                    Binary::from(b"contract_1"),
                    Owner {
                        address: "owner_1".into(),
                        public_key: Binary::from(b"pubkey_1"),
                    },
                )],
            },
        )
        .expect("failed to init");

        let msg = HandleMsg::InitContractState {
            contract_label: Binary::from(b"contract_2"),
            public_key: Binary::from(b"pubkey_2"),
        };

        let resp = handle(&mut deps, env, msg).unwrap();

        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);

        let owner_store = ContractsOwnerStore::from_readonly(&deps.storage);

        let owner = owner_store
            .read(&Binary::from(b"contract_2").as_slice(), &deps.api)
            .unwrap()
            .unwrap();

        assert_eq!(
            Owner {
                address: "alice".into(),
                public_key: Binary::from(b"pubkey_2"),
            },
            owner
        );
    }

    #[test]
    fn test_init_contract_state_ng() {
        let mut deps = mock_deps();
        let env = mock_env("alice", &[]);

        init(
            &mut deps,
            mock_env("admin", &[]),
            InitMsg {
                contract_owners: vec![(
                    Binary::from(b"contract_1"),
                    Owner {
                        address: "owner_1".into(),
                        public_key: Binary::from(b"pubkey_1"),
                    },
                )],
            },
        )
        .expect("failed to init");

        let msg = HandleMsg::InitContractState {
            contract_label: Binary::from(b"contract_1"),
            public_key: Binary::from(b"pubkey_1"),
        };

        let err = handle(&mut deps, env, msg).unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err("contract state is already instantiated")
        );
    }

    #[test]
    fn test_write_contract_state_single_set_ok() {
        let mut deps = mock_deps();
        let env = mock_env("alice", &[]);

        init(
            &mut deps,
            mock_env("admin", &[]),
            InitMsg {
                contract_owners: vec![(
                    Binary::from(b"contract_1"),
                    Owner {
                        address: "owner_1".into(),
                        public_key: Binary::from(b"pubkey_1"),
                    },
                )],
            },
        )
        .expect("failed to init");

        let contract_label = Binary::from(b"contract_2");

        let msg = HandleMsg::InitContractState {
            contract_label: contract_label.clone(),
            public_key: Binary::from(b"pubkey_2"),
        };

        let resp = handle(&mut deps, env.clone(), msg).unwrap();

        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);

        let msg = HandleMsg::WriteContractState {
            contract_label: contract_label.clone(),
            transaction: vec![(
                Binary::from(b"key1"),
                WriteAction::Set {
                    value: Binary::from(b"hoge"),
                },
            )],
        };

        let resp = handle(&mut deps, env, msg).unwrap();

        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);

        let contract_storage = PrefixedStorage::multilevel(
            &[CONTRACTS_STATE_KEY, contract_label.clone().as_slice()],
            &mut deps.storage,
        );

        let value = contract_storage.get(b"key1").unwrap();

        assert_eq!(b"hoge".as_slice(), value);

        // bob

        let env = mock_env("bob", &[]);
        let contract_label2 = Binary::from(b"contract_3");

        let msg = HandleMsg::InitContractState {
            contract_label: contract_label2.clone(),
            public_key: Binary::from(b"pubkey_3"),
        };

        let resp = handle(&mut deps, env.clone(), msg).unwrap();

        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);

        let msg = HandleMsg::WriteContractState {
            contract_label: contract_label2.clone(),
            transaction: vec![(
                Binary::from(b"key1"),
                WriteAction::Set {
                    value: Binary::from(b"hogehoge"),
                },
            )],
        };

        let resp = handle(&mut deps, env, msg).unwrap();

        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);

        let contract_storage = PrefixedStorage::multilevel(
            &[CONTRACTS_STATE_KEY, contract_label2.clone().as_slice()],
            &mut deps.storage,
        );

        let value = contract_storage.get(b"key1").unwrap();

        assert_eq!(b"hogehoge".as_slice(), value);

        // confirm not to overwrite
        let contract_storage = PrefixedStorage::multilevel(
            &[CONTRACTS_STATE_KEY, contract_label.clone().as_slice()],
            &mut deps.storage,
        );

        let value = contract_storage.get(b"key1").unwrap();

        assert_eq!(b"hoge".as_slice(), value);
    }

    #[test]
    fn test_write_contract_state_multi_set_ok() {
        let mut deps = mock_deps();
        let env = mock_env("alice", &[]);

        init(
            &mut deps,
            mock_env("admin", &[]),
            InitMsg {
                contract_owners: vec![(
                    Binary::from(b"contract_1"),
                    Owner {
                        address: "owner_1".into(),
                        public_key: Binary::from(b"pubkey_1"),
                    },
                )],
            },
        )
        .expect("failed to init");

        let contract_label = Binary::from(b"contract_2");

        let msg = HandleMsg::InitContractState {
            contract_label: contract_label.clone(),
            public_key: Binary::from(b"pubkey_2"),
        };

        let resp = handle(&mut deps, env.clone(), msg).unwrap();

        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);

        let msg = HandleMsg::WriteContractState {
            contract_label: contract_label.clone(),
            transaction: vec![
                (
                    Binary::from(b"key1"),
                    WriteAction::Set {
                        value: Binary::from(b"hoge"),
                    },
                ),
                (
                    Binary::from(b"key2"),
                    WriteAction::Set {
                        value: Binary::from(b"fuga"),
                    },
                ),
            ],
        };

        let resp = handle(&mut deps, env, msg).unwrap();

        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);

        let contract_storage = PrefixedStorage::multilevel(
            &[CONTRACTS_STATE_KEY, contract_label.clone().as_slice()],
            &mut deps.storage,
        );

        let value = contract_storage.get(b"key1").unwrap();
        assert_eq!(b"hoge".as_slice(), value);

        let value = contract_storage.get(b"key2").unwrap();
        assert_eq!(b"fuga".as_slice(), value);
    }

    #[test]
    fn test_write_contract_state_multi_reset_ok() {
        let mut deps = mock_deps();
        let env = mock_env("alice", &[]);

        init(
            &mut deps,
            mock_env("admin", &[]),
            InitMsg {
                contract_owners: vec![(
                    Binary::from(b"contract_1"),
                    Owner {
                        address: "owner_1".into(),
                        public_key: Binary::from(b"pubkey_1"),
                    },
                )],
            },
        )
        .expect("failed to init");

        let contract_label = Binary::from(b"contract_2");

        let msg = HandleMsg::InitContractState {
            contract_label: contract_label.clone(),
            public_key: Binary::from(b"pubkey_2"),
        };

        let resp = handle(&mut deps, env.clone(), msg).unwrap();

        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);

        let msg = HandleMsg::WriteContractState {
            contract_label: contract_label.clone(),
            transaction: vec![
                (
                    Binary::from(b"key1"),
                    WriteAction::Set {
                        value: Binary::from(b"hoge"),
                    },
                ),
                (
                    Binary::from(b"key2"),
                    WriteAction::Set {
                        value: Binary::from(b"fuga"),
                    },
                ),
            ],
        };

        let resp = handle(&mut deps, env.clone(), msg).unwrap();

        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);

        let contract_storage = PrefixedStorage::multilevel(
            &[CONTRACTS_STATE_KEY, contract_label.clone().as_slice()],
            &mut deps.storage,
        );

        let value = contract_storage.get(b"key1").unwrap();
        assert_eq!(b"hoge".as_slice(), value);

        let value = contract_storage.get(b"key2").unwrap();
        assert_eq!(b"fuga".as_slice(), value);

        let msg = HandleMsg::WriteContractState {
            contract_label: contract_label.clone(),
            transaction: vec![
                (
                    Binary::from(b"key1"),
                    WriteAction::Set {
                        value: Binary::from(b"hoge2"),
                    },
                ),
                (
                    Binary::from(b"key2"),
                    WriteAction::Set {
                        value: Binary::from(b"fuga2"),
                    },
                ),
            ],
        };

        let resp = handle(&mut deps, env.clone(), msg).unwrap();

        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);

        let contract_storage = PrefixedStorage::multilevel(
            &[CONTRACTS_STATE_KEY, contract_label.clone().as_slice()],
            &mut deps.storage,
        );

        let value = contract_storage.get(b"key1").unwrap();
        assert_eq!(b"hoge2".as_slice(), value);

        let value = contract_storage.get(b"key2").unwrap();
        assert_eq!(b"fuga2".as_slice(), value);
    }

    #[test]
    fn test_write_contract_state_multi_remove_ok() {
        let mut deps = mock_deps();
        let env = mock_env("alice", &[]);

        init(
            &mut deps,
            mock_env("admin", &[]),
            InitMsg {
                contract_owners: vec![(
                    Binary::from(b"contract_1"),
                    Owner {
                        address: "owner_1".into(),
                        public_key: Binary::from(b"pubkey_1"),
                    },
                )],
            },
        )
        .expect("failed to init");

        let contract_label = Binary::from(b"contract_2");

        let msg = HandleMsg::InitContractState {
            contract_label: contract_label.clone(),
            public_key: Binary::from(b"pubkey_2"),
        };

        let resp = handle(&mut deps, env.clone(), msg).unwrap();

        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);

        let msg = HandleMsg::WriteContractState {
            contract_label: contract_label.clone(),
            transaction: vec![
                (
                    Binary::from(b"key1"),
                    WriteAction::Set {
                        value: Binary::from(b"hoge"),
                    },
                ),
                (
                    Binary::from(b"key2"),
                    WriteAction::Set {
                        value: Binary::from(b"fuga"),
                    },
                ),
                (
                    Binary::from(b"key3"),
                    WriteAction::Set {
                        value: Binary::from(b"piyo"),
                    },
                ),
            ],
        };

        let resp = handle(&mut deps, env.clone(), msg).unwrap();

        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);

        let contract_storage = PrefixedStorage::multilevel(
            &[CONTRACTS_STATE_KEY, contract_label.clone().as_slice()],
            &mut deps.storage,
        );

        let value = contract_storage.get(b"key1").unwrap();
        assert_eq!(b"hoge".as_slice(), value);

        let value = contract_storage.get(b"key2").unwrap();
        assert_eq!(b"fuga".as_slice(), value);

        let msg = HandleMsg::WriteContractState {
            contract_label: contract_label.clone(),
            transaction: vec![
                (Binary::from(b"key1"), WriteAction::Remove {}),
                (Binary::from(b"key2"), WriteAction::Remove {}),
            ],
        };

        let resp = handle(&mut deps, env.clone(), msg).unwrap();

        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);

        let contract_storage = PrefixedStorage::multilevel(
            &[CONTRACTS_STATE_KEY, contract_label.clone().as_slice()],
            &mut deps.storage,
        );

        let value = contract_storage.get(b"key1");
        assert!(value.is_none());

        let value = contract_storage.get(b"key2");
        assert!(value.is_none());

        let value = contract_storage.get(b"key3").unwrap();
        assert_eq!(b"piyo".as_slice(), value);
    }

    #[test]
    fn test_write_contract_state_ng() {
        let mut deps = mock_deps();
        let env = mock_env("alice", &[]);

        init(
            &mut deps,
            mock_env("admin", &[]),
            InitMsg {
                contract_owners: vec![(
                    Binary::from(b"contract_1"),
                    Owner {
                        address: "owner_1".into(),
                        public_key: Binary::from(b"pubkey_1"),
                    },
                )],
            },
        )
        .expect("failed to init");

        let contract_label = Binary::from(b"contract_label");

        let msg = HandleMsg::InitContractState {
            contract_label: contract_label.clone(),
            public_key: Binary::from(b"pubkey_2"),
        };

        let resp = handle(&mut deps, env.clone(), msg).unwrap();

        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);

        let invalid_contract_label = Binary::from(b"invalid_contract_label");

        let msg = HandleMsg::WriteContractState {
            contract_label: invalid_contract_label.clone(),
            transaction: vec![
                (
                    Binary::from(b"key1"),
                    WriteAction::Set {
                        value: Binary::from(b"hoge"),
                    },
                ),
                (
                    Binary::from(b"key2"),
                    WriteAction::Set {
                        value: Binary::from(b"fuga"),
                    },
                ),
                (
                    Binary::from(b"key3"),
                    WriteAction::Set {
                        value: Binary::from(b"piyo"),
                    },
                ),
            ],
        };

        let err = handle(&mut deps, env.clone(), msg).unwrap_err();

        assert_eq!(
            err,
            StdError::generic_err("contract state is not initialized")
        );

        let contract_storage = PrefixedStorage::multilevel(
            &[
                CONTRACTS_STATE_KEY,
                invalid_contract_label.clone().as_slice(),
            ],
            &mut deps.storage,
        );

        let value = contract_storage.get(b"key1");
        assert!(value.is_none());

        let value = contract_storage.get(b"key2");
        assert!(value.is_none());

        let value = contract_storage.get(b"key3");
        assert!(value.is_none());
    }

    #[test]
    fn test_change_owner_by_admin_ok() {
        let mut deps = mock_deps();
        let env = mock_env("admin", &[]);

        init(
            &mut deps,
            env.clone(),
            InitMsg {
                contract_owners: vec![(
                    Binary::from(b"contract_1"),
                    Owner {
                        address: "owner_1".into(),
                        public_key: Binary::from(b"pubkey_1"),
                    },
                )],
            },
        )
        .expect("failed to init");

        // change owner by admin
        let contract_label = Binary::from(b"contract_1");

        let msg = HandleMsg::ChangeOwnerByAdmin {
            contract_label: contract_label.clone(),
            next_owner: Owner {
                address: "owner_2".into(),
                public_key: Binary::from(b"pubkey_2"),
            },
        };

        let resp = handle(&mut deps, env.clone(), msg).unwrap();

        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);

        let owner = ContractsOwnerStore::from_readonly(&deps.storage);

        let o1 = owner
            .read(contract_label.as_slice(), &deps.api)
            .unwrap()
            .unwrap();

        assert_eq!(
            o1,
            Owner {
                address: "owner_2".into(),
                public_key: Binary::from(b"pubkey_2"),
            }
        );
    }

    #[test]
    fn test_change_owner_by_admin_ng() {
        let mut deps = mock_deps();
        let env = mock_env("admin", &[]);

        init(
            &mut deps,
            env.clone(),
            InitMsg {
                contract_owners: vec![(
                    Binary::from(b"contract_1"),
                    Owner {
                        address: "owner_1".into(),
                        public_key: Binary::from(b"pubkey_1"),
                    },
                )],
            },
        )
        .expect("failed to init");

        // change owner by admin
        let contract_label = Binary::from(b"contract_1");

        let msg = HandleMsg::ChangeOwnerByAdmin {
            contract_label: contract_label.clone(),
            next_owner: Owner {
                address: "owner_2".into(),
                public_key: Binary::from(b"pubkey_2"),
            },
        };

        let not_admin_env = mock_env("not_admin", &[]);
        let err = handle(&mut deps, not_admin_env.clone(), msg).unwrap_err();

        assert_eq!(
            err,
            StdError::generic_err("message sender is not current admin")
        );

        let owner = ContractsOwnerStore::from_readonly(&deps.storage);

        let o1 = owner
            .read(contract_label.as_slice(), &deps.api)
            .unwrap()
            .unwrap();

        assert_eq!(
            o1,
            Owner {
                address: "owner_1".into(),
                public_key: Binary::from(b"pubkey_1"),
            }
        );
    }

    #[test]
    fn test_change_admin_ok() {
        let mut deps = mock_deps();

        init(
            &mut deps,
            mock_env("admin", &[]),
            InitMsg {
                contract_owners: vec![(
                    Binary::from(b"contract_1"),
                    Owner {
                        address: "owner_1".into(),
                        public_key: Binary::from(b"pubkey_1"),
                    },
                )],
            },
        )
        .expect("failed to init");

        let bob = HumanAddr::from("bob");

        let msg = HandleMsg::ChangeAdmin {
            next_admin: bob.clone(),
        };

        let env = mock_env("admin", &[]);
        let resp = handle(&mut deps, env, msg).unwrap();
        assert_eq!(resp.messages.len(), 0);
        assert_eq!(resp.log.len(), 0);
        assert!(resp.data.is_none());

        let admin = read_admin(&deps.storage);

        assert_eq!(admin, deps.api.canonical_address(&bob).unwrap());
    }

    #[test]
    fn test_change_admin_ng() {
        let mut deps = mock_deps();

        init(
            &mut deps,
            mock_env("admin", &[]),
            InitMsg {
                contract_owners: vec![(
                    Binary::from(b"contract_1"),
                    Owner {
                        address: "owner_1".into(),
                        public_key: Binary::from(b"pubkey_1"),
                    },
                )],
            },
        )
        .expect("failed to init");

        let bob = HumanAddr::from("bob");

        let msg = HandleMsg::ChangeAdmin {
            next_admin: bob.clone(),
        };

        let env = mock_env("not_admin", &[]);
        let err = handle(&mut deps, env, msg).unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err("message sender is not current admin")
        );

        let admin = read_admin(&deps.storage);

        assert_eq!(
            admin,
            deps.api
                .canonical_address(&HumanAddr::from("admin"))
                .unwrap()
        );
    }

    #[test]
    fn test_query_owner_ok() {
        let mut deps = mock_deps();
        let env = mock_env("admin", &[]);

        let owner = Owner {
            address: "owner_1".into(),
            public_key: Binary::from(b"pubkey_1"),
        };

        init(
            &mut deps,
            env.clone(),
            InitMsg {
                contract_owners: vec![(Binary::from(b"contract_1"), owner.clone())],
            },
        )
        .expect("failed to init");

        let contract_label = Binary::from(b"contract_1");

        let msg = QueryMsg::Owner { contract_label };

        let resp = query(&deps, msg).unwrap();

        assert_eq!(
            resp,
            to_binary(&QueryAnswer::Owner { owner: Some(owner) }).unwrap()
        );
    }

    #[test]
    fn test_query_owner_none() {
        let mut deps = mock_deps();
        let env = mock_env("admin", &[]);

        let owner = Owner {
            address: "owner_1".into(),
            public_key: Binary::from(b"pubkey_1"),
        };

        init(
            &mut deps,
            env.clone(),
            InitMsg {
                contract_owners: vec![(Binary::from(b"contract_1"), owner.clone())],
            },
        )
        .expect("failed to init");

        let non_existent_contract_label = Binary::from(b"contract_2");

        let msg = QueryMsg::Owner {
            contract_label: non_existent_contract_label,
        };

        let resp = query(&deps, msg).unwrap();

        assert_eq!(
            resp,
            to_binary(&QueryAnswer::Owner { owner: None }).unwrap()
        );
    }

    #[test]
    fn test_query_read_contract_state_ok() {
        let mut deps = mock_deps();
        let env = mock_env("admin", &[]);

        let mut rng = thread_rng();

        let secret_key = libsecp256k1::SecretKey::random(&mut rng);
        let private_key = Binary::from(secret_key.serialize());
        let public_key = libsecp256k1::PublicKey::from_secret_key(&secret_key);

        let owner = Owner {
            address: "owner_1".into(),
            public_key: Binary::from(public_key.serialize().as_slice()),
        };

        let contract_label = Binary::from(b"contract_1");

        init(
            &mut deps,
            env.clone(),
            InitMsg {
                contract_owners: vec![(contract_label.clone(), owner.clone())],
            },
        )
        .expect("failed to init");

        let signer = Secp256k1ApiSigner::new(&deps.api);

        let signature =
            ReadContractStateSignature::sign(private_key, &contract_label.as_slice(), &signer)
                .unwrap();

        let query_msg = QueryMsg::ReadContractState {
            signature: signature.clone(),
            key: Binary::from(b"key1"),
        };

        // without value
        let resp = query(&mut deps, query_msg).unwrap();
        assert_eq!(
            resp,
            to_binary(&QueryAnswer::ReadContractState { value: None }).unwrap(),
        );

        // set value
        let owner_env = mock_env("owner_1", &[]);

        let msg = HandleMsg::WriteContractState {
            contract_label: contract_label.clone(),
            transaction: vec![(
                Binary::from(b"key1"),
                WriteAction::Set {
                    value: Binary::from(b"hoge"),
                },
            )],
        };

        handle(&mut deps, owner_env, msg).unwrap();

        let query_msg = QueryMsg::ReadContractState {
            signature,
            key: Binary::from(b"key1"),
        };

        let resp = query(&mut deps, query_msg).unwrap();
        assert_eq!(
            resp,
            to_binary(&QueryAnswer::ReadContractState {
                value: Some(Binary::from(b"hoge"))
            })
            .unwrap(),
        );
    }

    #[test]
    fn test_query_read_contract_state_ng() {
        let mut deps = mock_deps();
        let env = mock_env("admin", &[]);

        let mut rng = thread_rng();

        let secret_key = libsecp256k1::SecretKey::random(&mut rng);
        let public_key = libsecp256k1::PublicKey::from_secret_key(&secret_key);

        let owner = Owner {
            address: "owner_1".into(),
            public_key: Binary::from(public_key.serialize().as_slice()),
        };

        let contract_label = Binary::from(b"contract_1");

        init(
            &mut deps,
            env.clone(),
            InitMsg {
                contract_owners: vec![(contract_label.clone(), owner.clone())],
            },
        )
        .expect("failed to init");

        let signer = Secp256k1ApiSigner::new(&deps.api);

        let another_secret_key = libsecp256k1::SecretKey::random(&mut rng);
        let another_private_key = Binary::from(another_secret_key.serialize());

        let signature = ReadContractStateSignature::sign(
            another_private_key,
            &contract_label.as_slice(),
            &signer,
        )
        .unwrap();

        let query_msg = QueryMsg::ReadContractState {
            signature: signature.clone(),
            key: Binary::from(b"key1"),
        };

        // without value
        let err = query(&mut deps, query_msg).unwrap_err();
        assert_eq!(err, StdError::generic_err("wrong owner"));

        // set value
        let owner_env = mock_env("owner_1", &[]);

        let msg = HandleMsg::WriteContractState {
            contract_label: contract_label.clone(),
            transaction: vec![(
                Binary::from(b"key1"),
                WriteAction::Set {
                    value: Binary::from(b"hoge"),
                },
            )],
        };

        handle(&mut deps, owner_env, msg).unwrap();

        let query_msg = QueryMsg::ReadContractState {
            signature,
            key: Binary::from(b"key1"),
        };

        let err = query(&mut deps, query_msg).unwrap_err();
        assert_eq!(err, StdError::generic_err("wrong owner"));
    }

    #[test]
    fn test_query_admin_ok() {
        let mut deps = mock_deps();
        let env = mock_env("admin", &[]);

        let owner = Owner {
            address: "owner_1".into(),
            public_key: Binary::from(b"pubkey_1"),
        };

        init(
            &mut deps,
            env.clone(),
            InitMsg {
                contract_owners: vec![(Binary::from(b"contract_1"), owner.clone())],
            },
        )
        .expect("failed to init");

        let msg = QueryMsg::Admin {};
        let resp = query(&deps, msg).unwrap();

        assert_eq!(
            resp,
            to_binary(&QueryAnswer::Admin {
                admin: HumanAddr::from("admin")
            })
            .unwrap()
        );
    }
}
