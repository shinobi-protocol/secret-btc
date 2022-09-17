use crate::state::{read_config, write_config};
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HandleResult, InitResponse, InitResult, Querier,
    QueryResult, StdError, Storage,
};
use secret_toolkit::utils::{pad_handle_result, HandleCallback};
use shared_types::shuriken::{HandleMsg, InitMsg, QueryAnswer, QueryMsg};
use shared_types::{bitcoin_spv, sfps, BLOCK_SIZE};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> InitResult {
    write_config(&mut deps.storage, msg.config, &deps.api)?;
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    let config = read_config(&deps.storage, &deps.api)?;
    let messages = match msg {
        HandleMsg::BitcoinSPVAddHeaders {
            tip_height,
            headers,
        } => {
            vec![bitcoin_spv::HandleMsg::AddHeaders {
                tip_height,
                headers,
            }
            .to_cosmos_msg(
                config.bitcoin_spv.hash,
                config.bitcoin_spv.address,
                None,
            )?]
        }
        HandleMsg::SFPSProxyAppendSubsequentHashes {
            committed_hashes,
            last_header,
        } => {
            let last_committed_hash = committed_hashes
                .hashes
                .following_hashes
                .last()
                .ok_or_else(|| StdError::generic_err("no committed hashes"))?
                .clone();
            if last_committed_hash.hash != sfps::sfps_lib::header::hash_header(&last_header) {
                return Err(StdError::generic_err(
                    "last_header does not match to committed_hashes",
                ));
            }
            vec![
                sfps::HandleMsg::AppendSubsequentHashes { committed_hashes }.to_cosmos_msg(
                    config.sfps.hash,
                    config.sfps.address,
                    None,
                )?,
            ]
        }
    };
    pad_handle_result(
        Ok(HandleResponse {
            messages,
            data: None,
            log: vec![],
        }),
        BLOCK_SIZE,
    )
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::Config {} => {
            let config = read_config(&deps.storage, &deps.api)?;
            to_binary(&QueryAnswer::Config(config))
        }
    }
}
