use crate::error::Error;
use crate::state::chaindb::StorageChainDB;
use crate::state::config::read_config;
use bitcoin::consensus::encode::deserialize;
use bitcoin::BlockHeader;
use bitcoin::Network;
use bitcoin_header_chain::header_chain::HeaderChain;
use cosmwasm_std::{Api, Binary, Env, Extern, HandleResponse, Querier, StdResult, Storage};
use secret_toolkit::utils::padding::pad_handle_result;
use shared_types::{bitcoin_spv::HandleMsg, BLOCK_SIZE};
use std::str::FromStr;

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    let result = match msg {
        HandleMsg::AddHeaders {
            tip_height,
            headers,
            ..
        } => try_add_headers(deps, env, tip_height, headers),
    };
    pad_handle_result(result.map_err(|e| e.into()), BLOCK_SIZE)
}

fn try_add_headers<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    tip_height: u32,
    headers: Vec<Binary>,
) -> Result<HandleResponse, Error> {
    let config = read_config(&deps.storage)?;
    let block_time = env.block.time as u32;
    if headers.is_empty() {
        return Err(Error::contract_err("no header in msg"));
    }
    let de_headers = {
        let mut de_headers = Vec::with_capacity(headers.len());
        for header in headers {
            let header: BlockHeader = deserialize::<BlockHeader>(header.as_slice())?;
            de_headers.push(header);
        }
        de_headers
    };
    let mut header_chain = HeaderChain::new(
        StorageChainDB::from_storage(&mut deps.storage),
        Network::from_str(&config.bitcoin_network)?,
    );
    header_chain.store_headers(tip_height, de_headers, block_time)?;
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}
