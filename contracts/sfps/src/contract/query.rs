use crate::contract::api_ed25519_verifier::ApiEd25519Verifier;
use crate::state::StorageLightClientDB;
use aes_siv::aead::generic_array::GenericArray;
use aes_siv::siv::Aes128Siv;
use cosmwasm_std::{
    to_binary, Api, Binary, Extern, Querier, QueryResult, StdError, StdResult, Storage,
};
use sfps_lib::light_block::header::Header;
use sfps_lib::light_block::LightBlock;
use sfps_lib::light_client::{LightClient, ReadonlyLightClientDB};
use sfps_lib::tx_result_proof::TxResultProof;
use shared_types::sfps::{QueryAnswer, QueryMsg};

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::MaxInterval {} => query_max_interval(&deps.storage),
        QueryMsg::CurrentHighestHeaderHash {} => query_current_highest_header_hash(&deps.storage),
        QueryMsg::HashByIndex { index } => query_hash_by_index(&deps.storage, index),
        QueryMsg::HashListLength {} => query_hash_list_length(&deps.storage),
        QueryMsg::VerifyTxResultProof {
            tx_result_proof,
            header_hash_index,
            encryption_key,
        } => {
            query_verify_tx_result_proof(&deps, tx_result_proof, header_hash_index, encryption_key)
        }
        QueryMsg::VerifySubsequentLightBlocks {
            current_highest_header,
            light_blocks,
        } => query_verify_subsequent_light_blocks(&deps, current_highest_header, light_blocks),
    }
}

fn query_max_interval<S: Storage>(storage: &S) -> QueryResult {
    let mut light_client_db = StorageLightClientDB::from_readonly_storage(storage);
    let max_interval = light_client_db.get_max_interval();
    let response = QueryAnswer::MaxInterval { max_interval };
    Ok(to_binary(&response)?)
}

fn query_current_highest_header_hash<S: Storage>(storage: &S) -> QueryResult {
    let mut light_client_db = StorageLightClientDB::from_readonly_storage(storage);
    // this unwrap is ok because light_client is initialized at contract initialization
    let hash = light_client_db.get_highest_hash().unwrap();
    let response = QueryAnswer::CurrentHighestHeaderHash {
        hash: Binary::from(hash),
    };
    Ok(to_binary(&response)?)
}

fn query_hash_list_length<S: Storage>(storage: &S) -> QueryResult {
    let mut light_client_db = StorageLightClientDB::from_readonly_storage(storage);
    let length = light_client_db.get_hash_list_length();
    let response = QueryAnswer::HashListLength {
        length: length as u64,
    };
    Ok(to_binary(&response)?)
}

fn query_hash_by_index<S: Storage>(storage: &S, index: u64) -> QueryResult {
    let mut light_client_db = StorageLightClientDB::from_readonly_storage(storage);
    let hash = light_client_db
        .get_hash_by_index(index as usize)
        .ok_or_else(|| StdError::generic_err("no hash"))?;
    let response = QueryAnswer::HashByIndex {
        hash: Binary::from(hash),
    };
    Ok(to_binary(&response)?)
}

fn query_verify_tx_result_proof<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    tx_result_proof: TxResultProof,
    header_hash_index: u64,
    encryption_key: Binary,
) -> QueryResult {
    // Verify Merkle Proof
    let chaindb = StorageLightClientDB::from_readonly_storage(&deps.storage);
    let mut light_client = LightClient::new(chaindb);
    light_client
        .verify_tx_result_proof(&tx_result_proof, header_hash_index as usize)
        .map_err(|e| StdError::generic_err(e.to_string()))?;

    // Decrypt response data from contract
    let encrypted_response_from_contract = &tx_result_proof
        .tx_result
        .tx_msg_data()
        .map_err(|e| StdError::generic_err(format!("failed to decode tx_msg_data: {}", e)))?
        .data[0]
        .data;
    let decrypted_data = decrypt_response_from_contract(
        encrypted_response_from_contract,
        encryption_key.as_slice(),
    )?;

    let res = QueryAnswer::VerifyTxResultProof { decrypted_data };
    Ok(to_binary(&res)?)
}

fn query_verify_subsequent_light_blocks<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    current_highest_header: Header,
    light_blocks: Vec<LightBlock>,
) -> QueryResult {
    let chaindb = StorageLightClientDB::from_readonly_storage(&deps.storage);
    let mut light_client = LightClient::new(chaindb);
    let committed_hashes = light_client
        .verify_subsequent_light_blocks(
            current_highest_header,
            light_blocks,
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

#[cfg(test)]
mod test {
    use super::*;
    use sfps_lib::tx_result_proof::TxMsgData;
    #[test]
    fn test_decrypt_response_from_contract() {
        let tx_msg_data = TxMsgData::from_bytes(Binary::from_base64("CpcDCiovc2VjcmV0LmNvbXB1dGUudjFiZXRhMS5Nc2dFeGVjdXRlQ29udHJhY3QS6AKBPee4v+Bpz0kNkY1ql6L69UewzstC5QTiNgUUusPP/Kww50gICa5ZF8VJdU4P8sz5NuylW8NInA6Oxi8K0DHGznya0GWWS3fYMueM3qL1GsGTBUsz1n5U9vsMd7jJDABh48xEtoLFsjnmAOtQWElBcpgwrwWV8UOq3cd1CNWBbUwkYt399cCMVI25fhGNphSaUIFkMDECSUDjb0dKLSPCG+s+rdCeZqRLIRsuWKudFTslz0/BKus6EHsiCrEtxc0mD0mmXLezAISyR18haN0kMk5N+cnTCvluIsYvC9LeSZgPbmFeYxZmCRLH7x+FM7J7/dxlQ96h+EFLuxK9qXPrOtpx4+YjCz/HQCEhWZnQkzZo2oHEYfe+eixCk4SSV5oYKwOzmk1NJjogpg4BhVUfURth9OwEjg/Pt9oHdxhUWmaappt2T3yS0qlmpvrhqAghrw2KVfJxtKdXv6O16odRsaKRGe2KECc=").unwrap().as_slice()).unwrap();
        let encrypted_response_from_contract = &tx_msg_data.data[0].data;
        let encryption_key =
            Binary::from_base64("c7eyjEabicAp7av5f+HN87ict5G9qSp234k13Amf0TQ=").unwrap();
        let decrypted = decrypt_response_from_contract(
            encrypted_response_from_contract.as_slice(),
            encryption_key.as_slice(),
        )
        .unwrap();
        assert_eq!(
            r#"{"request_release_btc":{"request_key":[148,207,4,220,2,94,93,52,68,131,85,214,9,210,94,242,169,36,60,84,108,145,195,180,49,200,180,228,134,27,75,102]}}                                                                                                         "#,
            std::str::from_utf8(decrypted.as_slice()).unwrap()
        );
    }
}
