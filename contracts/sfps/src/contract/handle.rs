use crate::state::{StorageChainDB, PREFIX_PRNG};
use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, HandleResult, Querier, StdError, Storage,
};
use secret_toolkit::utils::pad_handle_result;
use sfps_lib::header_chain::HeaderChain;
use sfps_lib::light_block::header::Header;
use sfps_lib::light_block::LightBlock;
use shared_types::prng::update_prng;
use shared_types::sfps::HandleMsg;
use shared_types::BLOCK_SIZE;

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    let response = match msg {
        HandleMsg::AddLightBlocks {
            current_highest_header,
            light_blocks,
            entropy,
            ..
        } => try_add_light_blocks(deps, env, current_highest_header, light_blocks, entropy),
    };
    pad_handle_result(response, BLOCK_SIZE)
}

fn try_add_light_blocks<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    mut current_highest_header: Header,
    light_blocks: Vec<LightBlock>,
    entropy: Binary,
) -> HandleResult {
    let mut rng = update_prng(
        &mut deps.storage,
        PREFIX_PRNG,
        &deps.api.canonical_address(&env.message.sender)?,
        entropy.as_slice(),
    )?;
    let chain_db = StorageChainDB::from_storage(&mut deps.storage);
    let mut header_chain = HeaderChain::new(chain_db);
    for light_block in light_blocks {
        header_chain
            .add_block_to_highest(&current_highest_header, light_block.clone(), &mut rng)
            .map_err(|e| StdError::generic_err(e.to_string()))?;
        current_highest_header = light_block.signed_header.header;
    }
    Ok(HandleResponse::default())
}
