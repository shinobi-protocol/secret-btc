use crate::state::{read_config, write_config};
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, InitResponse, InitResult, Querier, QueryResult,
    StdError, StdResult, Storage,
};
use secret_toolkit::utils::calls::HandleCallback;
use secret_toolkit::utils::pad_handle_result;
use shared_types::shuriken::{HandleMsg, InitMsg, QueryAnswer, QueryMsg};
use shared_types::state_proxy::client::{Secp256k1ApiSigner, StateProxyDeps};
use shared_types::{bitcoin_spv, sfps, BLOCK_SIZE};

const CONTRACT_LABEL: &[u8] = b"shuriken";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> InitResult {
    let mut deps = StateProxyDeps::init(
        &mut deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        msg.seed.clone(),
        msg.config.state_proxy.clone(),
        &Secp256k1ApiSigner::new(&deps.api),
    )?;
    write_config(&mut deps.storage, msg.config, &deps.api)?;
    Ok(InitResponse {
        messages: deps.storage.cosmos_msgs()?,
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    let mut deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )?;
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
        HandleMsg::SFPSProxyAppendSubsequentHashes { committed_hashes } => {
            vec![
                sfps::HandleMsg::AppendSubsequentHashes { committed_hashes }.to_cosmos_msg(
                    config.sfps.hash,
                    config.sfps.address,
                    None,
                )?,
            ]
        }
    };

    let messages = deps.storage.add_messages_to_state_proxy_msg(messages)?;
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
    let deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )?;
    match msg {
        QueryMsg::Config {} => {
            let config = read_config(&deps.storage, &deps.api)?;
            to_binary(&QueryAnswer::Config(config))
        }
    }
}
