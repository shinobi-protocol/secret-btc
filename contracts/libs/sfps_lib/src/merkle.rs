use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidLeafHash { given: Vec<u8>, proof: Vec<u8> },
    InvalidRootHash { given: Vec<u8>, proof: Vec<u8> },
    InvalidTotal,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidLeafHash { given, proof } => {
                write!(
                    f,
                    "invalid leaf hash: given {}, proof {}",
                    hex::encode(&given),
                    hex::encode(&proof)
                )
            }
            Error::InvalidRootHash { given, proof } => {
                write!(
                    f,
                    "invalid root hash: given {}, proof {}",
                    hex::encode(&given),
                    hex::encode(&proof)
                )
            }
            Error::InvalidTotal => f.write_str("invalid total"),
        }
    }
}

/// Compute a simple Merkle root from vectors of arbitrary byte vectors.
/// The leaves of the tree are the bytes of the given byte vectors in
/// the given order.
pub fn simple_hash_from_byte_vectors(byte_vecs: Vec<Vec<u8>>) -> Vec<u8> {
    simple_hash_from_byte_slices_inner(byte_vecs.as_slice())
}

#[derive(Serialize, Deserialize, Debug, schemars::JsonSchema, Clone, PartialEq)]
pub struct MerkleProof {
    pub total: u64,
    pub index: u64,
    #[schemars(with = "String")]
    #[serde(serialize_with = "hex::serde::serialize_upper")]
    #[serde(deserialize_with = "hex::serde::deserialize")]
    pub leaf_hash: Vec<u8>,
    #[schemars(with = "Vec<String>")]
    #[serde(with = "serde_aunts")]
    pub aunts: Vec<Vec<u8>>,
}

mod serde_aunts {
    use super::*;
    use serde::{Deserializer, Serializer};

    #[derive(Serialize, Deserialize, Debug, schemars::JsonSchema, Clone, PartialEq)]
    pub struct Aunt(
        #[schemars(with = "String")]
        #[serde(serialize_with = "hex::serde::serialize_upper")]
        #[serde(deserialize_with = "hex::serde::deserialize")]
        pub Vec<u8>,
    );

    #[derive(Serialize, Deserialize, Debug, schemars::JsonSchema, Clone, PartialEq)]
    pub struct Aunts(pub Vec<Aunt>);

    impl From<Aunts> for Vec<Vec<u8>> {
        fn from(aunts: Aunts) -> Vec<Vec<u8>> {
            aunts.0.into_iter().map(|aunt| aunt.0).collect()
        }
    }

    impl From<&[Vec<u8>]> for Aunts {
        fn from(aunts: &[Vec<u8>]) -> Aunts {
            Aunts(aunts.iter().map(|aunt| Aunt(aunt.clone())).collect())
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let aunts = Aunts::deserialize(deserializer)?;
        Ok(aunts.into())
    }

    pub fn serialize<S>(value: &[Vec<u8>], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let aunts: Aunts = value.into();
        aunts.serialize(serializer)
    }
}

impl MerkleProof {
    pub fn verify(&self, root_hash: Vec<u8>, leaf: &[u8]) -> Result<(), Error> {
        let leaf_hash = leaf_hash(leaf);
        if leaf_hash != self.leaf_hash {
            return Err(Error::InvalidLeafHash {
                given: leaf_hash,
                proof: self.leaf_hash.clone(),
            });
        }
        let computed_root_hash = self.compute_root_hash()?;
        if root_hash != computed_root_hash {
            return Err(Error::InvalidRootHash {
                given: root_hash,
                proof: computed_root_hash,
            });
        }
        Ok(())
    }
}

impl MerkleProof {
    fn compute_root_hash(&self) -> Result<Vec<u8>, Error> {
        compute_from_aunts(self.index, self.total, &self.leaf_hash, &self.aunts)
    }
}

// recurse into subtrees
fn simple_hash_from_byte_slices_inner(byte_slices: &[Vec<u8>]) -> Vec<u8> {
    let length = byte_slices.len() as u64;
    match length {
        0 => empty_hash(),
        1 => leaf_hash(byte_slices[0].as_slice()),
        _ => {
            let k = get_split_point(length) as usize;
            let left = simple_hash_from_byte_slices_inner(&byte_slices[..k]);
            let right = simple_hash_from_byte_slices_inner(&byte_slices[k..]);
            inner_hash(&left, &right)
        }
    }
}

fn get_split_point(length: u64) -> u64 {
    length.next_power_of_two() / 2
}

// tmhash(0x01 || left || right)
fn inner_hash(left: &[u8], right: &[u8]) -> Vec<u8> {
    // make a new array starting with 0x1 and copy in the bytes
    let mut inner_bytes = Vec::with_capacity(left.len() + right.len() + 1);
    inner_bytes.push(0x01);
    inner_bytes.extend_from_slice(left);
    inner_bytes.extend_from_slice(right);

    // hash it !
    Sha256::digest(&inner_bytes).to_vec()
}

// tmhash(0x00 || leaf)
pub fn leaf_hash(bytes: &[u8]) -> Vec<u8> {
    // make a new array starting with 0 and copy in the bytes
    let mut leaf_bytes = Vec::with_capacity(bytes.len() + 1);
    leaf_bytes.push(0x00);
    leaf_bytes.extend_from_slice(bytes);

    // hash it !
    Sha256::digest(&leaf_bytes).to_vec()
}

// tmhash({})
fn empty_hash() -> Vec<u8> {
    // the empty string / byte slice
    let empty = Vec::with_capacity(0);

    // hash it !
    Sha256::digest(&empty).to_vec()
}

fn compute_from_aunts(
    index: u64,
    total: u64,
    leaf_hash: &Vec<u8>,
    inner_hashes: &[Vec<u8>],
) -> Result<Vec<u8>, Error> {
    if index >= total || total == 0 {
        return Err(Error::InvalidTotal);
    }
    let inner_hash_num = inner_hashes.len();
    if inner_hash_num == 0 {
        if total == 1 {
            return Ok(leaf_hash.clone());
        }
        return Err(Error::InvalidTotal);
    }
    let num_left: u64 = get_split_point(total);
    if index < num_left {
        let left_hash = compute_from_aunts(
            index,
            num_left,
            leaf_hash,
            &inner_hashes[..inner_hash_num - 1],
        )?;
        return Ok(inner_hash(&left_hash, &inner_hashes[inner_hash_num - 1]));
    }
    let right_hash = compute_from_aunts(
        index - num_left,
        total - num_left,
        leaf_hash,
        &inner_hashes[..inner_hash_num - 1],
    )?;
    Ok(inner_hash(&inner_hashes[inner_hash_num - 1], &right_hash))
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_get_split_point() {
        assert_eq!(get_split_point(1), 0);
        assert_eq!(get_split_point(2), 1);
        assert_eq!(get_split_point(3), 2);
        assert_eq!(get_split_point(4), 2);
        assert_eq!(get_split_point(5), 4);
        assert_eq!(get_split_point(10), 8);
        assert_eq!(get_split_point(20), 16);
        assert_eq!(get_split_point(100), 64);
        assert_eq!(get_split_point(255), 128);
        assert_eq!(get_split_point(256), 128);
        assert_eq!(get_split_point(257), 256);
    }

    #[test]
    fn test_rfc6962_node_inner_hash() {
        let node_hash_hex = "AA217FE888E47007FA15EDAB33C2B492A722CB106C64667FC2B044444DE66BBB";
        let left_string = "N123";
        let right_string = "N456";

        let node_hash = hex::decode(node_hash_hex).unwrap();
        let hash = inner_hash(left_string.as_bytes(), right_string.as_bytes());
        assert_eq!(node_hash, hash);
    }

    #[test]
    fn test_rfc6962_empty_leaf_hash() {
        let empty_leaf_hash_hex =
            "6E340B9CFFB37A989CA544E6BB780A2C78901D3FB33738768511A30617AFA01D";
        let empty_leaf_hash = hex::decode(empty_leaf_hash_hex).unwrap();
        let hash = leaf_hash(&[]);
        assert_eq!(empty_leaf_hash, hash);
    }

    #[test]
    fn test_merkle_proof_sanity() {
        // 3 leaves
        let leaves = vec![b"1".to_vec(), b"2".to_vec(), b"3".to_vec()];

        let merkle_root =
            hex::decode("FE6E9D4604F578602851A2C15EF3894CA07B9517F7D5F7DEDC28179CA888580D")
                .unwrap();
        assert_eq!(merkle_root, simple_hash_from_byte_vectors(leaves.clone()));
        let leaf_hashes = [
            hex::decode("2215E8AC4E2B871C2A48189E79738C956C081E23AC2F2415BF77DA199DFD920C")
                .unwrap(),
            hex::decode("FA61E3DEC3439589F4784C893BF321D0084F04C572C7AF2B68E3F3360A35B486")
                .unwrap(),
            hex::decode("906C5D2485CAE722073A430F4D04FE1767507592CEF226629AEADB85A2EC909D")
                .unwrap(),
        ];
        for (i, leaf) in leaves.iter().enumerate() {
            let hash = leaf_hash(leaf);
            assert_eq!(leaf_hashes[i], hash);
        }

        let proof = MerkleProof {
            index: 0,
            total: 3,
            leaf_hash: leaf_hashes[0].clone(),
            aunts: vec![leaf_hashes[1].clone(), leaf_hashes[2].clone()],
        };
        let computed_root = proof.compute_root_hash().unwrap();
        assert_eq!(merkle_root, computed_root);
        proof.verify(computed_root, &leaves[0]).unwrap();

        let proof = MerkleProof {
            index: 1,
            total: 3,
            leaf_hash: leaf_hashes[1].clone(),
            aunts: vec![leaf_hashes[0].clone(), leaf_hashes[2].clone()],
        };
        let computed_root = proof.compute_root_hash().unwrap();
        assert_eq!(merkle_root, computed_root);
        proof.verify(computed_root, &leaves[1]).unwrap();

        let proof = MerkleProof {
            index: 2,
            total: 3,
            leaf_hash: leaf_hashes[2].clone(),
            aunts: vec![inner_hash(&leaf_hashes[0], &leaf_hashes[1])],
        };
        let computed_root = proof.compute_root_hash().unwrap();
        assert_eq!(merkle_root, computed_root);
        proof.verify(computed_root, &leaves[2]).unwrap();

        // 4 leaves
        let leaves = vec![b"1".to_vec(), b"2".to_vec(), b"3".to_vec(), b"4".to_vec()];

        let merkle_root =
            hex::decode("4C4B77FE3FC6CFB92E4D3C90B5ADE42F059A1F112A49827F07EDBB7BD4540E7B")
                .unwrap();
        assert_eq!(merkle_root, simple_hash_from_byte_vectors(leaves.clone()));
        let leaf_hashes = [
            hex::decode("2215E8AC4E2B871C2A48189E79738C956C081E23AC2F2415BF77DA199DFD920C")
                .unwrap(),
            hex::decode("FA61E3DEC3439589F4784C893BF321D0084F04C572C7AF2B68E3F3360A35B486")
                .unwrap(),
            hex::decode("906C5D2485CAE722073A430F4D04FE1767507592CEF226629AEADB85A2EC909D")
                .unwrap(),
            hex::decode("11E1F558223F4C71B6BE1CECFD1F0DE87146D2594877C27B29EC519F9040213C")
                .unwrap(),
        ];
        for (i, leaf) in leaves.iter().enumerate() {
            let hash = leaf_hash(leaf);
            assert_eq!(leaf_hashes[i], hash);
        }

        let proof = MerkleProof {
            index: 0,
            total: 4,
            leaf_hash: leaf_hashes[0].clone(),
            aunts: vec![
                leaf_hashes[1].clone(),
                inner_hash(&leaf_hashes[2], &leaf_hashes[3]),
            ],
        };
        let computed_root = proof.compute_root_hash().unwrap();
        assert_eq!(merkle_root, computed_root);
        proof.verify(computed_root, &leaves[0]).unwrap();

        let proof = MerkleProof {
            index: 1,
            total: 4,
            leaf_hash: leaf_hashes[1].clone(),
            aunts: vec![
                leaf_hashes[0].clone(),
                inner_hash(&leaf_hashes[2], &leaf_hashes[3]),
            ],
        };
        let computed_root = proof.compute_root_hash().unwrap();
        assert_eq!(merkle_root, computed_root);
        proof.verify(computed_root, &leaves[1]).unwrap();

        let proof = MerkleProof {
            index: 2,
            total: 4,
            leaf_hash: leaf_hashes[2].clone(),
            aunts: vec![
                leaf_hashes[3].clone(),
                inner_hash(&leaf_hashes[0], &leaf_hashes[1]),
            ],
        };
        let computed_root = proof.compute_root_hash().unwrap();
        assert_eq!(merkle_root, computed_root);
        proof.verify(computed_root, &leaves[2]).unwrap();

        let proof = MerkleProof {
            index: 3,
            total: 4,
            leaf_hash: leaf_hashes[3].clone(),
            aunts: vec![
                leaf_hashes[2].clone(),
                inner_hash(&leaf_hashes[0], &leaf_hashes[1]),
            ],
        };
        let computed_root = proof.compute_root_hash().unwrap();
        assert_eq!(merkle_root, computed_root);
        proof.verify(computed_root, &leaves[3]).unwrap();
    }

    #[test]
    fn test_merkle_proof_invalid_leaf_hash() {
        let merkle_root =
            hex::decode("FE6E9D4604F578602851A2C15EF3894CA07B9517F7D5F7DEDC28179CA888580D")
                .unwrap();
        let leaf_hashes = [
            hex::decode("2215E8AC4E2B871C2A48189E79738C956C081E23AC2F2415BF77DA199DFD920C")
                .unwrap(),
            hex::decode("FA61E3DEC3439589F4784C893BF321D0084F04C572C7AF2B68E3F3360A35B486")
                .unwrap(),
            hex::decode("906C5D2485CAE722073A430F4D04FE1767507592CEF226629AEADB85A2EC909D")
                .unwrap(),
        ];

        let proof = MerkleProof {
            index: 0,
            total: 3,
            leaf_hash: leaf_hashes[0].clone(),
            aunts: vec![leaf_hashes[1].clone(), leaf_hashes[2].clone()],
        };
        let err = proof.verify(merkle_root, b"4").unwrap_err();
        assert_eq!(
            err,
            Error::InvalidLeafHash {
                given: leaf_hash(b"4"),
                proof: proof.leaf_hash
            }
        )
    }

    #[test]
    fn test_merkle_proof_invalid_root_hash() {
        let invalid_merkle_root =
            hex::decode("EE6E9D4604F578602851A2C15EF3894CA07B9517F7D5F7DEDC28179CA888580D")
                .unwrap();
        let leaf_hashes = [
            hex::decode("2215E8AC4E2B871C2A48189E79738C956C081E23AC2F2415BF77DA199DFD920C")
                .unwrap(),
            hex::decode("FA61E3DEC3439589F4784C893BF321D0084F04C572C7AF2B68E3F3360A35B486")
                .unwrap(),
            hex::decode("906C5D2485CAE722073A430F4D04FE1767507592CEF226629AEADB85A2EC909D")
                .unwrap(),
        ];

        let proof = MerkleProof {
            index: 0,
            total: 3,
            leaf_hash: leaf_hashes[0].clone(),
            aunts: vec![leaf_hashes[1].clone(), leaf_hashes[2].clone()],
        };
        let err = proof.verify(invalid_merkle_root.clone(), b"1").unwrap_err();
        assert_eq!(
            err,
            Error::InvalidRootHash {
                given: invalid_merkle_root,
                proof: proof.compute_root_hash().unwrap(),
            }
        )
    }

    #[test]
    fn test_deserialize_merkle_proof() {
        let json = r#"
        {
            "total": 1,
            "index": 3,
            "leaf_hash": "EE6E9D4604F578602851A2C15EF3894CA07B9517F7D5F7DEDC28179CA888580D",
            "aunts": [
                "2215E8AC4E2B871C2A48189E79738C956C081E23AC2F2415BF77DA199DFD920C",
                "FA61E3DEC3439589F4784C893BF321D0084F04C572C7AF2B68E3F3360A35B486",
                "906C5D2485CAE722073A430F4D04FE1767507592CEF226629AEADB85A2EC909D"
            ]
        }
        "#;
        let proof: MerkleProof = serde_json::from_str(json).unwrap();
        assert_eq!(
            proof,
            MerkleProof {
                total: 1,
                index: 3,
                leaf_hash: hex::decode(
                    "EE6E9D4604F578602851A2C15EF3894CA07B9517F7D5F7DEDC28179CA888580D"
                )
                .unwrap(),
                aunts: vec![
                    hex::decode("2215E8AC4E2B871C2A48189E79738C956C081E23AC2F2415BF77DA199DFD920C")
                        .unwrap(),
                    hex::decode("FA61E3DEC3439589F4784C893BF321D0084F04C572C7AF2B68E3F3360A35B486")
                        .unwrap(),
                    hex::decode("906C5D2485CAE722073A430F4D04FE1767507592CEF226629AEADB85A2EC909D")
                        .unwrap(),
                ]
            }
        )
    }
}
