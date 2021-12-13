use crate::rng::{gen_seed, rng, Seed};
use crate::state::chaindb::StorageChainDB;
use crate::state::prng_seed::{read_prng_seed, write_prng_seed};
use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, HandleResult, Querier, StdError, Storage,
};
use secret_toolkit::utils::pad_handle_result;
use sfps_lib::header_chain::HeaderChain;
use sfps_lib::light_block::header::Header;
use sfps_lib::light_block::LightBlock;
use shared_types::sfps::HandleMsg;
use shared_types::BLOCK_SIZE;

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    let response = match msg {
        HandleMsg::AddLightBlock {
            current_highest_header,
            light_block,
            ..
        } => try_add_light_block(deps, env, current_highest_header, light_block),
        HandleMsg::AddEntropy { entropy, .. } => try_add_entropy(deps, env, entropy),
    };
    pad_handle_result(response, BLOCK_SIZE)
}

fn try_add_light_block<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    current_highest_header: Header,
    light_block: LightBlock,
) -> HandleResult {
    let byte = cosmwasm_std::to_vec(&light_block)?;
    let seed = update_seed(&mut deps.storage, &env, &byte);
    let mut prng = rng(seed);
    let chain_db = StorageChainDB::from_storage(&mut deps.storage);
    let mut header_chain = HeaderChain::new(chain_db);
    header_chain
        .add_block_to_highest(&current_highest_header, light_block, &mut prng)
        .map_err(|e| StdError::generic_err(e.to_string()))?;
    Ok(HandleResponse::default())
}

fn try_add_entropy<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: Binary,
) -> HandleResult {
    update_seed(&mut deps.storage, &env, entropy.as_slice());
    Ok(HandleResponse::default())
}

fn update_seed<S: Storage>(storage: &mut S, env: &Env, entropy: &[u8]) -> Seed {
    let prev_seed = read_prng_seed(storage);
    let new_seed = gen_seed(prev_seed, env, entropy);
    write_prng_seed(storage, &new_seed);
    new_seed
}
