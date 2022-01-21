use crate::error::Error;
use crate::state::chaindb::StorageChainDB;
use crate::state::config::write_config;
use bitcoin::consensus::encode::deserialize;
use bitcoin::BlockHeader;
use bitcoin::Network;
use bitcoin_header_chain::header_chain::HeaderChain;
use cosmwasm_std::{Api, Env, Extern, InitResponse, Querier, StdResult, Storage};
use shared_types::bitcoin_spv::{Config, InitMsg};
use std::str::FromStr;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    Ok(try_init(deps, env, msg)?)
}

fn try_init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> Result<InitResponse, Error> {
    let config = Config {
        bitcoin_network: msg.bitcoin_network,
        confirmation: msg.confirmation,
    };
    write_config(&mut deps.storage, &config)?;

    // init bitcoin header chain
    let chaindb = StorageChainDB::from_storage(&mut deps.storage);
    let mut header_chain = HeaderChain::new(chaindb, Network::from_str(&config.bitcoin_network)?);
    match msg.initial_header {
        Some(initial_header) => {
            let header: BlockHeader = deserialize::<BlockHeader>(initial_header.header.as_slice())?;
            header_chain.init_to_header(initial_header.height, header, env.block.time as u32)?;
        }
        None => {
            header_chain.init_to_genesis()?;
        }
    }

    Ok(InitResponse {
        messages: vec![],
        log: vec![],
    })
}
