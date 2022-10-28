use crate::state::{read_config, write_config, LogStorage, PREFIX_VIEWING_KEY, PRNG_KEY};
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HandleResult, InitResponse, InitResult, Querier,
    QueryResult, StdError, Storage,
};
use secret_toolkit::utils::pad_handle_result;
use shared_types::log::{
    event::EventSource, HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg,
};
use shared_types::{prng, viewing_key, BLOCK_SIZE};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> InitResult {
    prng::init_prng(&mut deps.storage, PRNG_KEY, &env, msg.entropy.as_slice())?;
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    let handle_result = match msg {
        HandleMsg::Setup { config } => {
            write_config(&mut deps.storage, config, &deps.api)?;
            Ok(HandleResponse::default())
        }
        HandleMsg::AddEvents { events } => {
            let config = read_config(&deps.storage, &deps.api)?;
            for (address, event) in events {
                // Check if the message sender is authorized source of event.
                let authorized_source_address = match event.authorized_source() {
                    EventSource::Gateway => Some(&config.gateway.address),
                    EventSource::User => Some(&address),
                    EventSource::Any => None,
                };
                match authorized_source_address {
                    Some(authorized_source_address) => {
                        if &env.message.sender != authorized_source_address {
                            return Err(StdError::unauthorized());
                        }
                    }
                    None => {}
                }
                let address = &deps.api.canonical_address(&address)?;
                let mut store = LogStorage::from_storage(&mut deps.storage, address);
                store.append(&event)?;
            }
            Ok(HandleResponse::default())
        }
        HandleMsg::CreateViewingKey { entropy } => {
            let sender = &deps.api.canonical_address(&env.message.sender)?;
            let mut prng =
                prng::update_prng(&mut deps.storage, PRNG_KEY, sender, entropy.as_bytes())?;
            let key = viewing_key::ViewingKey::new(&mut prng);
            let mut viewing_key_hash_store = viewing_key::ViewingKeyHashStore::from_storage(
                &mut deps.storage,
                PREFIX_VIEWING_KEY,
            );
            viewing_key_hash_store.write(&sender, &key.hash());
            Ok(HandleResponse {
                messages: vec![],
                data: Some(to_binary(&HandleAnswer::CreateViewingKey { key })?),
                log: vec![],
            })
        }
        HandleMsg::SetViewingKey { key } => {
            let sender = &deps.api.canonical_address(&env.message.sender)?;
            let mut viewing_key_hash_store = viewing_key::ViewingKeyHashStore::from_storage(
                &mut deps.storage,
                PREFIX_VIEWING_KEY,
            );
            viewing_key_hash_store.write(&sender, &key.hash());
            Ok(HandleResponse::default())
        }
    };
    pad_handle_result(handle_result, BLOCK_SIZE)
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::Log {
            address,
            key,
            page,
            page_size,
        } => {
            let address = deps.api.canonical_address(&address)?;
            let viewing_key_hash_store = viewing_key::ViewingKeyHashStore::from_readonly_storage(
                &deps.storage,
                PREFIX_VIEWING_KEY,
            );
            if viewing_key_hash_store.read(&address).unwrap_or_default() != key.hash() {
                return Err(StdError::generic_err("wrong viewing key"));
            }
            let log_storage = LogStorage::from_readonly_storage(&deps.storage, &address);
            let logs = log_storage.get_logs(page, page_size)?;
            to_binary(&QueryAnswer::Log { logs })
        }
        QueryMsg::Config {} => {
            let config = read_config(&deps.storage, &deps.api)?;
            to_binary(&QueryAnswer::Config(config))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::*;
    use shared_types::log::event::*;
    use shared_types::viewing_key::ViewingKey;

    #[test]
    fn test_query_log() {
        let mut deps = mock_dependencies(20, &[]);
        let canonical_addr = deps.api.canonical_address(&"lebron".into()).unwrap();
        let mut log_storage = LogStorage::from_storage(&mut deps.storage, &canonical_addr);
        let viewing_key = ViewingKey("viewing_key".to_string());
        let time = 1000;

        for i in 0..40 {
            log_storage
                .append(&Event::MintStarted(MintStartedData {
                    time: time + i,
                    address: "address".to_string(),
                }))
                .unwrap()
        }

        handle(
            &mut deps,
            mock_env("lebron", &[]),
            HandleMsg::SetViewingKey {
                key: viewing_key.clone(),
            },
        )
        .unwrap();

        assert_eq!(
            query(
                &deps,
                QueryMsg::Log {
                    address: "lebron".into(),
                    key: viewing_key.clone(),
                    page: 0,
                    page_size: 0,
                },
            )
            .unwrap(),
            to_binary(&QueryAnswer::Log { logs: vec![] }).unwrap()
        );
        assert_eq!(
            query(
                &deps,
                QueryMsg::Log {
                    address: "lebron".into(),
                    key: viewing_key.clone(),
                    page: 0,
                    page_size: 2,
                },
            )
            .unwrap(),
            to_binary(&QueryAnswer::Log {
                logs: vec![
                    Event::MintStarted(MintStartedData {
                        time: 1039,
                        address: "address".to_string()
                    }),
                    Event::MintStarted(MintStartedData {
                        time: 1038,
                        address: "address".to_string()
                    })
                ]
            })
            .unwrap()
        );
        assert_eq!(
            query(
                &deps,
                QueryMsg::Log {
                    address: "lebron".into(),
                    key: viewing_key,
                    page: 1,
                    page_size: 2,
                },
            )
            .unwrap(),
            to_binary(&QueryAnswer::Log {
                logs: vec![
                    Event::MintStarted(MintStartedData {
                        time: 1037,
                        address: "address".to_string()
                    }),
                    Event::MintStarted(MintStartedData {
                        time: 1036,
                        address: "address".to_string()
                    })
                ]
            })
            .unwrap()
        );
        assert_eq!(
            query(
                &deps,
                QueryMsg::Log {
                    address: "lebron".into(),
                    key: ViewingKey("wrong_viewing_key".into()),
                    page: 1,
                    page_size: 2,
                },
            )
            .unwrap_err(),
            StdError::generic_err("wrong viewing key")
        );
    }
}
