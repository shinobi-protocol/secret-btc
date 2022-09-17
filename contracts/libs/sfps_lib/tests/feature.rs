use cosmos_proto::prost::Message;
use cosmos_proto::tendermint::abci::ResponseDeliverTx;
use cosmos_proto::tendermint::types::{Header, LightBlock};
use serde::{Deserialize, Serialize};
use sfps_lib::header_chain::{ChainDB, HeaderChain, ReadonlyChainDB};
use sfps_lib::merkle::MerkleProof;
use sfps_lib::response_deliver_tx_proof::encode_response_deliver_tx_as_merkle_leaf;
use sfps_lib::response_deliver_tx_proof::ResponseDeliverTxProof;
use std::fs::File;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ChainData {
    initial_header: String,
    light_blocks: Vec<String>,
    response_deliver_txs_at_last_block: Vec<String>,
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
    fn get_hash_by_index(&mut self, index: usize) -> Option<Vec<u8>> {
        self.hashes.get(index as usize).map(|hash| hash.clone())
    }
    fn get_highest_hash(&mut self) -> Option<Vec<u8>> {
        self.hashes.last().map(|hash| hash.clone())
    }
    fn get_hash_list_length(&mut self) -> usize {
        self.hashes.len()
    }
    fn get_max_interval(&mut self) -> u64 {
        self.max_interval
    }
}

impl ChainDB for MemoryChainDB {
    type Error = String;
    fn append_block_hash(&mut self, hash: Vec<u8>) -> Result<(), Self::Error> {
        self.hashes.push(hash);
        Ok(())
    }
    fn store_max_interval(&mut self, max_interval: u64) -> Result<(), Self::Error> {
        self.max_interval = max_interval;
        Ok(())
    }
}

struct DalekEd25519Verifier();

impl sfps_lib::light_block::Ed25519Verifier<String> for DalekEd25519Verifier {
    fn verify_batch(
        &mut self,
        messages: &[&[u8]],
        signatures: &[&[u8]],
        public_keys: &[&[u8]],
    ) -> Result<(), String> {
        let mut sig = Vec::with_capacity(signatures.len());
        for signature in signatures.iter() {
            sig.push(ed25519_dalek::Signature::from_bytes(signature).unwrap())
        }
        let mut pubkeys = Vec::with_capacity(public_keys.len());
        for public_key in public_keys.iter() {
            pubkeys.push(ed25519_dalek::PublicKey::from_bytes(public_key).unwrap())
        }
        ed25519_dalek::verify_batch(messages, &sig, &pubkeys)
            .map_err(|_| "ed25519 error".to_string())
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
    let initial_header =
        Header::decode(base64::decode(&data.initial_header).unwrap().as_slice()).unwrap();
    let mut last_header = initial_header.clone();
    header_chain.init(initial_header, 10).unwrap();
    let light_blocks: Vec<LightBlock> = data
        .light_blocks
        .iter()
        .map(|block| LightBlock::decode(base64::decode(block).unwrap().as_slice()).unwrap())
        .collect();
    for block in &light_blocks {
        header_chain
            .add_block_to_highest(&last_header, block.clone(), &mut DalekEd25519Verifier {})
            .unwrap();
        last_header = block
            .signed_header
            .as_ref()
            .unwrap()
            .header
            .as_ref()
            .unwrap()
            .clone();
    }

    // https://secretnodes.com/secret/chains/secret-4/light_blocks/1000008/transactions/EDA6A86BA2C36AF0B919E448A55755607622E85AC384D8B969F60FFED1BD1D46?format=json
    // verify tx result merkle proof at height 1000008 (last_result_hash at height 1000009)
    let response_deliver_tx = ResponseDeliverTx::decode(
        base64::decode(&data.response_deliver_txs_at_last_block[0])
            .unwrap()
            .as_slice(),
    )
    .unwrap();
    let merkle_proof = MerkleProof {
        total: 1,
        index: 0,
        leaf: encode_response_deliver_tx_as_merkle_leaf(&response_deliver_tx),
        aunts: vec![],
    };
    let response_deliver_tx_proof = ResponseDeliverTxProof {
        merkle_proof,
        headers: vec![light_blocks
            .last()
            .as_ref()
            .unwrap()
            .signed_header
            .as_ref()
            .unwrap()
            .header
            .as_ref()
            .unwrap()
            .clone()],
    };
    header_chain
        .verify_response_deliver_tx_proof(&response_deliver_tx_proof, 9)
        .unwrap()
}
