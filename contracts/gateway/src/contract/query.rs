use crate::contract::query_bitcoin_network::query_bitcoin_network;
use crate::error::Error;
use crate::state::config::read_config;
use crate::state::mint_key::read_mint_key;
use crate::state::prefix::PREFIX_VIEW_KEY;
use crate::state::suspension_switch::suspension_switch;
use bitcoin::Address;
use cosmwasm_std::{
    to_binary, Api, Extern, HumanAddr, Querier, QueryResponse, QueryResult, Storage,
};
use shared_types::gateway::{QueryAnswer, QueryMsg};
use shared_types::viewing_key;
use std::string::ToString;

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    let result = match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::SuspensionSwitch {} => query_suspension_switch(deps),
        _ => authenticated_queries(deps, msg),
    };
    Ok(result?)
}

fn get_validation_params(query_msg: &QueryMsg) -> (Vec<&HumanAddr>, viewing_key::ViewingKey) {
    match query_msg {
        QueryMsg::MintAddress { address, key, .. } => (vec![address], key.clone()),
        _ => panic!("This query type does not require authentication"),
    }
}

fn authenticated_queries<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> Result<QueryResponse, Error> {
    let (addresses, key) = get_validation_params(&msg);

    for address in addresses {
        let canonical_addr = deps.api.canonical_address(address)?;

        let store =
            viewing_key::ViewingKeyHashStore::from_readonly_storage(&deps.storage, PREFIX_VIEW_KEY);
        // Checking the key will take significant time. We don't want to exit immediately if it isn't set
        // in a way which will allow to time the command and determine if a viewing key doesn't exist
        let expected_key_hash = store.read(&canonical_addr).unwrap_or_default();
        if key.hash() == expected_key_hash {
            return match msg {
                // Base
                QueryMsg::MintAddress { address, .. } => query_mint_address(&deps, address),
                _ => panic!("This query type does not require authentication"),
            };
        }
    }

    Ok(to_binary(&QueryAnswer::ViewingKeyError {
        msg: "Wrong viewing key for this address or viewing key not set".to_string(),
    })?)
}

fn query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> Result<QueryResponse, Error> {
    let config = read_config(&deps.storage, &deps.api)?;
    Ok(to_binary(&QueryAnswer::Config(config))?)
}

fn query_suspension_switch<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> Result<QueryResponse, Error> {
    let switch = suspension_switch(&deps.storage)?;
    Ok(to_binary(&QueryAnswer::SuspensionSwitch(switch))?)
}

fn query_mint_address<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> Result<QueryResponse, Error> {
    let address = deps.api.canonical_address(&address)?;
    let config = read_config(&deps.storage, &deps.api)?;
    let network = query_bitcoin_network(&deps.querier, config.bitcoin_spv)?;
    let response = match read_mint_key(&deps.storage, &address, network)? {
        Some(key) => {
            let mint_address = Address::p2wpkh(&key.public_key(), key.network)?;
            QueryAnswer::MintAddress {
                address: Some(mint_address.to_string()),
            }
        }
        None => QueryAnswer::MintAddress { address: None },
    };
    Ok(to_binary(&response)?)
}
