use super::*;
use crate::header_chain::chaindb::ChainDBResult;
use bitcoin::blockdata::constants::genesis_block;
use bitcoin::hash_types::{BlockHash, TxMerkleNode};
use bitcoin::BlockHeader;
use bitcoin::Network;
use mock_chain_db::MockChainDB;
use std::convert::TryInto;
use std::str::FromStr;
use std::time;

const NETWORKS: [Network; 3] = [Network::Bitcoin, Network::Testnet, Network::Regtest];

fn block_header(
    version: i32,
    prev_blockhash: &str,
    merkle_root: &str,
    time: u32,
    bits: u32,
    nonce: u32,
) -> BlockHeader {
    BlockHeader {
        version: version,
        prev_blockhash: BlockHash::from_str(&prev_blockhash).unwrap(),
        merkle_root: TxMerkleNode::from_str(&merkle_root).unwrap(),
        time: time,
        bits: bits,
        nonce: nonce,
    }
}

mod mock_chain_db {
    use super::*;
    use std::collections::HashMap;
    #[derive(Debug, PartialEq, Clone)]
    pub struct MockChainDB {
        pub map: HashMap<u32, StoredBlockHeader>,
        pub tip_height: Option<u32>,
    }

    impl MockChainDB {
        pub fn new() -> Self {
            Self {
                map: HashMap::default(),
                tip_height: None,
            }
        }
    }

    impl ReadonlyChainDB for MockChainDB {
        fn header_at(&mut self, height: u32) -> ChainDBResult<Option<StoredBlockHeader>> {
            Ok(self.map.get(&height).cloned())
        }

        fn tip_height(&mut self) -> ChainDBResult<Option<u32>> {
            Ok(self.tip_height)
        }
    }

    impl ChainDB for MockChainDB {
        fn store_header(
            &mut self,
            height: u32,
            block_header: StoredBlockHeader,
        ) -> ChainDBResult<()> {
            if self.tip_height.is_none() || self.tip_height()?.unwrap() < height {
                self.tip_height = Some(height);
            }
            self.map.insert(height, block_header);
            Ok(())
        }
    }
}

mod random_chain {
    use super::*;
    use bitcoin::hash_types::TxMerkleNode;
    use bitcoin::hashes::Hash;
    use rand::thread_rng;
    use rand::Rng;
    pub const HIGH_TARGET: Uint256 = Uint256([
        0xffffffffffffffff,
        0xffffffffffffffff,
        0xffffffffffffffff,
        0x1fffffffffffffff,
    ]);

    fn random_merkle_root() -> TxMerkleNode {
        let bytes: [u8; 32] = thread_rng().gen();
        TxMerkleNode::from_inner(bytes)
    }

    pub fn random_header(prev_blockhash: BlockHash, time: u32, target: Uint256) -> BlockHeader {
        let bits = BlockHeader::compact_target_from_u256(&target);
        let merkle_root = random_merkle_root();
        let mut header = BlockHeader {
            version: 1,
            prev_blockhash,
            merkle_root,
            time,
            bits,
            nonce: 0,
        };
        loop {
            if is_valid_proof_of_work_hash(&target, &header.block_hash()) {
                return header;
            }
            header.nonce += 1;
        }
    }
}

#[test]
fn test_reverse_hash() {
    let hash =
        BlockHash::from_str("89b4f223789e40b5b475af6483bb05bceda54059e17d2053334b358f6bb310ac")
            .unwrap();
    let reversed = reverse_hash(&hash);
    assert_eq!(
        format!("{}", reversed),
        "0x89b4f223789e40b5b475af6483bb05bceda54059e17d2053334b358f6bb310ac"
    );
}

#[test]
fn test_is_valid_proof_of_work_hash() {
    let target = Uint256([
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        0x00000000ffff0000,
    ]);
    assert_eq!(
        target,
        bitcoin::BlockHeader::u256_from_compact_target(486604799)
    );
    assert_eq!(
        format!("{}", target),
        "0x00000000ffff0000000000000000000000000000000000000000000000000000"
    );
    let hash =
        BlockHash::from_str("00000000ffff0000000000000000000000000000000000000000000000000000")
            .unwrap();
    assert_eq!(is_valid_proof_of_work_hash(&target, &hash), true);
    let hash =
        BlockHash::from_str("00000000ffff0000000000000000000000000000000000000000000000000001")
            .unwrap();
    assert_eq!(is_valid_proof_of_work_hash(&target, &hash), false);
    let hash =
        BlockHash::from_str("00000000efffffffffffffffffffffffffffffffffffffffffffffffffffffff")
            .unwrap();
    assert_eq!(is_valid_proof_of_work_hash(&target, &hash), true);
}

#[test]
fn test_validate_work_sanity() {
    // header at 32255 from mainnet
    let header = block_header(
        1,
        "000000006baebaa74cecde6c6787c26ee0a616a3c333261bff36653babdac149",
        "89b4f223789e40b5b475af6483bb05bceda54059e17d2053334b358f6bb310ac",
        1262152739,
        486604799,
        312762301,
    );
    assert!(validate_work(&header).is_ok());

    // header at 340703 from mainnet
    let header = block_header(
        2,
        "00000000000000000fd4c2d64852112cd5d3fce29271fa74660dbf286b280866",
        "c40633ce7b7f4c29cfbe6d2b19686885c289057574e03248dec5f82d19464255",
        1422372768,
        404291887,
        2944237276,
    );
    assert!(validate_work(&header).is_ok());

    // header at 10079 from testnet
    let header = block_header(
        1,
        "00000000130d0a10ac76fe510ad42bf6c88b58b9a190f1d34f8e3ecc3b07abaf",
        "cfce135579249a4e5019bd1739468a7f4c8e5aaa5c4a425b923ecd823fd8e796",
        1343019339,
        473956288,
        1248357270,
    );
    assert!(validate_work(&header).is_ok());

    // header at 10081 from testnet
    let header = block_header(
        1,
        "000000007c4fc01a14af8067762fb807144b1b59cd4ec79ffc61efae3439757d",
        "7e1c39d9d7e9f2c49d014ab1c858adfabfd1ec3a603c50a0b1bc81294b4a408f",
        1338181274,
        486604799,
        1603192096,
    );
    assert!(validate_work(&header).is_ok());
}

#[test]
fn test_validate_work_insufficient_work() {
    let invalid_nonce = 1;
    // header at 32255 from mainnet
    let header = block_header(
        1,
        "000000006baebaa74cecde6c6787c26ee0a616a3c333261bff36653babdac149",
        "89b4f223789e40b5b475af6483bb05bceda54059e17d2053334b358f6bb310ac",
        1262152739,
        486604799,
        invalid_nonce,
    );
    assert!(validate_work(&header).is_err());

    // header at 340703 from mainnet
    let header = block_header(
        2,
        "00000000000000000fd4c2d64852112cd5d3fce29271fa74660dbf286b280866",
        "c40633ce7b7f4c29cfbe6d2b19686885c289057574e03248dec5f82d19464255",
        1422372768,
        404291887,
        invalid_nonce,
    );
    assert!(validate_work(&header).is_err());

    // header at 10079 from testnet
    let header = block_header(
        1,
        "00000000130d0a10ac76fe510ad42bf6c88b58b9a190f1d34f8e3ecc3b07abaf",
        "cfce135579249a4e5019bd1739468a7f4c8e5aaa5c4a425b923ecd823fd8e796",
        1343019339,
        473956288,
        invalid_nonce,
    );
    assert!(validate_work(&header).is_err());

    // header at 10081 from testnet
    let header = block_header(
        1,
        "000000007c4fc01a14af8067762fb807144b1b59cd4ec79ffc61efae3439757d",
        "7e1c39d9d7e9f2c49d014ab1c858adfabfd1ec3a603c50a0b1bc81294b4a408f",
        1338181274,
        486604799,
        invalid_nonce,
    );
    assert!(validate_work(&header).is_err());
}

#[test]
fn test_required_target_increase_difficulty() {
    // header at 30240
    let start_header = block_header(
        1,
        "000000005107662c86452e7365f32f8ffdc70d8d87aa6f78630a79f7d77fbfe6",
        "5533b5a4273dbe0c1972339148046f0197188a3515b6ff4d83b6c7652f340d70",
        1261130161,
        486604799,
        421900479,
    );
    // header at 32255
    let end_header = block_header(
        1,
        "000000006baebaa74cecde6c6787c26ee0a616a3c333261bff36653babdac149",
        "89b4f223789e40b5b475af6483bb05bceda54059e17d2053334b358f6bb310ac",
        1262152739,
        486604799,
        312762301,
    );
    // header at 32256
    let next_header = block_header(
        1,
        "00000000984f962134a7291e3693075ae03e521f0ee33378ec30a334d860034b",
        "64b5e5f5a262f47af443a0120609206a3305877693edfe03e994f20a024ab627",
        1262153464,
        486594666,
        121087187,
    );
    let mut db = MockChainDB::new();
    db.store_header(
        30240,
        StoredBlockHeader {
            header: start_header.clone(),
            work: Uint256::default(),
        },
    )
    .unwrap();
    let mut header_chain = HeaderChain::new(db, Network::Bitcoin);
    let target = header_chain
        .required_target(32255, &end_header, &next_header)
        .unwrap();
    assert_eq!(target, next_header.target());
}

#[test]
fn test_required_target_decrease_difficulty() {
    // header at 338688
    let start_header = block_header(
        2,
        "0000000000000000031b14ece1cfda0e23774e473cd2676834f73155e4f46a2b",
        "d99d387f05dfc062b35bdd0e70c21e1602618f40f78776e7160db3705b2bf60c",
        1421084073,
        404291887,
        4101859333,
    );
    // header at 340703
    let end_header = block_header(
        2,
        "00000000000000000fd4c2d64852112cd5d3fce29271fa74660dbf286b280866",
        "c40633ce7b7f4c29cfbe6d2b19686885c289057574e03248dec5f82d19464255",
        1422372768,
        404291887,
        2944237276,
    );
    // header at 340704
    let next_header = block_header(
        2,
        "000000000000000010bfa427c8d305d861ab5ee4776d87d6d911f5fb3045c754",
        "4dcc31ecba5dcbb6e9f5e6fc966e1c61101772a9150b84dc20bbd1e326be6485",
        1422372946,
        404399040,
        1717099203,
    );

    let mut db = MockChainDB::new();
    db.store_header(
        338688,
        StoredBlockHeader {
            header: start_header.clone(),
            work: Uint256::default(),
        },
    )
    .unwrap();
    let mut header_chain = HeaderChain::new(db, Network::Bitcoin);
    let target = header_chain
        .required_target(340703, &end_header, &next_header)
        .unwrap();
    assert_eq!(target, next_header.target());
}

#[test]
fn test_required_target_testnet_reset_difficulty() {
    let prev_header = block_header(
        1,
        "0000000000000000000000000000000000000000000000000000000000000000",
        "0000000000000000000000000000000000000000000000000000000000000000",
        1000,
        0,
        0,
    );

    let next_header = block_header(
        1,
        "0000000000000000000000000000000000000000000000000000000000000000",
        "0000000000000000000000000000000000000000000000000000000000000000",
        2201,
        0,
        0,
    );

    let mut db = MockChainDB::new();
    db.store_header(
        0,
        StoredBlockHeader {
            header: prev_header,
            work: Uint256::default(),
        },
    )
    .unwrap();

    let mut header_chain = HeaderChain::new(db, Network::Testnet);
    let target = header_chain
        .required_target(0, &prev_header, &next_header)
        .unwrap();
    assert_eq!(target, max_target());
}

#[test]
fn test_required_target_testnet_scan_back() {
    let max_target = max_target();
    let scan_backed_header = block_header(
        1,
        "0000000000000000000000000000000000000000000000000000000000000000",
        "0000000000000000000000000000000000000000000000000000000000000000",
        0,
        404399040,
        0,
    );

    let max_target_header = block_header(
        1,
        "0000000000000000000000000000000000000000000000000000000000000000",
        "0000000000000000000000000000000000000000000000000000000000000000",
        0,
        BlockHeader::compact_target_from_u256(&max_target),
        0,
    );

    let next_header = block_header(
        1,
        "0000000000000000000000000000000000000000000000000000000000000000",
        "0000000000000000000000000000000000000000000000000000000000000000",
        0,
        0,
        0,
    );

    let mut db = MockChainDB::new();
    db.store_header(
        0,
        StoredBlockHeader {
            header: scan_backed_header,
            work: Uint256::default(),
        },
    )
    .unwrap();

    db.store_header(
        1,
        StoredBlockHeader {
            header: max_target_header,
            work: Uint256::default(),
        },
    )
    .unwrap();

    let mut header_chain = HeaderChain::new(db, Network::Testnet);
    let target = header_chain
        .required_target(1, &max_target_header, &next_header)
        .unwrap();
    assert_eq!(target, BlockHeader::u256_from_compact_target(404399040));
}

#[test]
fn test_required_target_testnet_scan_back_max_target() {
    let max_target = max_target();
    let older_header = block_header(
        1,
        "0000000000000000000000000000000000000000000000000000000000000000",
        "0000000000000000000000000000000000000000000000000000000000000000",
        0,
        404399040,
        0,
    );
    let max_target_header = block_header(
        1,
        "0000000000000000000000000000000000000000000000000000000000000000",
        "0000000000000000000000000000000000000000000000000000000000000000",
        0,
        BlockHeader::compact_target_from_u256(&max_target),
        0,
    );

    let next_header = block_header(
        1,
        "0000000000000000000000000000000000000000000000000000000000000000",
        "0000000000000000000000000000000000000000000000000000000000000000",
        0,
        0,
        0,
    );

    let mut db = MockChainDB::new();
    db.store_header(
        2015,
        StoredBlockHeader {
            header: older_header,
            work: Uint256::default(),
        },
    )
    .unwrap();

    db.store_header(
        2016,
        StoredBlockHeader {
            header: max_target_header,
            work: Uint256::default(),
        },
    )
    .unwrap();

    let mut header_chain = HeaderChain::new(db, Network::Testnet);
    let target = header_chain
        .required_target(2016, &max_target_header, &next_header)
        .unwrap();
    assert_eq!(target, max_target);
}

#[test]
fn test_init_to_genesis_sanity() {
    for network in NETWORKS.iter() {
        let db = MockChainDB::new();
        let mut header_chain = HeaderChain::new(db, *network);
        header_chain.init_to_genesis().unwrap();
        let genesis_header = genesis_block(*network).header;
        let genesis_hash = genesis_header.block_hash();
        assert_eq!(
            header_chain.tip().unwrap().unwrap().header.block_hash(),
            genesis_hash
        );
        assert_eq!(
            header_chain.header_at(0).unwrap().unwrap(),
            StoredBlockHeader {
                header: genesis_header,
                work: genesis_header.work()
            }
        );
    }
}

#[test]
fn test_init_to_genesis_after_initialized() {
    for network in NETWORKS.iter() {
        let db = MockChainDB::new();
        let mut header_chain = HeaderChain::new(db, *network);
        header_chain.init_to_genesis().unwrap();
        assert_eq!(
            header_chain.init_to_genesis().unwrap_err(),
            Error::AlreadyInitialized
        );
    }
}

#[test]
fn store_headers_sanity() {
    let network = Network::Bitcoin;
    let db = MockChainDB::new();
    let mut header_chain = HeaderChain::new(db, network);
    header_chain.init_to_genesis().unwrap();

    // block 1 in mainnet
    let next_block_header = BlockHeader {
        version: 1,
        prev_blockhash: genesis_block(network).header.block_hash(),
        merkle_root: TxMerkleNode::from_str(
            "0e3e2357e806b6cdb1f70b54c3a3a17b6714ee1f0e68bebb44a74b1efd512098",
        )
        .unwrap(),
        time: 1231469665,
        bits: 0x1d00ffff,
        nonce: 2573394689,
    };
    let genesis = header_chain.tip().unwrap().unwrap();
    header_chain
        .store_headers(1, vec![next_block_header], next_block_header.time)
        .unwrap();
    let header_tip = header_chain.tip().unwrap().unwrap();
    assert_eq!(header_tip.header, next_block_header);
    assert_eq!(header_tip.work, Uint256::from_u64(0x200020002).unwrap());
    assert_eq!(header_chain.header_at(0).unwrap().unwrap(), genesis);
    assert_eq!(header_chain.header_at(1).unwrap().unwrap(), header_tip);
}

#[test]
fn store_headers_repeating() {
    let network = Network::Bitcoin;
    let db = MockChainDB::new();
    let mut header_chain = HeaderChain::new(db, network);
    header_chain.init_to_genesis().unwrap();

    // block 1 in mainnet
    let next_block_header = BlockHeader {
        version: 1,
        prev_blockhash: genesis_block(network).header.block_hash(),
        merkle_root: TxMerkleNode::from_str(
            "0e3e2357e806b6cdb1f70b54c3a3a17b6714ee1f0e68bebb44a74b1efd512098",
        )
        .unwrap(),
        time: 1231469665,
        bits: 0x1d00ffff,
        nonce: 2573394689,
    };
    for _ in 1..100 {
        header_chain
            .store_headers(1, vec![next_block_header.clone()], next_block_header.time)
            .unwrap();
        let header_tip = header_chain.tip().unwrap().unwrap();
        assert_eq!(header_tip.header, next_block_header);
        assert_eq!(header_tip.work, Uint256::from_u64(0x200020002).unwrap());
    }
}

#[test]
fn store_headers_bad_proof_of_work() {
    let network = Network::Bitcoin;
    let db = MockChainDB::new();
    let mut header_chain = HeaderChain::new(db, network);
    header_chain.init_to_genesis().unwrap();
    let correct_block_header = BlockHeader {
        version: 1,
        prev_blockhash: genesis_block(network).header.block_hash(),
        merkle_root: TxMerkleNode::from_str(
            "0e3e2357e806b6cdb1f70b54c3a3a17b6714ee1f0e68bebb44a74b1efd512098",
        )
        .unwrap(),
        time: 1231469665,
        bits: 0x1d00ffff,
        nonce: 2573394689,
    };

    let mut invalid_hash_version = correct_block_header.clone();
    invalid_hash_version.version = 2;
    let mut invalid_hash_prev_block_hash = correct_block_header.clone();
    invalid_hash_prev_block_hash.prev_blockhash =
        BlockHash::from_str("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26e")
            .unwrap();
    let mut invalid_hash_merkle_root = correct_block_header.clone();
    invalid_hash_merkle_root.merkle_root =
        TxMerkleNode::from_str("0e3e2357e806b6cdb1f70b54c3a3a17b6714ee1f0e68bebb44a74b1efd512097")
            .unwrap();
    let mut invalid_hash_time = correct_block_header.clone();
    invalid_hash_time.time = 1231469664;
    let mut invalid_hash_bits = correct_block_header.clone();
    invalid_hash_bits.bits = 0x1d00fffe;
    let mut invalid_hash_nonce = correct_block_header.clone();
    invalid_hash_nonce.nonce = 2753394688;

    assert_eq!(
        header_chain
            .store_headers(1, vec![invalid_hash_version], 1231469665)
            .unwrap_err(),
        Error::BadProofOfWork
    );
    assert_eq!(
        header_chain
            .store_headers(1, vec![invalid_hash_prev_block_hash], 1231469665)
            .unwrap_err(),
        Error::UnconnectedHeader
    );
    assert_eq!(
        header_chain
            .store_headers(1, vec![invalid_hash_merkle_root], 1231469665)
            .unwrap_err(),
        Error::BadProofOfWork
    );
    assert_eq!(
        header_chain
            .store_headers(1, vec![invalid_hash_time], 1231469665)
            .unwrap_err(),
        Error::BadProofOfWork
    );
    assert_eq!(
        header_chain
            .store_headers(1, vec![invalid_hash_bits], 1231469665)
            .unwrap_err(),
        Error::InvalidTarget
    );
    assert_eq!(
        header_chain
            .store_headers(1, vec![invalid_hash_nonce], 1231469665)
            .unwrap_err(),
        Error::BadProofOfWork
    );
    header_chain
        .store_headers(1, vec![correct_block_header], 1231469665)
        .unwrap();
}

#[test]
fn test_extend_tip() {
    use random_chain::*;

    // Create Random Header Chain
    const BLOCK_TIMESPAN: u32 = 600;
    let genesis = random_header(BlockHash::default(), 1231469665, HIGH_TARGET);
    let mut headers = vec![];
    let mut prev_header = genesis.clone();
    for _ in 0..100 {
        let header = random_header(
            prev_header.block_hash(),
            prev_header.time + BLOCK_TIMESPAN,
            HIGH_TARGET,
        );
        headers.push(header.clone());
        prev_header = header;
    }

    let db = MockChainDB::new();
    let mut header_chain = HeaderChain::new(db, Network::Regtest);

    // init genesis
    header_chain
        .init_to_header(0, genesis, genesis.time)
        .unwrap();

    // add headers
    for (i, header) in headers.iter().enumerate() {
        header_chain
            .store_headers((i + 1) as u32, vec![*header], header.time)
            .unwrap();
    }

    // assert headers in chain
    for (i, header) in headers.iter().enumerate() {
        let stored_header = header_chain.header_at((i + 1) as u32).unwrap().unwrap();
        // assert work
        assert_eq!(
            stored_header.work,
            Uint256::from_u64((8 * (i + 2)) as u64).unwrap()
        );
        // assert header
        assert_eq!(stored_header.header, *header);
    }

    // assert tip is last added header
    assert_eq!(
        header_chain.tip().unwrap().unwrap().header,
        *headers.last().unwrap()
    );
}

#[test]
fn test_extend_tip_bulk() {
    use random_chain::*;

    // Create Random Header Chain
    const BLOCK_TIMESPAN: u32 = 600;
    let genesis = random_header(BlockHash::default(), 1231469665, HIGH_TARGET);
    let mut headers = vec![];
    let mut prev_header = genesis.clone();
    for _ in 0..100 {
        let header = random_header(
            prev_header.block_hash(),
            prev_header.time + BLOCK_TIMESPAN,
            HIGH_TARGET,
        );
        headers.push(header.clone());
        prev_header = header;
    }

    let db = MockChainDB::new();
    let mut header_chain = HeaderChain::new(db, Network::Regtest);

    // init genesis
    header_chain
        .init_to_header(0, genesis, genesis.time)
        .unwrap();

    // add headers
    header_chain
        .store_headers(100 as u32, headers.clone(), headers.last().unwrap().time)
        .unwrap();

    // assert headers in chain
    for (i, header) in headers.iter().enumerate() {
        let stored_header = header_chain.header_at((i + 1) as u32).unwrap().unwrap();
        // assert work
        assert_eq!(
            stored_header.work,
            Uint256::from_u64((8 * (i + 2)) as u64).unwrap()
        );
        // assert header
        assert_eq!(stored_header.header, *header);
    }

    // assert tip is last added header
    assert_eq!(
        header_chain.tip().unwrap().unwrap().header,
        *headers.last().unwrap()
    );
}

#[test]
fn test_replace_with_larger_work_chain() {
    use random_chain::*;
    // Create Random Header Chain
    const BLOCK_TIMESPAN: u32 = 600;
    let genesis = random_header(BlockHash::default(), 1231469665, HIGH_TARGET);
    let mut header_chain = HeaderChain::new(MockChainDB::new(), Network::Regtest);
    header_chain.init_to_header(0, genesis, 1231469665).unwrap();
    let mut first_chain = vec![];
    let mut prev_header = genesis.clone();
    // 6 blocks after genesis
    // tip height 6
    for _ in 0..6 {
        let header = random_header(
            prev_header.block_hash(),
            prev_header.time + BLOCK_TIMESPAN,
            HIGH_TARGET,
        );
        first_chain.push(header.clone());
        prev_header = header;
    }
    header_chain
        .store_headers(6, first_chain.clone(), 1231469665 + 6 * BLOCK_TIMESPAN)
        .unwrap();
    let mut expected_work = genesis.work();
    for height in 1..7 {
        let stored_header = header_chain.header_at(height).unwrap().unwrap();
        expected_work = expected_work + first_chain[height as usize - 1].work();
        assert_eq!(
            stored_header,
            StoredBlockHeader {
                header: first_chain[height as usize - 1],
                work: expected_work
            }
        );
    }
    // fork at height 6
    // each block has same parent, block at height 5
    // tip height 7
    let mut replace_chain = vec![];
    prev_header = first_chain[4];
    for _ in 0..2 {
        let header = random_header(
            prev_header.block_hash(),
            prev_header.time + BLOCK_TIMESPAN,
            HIGH_TARGET,
        );
        replace_chain.push(header.clone());
        prev_header = header;
    }
    header_chain
        .store_headers(7, replace_chain.clone(), 1231469665 + 7 * BLOCK_TIMESPAN)
        .unwrap();
    let mut expected_work = genesis.work();
    for height in 1..6 {
        let stored_header = header_chain.header_at(height).unwrap().unwrap();
        expected_work = expected_work + first_chain[height as usize - 1].work();
        assert_eq!(
            stored_header,
            StoredBlockHeader {
                header: first_chain[height as usize - 1],
                work: expected_work
            }
        );
    }
    for height in 6..8 {
        let stored_header = header_chain.header_at(height).unwrap().unwrap();
        expected_work = expected_work + replace_chain[height as usize - 6].work();
        assert_eq!(
            stored_header,
            StoredBlockHeader {
                header: replace_chain[height as usize - 6],
                work: expected_work
            }
        );
    }
}

#[test]
fn test_try_to_replace_with_same_length_larger_work_chain() {
    use random_chain::*;
    let genesis = random_header(BlockHash::default(), 1231469665, HIGH_TARGET);
    let mut db = MockChainDB::new();
    db.store_header(
        0,
        StoredBlockHeader {
            header: genesis,
            work: genesis.work() - Uint256::from_u64(1).unwrap(),
        },
    )
    .unwrap();
    // set network as bitcoin for retargetting difficulty
    let mut header_chain = HeaderChain::new(db, Network::Bitcoin);
    let higher_work_header = random_header(genesis.block_hash(), 1231469665, HIGH_TARGET);
    header_chain
        .store_headers(1, vec![higher_work_header], 1231469665)
        .unwrap();
}

#[test]
fn test_try_to_replace_with_shorter_length_chain() {
    use random_chain::*;
    // Create Random Header Chain
    const BLOCK_TIMESPAN: u32 = 600;
    let genesis = random_header(BlockHash::default(), 1231469665, HIGH_TARGET);
    let mut header_chain = HeaderChain::new(MockChainDB::new(), Network::Regtest);
    header_chain.init_to_header(0, genesis, 1231469665).unwrap();
    let mut first_chain = vec![];
    let mut prev_header = genesis.clone();
    // 6 blocks after genesis
    // tip height 6
    for _ in 0..6 {
        let header = random_header(
            prev_header.block_hash(),
            prev_header.time + BLOCK_TIMESPAN,
            HIGH_TARGET,
        );
        first_chain.push(header.clone());
        prev_header = header;
    }
    header_chain
        .store_headers(6, first_chain.clone(), 1231469665 + 6 * BLOCK_TIMESPAN)
        .unwrap();
    let mut expected_work = genesis.work();
    for height in 1..7 {
        let stored_header = header_chain.header_at(height).unwrap().unwrap();
        expected_work = expected_work + first_chain[height as usize - 1].work();
        assert_eq!(
            stored_header,
            StoredBlockHeader {
                header: first_chain[height as usize - 1],
                work: expected_work
            }
        );
    }
    // fork at height 3
    // each block has same parent, block at height 2
    // tip height 5
    let mut replace_chain = vec![];
    prev_header = first_chain[2];
    for _ in 0..2 {
        let header = random_header(
            prev_header.block_hash(),
            prev_header.time + BLOCK_TIMESPAN,
            HIGH_TARGET,
        );
        replace_chain.push(header.clone());
        prev_header = header;
    }
    // fail to replace
    let err = header_chain
        .store_headers(5, replace_chain.clone(), 1231469665 + 5 * BLOCK_TIMESPAN)
        .unwrap_err();
    assert_eq!(err, Error::InvalidTipHeight);
}

#[test]
fn test_testnet_blockchain() {
    // height 2014704~2014707
    let initial_block_height = 2104704;

    let initial_block_header = block_header(
        536870916,
        "000000000000002077102f09a5563275c1efdbea9f6395f5146f1d6037970d7b",
        "fb7266e26225974ad70668067ee92613aa0256bd48ec58d02a1b0badc00ed2e7",
        1637391579,
        423027356,
        2073682798,
    );
    let blocks = vec![
        block_header(
            536870912,
            "0000000000000025ed3e9ee35a5d6c37509083df563c4e1ebe1857ddb361e3bb",
            "1a3187fa39c94ee54f20fb91bbabcec971ed04e5f0be56ecdad9e32cb0ca718f",
            1637391953,
            423027356,
            3111837443,
        ),
        block_header(
            549453824,
            "000000000000001efd6ddbc884149a03b77aa834d03eea39159d210a0eebe20e",
            "b0189c2ba082b1f95cc8847a2ba77edbca840e8dabc9db80070823274be3ce85",
            1637393155, // timestamp: more than 1200 seconds later from prev block
            486604799,  // bits: max target
            3993466129,
        ),
        block_header(
            547356672,
            "0000000000b025ecdcc28258a5d180867d6bf249d9dadeab8f0e95b2fa2b7fb3",
            "35619bb5e5d1757fe60402d6ab91b08ae62d5822ee0a0d6a04782d52ea02453c",
            1637393571, // timestamp: less than 1200 seconds later from prev block
            423027356,  // return to calculated target
            1845517584,
        ),
    ];

    let now = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .try_into()
        .unwrap();

    // store blocks one by one
    let mut header_chain = HeaderChain::new(MockChainDB::new(), Network::Testnet);
    header_chain
        .init_to_header(initial_block_height, initial_block_header, now)
        .unwrap();
    let mut tip_height = initial_block_height + 1;
    for block in blocks.clone() {
        header_chain
            .store_headers(tip_height, vec![block], now)
            .unwrap();
        tip_height += 1;
    }

    // bulk store blocks
    let mut header_chain = HeaderChain::new(MockChainDB::new(), Network::Testnet);
    header_chain
        .init_to_header(initial_block_height, initial_block_header.clone(), now)
        .unwrap();
    let tip_height = 2104707;
    header_chain.store_headers(tip_height, blocks, now).unwrap();
}
