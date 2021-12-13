use crate::light_block::header::fields::{BlockId, Timestamp};
use prost::Message;

pub fn signature_message(
    vote_type: i8,
    height: i64,
    round: i32,
    block_id: Option<BlockId>,
    timestamp: Timestamp,
    chain_id: String,
) -> Vec<u8> {
    CanonicalVote::new(vote_type, height, round, block_id, timestamp, chain_id).signature_message()
}

#[derive(prost::Message)]
struct CanonicalVote {
    /// Type of vote (prevote or precommit)
    #[prost(int32, tag = "1")]
    vote_type: i32,

    /// Block height
    #[prost(sfixed64, tag = "2")]
    height: i64,

    /// Round
    #[prost(sfixed64, tag = "3")]
    round: i64,

    /// Block ID
    #[prost(message, optional, tag = "4")]
    block_id: Option<BlockId>,

    /// Timestamp
    #[prost(message, optional, tag = "5")]
    timestamp: Option<Timestamp>,

    /// Chain ID
    #[prost(string, tag = "6")]
    chain_id: String,
}

impl CanonicalVote {
    fn new(
        vote_type: i8,
        height: i64,
        round: i32,
        block_id: Option<BlockId>,
        timestamp: Timestamp,
        chain_id: String,
    ) -> Self {
        Self {
            vote_type: vote_type.into(),
            height: height,
            round: round.into(),
            block_id: block_id,
            timestamp: Some(timestamp),
            chain_id: chain_id,
        }
    }

    fn signature_message(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.encoded_len());
        self.encode_length_delimited(&mut bytes).unwrap();
        bytes
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::light_block::header::fields::PartSetHeader;

    #[test]
    fn test_signature_message() {
        let vote = CanonicalVote::new(
            2,
            2,
            0,
            Some(BlockId {
                hash: hex::decode(
                    "E26068660D03B9A8D00A7EB427AB111D6EDC37C4381C7AF9BC100D0D788BEACB",
                )
                .unwrap(),
                parts: Some(PartSetHeader {
                    total: 1,
                    hash: hex::decode(
                        "525FC749ED31DFB174BC2FAACEC72D8FA725E3B63BB41CBB7605E56C3895C54F",
                    )
                    .unwrap(),
                }),
            }),
            serde_json::from_str(r#""2020-09-15T14:36:42.293798634Z""#).unwrap(),
            "secret-2".to_string(),
        );
        assert_eq!(
            vote.signature_message(),
            [
                109, 8, 2, 17, 2, 0, 0, 0, 0, 0, 0, 0, 34, 72, 10, 32, 226, 96, 104, 102, 13, 3,
                185, 168, 208, 10, 126, 180, 39, 171, 17, 29, 110, 220, 55, 196, 56, 28, 122, 249,
                188, 16, 13, 13, 120, 139, 234, 203, 18, 36, 8, 1, 18, 32, 82, 95, 199, 73, 237,
                49, 223, 177, 116, 188, 47, 170, 206, 199, 45, 143, 167, 37, 227, 182, 59, 180, 28,
                187, 118, 5, 229, 108, 56, 149, 197, 79, 42, 12, 8, 250, 162, 131, 251, 5, 16, 234,
                133, 140, 140, 1, 50, 8, 115, 101, 99, 114, 101, 116, 45, 50
            ]
        );
    }
}
