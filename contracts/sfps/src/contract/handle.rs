use crate::state::StorageLightClientDB;
use cosmwasm_std::{
    Api, Env, Extern, HandleResponse, HandleResult, Querier, StdError, StdResult, Storage,
};
use secret_toolkit::utils::pad_handle_result;
use sfps_lib::light_client::LightClient;
use sfps_lib::subsequent_hashes::CommittedHashes;
use shared_types::sfps::HandleMsg;
use shared_types::state_proxy::client::{Secp256k1ApiSigner, StateProxyDeps};
use shared_types::BLOCK_SIZE;

use super::CONTRACT_LABEL;

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    let mut deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )?;
    let result = match msg {
        HandleMsg::AppendSubsequentHashes { committed_hashes } => {
            try_append_subsequent_hashes(&mut deps, committed_hashes)
        }
    };

    let result = match result {
        Ok(mut response) => {
            response.messages = deps
                .storage
                .add_messages_to_state_proxy_msg(response.messages)?;
            Ok(response)
        }
        Err(e) => Err(e),
    };
    pad_handle_result(result, BLOCK_SIZE)
}

fn try_append_subsequent_hashes<A: Api, Q: Querier>(
    deps: &mut StateProxyDeps<A, Q>,
    committed_hashes: CommittedHashes,
) -> HandleResult {
    let light_client_db = StorageLightClientDB::from_storage(&mut deps.storage);
    let mut light_client = LightClient::new(light_client_db);
    light_client
        .append_subsequent_hashes(committed_hashes)
        .map_err(|e| StdError::generic_err(e.to_string()))?;
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}
