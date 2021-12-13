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
    prng::init_prng(&mut deps.storage, PRNG_KEY, &env, msg.prng_seed.as_slice())?;
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
                    EventSource::Gateway => &config.gateway.address,
                    EventSource::Treasury => &config.treasury.address,
                    EventSource::User => &address,
                };
                if &env.message.sender != authorized_source_address {
                    return Err(StdError::unauthorized());
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

    #[test]
    fn test_query_log() {
        let mut deps = mock_dependencies(20, &[]);
        let canonical_addr = deps.api.canonical_address(&"lebron".into()).unwrap();
        let mut log_storage = LogStorage::from_storage(&mut deps.storage, &canonical_addr);
        let time = 1000;
        let max_time = 1039;
        for i in 0..40 {
            log_storage
                .append(&Event::MintStarted(MintStartedData {
                    time: time + i,
                    address: "address".to_string(),
                }))
                .unwrap()
        }
        // page_size = 10
        for page in 0u32..4u32 {
            let logs = log_storage.get_logs(page, 10).unwrap();
            assert_eq!(logs.len(), 10);
            for (i, log) in logs.iter().enumerate() {
                assert_eq!(
                    log,
                    &Event::MintStarted(MintStartedData {
                        time: max_time - (page as u64) * 10 - i as u64,
                        address: "address".to_string()
                    })
                )
            }
        }
        assert_eq!(log_storage.get_logs(4, 10).unwrap(), vec![]);
        assert_eq!(log_storage.get_logs(5, 10).unwrap(), vec![]);
        // page_size = 20
        for page in 0u32..2u32 {
            let logs = log_storage.get_logs(page, 20).unwrap();
            assert_eq!(logs.len(), 20);
            for (i, log) in logs.iter().enumerate() {
                assert_eq!(
                    log,
                    &Event::MintStarted(MintStartedData {
                        time: max_time - (page as u64) * 20 - i as u64,
                        address: "address".to_string()
                    })
                )
            }
        }
        assert_eq!(log_storage.get_logs(2, 20).unwrap(), vec![]);
        // page_size = 30
        assert_eq!(log_storage.get_logs(0, 30).unwrap().len(), 30);
        assert_eq!(log_storage.get_logs(1, 30).unwrap().len(), 10);
        // page_size = 50
        assert_eq!(log_storage.get_logs(0, 50).unwrap().len(), 40);
    }
}
