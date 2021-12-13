use crate::state::{read_config, write_config};
use cosmwasm_std::{
    to_binary, Api, CosmosMsg, Env, Extern, HandleResponse, HandleResult, InitResponse, InitResult,
    Querier, QueryResult, StdError, StdResult, Storage,
};
use secret_toolkit::snip20::{send_from_msg, send_msg, set_viewing_key_msg};
use secret_toolkit::utils::pad_handle_result;
use secret_toolkit::utils::HandleCallback;
use shared_types::log;
use shared_types::treasury::{
    Config, HandleMsg, InitMsg, Operation, QueryAnswer, QueryMsg, TREASURY_VIEWING_KEY,
};
use shared_types::BLOCK_SIZE;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> InitResult {
    let config = msg.config;
    let message = set_viewing_key_msg(
        TREASURY_VIEWING_KEY.into(),
        None,
        BLOCK_SIZE,
        config.snb.hash.clone(),
        config.snb.address.clone(),
    )?;
    write_config(&mut deps.storage, &deps.api, config)?;
    Ok(InitResponse {
        messages: vec![message],
        log: vec![],
    })
}

fn check_sender_is_owner(config: &Config, env: &Env) -> StdResult<()> {
    if env.message.sender != config.owner {
        Err(StdError::unauthorized())
    } else {
        Ok(())
    }
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    let response = match msg {
        HandleMsg::TransferOwnership { owner } => {
            let mut config = read_config(&deps.storage, &deps.api)?;
            check_sender_is_owner(&config, &env)?;
            config.owner = owner;
            write_config(&mut deps.storage, &deps.api, config)?;
            Ok(HandleResponse::default())
        }
        HandleMsg::Operate { operations } => {
            let config = read_config(&deps.storage, &deps.api)?;
            check_sender_is_owner(&config, &env)?;
            let token_address = config.snb.address;
            let token_hash = config.snb.hash;
            let log_address = config.log.address;
            let log_hash = config.log.hash;
            let messages: Vec<CosmosMsg> = operations
                .into_iter()
                .flat_map(|operation| match operation {
                    Operation::Send { to, amount } => vec![
                        send_msg(
                            to.clone(),
                            amount,
                            None,
                            None,
                            BLOCK_SIZE,
                            token_hash.clone(),
                            token_address.clone(),
                        ),
                        log::HandleMsg::AddEvents {
                            events: vec![(
                                to,
                                log::Event::ReceivedFromTreasury(
                                    log::event::ReceivedFromTreasuryData {
                                        time: env.block.time,
                                        amount: amount,
                                    },
                                ),
                            )],
                        }
                        .to_cosmos_msg(
                            log_hash.clone(),
                            log_address.clone(),
                            None,
                        ),
                    ],
                    Operation::ReceiveFrom { from, amount } => vec![
                        send_from_msg(
                            from.clone(),
                            env.contract.address.clone(),
                            amount,
                            None,
                            None,
                            BLOCK_SIZE,
                            token_hash.clone(),
                            token_address.clone(),
                        ),
                        log::HandleMsg::AddEvents {
                            events: vec![(
                                from,
                                log::Event::SentToTreasury(log::event::SentToTreasuryData {
                                    time: env.block.time,
                                    amount: amount,
                                }),
                            )],
                        }
                        .to_cosmos_msg(
                            log_hash.clone(),
                            log_address.clone(),
                            None,
                        ),
                    ],
                })
                .collect::<StdResult<Vec<CosmosMsg>>>()?;
            Ok(HandleResponse {
                messages,
                log: vec![],
                data: None,
            })
        }
    };
    pad_handle_result(response, BLOCK_SIZE)
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::Config {} => {
            let config = read_config(&deps.storage, &deps.api)?;
            let answer = QueryAnswer::Config(config);
            Ok(to_binary(&answer)?)
        }
    }
}
