use crate::error::Error;
use crate::state::chaindb::StorageChainDB;
use crate::state::config::read_config;
use bitcoin::blockdata::transaction::Transaction;
use bitcoin::consensus::encode::deserialize;
use bitcoin::consensus::encode::serialize;
use bitcoin::hash_types::TxMerkleNode;
use bitcoin::Network;
use bitcoin_header_chain::header_chain::{HeaderChain, StoredBlockHeader};
use bitcoin_header_chain::merkle_proof::MerkleProof;
use cosmwasm_std::{
    to_binary, Api, Binary, Extern, Querier, QueryResponse, QueryResult, StdError, Storage,
};
use shared_types::bitcoin_spv::{MerkleProofMsg, QueryAnswer, QueryMsg};
use std::str::FromStr;
use std::string::ToString;

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    let result = match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::BestHeaderHash {} => query_best_header_hash(&deps.storage),
        QueryMsg::BlockHeader { height, .. } => query_block_header(&deps.storage, height),
        QueryMsg::VerifyMerkleProof {
            height,
            tx,
            merkle_proof,
            ..
        } => query_verify_merkle_proof(&deps.storage, height, tx, merkle_proof),
    };
    Ok(result?)
}

fn query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> Result<QueryResponse, Error> {
    let config = read_config(&deps.storage)?;
    Ok(to_binary(&QueryAnswer::Config(config))?)
}

fn query_best_header_hash<S: Storage>(storage: &S) -> Result<QueryResponse, Error> {
    let config = read_config(storage)?;
    let chaindb = StorageChainDB::from_readonly_storage(storage);
    let mut header_chain = HeaderChain::new(chaindb, Network::from_str(&config.bitcoin_network)?);
    let tip = header_chain
        .tip()?
        .ok_or_else(|| StdError::generic_err("no tip of chain db"))?;
    let result = QueryAnswer::BestHeaderHash {
        hash: tip.header.block_hash().to_string(),
    };
    Ok(to_binary(&result)?)
}

fn query_block_header<S: Storage>(storage: &S, height: u32) -> Result<QueryResponse, Error> {
    let config = read_config(storage)?;
    let chaindb = StorageChainDB::from_readonly_storage(storage);
    let mut header_chain = HeaderChain::new(chaindb, Network::from_str(&config.bitcoin_network)?);
    let block_header: StoredBlockHeader = header_chain
        .header_at(height)?
        .ok_or_else(|| Error::contract_err("there is not the block_header in header_chain"))?;
    let result = QueryAnswer::BlockHeader {
        header: Binary(serialize(&block_header.header)),
    };
    Ok(to_binary(&result)?)
}

fn query_verify_merkle_proof<S: Storage>(
    storage: &S,
    height: u32,
    tx: Binary,
    merkle_proof: MerkleProofMsg,
) -> Result<QueryResponse, Error> {
    let config = read_config(storage)?;
    let chaindb = StorageChainDB::from_readonly_storage(storage);
    let tx: Transaction = deserialize::<Transaction>(tx.as_slice())?;
    let txid = tx.txid();
    let merkle_proof = msg_to_merkle_proof(merkle_proof)?;
    let mut header_chain = HeaderChain::new(chaindb, Network::from_str(&config.bitcoin_network)?);
    let tip_height = header_chain
        .tip_height()?
        .ok_or_else(|| Error::contract_err("no tip"))?;
    let header = header_chain
        .header_at(height)?
        .ok_or_else(|| Error::contract_err("block not found"))?;

    //
    // Validate Tx Confirmed
    //
    // check if merkle path contains tx
    if merkle_proof.leaf()?.as_hash() != txid.as_hash() {
        return Err(Error::contract_err("merkle path and tx does not match"));
    }

    // get block header from storage
    if tip_height.saturating_sub(height) + 1 < config.confirmation.into() {
        return Err(Error::contract_err("not confirmed yet"));
    }

    // check if the block contains merkle root of merkle path
    if merkle_proof.merkle_root()? != header.header.merkle_root {
        return Err(Error::contract_err("invalid merkle root"));
    }
    Ok(to_binary(&QueryAnswer::VerifyMerkleProof {
        success: true,
    })?)
}

pub fn msg_to_merkle_proof(msg: MerkleProofMsg) -> Result<MerkleProof, Error> {
    let siblings: Result<Vec<TxMerkleNode>, _> = msg
        .siblings
        .iter()
        .map(|x| TxMerkleNode::from_str(&x))
        .collect();
    Ok(MerkleProof {
        prefix: msg.prefix,
        siblings: siblings?,
    })
}
