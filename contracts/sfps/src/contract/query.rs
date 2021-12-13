use crate::state::chaindb::StorageChainDB;
use aes_siv::aead::generic_array::GenericArray;
use aes_siv::siv::Aes128Siv;
use cosmwasm_std::{
    to_binary, Api, Binary, Extern, Querier, QueryResult, StdError, StdResult, Storage,
};
use sfps_lib::header_chain::HeaderChain;
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
    }
}

fn query_max_interval<S: Storage>(storage: &S) -> QueryResult {
    let chain_db = StorageChainDB::from_readonly_storage(storage);
    let header_chain = HeaderChain::new(chain_db);
    let max_interval = header_chain.get_max_interval();
    let response = QueryAnswer::MaxInterval { max_interval };
    Ok(to_binary(&response)?)
}

fn query_current_highest_header_hash<S: Storage>(storage: &S) -> QueryResult {
    let chain_db = StorageChainDB::from_readonly_storage(storage);
    let header_chain = HeaderChain::new(chain_db);
    // this unwrap is ok because header_chain is initialized at contract initialization
    let hash = header_chain.get_highest_hash().unwrap();
    let response = QueryAnswer::CurrentHighestHeaderHash {
        hash: Binary::from(hash),
    };
    Ok(to_binary(&response)?)
}

fn query_hash_list_length<S: Storage>(storage: &S) -> QueryResult {
    let chain_db = StorageChainDB::from_readonly_storage(storage);
    let header_chain = HeaderChain::new(chain_db);
    let length = header_chain.get_hash_list_length();
    let response = QueryAnswer::HashListLength {
        length: length as u64,
    };
    Ok(to_binary(&response)?)
}

fn query_hash_by_index<S: Storage>(storage: &S, index: u64) -> QueryResult {
    let chain_db = StorageChainDB::from_readonly_storage(storage);
    let header_chain = HeaderChain::new(chain_db);
    let hash = header_chain
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
    let chaindb = StorageChainDB::from_readonly_storage(&deps.storage);
    let header_chain = HeaderChain::new(chaindb);
    header_chain
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
