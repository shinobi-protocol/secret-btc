// Init Tests
use super::*;
use crate::state::prefix::PREFIX_PRNG;
use common_macros::hash_set;
use contract_test_utils::contract_runner::ContractRunner;
use cosmwasm_std::{from_binary, ReadonlyStorage};
use shared_types::gateway::{QueryAnswer, QueryMsg};
use shared_types::state_proxy::client::Secp256k1ApiSigner;
use shared_types::state_proxy::client::StateProxyDeps;

#[test]
fn test_init_sanity() {
    let mut context = init_helper();
    match from_binary(&GatewayRunner::run_query(&mut context, QueryMsg::Config {}).unwrap())
        .unwrap()
    {
        QueryAnswer::Config(config) => {
            assert_eq!(config.btc_tx_values, hash_set! {100000000, 10000000});
        }
        _ => unreachable!(),
    }
    let deps = context.client_deps();
    let deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )
    .unwrap();
    let seed = deps.storage.get(PREFIX_PRNG).unwrap();
    assert_eq!(
        seed.as_slice(),
        [
            22, 217, 248, 105, 79, 175, 140, 16, 171, 211, 127, 13, 201, 71, 148, 62, 84, 178, 158,
            228, 251, 104, 219, 82, 84, 106, 72, 159, 209, 236, 193, 235
        ]
    );
}
