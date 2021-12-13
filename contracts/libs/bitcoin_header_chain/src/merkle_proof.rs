use bitcoin::consensus::Encodable;
use bitcoin::hash_types::TxMerkleNode;
use bitcoin::hashes::Hash;
use serde_derive::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// Std Io error
    StdIoError(std::io::Error),
    /// No sibling
    NoSibling,
    // Invalid MerkleProof
    InvalidMerkleProof,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::NoSibling => write!(f, "no sibling"),
            Error::InvalidMerkleProof => write!(f, "invalid merkle path"),
            Error::StdIoError(ref e) => write!(f, "std io error {}", e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::StdIoError(e)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
// also known as MerkleProof.
pub struct MerkleProof {
    pub prefix: Vec<bool>,
    pub siblings: Vec<TxMerkleNode>,
}

impl MerkleProof {
    pub fn leaf(&self) -> Result<&TxMerkleNode, Error> {
        self.siblings.first().ok_or(Error::NoSibling)
    }

    pub fn merkle_root(&self) -> Result<TxMerkleNode, Error> {
        let mut current = *self.leaf()?;
        if self.siblings.len() - 1 != self.prefix.len() {
            return Err(Error::InvalidMerkleProof);
        }
        for (i, prefix) in self.prefix.iter().enumerate() {
            let sibling = &self.siblings[i + 1];
            match prefix {
                true => {
                    current = step(sibling, &current)?;
                }
                false => {
                    current = step(&current, sibling)?;
                }
            }
        }
        Ok(current)
    }
}

fn step(left: &TxMerkleNode, right: &TxMerkleNode) -> Result<TxMerkleNode, Error> {
    let mut encoder = TxMerkleNode::engine();
    left.consensus_encode(&mut encoder)?;
    right.consensus_encode(&mut encoder)?;
    Ok(TxMerkleNode::from_engine(encoder))
}

#[cfg(test)]
mod test {
    use super::*;

    type MerkleTree = Vec<TxMerkleNode>;

    fn tx_merkle_node(str: &str) -> TxMerkleNode {
        TxMerkleNode::from_slice(&hex::decode(str).unwrap()).unwrap()
    }

    fn gen_merkle_tree(strs: Vec<&str>) -> MerkleTree {
        let mut merkle_tree = vec![];
        for s in strs {
            merkle_tree.push(tx_merkle_node(s))
        }
        merkle_tree
    }

    struct TestCase {
        merkle_root: TxMerkleNode,
        merkle_proofs: Vec<MerkleProof>,
    }

    fn test_cases() -> Vec<TestCase> {
        let mut test_cases = vec![];
        // Only 1 Tx In Block
        test_cases.push(TestCase {
            merkle_root: tx_merkle_node(
                "352e4e0300dcb283d7aae269b49724b8cdcf973e5827d0ab1e7c94cb82264469",
            ),
            merkle_proofs: vec![MerkleProof {
                prefix: vec![],
                siblings: vec![tx_merkle_node(
                    "352e4e0300dcb283d7aae269b49724b8cdcf973e5827d0ab1e7c94cb82264469",
                )],
            }],
        });
        // Tree 3 Leaves
        let merkle_tree = gen_merkle_tree(vec![
            // leaves
            "00ffb46c84f38500f566ffd235caed6e61ecb37793184a08d9f077e5cc9c63d4",
            "ce85c79cfb3af65a6416762a5b47ecb9071bfcf41531d9aca5e4c277125a5fbc",
            "5dee81eeeaeaeb470f9d15670ed68237b2ee7b309e6d1ad1ed4339f59fc6e19b",
            // height = 1
            "8fa1c2b671bba5eb667894218b3cf2855d8e0e85c91dc0b0b899b7cc8b0670e8",
            "3b275abb86323974940fcdda044ef8e310a48d2135228cc03d89fb3c5c51507c",
            // merkle root
            "6624bd5c857daa7202c7b3eaa4285dab60595116b28f1ab4212ef480d79ef667",
        ]);
        test_cases.push(TestCase {
            merkle_root: merkle_tree.last().unwrap().clone(),
            merkle_proofs: vec![
                MerkleProof {
                    prefix: vec![false, false],
                    siblings: vec![merkle_tree[0], merkle_tree[1], merkle_tree[4]],
                },
                MerkleProof {
                    prefix: vec![true, false],
                    siblings: vec![merkle_tree[1], merkle_tree[0], merkle_tree[4]],
                },
                MerkleProof {
                    prefix: vec![false, true],
                    siblings: vec![merkle_tree[2], merkle_tree[2], merkle_tree[3]],
                },
            ],
        });
        // 4 Leaves
        let merkle_tree = gen_merkle_tree(vec![
            "37221d338269b6d12ad29a20e4beb9506526dded90eadb89e6074d231ac4d1f6",
            "03fbad67c14af86d280218ed971a98c7d14fe7f10417c1f350403b411f97a9dc",
            "7af61424f4f131892d6e972bec2d599b84affeafd958a7e7a8530aa0f1004790",
            "829b2a5ad886e897cc7a71da7d89519b11b0c474e650238ffb0f5163bac588cf",
            "265144e841c7866949f8b2795a44161fb771dcf06345dacd74b880a57a7a4b01",
            "bca5faae4e577c6ba65e951a800c1c7409a7903aaedd173e51bcd54804fcffa9",
            "32c7b4757d148e39aabf4ff74d3a898effbef3240ec948d23cac448cbd5c457b",
        ]);
        test_cases.push(TestCase {
            merkle_root: merkle_tree.last().unwrap().clone(),
            merkle_proofs: vec![
                MerkleProof {
                    prefix: vec![false, false],
                    siblings: vec![merkle_tree[0], merkle_tree[1], merkle_tree[5]],
                },
                MerkleProof {
                    prefix: vec![true, false],
                    siblings: vec![merkle_tree[1], merkle_tree[0], merkle_tree[5]],
                },
                MerkleProof {
                    prefix: vec![false, true],
                    siblings: vec![merkle_tree[2], merkle_tree[3], merkle_tree[4]],
                },
                MerkleProof {
                    prefix: vec![true, true],
                    siblings: vec![merkle_tree[3], merkle_tree[2], merkle_tree[4]],
                },
            ],
        });
        // 5 Leaves
        let merkle_tree = gen_merkle_tree(vec![
            "ce229a4b45813c067e0752417048b43ef7b5a480892a8c0f6c48429a55421f9a",
            "c3e6319f1df366b4cde09b272654e706229c10f195d1c00d52c578281e13bf3c",
            "ab59ec0da51d9502362c4c37c105c68ba85ff22ea92b9c9dafe17a5315b295d5",
            "97920a47828417036deb5096694c92f84ba10924459b414909cabc808b7924cc",
            "d416e6583cda8c43585b85a3b9875b52f0148736171a270545b67d68b8e8e172",
            "47433736eceb0827077b985393457141d0b0eb988007b46a2f403cf1953721cd",
            "10c870737f63468f9556fbd778aa89906d57c0405bf7dd38e2f9745cc1fe9be8",
            "171a1ae320886955bda7412e34627e400460113668b3028240f52d65876f4d28",
            "22c2ab3277225102f47d0884168e9a89978d21ca8ccf07663b86838f0ddd7734",
            "bcb1f9da6828eaeed40c35e287028ec590025fb749c04591ed8b4f17ebb87337",
            "b861b10180d112c235e55df04960ae67a430d69ddf007c923546a892b726ecc7",
        ]);
        test_cases.push(TestCase {
            merkle_root: merkle_tree.last().unwrap().clone(),
            merkle_proofs: vec![
                MerkleProof {
                    prefix: vec![false, false, false],
                    siblings: vec![
                        merkle_tree[0],
                        merkle_tree[1],
                        merkle_tree[6],
                        merkle_tree[9],
                    ],
                },
                MerkleProof {
                    prefix: vec![true, false, false],
                    siblings: vec![
                        merkle_tree[1],
                        merkle_tree[0],
                        merkle_tree[6],
                        merkle_tree[9],
                    ],
                },
                MerkleProof {
                    prefix: vec![false, true, false],
                    siblings: vec![
                        merkle_tree[2],
                        merkle_tree[3],
                        merkle_tree[5],
                        merkle_tree[9],
                    ],
                },
                MerkleProof {
                    prefix: vec![true, true, false],
                    siblings: vec![
                        merkle_tree[3],
                        merkle_tree[2],
                        merkle_tree[5],
                        merkle_tree[9],
                    ],
                },
                MerkleProof {
                    prefix: vec![false, false, true],
                    siblings: vec![
                        merkle_tree[4],
                        merkle_tree[4],
                        merkle_tree[7],
                        merkle_tree[8],
                    ],
                },
            ],
        });
        // 7 Leaves
        let merkle_tree = gen_merkle_tree(vec![
            // leaves
            "221e820e1baeb77ae33f1464a1a5c6fd2a318af254cd8539c25b31f0a4ce962a",
            "3bb13029ce7b1f559ef5e747fcac439f1455a2ec7c5f09b72290795e70665044",
            "8bfe43511ba3cff574389106072f280c69bef6d59f247f25ef5ad9b28db2d0c0",
            "e42a42699c63ac67dfa99e6fdbd50605e9b966a2178b8c4ede67013b07d7c5a5",
            "1ebc954a5a4cca3a8e76a59a7c0b54d3ecb27881de4427a47d63f4c09db13f8d",
            "63e3d6c1c162e0a2de745c354bdae28a25ee5e6b399fff5b04fc175ad30eb68a",
            "9c4302d63b378d5f38164dc4d902707de5b0d639fe34f14b7740f783036e3c00",
            // height = 1
            "3081b38382bb39b656594fe4950ee7a0439ed593eb4e31fe8e0b1a7c254dd45b",
            "606db11984d83d0786ac70ec276c527d02100515234a5ccb023c02c88f5f5cac",
            "28f27bffc446270f9d19ad9e3a52b1953304a55f9ba431fdd9c56f83e0730605",
            "5dbbcb9ac11c045c5ebf652c5038f0d74df7b9c31b26b0f9fc15bf70916cdb4d",
            // height = 2
            "774e8ab22f1430618d0132105f96162615f8cee0f459969cc06ac195f6402027",
            "045778e30e7e06bc0f32666b1636f9026ba8b2638e85a7f9345de73bb430ef0a",
            // root
            "2d99961074ab1dfb3a25c8d1543f851563f1807f785219b74f6ab05ee26c0b95",
        ]);
        test_cases.push(TestCase {
            merkle_root: merkle_tree.last().unwrap().clone(),
            merkle_proofs: vec![
                MerkleProof {
                    prefix: vec![false, false, false],
                    siblings: vec![
                        merkle_tree[0],
                        merkle_tree[1],
                        merkle_tree[8],
                        merkle_tree[12],
                    ],
                },
                MerkleProof {
                    prefix: vec![true, false, false],
                    siblings: vec![
                        merkle_tree[1],
                        merkle_tree[0],
                        merkle_tree[8],
                        merkle_tree[12],
                    ],
                },
                MerkleProof {
                    prefix: vec![false, true, false],
                    siblings: vec![
                        merkle_tree[2],
                        merkle_tree[3],
                        merkle_tree[7],
                        merkle_tree[12],
                    ],
                },
                MerkleProof {
                    prefix: vec![true, true, false],
                    siblings: vec![
                        merkle_tree[3],
                        merkle_tree[2],
                        merkle_tree[7],
                        merkle_tree[12],
                    ],
                },
                MerkleProof {
                    prefix: vec![false, false, true],
                    siblings: vec![
                        merkle_tree[4],
                        merkle_tree[5],
                        merkle_tree[10],
                        merkle_tree[11],
                    ],
                },
                MerkleProof {
                    prefix: vec![true, false, true],
                    siblings: vec![
                        merkle_tree[5],
                        merkle_tree[4],
                        merkle_tree[10],
                        merkle_tree[11],
                    ],
                },
                MerkleProof {
                    prefix: vec![false, true, true],
                    siblings: vec![
                        merkle_tree[6],
                        merkle_tree[6],
                        merkle_tree[9],
                        merkle_tree[11],
                    ],
                },
            ],
        });
        return test_cases;
    }

    #[test]
    fn merkle_proof() {
        let test_cases: Vec<TestCase> = test_cases();
        for test_case in test_cases {
            for merkle_proof in test_case.merkle_proofs {
                assert_eq!(
                    merkle_proof.leaf().unwrap().as_hash(),
                    merkle_proof.siblings[0].as_hash()
                );
                assert_eq!(merkle_proof.merkle_root().unwrap(), test_case.merkle_root,)
            }
        }
    }

    #[test]
    fn no_sibling_path() {
        let merkle_proof = MerkleProof {
            prefix: vec![],
            siblings: vec![],
        };
        assert!(merkle_proof.leaf().is_err());
        assert!(merkle_proof.merkle_root().is_err());
    }

    #[test]
    fn unmatched_prefix_num() {
        let merkle_proof = MerkleProof {
            prefix: vec![false],
            siblings: vec![
                tx_merkle_node("352e4e0300dcb283d7aae269b49724b8cdcf973e5827d0ab1e7c94cb82264469"),
                tx_merkle_node("2d99961074ab1dfb3a25c8d1543f851563f1807f785219b74f6ab05ee26c0b95"),
                tx_merkle_node("045778e30e7e06bc0f32666b1636f9026ba8b2638e85a7f9345de73bb430ef0a"),
            ],
        };
        assert!(merkle_proof.merkle_root().is_err());
    }
}
