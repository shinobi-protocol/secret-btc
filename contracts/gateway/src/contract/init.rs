use crate::contract::CONTRACT_LABEL;
use crate::error::Error;
use crate::state::config::write_config;
use crate::state::prefix::PREFIX_PRNG;
use cosmwasm_std::{Api, Env, Extern, InitResponse, Querier, StdResult, Storage};
use shared_types::gateway::InitMsg;
use shared_types::prng::{init_prng, update_prng};
use shared_types::state_proxy::client::{Secp256k1ApiSigner, StateProxyDeps};

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
    let mut deps = StateProxyDeps::init(
        &mut deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        msg.seed.clone(),
        msg.config.state_proxy.clone(),
        &Secp256k1ApiSigner::new(&deps.api),
    )?;
    init_prng(&mut deps.storage, PREFIX_PRNG, &env, msg.seed.as_ref())?;
    write_config(&mut deps.storage, msg.config, &deps.api)?;

    Ok(InitResponse {
        messages: deps.storage.cosmos_msgs()?,
        log: vec![],
    })
}
