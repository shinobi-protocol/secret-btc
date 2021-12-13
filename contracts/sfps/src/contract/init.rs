use crate::rng::gen_seed;
use crate::state::chaindb::StorageChainDB;
use crate::state::prng_seed::write_prng_seed;
use cosmwasm_std::{Api, Env, Extern, InitResponse, Querier, StdError, StdResult, Storage};
use sfps_lib::header_chain::HeaderChain;
use shared_types::sfps::InitMsg;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let seed = gen_seed([0; 32], &env, msg.entropy.as_slice());
    write_prng_seed(&mut deps.storage, &seed);

    let chaindb = StorageChainDB::from_storage(&mut deps.storage);
    let mut header_chain = HeaderChain::new(chaindb);
    header_chain
        .init(msg.initial_header, msg.max_interval)
        .map_err(|e| StdError::generic_err(e.to_string()))?;

    Ok(InitResponse::default())
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::*;
    #[test]
    fn test_init() {
        let init_msg_json = r#"{"max_interval":10,"initial_header":{"version":{"block":"11"},"chain_id":"supernova-1","height":"3","time":"2021-11-29T14:15:06.76915965Z","last_block_id":{"hash":"644AEAC7EAD429C34FC4035B7945E618C1C01CB55F81D9FD87DD26D71F8884A2","parts":{"total":2,"hash":"D968295666C909414C18B39BE8ECF5AA5F52ECDB1E6F6DFCF961C1390E9097DC"}},"last_commit_hash":"8C1AC7ABA871D69AE067F3DA9111A4A0474154058810E7FC3532A7FC9BDD75C9","data_hash":"F3CB4DDD588708EA6506536DA748239CEE190E0E0D6ECD4D5913764EEE01BDE0","validators_hash":"7BE60BAB30F3F6083D8930160E22715328DF50DC13E34B3F012DE8400D89E5C9","next_validators_hash":"7BE60BAB30F3F6083D8930160E22715328DF50DC13E34B3F012DE8400D89E5C9","consensus_hash":"048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F","app_hash":"3447F574FCFAEF48DB6BBF6F692D546986A47B7984AEC96DDD93817AFBB65214","last_results_hash":"96884A5D5334E36F52ED18B04DA5F930C61D60B5192019DB2088D9F8DE24D5A0","evidence_hash":"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855","proposer_address":"D7129578F042BA3240D6BD94F31104B7D1440A5E"},"entropy":"iJiTDs6+YrZHITULnyhjFWW4ciVxKWJ3+O5PYt2pBSM="}"#;
        let init_msg = cosmwasm_std::from_slice(init_msg_json.as_bytes()).unwrap();
        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env("initializer", &[]);
        init(&mut deps, env, init_msg).unwrap();
    }
}
