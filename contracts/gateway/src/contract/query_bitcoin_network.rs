use crate::error::Error;
use bitcoin::Network;
use cosmwasm_std::Querier;
use secret_toolkit::utils::calls::Query;
use shared_types::{bitcoin_spv, ContractReference};
use std::str::FromStr;

pub fn query_bitcoin_network<Q: Querier>(
    querier: &Q,
    bitcoin_spv: ContractReference,
) -> Result<Network, Error> {
    if let bitcoin_spv::QueryAnswer::Config(config) =
        (bitcoin_spv::QueryMsg::Config {}).query(querier, bitcoin_spv.hash, bitcoin_spv.address)?
    {
        let network = Network::from_str(&config.bitcoin_network)?;
        Ok(network)
    } else {
        Err(Error::contract_err(
            "unexpected query answer from bitcoin spv",
        ))
    }
}
