use crate::error::Error;
use crate::state::config::write_config;
use crate::state::prefix::PREFIX_PRNG;
use cosmwasm_std::{Api, Env, Extern, InitResponse, Querier, StdResult, Storage};
use shared_types::gateway::InitMsg;
use shared_types::prng::init_prng;

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
    init_prng(&mut deps.storage, PREFIX_PRNG, &env, msg.entropy.as_slice())?;
    write_config(&mut deps.storage, msg.config, &deps.api)?;

    Ok(InitResponse {
        messages: vec![],
        log: vec![],
    })
}
