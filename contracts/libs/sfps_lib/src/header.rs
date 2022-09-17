use crate::merkle::simple_hash_from_byte_vectors;
use cosmos_proto::prost::Message;
use cosmos_proto::tendermint::types::Header;

const FIELDS_NUM: usize = 14;
pub fn hash_header(header: &Header) -> Vec<u8> {
    let bytes = hash_bytes(header);
    simple_hash_from_byte_vectors(bytes.to_vec())
}

fn hash_bytes(header: &Header) -> [Vec<u8>; FIELDS_NUM] {
    [
        header.version.clone().unwrap().encode_to_vec(),
        header.chain_id.encode_to_vec(),
        header.height.encode_to_vec(),
        header.time.clone().unwrap().encode_to_vec(),
        header.last_block_id.clone().unwrap().encode_to_vec(),
        header.last_commit_hash.encode_to_vec(),
        header.data_hash.encode_to_vec(),
        header.validators_hash.encode_to_vec(),
        header.next_validators_hash.encode_to_vec(),
        header.consensus_hash.encode_to_vec(),
        header.app_hash.encode_to_vec(),
        header.last_results_hash.encode_to_vec(),
        header.evidence_hash.encode_to_vec(),
        header.proposer_address.encode_to_vec(),
    ]
}

#[cfg(test)]
mod test {
    use crate::header::hash_header;
    use cosmos_proto::prost_types::*;
    use cosmos_proto::tendermint::types::*;
    use cosmos_proto::tendermint::version::*;

    #[test]
    fn test_hash_header() {
        // genesis header
        let header = Header {
            version: Some(Consensus { block: 11, app: 0 }),
            chain_id: "secret-4".to_string(),
            height: 1000001,
            time: Some(Timestamp {
                seconds: 1637683657,
                nanos: 575128904,
            }),
            last_block_id: Some(BlockId {
                hash: hex::decode(
                    "EDFFB7C581AEDADA9A41E355BEED063E15BB1B2C281957AE864E0392B5DEDE2F",
                )
                .unwrap(),
                part_set_header: Some(PartSetHeader {
                    total: 1,
                    hash: hex::decode(
                        "01932DD4FA96010353DA4A60AC99EED3865C28D8AEADB67DFD9949A9A82D1940",
                    )
                    .unwrap(),
                }),
            }),
            last_commit_hash: hex::decode(
                "7FE08F2D155922661247762A68A16C0439E894336FE3E87C07ED3ED0FDECE418",
            )
            .unwrap(),
            data_hash: hex::decode(
                "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            )
            .unwrap(),
            validators_hash: hex::decode(
                "9EFBBA1CEA6B4CAE8F27C7F16E830FBBFEED6AB8D35245DE263D63FE4F7211B0",
            )
            .unwrap(),
            next_validators_hash: hex::decode(
                "9EFBBA1CEA6B4CAE8F27C7F16E830FBBFEED6AB8D35245DE263D63FE4F7211B0",
            )
            .unwrap(),
            consensus_hash: hex::decode(
                "717BE5422EFEECF5C48B9B5EA4AFD9C4C2E002A676CA2E9512CAE7802DA37D92",
            )
            .unwrap(),
            app_hash: hex::decode(
                "EE32DBF923F4B63F0CCCBB7CABE3BFE58AD33B6749A2D2DC7130E683ED41FE91",
            )
            .unwrap(),
            last_results_hash: hex::decode(
                "D0C1BDF3B1811F72A1DA190266D06CE950B465DE1681436AF826619D7DC92A79",
            )
            .unwrap(),
            evidence_hash: hex::decode(
                "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            )
            .unwrap(),
            proposer_address: hex::decode("B89CDCF017D80A946FBFFC41A2583C03190E8613").unwrap(),
        };
        assert_eq!(
            hex::encode_upper(&hash_header(&header)),
            "275AFF29FB91FCFC3E6581B3522502205F03758864CD1030D2A5E2212AA4FBE2",
        );
    }
}
