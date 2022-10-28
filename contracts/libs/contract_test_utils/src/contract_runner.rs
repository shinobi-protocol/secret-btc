use cosmwasm_std::testing::*;
use cosmwasm_std::{
    from_binary, Binary, CosmosMsg, Env, HandleResponse, HandleResult, InitResponse, StdResult,
    WasmMsg,
};
use serde::Serialize;

use crate::context::{ClientDeps, Context};

pub trait ContractRunner {
    type InitMsg: Serialize;
    type HandleMsg: Serialize;
    type QueryMsg: Serialize;

    fn init(deps: &mut ClientDeps, env: Env, msg: Self::InitMsg) -> StdResult<InitResponse>;
    fn handle(deps: &mut ClientDeps, env: Env, msg: Self::HandleMsg) -> HandleResult;
    fn query(deps: &ClientDeps, msg: Self::QueryMsg) -> StdResult<Binary>;

    fn run_init(context: &mut Context, env: Env, msg: Self::InitMsg) -> StdResult<InitResponse> {
        let mut deps = context.client_deps();
        let result = Self::init(&mut deps, env, msg);
        if let Ok(response) = &result {
            context.exec_state_contract_messages(&response.messages)
        }
        result
    }

    fn run_handle(
        context: &mut Context,
        env: Env,
        msg: Self::HandleMsg,
    ) -> StdResult<HandleResponse> {
        let mut deps = context.client_deps();
        let result = Self::handle(&mut deps, env, msg);
        if let Ok(response) = &result {
            context.exec_state_contract_messages(&response.messages)
        }
        result
    }

    fn run_query(context: &mut Context, msg: Self::QueryMsg) -> StdResult<Binary> {
        Self::query(&context.client_deps(), msg)
    }
}
