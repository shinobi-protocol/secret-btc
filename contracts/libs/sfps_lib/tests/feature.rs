use prost::Message;
use serde::{Deserialize, Serialize};
use sfps_lib::header_chain::{ChainDB, HeaderChain, ReadonlyChainDB};
use sfps_lib::light_block::header::Header;
use sfps_lib::light_block::LightBlock;
use sfps_lib::merkle::{leaf_hash, MerkleProof};
use sfps_lib::rand;
use sfps_lib::tx_result_proof::{TxResult, TxResultProof};
use std::fs::File;

#[derive(Clone, Deserialize, Serialize, Debug)]
struct ChainData {
    initial_header: Header,
    blocks: Vec<LightBlock>,
    tx_result: TxResult,
}

#[derive(Debug, Default)]
struct MemoryChainDB {
    hashes: Vec<Vec<u8>>,
    max_interval: u64,
}

impl MemoryChainDB {
    pub fn new() -> Self {
        Self {
            hashes: vec![],
            max_interval: 0,
        }
    }
}

impl ReadonlyChainDB for MemoryChainDB {
    fn get_hash_by_index(&self, index: usize) -> Option<Vec<u8>> {
        self.hashes.get(index as usize).map(|hash| hash.clone())
    }
    fn get_highest_hash(&self) -> Option<Vec<u8>> {
        self.hashes.last().map(|hash| hash.clone())
    }
    fn get_hash_list_length(&self) -> usize {
        self.hashes.len()
    }
    fn get_max_interval(&self) -> u64 {
        self.max_interval
    }
}

impl ChainDB for MemoryChainDB {
    type Error = String;
    fn append_header_hash(&mut self, hash: Vec<u8>) -> Result<(), Self::Error> {
        self.hashes.push(hash);
        Ok(())
    }
    fn store_max_interval(&mut self, max_interval: u64) -> Result<(), Self::Error> {
        self.max_interval = max_interval;
        Ok(())
    }
}

#[test]
fn feature_test() {
    // read data from json file
    let file = File::open("tests/testdata.json").unwrap();
    let data: ChainData = serde_json::from_reader(file).unwrap();

    // instantiate header chain object
    let mut header_chain = HeaderChain::new(MemoryChainDB::new());

    // construct header chain object
    let mut rng = rand::thread_rng();
    let mut last_header = data.initial_header.clone();
    header_chain.init(data.initial_header, 10).unwrap();
    for block in data.blocks.clone().into_iter() {
        header_chain
            .add_block_to_highest(&last_header, block.clone(), &mut rng)
            .unwrap();
        last_header = block.signed_header.header;
    }

    // https://secretnodes.com/secret/chains/secret-4/blocks/1000008/transactions/EDA6A86BA2C36AF0B919E448A55755607622E85AC384D8B969F60FFED1BD1D46?format=json
    // verify tx result merkle proof at height 1000008 (last_result_hash at height 1000009)
    let merkle_proof = MerkleProof {
        total: 1,
        index: 0,
        leaf_hash: leaf_hash(&data.tx_result.encode_to_vec()),
        aunts: vec![],
    };
    let tx_result_proof = TxResultProof {
        tx_result: data.tx_result,
        merkle_proof,
        headers: vec![data.blocks.last().unwrap().signed_header.header.clone()],
    };
    header_chain
        .verify_tx_result_proof(&tx_result_proof, 9)
        .unwrap()
}
