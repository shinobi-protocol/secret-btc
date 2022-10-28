use crate::contract::api_ed25519_verifier::ApiEd25519Verifier;
use crate::state::StorageLightClientDB;
use aes_siv::aead::generic_array::GenericArray;
use aes_siv::siv::Aes128Siv;
use cosmwasm_std::QueryResponse;
use cosmwasm_std::{
    to_binary, Api, Binary, Extern, Querier, QueryResult, StdError, StdResult, Storage,
};
use sfps_lib::cosmos_proto::tendermint::types::Header;
use sfps_lib::cosmos_proto::tendermint::types::LightBlock;
use sfps_lib::light_client::{LightClient, ReadonlyLightClientDB};
use sfps_lib::merkle::MerkleProof;
use sfps_lib::response_deliver_tx_proof::tx_msg_data_of_response_deliver_tx;
use sfps_lib::response_deliver_tx_proof::ResponseDeliverTxProof;
use shared_types::sfps::{QueryAnswer, QueryMsg};
use shared_types::state_proxy::client::Secp256k1ApiSigner;
use shared_types::state_proxy::client::StateProxyDeps;

use super::CONTRACT_LABEL;

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    let deps = StateProxyDeps::restore(
        &deps.storage,
        &deps.api,
        &deps.querier,
        CONTRACT_LABEL,
        &Secp256k1ApiSigner::new(&deps.api),
    )?;

    let result = match msg {
        QueryMsg::MaxInterval {} => query_max_interval(&deps.storage),
        QueryMsg::CurrentHighestHeaderHash {} => query_current_highest_block_hash(&deps.storage),
        QueryMsg::HashByIndex { index } => query_hash_by_index(&deps.storage, index),
        QueryMsg::HashListLength {} => query_hash_list_length(&deps.storage),
        QueryMsg::VerifyResponseDeliverTxProof {
            merkle_proof,
            headers,
            block_hash_index,
            encryption_key,
        } => query_verify_response_deliver_tx_proof(
            &deps,
            merkle_proof,
            headers,
            block_hash_index,
            encryption_key,
        ),
        QueryMsg::VerifySubsequentLightBlocks {
            anchor_header,
            anchor_header_index,
            following_light_blocks,
            commit_flags,
        } => query_verify_subsequent_light_blocks(
            &deps,
            anchor_header,
            anchor_header_index,
            following_light_blocks,
            commit_flags,
        ),
    };
    Ok(result?)
}

fn query_max_interval<S: Storage>(storage: &S) -> Result<QueryResponse, StdError> {
    let mut light_client_db = StorageLightClientDB::from_readonly_storage(storage);
    let max_interval = light_client_db.get_max_interval();
    let response = QueryAnswer::MaxInterval { max_interval };
    Ok(to_binary(&response)?)
}

fn query_current_highest_block_hash<S: Storage>(storage: &S) -> Result<QueryResponse, StdError> {
    let mut light_client_db = StorageLightClientDB::from_readonly_storage(storage);
    // this unwrap is ok because light_client is initialized at contract initialization
    let hash = light_client_db.get_highest_hash().unwrap();
    let response = QueryAnswer::CurrentHighestHeaderHash {
        hash: Binary::from(hash.hash),
        height: hash.height,
    };
    Ok(to_binary(&response)?)
}

fn query_hash_list_length<S: Storage>(storage: &S) -> Result<QueryResponse, StdError> {
    let mut light_client_db = StorageLightClientDB::from_readonly_storage(storage);
    let length = light_client_db.get_hash_list_length();
    let response = QueryAnswer::HashListLength {
        length: length as u64,
    };
    Ok(to_binary(&response)?)
}

fn query_hash_by_index<S: Storage>(storage: &S, index: u64) -> Result<QueryResponse, StdError> {
    let mut light_client_db = StorageLightClientDB::from_readonly_storage(storage);
    let hash = light_client_db
        .get_hash_by_index(index as usize)
        .ok_or_else(|| StdError::generic_err("no hash"))?;
    let response = QueryAnswer::HashByIndex {
        hash: Binary::from(hash.hash),
        height: hash.height,
    };
    Ok(to_binary(&response)?)
}

fn query_verify_response_deliver_tx_proof<A: Api, Q: Querier>(
    deps: &StateProxyDeps<A, Q>,
    merkle_proof: MerkleProof,
    headers: Vec<Header>,
    block_hash_index: u64,
    encryption_key: Binary,
) -> Result<QueryResponse, StdError> {
    let response_deliver_tx_proof = ResponseDeliverTxProof {
        merkle_proof,
        headers,
    };
    // Verify Merkle Proof
    let chaindb = StorageLightClientDB::from_readonly_storage(&deps.storage);
    let mut light_client = LightClient::new(chaindb);
    light_client
        .verify_response_deliver_tx_proof(&response_deliver_tx_proof, block_hash_index as usize)
        .map_err(|e| StdError::generic_err(e.to_string()))?;

    // Decrypt response data from contract
    let encrypted_response_from_contract = &tx_msg_data_of_response_deliver_tx(
        &response_deliver_tx_proof
            .leaf_response_deliver_tx()
            .map_err(|e| StdError::generic_err(e.to_string()))?,
    )
    .data[0]
        .data;
    let decrypted_data = decrypt_response_from_contract(
        encrypted_response_from_contract,
        encryption_key.as_slice(),
    )?;

    let res = QueryAnswer::VerifyResponseDeliverTxProof { decrypted_data };
    Ok(to_binary(&res)?)
}

fn query_verify_subsequent_light_blocks<A: Api, Q: Querier>(
    deps: &StateProxyDeps<A, Q>,
    anchor_header: Header,
    anchor_header_index: u64,
    following_light_blocks: Vec<LightBlock>,
    commit_flags: Vec<bool>,
) -> Result<QueryResponse, StdError> {
    let chaindb = StorageLightClientDB::from_readonly_storage(&deps.storage);
    let mut light_client = LightClient::new(chaindb);
    let committed_hashes = light_client
        .verify_subsequent_light_blocks(
            anchor_header,
            anchor_header_index as usize,
            following_light_blocks,
            commit_flags,
            &mut ApiEd25519Verifier { api: &deps.api },
        )
        .map_err(|e| StdError::generic_err(e.to_string()))?;
    let res = QueryAnswer::VerifySubsequentLightBlocks { committed_hashes };
    Ok(to_binary(&res)?)
}

pub fn decrypt_response_from_contract(data: &[u8], encryption_key: &[u8]) -> StdResult<Binary> {
    let mut cipher = Aes128Siv::new(*GenericArray::from_slice(encryption_key));
    let plain_text = cipher
        .decrypt(&[&[]], data)
        .map_err(|_| StdError::generic_err("decryption error".to_string()))?;
    let base64_text =
        String::from_utf8(plain_text).map_err(|e| StdError::generic_err(e.to_string()))?;
    Binary::from_base64(&base64_text)
}
