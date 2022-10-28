use crate::contract::CONTRACT_LABEL;
use crate::state::StorageLightClientDB;
use cosmwasm_std::{Api, Env, Extern, InitResponse, Querier, StdError, StdResult, Storage};
use sfps_lib::light_client::LightClient;
use shared_types::sfps::InitMsg;
use shared_types::state_proxy::client::{Secp256k1ApiSigner, StateProxyDeps};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let mut deps = StateProxyDeps::init(
        &mut deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        msg.seed.clone(),
        msg.config.state_proxy.clone(),
        &Secp256k1ApiSigner::new(&deps.api),
    )?;

    let chaindb = StorageLightClientDB::from_storage(&mut deps.storage);
    let mut light_client = LightClient::new(chaindb);
    light_client
        .init(msg.initial_header, msg.max_interval)
        .map_err(|e| StdError::generic_err(e.to_string()))?;

    Ok(InitResponse {
        messages: deps.storage.cosmos_msgs()?,
        log: vec![],
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::*;
    #[test]
    fn test_init() {
        let init_msg_json = r#"{"seed": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=","config":{"state_proxy":{"address":"address","hash": "hash"}},"max_interval":10,"initial_header":"CgIICxIIc2VjcmV0LTQYwIQ9IgwIw6P0jAYQtaOfsQEqSAogtuqUfryBu9/yUD5L5ohj/Niyzz44955n/iQO83cWzeoSJAgBEiCU0ANcdBhca90ER7l69oj7YDHNEb0pz7lyoRqU+pHbvzIgsD/4zfRY74ub9Zb45/CqswPKk5Iiege/cqgaS/CDzwk6ICZVWiA4fVTWRBskcgDJ34oGMbYNLbvk2MJ0nMIBD/ifQiCe+7oc6mtMro8nx/Fugw+7/u1quNNSRd4mPWP+T3IRsEognvu6HOprTK6PJ8fxboMPu/7tarjTUkXeJj1j/k9yEbBSIHF75UIu/uz1xIubXqSv2cTC4AKmdsoulRLK54Ato32SWiCLihxZRJGEH8Bx0hvuPhW6tC3nOY6EWR9bfeoQnP66QGIgxzJ8au+AnupslgZSUUVNV9m8ae9045iV3D0Pkr33p7tqIOOwxEKY/BwUmvv0yJlvuSQnrkHkZJuTTKSVmRt4UrhVchQDXA/dn7uUwoktl7saawriZL0wGA==","entropy":"iJiTDs6+YrZHITULnyhjFWW4ciVxKWJ3+O5PYt2pBSM="}"#;
        let init_msg = cosmwasm_std::from_slice(init_msg_json.as_bytes()).unwrap();
        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env("initializer", &[]);
        init(&mut deps, env, init_msg).unwrap();
    }
}
