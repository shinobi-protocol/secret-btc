use crate::state::{read_config, write_config};
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HandleResult, InitResponse, InitResult, Querier,
    QueryResult, StdError, Storage,
};
use secret_toolkit::utils::{pad_handle_result, HandleCallback};
use shared_types::shuriken::{HandleMsg, InitMsg, QueryAnswer, QueryMsg};
use shared_types::{bitcoin_spv, finance_admin, sfps, BLOCK_SIZE};
use std::convert::TryInto;

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
        HandleMsg::ChangeFinanceAdmin { new_finance_admin } => {
            let mut config = read_config(&deps.storage, &deps.api)?;
            if env.message.sender != config.finance_admin.address {
                return Err(StdError::generic_err("not finance admin"));
            }
            config.finance_admin = new_finance_admin;
            write_config(&mut deps.storage, config, &deps.api)?;
            vec![]
        }
        HandleMsg::BitcoinSPVProxy { msg } => {
            let best_height = match msg {
                bitcoin_spv::HandleMsg::AddHeaders { tip_height, .. } => tip_height,
            };
            vec![
                msg.to_cosmos_msg(config.bitcoin_spv.hash, config.bitcoin_spv.address, None)?,
                finance_admin::CommonHandleMsg::MintBitcoinSPVReward {
                    executer: env.message.sender,
                    best_height: best_height,
                }
                .to_cosmos_msg(
                    config.finance_admin.hash,
                    config.finance_admin.address,
                    None,
                )?,
            ]
        }
        HandleMsg::SFPSProxy { msg } => {
            let best_height = match &msg {
                sfps::HandleMsg::AddLightBlock { light_block, .. } => {
                    light_block.signed_header.header.height
                }
                _ => return Err(StdError::generic_err("sfps proxy msg is not AddLightBlock")),
            };
            vec![
                msg.to_cosmos_msg(config.sfps.hash, config.sfps.address, None)?,
                finance_admin::CommonHandleMsg::MintSFPSReward {
                    executer: env.message.sender,
                    best_height: best_height.try_into().unwrap(),
                }
                .to_cosmos_msg(
                    config.finance_admin.hash,
                    config.finance_admin.address,
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
