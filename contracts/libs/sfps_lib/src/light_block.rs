use crate::signed_header::validate_signed_header;
use crate::validator_set::bytes_of_pub_key;
use crate::validator_set::hash_validator_set;
use crate::validator_set::total_voting_power;
use crate::vote::Vote;
use cosmos_proto::tendermint::types::LightBlock;
use std::convert::TryInto;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidValidatorsHash {
        hash_of_validators: Vec<u8>,
        validators_hash_in_header: Vec<u8>,
    },
    NoSignedHeader,
    DecodeSignature(),
    NoValidatorSet,
    NoValidatorPubKey,
    NotEnoughVotingPowerSigned,
    Vote(crate::vote::Error),
    SignedHeader(crate::signed_header::Error),
    Ed25519Verifier(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidValidatorsHash {
                hash_of_validators,
                validators_hash_in_header,
            } => write!(
                f,
                "invalid validators: hash ov validators {}, validators_hash in header {}",
                hex::encode(&hash_of_validators),
                hex::encode(&validators_hash_in_header)
            ),
            Error::NoSignedHeader => f.write_str("no signed header"),
            Error::DecodeSignature() => write!(f, "failed to decode signature error"),
            Error::NoValidatorSet => f.write_str("no validator set"),
            Error::NoValidatorPubKey => f.write_str("no validator pub key"),
            //Error::Base64(e) => write!(f, "base64 error: {}", e),
            Error::NotEnoughVotingPowerSigned => f.write_str("not enough voting power signed"),
            //Error::VerifyBatchFailed => f.write_str("verify batch failed"),
            Error::Vote(e) => write!(f, "vote error {}", e),
            Error::SignedHeader(e) => write!(f, "signed header error {}", e),
            Error::Ed25519Verifier(str) => write!(f, "ed25519 verifier error {}", str),
        }
    }
}

impl From<crate::vote::Error> for Error {
    fn from(e: crate::vote::Error) -> Self {
        Self::Vote(e)
    }
}

impl From<crate::signed_header::Error> for Error {
    fn from(e: crate::signed_header::Error) -> Self {
        Self::SignedHeader(e)
    }
}

pub trait Ed25519Verifier<S: ToString> {
    fn verify_batch(
        &mut self,
        messages: &[&[u8]],
        signatures: &[&[u8]],
        public_keys: &[&[u8]],
    ) -> Result<(), S>;
}

pub fn validate_light_block(light_block: &LightBlock) -> Result<(), Error> {
    let signed_header = light_block
        .signed_header
        .as_ref()
        .ok_or_else(|| Error::NoSignedHeader)?;
    validate_signed_header(signed_header)?;
    let header = signed_header.header.as_ref().unwrap();
    let validator_set = light_block
        .validator_set
        .as_ref()
        .ok_or_else(|| Error::NoValidatorSet)?;
    let hash_of_validators: Vec<u8> = hash_validator_set(validator_set);
    if hash_of_validators == header.validators_hash {
        Ok(())
    } else {
        Err(Error::InvalidValidatorsHash {
            hash_of_validators: hash_of_validators,
            validators_hash_in_header: header.validators_hash.clone(),
        })
    }
}

//https://github.com/tendermint/tendermint/blob/5e52a6ec558f789b642a231c257f8754b97637bc/types/validator_set.go#L636
pub fn verify_light_block<S: ToString, E: Ed25519Verifier<S>>(
    light_block: &LightBlock,
    ed25519_verifier: &mut E,
) -> Result<(), Error> {
    if verified_voting_power(light_block, ed25519_verifier)?
        <= total_voting_power(light_block.validator_set.as_ref().unwrap()) * 2 / 3
    {
        return Err(Error::NotEnoughVotingPowerSigned);
    }
    Ok(())
}

fn verified_voting_power<S: ToString, E: Ed25519Verifier<S>>(
    light_block: &LightBlock,
    ed25519_verifier: &mut E,
) -> Result<i64, Error> {
    let header = light_block
        .signed_header
        .as_ref()
        .unwrap()
        .header
        .as_ref()
        .unwrap();
    let commit = light_block
        .signed_header
        .as_ref()
        .unwrap()
        .commit
        .as_ref()
        .unwrap();
    let block_id = commit.block_id.as_ref().unwrap();
    let mut voting_power = 0;
    let mut messages = Vec::with_capacity(commit.signatures.len());
    let mut signatures = Vec::with_capacity(commit.signatures.len());
    let mut public_keys = Vec::with_capacity(commit.signatures.len());
    for (index, commit_sig) in commit.signatures.iter().enumerate() {
        let (is_commit, signature_message, signature) = match commit_sig.try_into()? {
            Vote::Absent => continue,
            Vote::Commit(vote) => (
                true,
                vote.signature_message(
                    commit.height,
                    commit.round.into(),
                    block_id.clone(),
                    header.chain_id.clone(),
                ),
                vote.signature,
            ),
            Vote::Nil(vote) => (
                false,
                vote.signature_message(commit.height, commit.round.into(), header.chain_id.clone()),
                vote.signature,
            ),
        };
        // validators and commit have a 1-to-1 correspondance.
        // This means we don't need the validator address or to do any lookup.
        // get validator info from validator set
        let validator_info = light_block
            .validator_set
            .as_ref()
            .unwrap()
            .validators
            .get(index)
            .unwrap();
        messages.push(signature_message);
        signatures.push(signature);
        public_keys.push(bytes_of_pub_key(
            validator_info
                .pub_key
                .as_ref()
                .ok_or_else(|| Error::NoValidatorPubKey)?,
        ));
        if is_commit {
            voting_power += validator_info.voting_power;
        }
    }
    let mut msg = Vec::with_capacity(messages.len());
    for message in messages.iter() {
        msg.push(message.as_slice());
    }
    let mut sig = Vec::with_capacity(signatures.len());
    for signature in signatures.iter() {
        sig.push(signature.as_slice());
    }
    ed25519_verifier
        .verify_batch(msg.as_slice(), sig.as_slice(), public_keys.as_slice())
        .map_err(|str| Error::Ed25519Verifier(str.to_string()))?;
    Ok(voting_power)
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmos_proto::prost::Message;

    struct DalekEd25519Verifier();

    impl Ed25519Verifier<String> for DalekEd25519Verifier {
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
                .map_err(|_| "verify batch failed".to_string())
        }
    }

    const LIGHT_BLOCK_BINARY: &str = "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";

    #[test]
    fn test_veirfy_sanity() {
        let light_block: LightBlock =
            LightBlock::decode(base64::decode(LIGHT_BLOCK_BINARY).unwrap().as_slice()).unwrap();
        validate_light_block(&light_block).unwrap();
        verify_light_block(&light_block, &mut DalekEd25519Verifier {}).unwrap();
    }

    /*
    #[test]
    fn test_veirfy_invalid_block_hash() {
        let mut light_block: LightBlock = serde_json::from_str(LIGHT_BLOCK_JSON).unwrap();
        light_block.signed_header.commit.block_id.hash = vec![0, 1, 2];
        assert_eq!(
            light_block
                .verify(&mut DalekEd25519Verifier {})
                .unwrap_err(),
            Error::SignedHeader(signed_header::Error::InvalidHeaderHash {
                commit: light_block.signed_header.commit.block_id.hash,
                header: light_block.signed_header.header.hash().into()
            })
        )
    }

    #[test]
    fn test_veirfy_commit_for_different_hash() {
        let mut light_block: LightBlock = serde_json::from_str(LIGHT_BLOCK_JSON).unwrap();
        light_block.signed_header.commit.height += 1;
        assert_eq!(
            light_block
                .verify(&mut DalekEd25519Verifier {})
                .unwrap_err(),
            Error::SignedHeader(signed_header::Error::InvalidHeight {
                commit: light_block.signed_header.commit.height,
                header: light_block.signed_header.header.height
            })
        )
    }

    #[test]
    fn test_veirfy_commit_for_different_height() {
        let mut light_block: LightBlock = serde_json::from_str(LIGHT_BLOCK_JSON).unwrap();
        light_block.signed_header.commit.height += 1;
        assert_eq!(
            light_block
                .verify(&mut DalekEd25519Verifier {})
                .unwrap_err(),
            Error::SignedHeader(signed_header::Error::InvalidHeight {
                commit: light_block.signed_header.commit.height,
                header: light_block.signed_header.header.height
            })
        )
    }

    #[test]
    fn test_veirfy_invalid_validators() {
        let mut light_block: LightBlock = serde_json::from_str(LIGHT_BLOCK_JSON).unwrap();
        light_block.validators.0.pop();
        assert_eq!(
            light_block
                .verify(&mut DalekEd25519Verifier {})
                .unwrap_err(),
            Error::InvalidValidatorsHash {
                hash_of_validators: light_block.validators.hash(),
                validators_hash_in_header: light_block.signed_header.header.validators_hash
            }
        )
    }

    #[test]
    fn test_veirfy_signature_for_different_block_hash() {
        let mut light_block: LightBlock = serde_json::from_str(LIGHT_BLOCK_JSON).unwrap();
        // change app hash for change header hash
        light_block.signed_header.header.app_hash = vec![0, 1, 2];
        light_block.signed_header.commit.block_id.hash = light_block.signed_header.header.hash();
        assert_eq!(
            light_block
                .verify(&mut DalekEd25519Verifier {})
                .unwrap_err(),
            Error::VerifyBatchFailed
        )
    }

    #[test]
    fn test_veirfy_invalid_signature_commit_sig_as_nil() {
        let mut light_block: LightBlock = serde_json::from_str(LIGHT_BLOCK_JSON).unwrap();
        // change app hash for change header hash
        let sig = match light_block.signed_header.commit.signatures.pop().unwrap() {
            Vote::Commit(vote) => Vote::Nil(NilVote {
                validator_address: vote.validator_address,
                timestamp: vote.timestamp,
                signature: vote.signature,
            }),
            _ => panic!("unexpected"),
        };
        light_block.signed_header.commit.signatures.push(sig);
        assert_eq!(
            light_block
                .verify(&mut DalekEd25519Verifier {})
                .unwrap_err(),
            Error::VerifyBatchFailed
        )
    }

    #[test]
    fn test_veirfy_not_enough_voting_power() {
        let mut light_block: LightBlock = serde_json::from_str(LIGHT_BLOCK_JSON).unwrap();
        // total voting power = 91044884
        // make firls 20 signatures absent
        // validator power = 38054128
        for i in 0..20 {
            light_block.signed_header.commit.signatures[i] = Vote::Absent;
        }
        assert_eq!(
            light_block
                .verify(&mut DalekEd25519Verifier {})
                .unwrap_err(),
            Error::NotEnoughVotingPowerSigned
        )
    }

    #[test]
    fn test_veirfy_empty_commit() {
        let mut light_block: LightBlock = serde_json::from_str(LIGHT_BLOCK_JSON).unwrap();
        light_block.signed_header.commit.signatures = Vec::default();
        assert_eq!(
            light_block
                .verify(&mut DalekEd25519Verifier {})
                .unwrap_err(),
            Error::NotEnoughVotingPowerSigned
        )
    }
    */
}
