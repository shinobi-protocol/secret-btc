use crate::state::StorageLightClientDB;
use cosmwasm_std::{Api, Env, Extern, HandleResponse, HandleResult, Querier, StdError, Storage};
use secret_toolkit::utils::pad_handle_result;
use sfps_lib::light_client::LightClient;
use sfps_lib::subsequent_hashes::CommittedHashes;
use shared_types::sfps::HandleMsg;
use shared_types::BLOCK_SIZE;

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: HandleMsg,
) -> HandleResult {
    let response = match msg {
        HandleMsg::AppendSubsequentHashes { committed_hashes } => {
            try_append_subsequent_hashes(deps, committed_hashes)
        }
    };
    pad_handle_result(response, BLOCK_SIZE)
}

fn try_append_subsequent_hashes<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    committed_hashes: CommittedHashes,
) -> HandleResult {
    let light_client_db = StorageLightClientDB::from_storage(&mut deps.storage);
    let mut light_client = LightClient::new(light_client_db);
    light_client
        .append_subsequent_hashes(committed_hashes)
        .map_err(|e| StdError::generic_err(e.to_string()))?;
    Ok(HandleResponse::default())
}
