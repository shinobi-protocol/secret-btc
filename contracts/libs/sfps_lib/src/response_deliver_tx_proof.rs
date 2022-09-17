use crate::header::hash_header;
use crate::merkle::{Error as MerkleProofError, MerkleProof};
use cosmos_proto::cosmos::base::abci::v1beta1::TxMsgData;
use cosmos_proto::prost::Message;
use cosmos_proto::tendermint::abci::ResponseDeliverTx;
use cosmos_proto::tendermint::types::Header;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    EmptyHeaders,
    NoLastHeaderHash,
    UnconnectedHeaders,
    ProstError(cosmos_proto::prost::DecodeError),
    MerkleProof(MerkleProofError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::EmptyHeaders => f.write_str("empty headers"),
            Error::NoLastHeaderHash => f.write_str("no last header hash"),
            Error::UnconnectedHeaders => f.write_str("unconnected headers"),
            Error::ProstError(e) => write!(f, "prost error: {}", e),
            Error::MerkleProof(e) => write!(f, "merkle proof error: {}", e),
        }
    }
}

impl From<cosmos_proto::prost::DecodeError> for Error {
    fn from(e: cosmos_proto::prost::DecodeError) -> Self {
        Self::ProstError(e)
    }
}

impl From<MerkleProofError> for Error {
    fn from(e: MerkleProofError) -> Self {
        Self::MerkleProof(e)
    }
}

pub fn tx_msg_data_of_response_deliver_tx(response_deliver_tx: &ResponseDeliverTx) -> TxMsgData {
    TxMsgData::decode(response_deliver_tx.data.as_slice()).unwrap()
}

pub fn encode_response_deliver_tx_as_merkle_leaf(
    response_deliver_tx: &ResponseDeliverTx,
) -> Vec<u8> {
    response_deliver_tx.encode_to_vec()
}

// Merkle Proof for ResponseDeliverTx.
// This proof is valid when and only when all of 1-3 proposition are correct.
// 1. (1 < a < length(headers)) header[a] contains hash of header[a-1] as it's last_block_id.hash'
// 2. the first element of headers(= the lowest header of headers) contains merkle root of merkle proof as it's 'last_result_hash'
// 3. merkle proof includes response_deliver_tx as it's leaf
#[derive(Clone, Debug, PartialEq)]
pub struct ResponseDeliverTxProof {
    pub merkle_proof: MerkleProof,
    pub headers: Vec<Header>,
}

impl ResponseDeliverTxProof {
    // if tx result proof is correct, returns hash of the highest header of connected headers.
    pub fn verify(&self) -> Result<Vec<u8>, Error> {
        let highest_header_hash = verify_headers_connected(&self.headers)?;
        let results_hash = self.headers.first().unwrap().last_results_hash.clone();
        self.merkle_proof.verify(results_hash)?;
        Ok(highest_header_hash)
    }

    pub fn leaf_response_deliver_tx(&self) -> Result<ResponseDeliverTx, Error> {
        Ok(ResponseDeliverTx::decode(
            self.merkle_proof.leaf.as_slice(),
        )?)
    }
}

// If given headers is connected, returns hash of the highest header.
fn verify_headers_connected(headers: &[Header]) -> Result<Vec<u8>, Error> {
    match headers.len() {
        0 => Err(Error::EmptyHeaders),
        _ => {
            let mut iter = headers.iter();
            let mut last_hash = hash_header(iter.next().unwrap());
            for header in iter {
                if last_hash
                    != header
                        .last_block_id
                        .clone()
                        .ok_or(Error::NoLastHeaderHash)?
                        .hash
                {
                    return Err(Error::UnconnectedHeaders);
                }
                last_hash = hash_header(header);
            }
            Ok(last_hash)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::merkle::simple_hash_from_byte_vectors;
    use cosmos_proto::cosmos::base::abci::v1beta1::MsgData;
    use cosmos_proto::prost_types::Timestamp;
    use cosmos_proto::tendermint::types::*;
    use cosmos_proto::tendermint::version::*;

    #[test]
    fn test_response_deliver_tx_encode_as_merkle_leaf() {
        let response_deliver_tx = ResponseDeliverTx {
            code: 0,
            data: base64::decode("CpcDCiovc2VjcmV0LmNvbXB1dGUudjFiZXRhMS5Nc2dFeGVjdXRlQ29udHJhY3QS6AJVzmyAo99HQULa7zPh5czf0jm6vUIgLaV5GqHcbtdXEEIK0ZGQvykkyg6ikiUk3o5ynhv8kWK/3I+Eg9FEESWZuBsML2e4q/a4tFJk6eogjMhfJQ2uO4SKdPg4NTjB0An4bCC1uQdryNwQOpRM//GgidK55QZCeVIfEOAaD1GdVomol5t12V2qbjuM2vk/U8OLuLdjGgV2aMxju5tmjkdfwLiw2EpJlOG4qkuF9Lef0IUiFI627ED7G7JM2bzjjeI3ihXbYEgmmBfIe6gExN6K7yrJnWjzxrSVp3wAAkDkHu968GE+nX/mAuMyW7ME3yAnRv29wbyGgQi6tOByLIcKPy/1c3j3PEkmV2PQ/003zh5giCRJ75vbBs6xKcOM2V+l6m22N0KjVZZGfpJyOdJLQQPr7DYAQYarMEO81NayHr/SBjDbfOcfZoFcosg+dr7xfzSqqegs81dZUBSwsvboa0jILtzzbOE=").unwrap(),
            log: "".to_string(),
            info: "".to_string(),
            gas_used: 23757,
            gas_wanted: 50000,
            events: vec![],
            codespace: "".to_string(),
        };
        let encoded = encode_response_deliver_tx_as_merkle_leaf(&response_deliver_tx);
        assert_eq!(
            hex::encode_upper(&encoded),
            "129A030A97030A2A2F7365637265742E636F6D707574652E763162657461312E4D736745786563757465436F6E747261637412E80255CE6C80A3DF474142DAEF33E1E5CCDFD239BABD42202DA5791AA1DC6ED75710420AD19190BF2924CA0EA2922524DE8E729E1BFC9162BFDC8F8483D144112599B81B0C2F67B8ABF6B8B45264E9EA208CC85F250DAE3B848A74F8383538C1D009F86C20B5B9076BC8DC103A944CFFF1A089D2B9E5064279521F10E01A0F519D5689A8979B75D95DAA6E3B8CDAF93F53C38BB8B7631A057668CC63BB9B668E475FC0B8B0D84A4994E1B8AA4B85F4B79FD08522148EB6EC40FB1BB24CD9BCE38DE2378A15DB6048269817C87BA804C4DE8AEF2AC99D68F3C6B495A77C000240E41EEF7AF0613E9D7FE602E3325BB304DF202746FDBDC1BC868108BAB4E0722C870A3F2FF57378F73C49265763D0FF4D37CE1E60882449EF9BDB06CEB129C38CD95FA5EA6DB63742A35596467E927239D24B4103EBEC36004186AB3043BCD4D6B21EBFD20630DB7CE71F66815CA2C83E76BEF17F34AAA9E82CF357595014B0B2F6E86B48C82EDCF36CE128D0860330CDB901"
        );
        let leaf_hash = simple_hash_from_byte_vectors(vec![encoded.clone()]);
        assert_eq!(
            hex::encode_upper(leaf_hash),
            "3F54943D32A473050D0C96E69E0C5778BE11A540BAEFBFE005D86883A60D6CC3"
        );
        let root = simple_hash_from_byte_vectors(vec![encoded]);
        assert_eq!(
            hex::encode_upper(root),
            "3F54943D32A473050D0C96E69E0C5778BE11A540BAEFBFE005D86883A60D6CC3"
        );
    }

    #[test]
    fn test_response_deliver_tx_tx_msg_data() {
        let response_deliver_tx = ResponseDeliverTx {
            code: 0,
            data: base64::decode("CpcDCiovc2VjcmV0LmNvbXB1dGUudjFiZXRhMS5Nc2dFeGVjdXRlQ29udHJhY3QS6AJVzmyAo99HQULa7zPh5czf0jm6vUIgLaV5GqHcbtdXEEIK0ZGQvykkyg6ikiUk3o5ynhv8kWK/3I+Eg9FEESWZuBsML2e4q/a4tFJk6eogjMhfJQ2uO4SKdPg4NTjB0An4bCC1uQdryNwQOpRM//GgidK55QZCeVIfEOAaD1GdVomol5t12V2qbjuM2vk/U8OLuLdjGgV2aMxju5tmjkdfwLiw2EpJlOG4qkuF9Lef0IUiFI627ED7G7JM2bzjjeI3ihXbYEgmmBfIe6gExN6K7yrJnWjzxrSVp3wAAkDkHu968GE+nX/mAuMyW7ME3yAnRv29wbyGgQi6tOByLIcKPy/1c3j3PEkmV2PQ/003zh5giCRJ75vbBs6xKcOM2V+l6m22N0KjVZZGfpJyOdJLQQPr7DYAQYarMEO81NayHr/SBjDbfOcfZoFcosg+dr7xfzSqqegs81dZUBSwsvboa0jILtzzbOE=").unwrap(),
            log: "".to_string(),
            info: "".to_string(),
            gas_used: 23757,
            gas_wanted: 50000,
            events: vec![],
            codespace: "".to_string(),
        };
        let tx_msg_data = tx_msg_data_of_response_deliver_tx(&response_deliver_tx);
        assert_eq!(
            tx_msg_data,
            TxMsgData {
                data: vec![MsgData {
                    msg_type: "/secret.compute.v1beta1.MsgExecuteContract".into(),
                    data: vec![
                        85, 206, 108, 128, 163, 223, 71, 65, 66, 218, 239, 51, 225, 229, 204, 223,
                        210, 57, 186, 189, 66, 32, 45, 165, 121, 26, 161, 220, 110, 215, 87, 16,
                        66, 10, 209, 145, 144, 191, 41, 36, 202, 14, 162, 146, 37, 36, 222, 142,
                        114, 158, 27, 252, 145, 98, 191, 220, 143, 132, 131, 209, 68, 17, 37, 153,
                        184, 27, 12, 47, 103, 184, 171, 246, 184, 180, 82, 100, 233, 234, 32, 140,
                        200, 95, 37, 13, 174, 59, 132, 138, 116, 248, 56, 53, 56, 193, 208, 9, 248,
                        108, 32, 181, 185, 7, 107, 200, 220, 16, 58, 148, 76, 255, 241, 160, 137,
                        210, 185, 229, 6, 66, 121, 82, 31, 16, 224, 26, 15, 81, 157, 86, 137, 168,
                        151, 155, 117, 217, 93, 170, 110, 59, 140, 218, 249, 63, 83, 195, 139, 184,
                        183, 99, 26, 5, 118, 104, 204, 99, 187, 155, 102, 142, 71, 95, 192, 184,
                        176, 216, 74, 73, 148, 225, 184, 170, 75, 133, 244, 183, 159, 208, 133, 34,
                        20, 142, 182, 236, 64, 251, 27, 178, 76, 217, 188, 227, 141, 226, 55, 138,
                        21, 219, 96, 72, 38, 152, 23, 200, 123, 168, 4, 196, 222, 138, 239, 42,
                        201, 157, 104, 243, 198, 180, 149, 167, 124, 0, 2, 64, 228, 30, 239, 122,
                        240, 97, 62, 157, 127, 230, 2, 227, 50, 91, 179, 4, 223, 32, 39, 70, 253,
                        189, 193, 188, 134, 129, 8, 186, 180, 224, 114, 44, 135, 10, 63, 47, 245,
                        115, 120, 247, 60, 73, 38, 87, 99, 208, 255, 77, 55, 206, 30, 96, 136, 36,
                        73, 239, 155, 219, 6, 206, 177, 41, 195, 140, 217, 95, 165, 234, 109, 182,
                        55, 66, 163, 85, 150, 70, 126, 146, 114, 57, 210, 75, 65, 3, 235, 236, 54,
                        0, 65, 134, 171, 48, 67, 188, 212, 214, 178, 30, 191, 210, 6, 48, 219, 124,
                        231, 31, 102, 129, 92, 162, 200, 62, 118, 190, 241, 127, 52, 170, 169, 232,
                        44, 243, 87, 89, 80, 20, 176, 178, 246, 232, 107, 72, 200, 46, 220, 243,
                        108, 225
                    ],
                }]
            }
        );
    }

    #[test]
    fn test_verify_headers_connected_sanity() {
        let headers = {
            let mut prev_header_hash =
                hex::decode("AAAE99EE9C5A976417E4343837BD046CBE6E66BB2C634D0BDA699F0263C64803")
                    .unwrap();
            let mut headers = vec![];
            for i in 0..5 {
                let header = Header {
                    version: Some(Consensus { app: 0, block: 0 }),
                    chain_id: "chain_id".to_string(),
                    height: i + 1,
                    time: Some(Timestamp {
                        seconds: 0,
                        nanos: 0,
                    }),
                    last_block_id: Some(BlockId {
                        hash: prev_header_hash,
                        part_set_header: Some(PartSetHeader {
                            total: 0,
                            hash: [0; 32].to_vec(),
                        }),
                    }),
                    last_commit_hash: [0; 32].to_vec(),
                    data_hash: [0; 32].to_vec(),
                    validators_hash: [0; 32].to_vec(),
                    next_validators_hash: [0; 32].to_vec(),
                    consensus_hash: [0; 32].to_vec(),
                    app_hash: [0; 32].to_vec(),
                    last_results_hash: [0; 32].to_vec(),
                    evidence_hash: [0; 32].to_vec(),
                    proposer_address: [0; 32].to_vec(),
                };
                prev_header_hash = hash_header(&header);
                headers.push(header);
            }
            headers
        };
        assert_eq!(
            verify_headers_connected(&headers).unwrap(),
            hash_header(headers.last().unwrap())
        )
    }

    #[test]
    fn test_verify_headers_connected_1_header() {
        let header = Header {
            version: Some(Consensus { app: 0, block: 0 }),
            chain_id: "chain_id".to_string(),
            height: 1,
            time: Some(Timestamp {
                seconds: 0,
                nanos: 0,
            }),
            last_block_id: Some(BlockId {
                hash: hex::decode(
                    "AAAE99EE9C5A976417E4343837BD046CBE6E66BB2C634D0BDA699F0263C64803",
                )
                .unwrap(),
                part_set_header: Some(PartSetHeader {
                    total: 0,
                    hash: [0; 32].to_vec(),
                }),
            }),
            last_commit_hash: [0; 32].to_vec(),
            data_hash: [0; 32].to_vec(),
            validators_hash: [0; 32].to_vec(),
            next_validators_hash: [0; 32].to_vec(),
            consensus_hash: [0; 32].to_vec(),
            app_hash: [0; 32].to_vec(),
            last_results_hash: [0; 32].to_vec(),
            evidence_hash: [0; 32].to_vec(),
            proposer_address: [0; 32].to_vec(),
        };
        let headers = vec![header.clone()];
        assert_eq!(
            verify_headers_connected(&headers).unwrap(),
            hash_header(&header)
        )
    }

    #[test]
    fn test_verify_headers_connected_0_header() {
        let headers = vec![];
        assert_eq!(
            verify_headers_connected(&headers).unwrap_err(),
            Error::EmptyHeaders
        )
    }

    #[test]
    fn test_verify_headers_connected_not_connected() {
        let change_hash = [0; 32].to_vec();
        for hash_change_index in 1..5 {
            let mut prev_header_hash =
                hex::decode("AAAE99EE9C5A976417E4343837BD046CBE6E66BB2C634D0BDA699F0263C64803")
                    .unwrap();
            let mut headers = vec![];
            for i in 0..5 {
                if i == hash_change_index {
                    prev_header_hash = change_hash.clone();
                }
                let header = Header {
                    version: Some(Consensus { app: 0, block: 0 }),
                    chain_id: "chain_id".to_string(),
                    height: i + 1,
                    time: Some(Timestamp {
                        seconds: 0,
                        nanos: 0,
                    }),
                    last_block_id: Some(BlockId {
                        hash: prev_header_hash,
                        part_set_header: Some(PartSetHeader {
                            total: 0,
                            hash: [0; 32].to_vec(),
                        }),
                    }),
                    last_commit_hash: [0; 32].to_vec(),
                    data_hash: [0; 32].to_vec(),
                    validators_hash: [0; 32].to_vec(),
                    next_validators_hash: [0; 32].to_vec(),
                    consensus_hash: [0; 32].to_vec(),
                    app_hash: [0; 32].to_vec(),
                    last_results_hash: [0; 32].to_vec(),
                    evidence_hash: [0; 32].to_vec(),
                    proposer_address: [0; 32].to_vec(),
                };
                prev_header_hash = hash_header(&header);
                headers.push(header);
            }
            assert_eq!(
                verify_headers_connected(&headers).unwrap_err(),
                Error::UnconnectedHeaders
            )
        }
    }

    #[test]
    fn test_verify_response_deliver_tx_proof() {
        let headers = vec! [
            Header::decode(base64::decode("CgIICxILc2VjcmV0ZGV2LTEYqQEiDAiX4b2WBhCVg5TxAipICiCoWqI3O5cj3nZBf56slcPSTqn9dMOMVMkYb6ydoAHg4hIkCAESIKPNnEGXKSA/aryDs0kAF84CLRZP1wtPW8nYZYbY9PQVMiBVJkXMzCbsJyQkF8KRiYf66aktzd1D0ljrpz+PcsXgWzog47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFVCIAbegxoNWmEdX6w0dvOJg6V/lLpaJ0GhWcuFqgZM78VqSiAG3oMaDVphHV+sNHbziYOlf5S6WidBoVnLhaoGTO/FalIgBICRvH3cKD93v7+R1zxE2ljD34qcvIZ0Bdi389qtoi9aIFlj7IJQr3N9NldF+cH5b+2+9ZEAGqMYfjgmY/3WFJBRYiCOVvbDbHwsHpXQ40BFcPp/y6zxzhVmS0xQzyYsN4Fspmog47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFVyFBA+n7uzeNOW3kX1ZKV9QpdPXWpw").unwrap().as_slice()).unwrap(),
            Header::decode(base64::decode("CgIICxILc2VjcmV0ZGV2LTEYqgEiDAic4b2WBhDi1pr0AipICiBOxz8DZ87CbclYVz8/lkrhMkNsaBLFinZ6l+RMiIPRZhIkCAESIPPDA2c61jgboW8LDJ8zpaiAsiTW6bxDVpkjFkYaK9/tMiC9dCIGbeQTCFHVGS/IHBU9f/O7bI1HCZtEwDGf0U5q/zoghwEm7EcvdPGQNIRA3rm5h6JODHONp1KcKJFIeqPEnfxCIAbegxoNWmEdX6w0dvOJg6V/lLpaJ0GhWcuFqgZM78VqSiAG3oMaDVphHV+sNHbziYOlf5S6WidBoVnLhaoGTO/FalIgBICRvH3cKD93v7+R1zxE2ljD34qcvIZ0Bdi389qtoi9aIPds0qgj1KQpxlZZ75UKO9FxvFwjImngjD8c5dXTX6VvYiDjsMRCmPwcFJr79MiZb7kkJ65B5GSbk0yklZkbeFK4VWog47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFVyFBA+n7uzeNOW3kX1ZKV9QpdPXWpw").unwrap().as_slice()).unwrap()
        ];
        let merkle_proof = MerkleProof{
            total: 1,
            index: 0,
            leaf: hex::decode("129a030a97030a2a2f7365637265742e636f6d707574652e763162657461312e4d736745786563757465436f6e747261637412e802a58d9a975d9ed398b813cbc932bcd0ee126f22067c39f2838102ac81d9992d9406ab406a927854aa5f49ce2923e46a1b76c26411b61bb7c6829e5bcab1411663295eabb92b239eb31a2fd2004b92225e4e47d2fba51e2b5fd67262833dc1dfb7dc3d96b187f6e637d18b0d8596f100bf286bfcec17fc95c09256d03c7a97bd6dfd0463204d0267454b16aa1f538a93fd5176c3e50d08e87ab083fe41bed2832058feecdb67dee1d3cff1e09718df1e06757f867768be6f36d0197d8e362c44a7e30120b480b5b4caefc54032e426975ccb36245b817f9c7ed77fdeb1a4f9eb8d390944277c547eb06ea6165d1866c690a8d4cfb97f23dfdd9611f07d693f7d4ba90ed31016d9782af9f72af9629a8ba34d5c0d742b4e84bc0788b25c303a711ed3fa9de629134c3b7bc8cc06805bd4888575d2c4a2e8e188ccd1d006a44ef314a051f0c8d1af85b77e61ca4004af467a50d0a709f84ecead1e18f936c4b41db831448a43fb01ee1728c0cf2430f1bc08").unwrap(),
            aunts: vec![],
        };
        let response_deliver_tx_proof = ResponseDeliverTxProof {
            headers,
            merkle_proof,
        };
        response_deliver_tx_proof.verify().unwrap();
    }
}
