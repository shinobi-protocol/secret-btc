use crate::merkle::simple_hash_from_byte_vectors;
use cosmos_proto::prost::Message;
use cosmos_proto::tendermint::crypto::public_key::Sum;
use cosmos_proto::tendermint::crypto::PublicKey;
use cosmos_proto::tendermint::types::SimpleValidator;
use cosmos_proto::tendermint::types::Validator;
use cosmos_proto::tendermint::types::ValidatorSet;

pub fn total_voting_power(validator_set: &ValidatorSet) -> i64 {
    validator_set
        .validators
        .iter()
        .map(|validator| validator.voting_power)
        .sum()
}
pub fn hash_validator_set(validator_set: &ValidatorSet) -> Vec<u8> {
    let leaves: Vec<Vec<u8>> = validator_set
        .validators
        .iter()
        .map(|validator| hash_bytes(validator))
        .collect();
    simple_hash_from_byte_vectors(leaves)
}

fn hash_bytes(validator: &Validator) -> Vec<u8> {
    SimpleValidator {
        pub_key: validator.pub_key.clone(),
        voting_power: validator.voting_power.clone(),
    }
    .encode_to_vec()
}

pub fn bytes_of_sum(sum: &Sum) -> &[u8] {
    let bytes = match sum {
        Sum::Ed25519(bytes) => bytes,
        Sum::Secp256k1(bytes) => bytes,
    };
    bytes
}

pub fn bytes_of_pub_key(pub_key: &PublicKey) -> &[u8] {
    let bytes = match &pub_key.sum {
        Some(sum) => bytes_of_sum(&sum),
        None => &[],
    };
    bytes
}
