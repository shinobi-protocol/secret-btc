pub mod header;
pub mod signed_header;
pub mod validators;

use serde::{Deserialize, Serialize};
use signed_header::SignedHeader;
use signed_header::Vote;
use std::fmt;
use validators::Validators;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidValidatorsHash {
        hash_of_validators: Vec<u8>,
        validators_hash_in_header: Vec<u8>,
    },
    SignedHeader(signed_header::Error),
    NotEnoughVotingPowerSigned,
    VerifyBatchFailed,
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
            Error::SignedHeader(e) => write!(f, "signed header error: {}", e),
            Error::NotEnoughVotingPowerSigned => f.write_str("not enough voting power signed"),
            Error::VerifyBatchFailed => f.write_str("verify batch failed"),
        }
    }
}

impl From<signed_header::Error> for Error {
    fn from(e: signed_header::Error) -> Self {
        Self::SignedHeader(e)
    }
}

pub trait Ed25519Verifier {
    fn verify_batch(
        &mut self,
        messages: &[&[u8]],
        signatures: &[&[u8]],
        public_keys: &[&[u8]],
    ) -> Result<(), Error>;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct LightBlock {
    pub signed_header: SignedHeader,
    pub validators: Validators,
}

impl LightBlock {
    pub fn verify<E: Ed25519Verifier>(&self, ed25519_verifier: &mut E) -> Result<(), Error> {
        self.validate_basic()?;
        self.verify_commit(ed25519_verifier)
    }

    fn validate_basic(&self) -> Result<(), Error> {
        self.signed_header.validate_basic()?;
        self.validate_validators_hash()
    }

    fn validate_validators_hash(&self) -> Result<(), Error> {
        let hash_of_validators: Vec<u8> = self.validators.hash().into();
        if hash_of_validators == self.signed_header.header.validators_hash {
            Ok(())
        } else {
            Err(Error::InvalidValidatorsHash {
                hash_of_validators: hash_of_validators,
                validators_hash_in_header: self.signed_header.header.validators_hash.clone(),
            })
        }
    }

    //https://github.com/tendermint/tendermint/blob/5e52a6ec558f789b642a231c257f8754b97637bc/types/validator_set.go#L636
    pub fn verify_commit<E: Ed25519Verifier>(&self, ed25519_verifier: &mut E) -> Result<(), Error> {
        if self.voting_power(ed25519_verifier)? <= self.validators.total_voting_power() * 2 / 3 {
            return Err(Error::NotEnoughVotingPowerSigned);
        }
        Ok(())
    }

    fn voting_power<E: Ed25519Verifier>(&self, ed25519_verifier: &mut E) -> Result<i64, Error> {
        let commit = self.signed_header.commit.clone();
        let mut voting_power = 0;
        let mut messages = Vec::with_capacity(commit.signatures.len());
        let mut signatures = Vec::with_capacity(commit.signatures.len());
        let mut public_keys = Vec::with_capacity(commit.signatures.len());
        for (index, commit_sig) in commit.signatures.iter().enumerate() {
            let (is_commit, signature_message, signature) = match commit_sig {
                Vote::Absent => continue,
                Vote::Commit(vote) => (
                    true,
                    vote.signature_message(
                        commit.height,
                        commit.round,
                        commit.block_id.clone(),
                        self.signed_header.header.chain_id.clone(),
                    ),
                    vote.signature,
                ),
                Vote::Nil(vote) => (
                    false,
                    vote.signature_message(
                        commit.height,
                        commit.round,
                        self.signed_header.header.chain_id.clone(),
                    ),
                    vote.signature,
                ),
            };
            // validators and commit have a 1-to-1 correspondance.
            // This means we don't need the validator address or to do any lookup.
            // get validator info from validator set
            let validator_info = self.validators.0.get(index).unwrap();
            messages.push(signature_message);
            signatures.push(signature);
            public_keys.push(validator_info.pub_key.as_bytes());
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
            sig.push(signature.as_bytes());
        }
        ed25519_verifier.verify_batch(msg.as_slice(), sig.as_slice(), public_keys.as_slice())?;
        Ok(voting_power)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::light_block::signed_header::commit::vote::NilVote;

    struct DalekEd25519Verifier();

    impl Ed25519Verifier for DalekEd25519Verifier {
        fn verify_batch(
            &mut self,
            messages: &[&[u8]],
            signatures: &[&[u8]],
            public_keys: &[&[u8]],
        ) -> Result<(), Error> {
            let mut sig = Vec::with_capacity(signatures.len());
            for signature in signatures.iter() {
                sig.push(ed25519_dalek::Signature::from_bytes(signature).unwrap())
            }
            let mut pubkeys = Vec::with_capacity(public_keys.len());
            for public_key in public_keys.iter() {
                pubkeys.push(ed25519_dalek::PublicKey::from_bytes(public_key).unwrap())
            }
            ed25519_dalek::verify_batch(messages, &sig, &pubkeys)
                .map_err(|_| Error::VerifyBatchFailed {})
        }
    }

    const LIGHT_BLOCK_JSON: &str = r#"
    {
        "signed_header": {
            "header": {
                "version": {
                    "block": "11"
                },
                "chain_id": "secret-4",
                "height": "1000001",
                "time": "2021-11-23T16:07:37.575128904Z",
                "last_block_id": {
                    "hash": "EDFFB7C581AEDADA9A41E355BEED063E15BB1B2C281957AE864E0392B5DEDE2F",
                    "parts": {
                        "total": 1,
                        "hash": "01932DD4FA96010353DA4A60AC99EED3865C28D8AEADB67DFD9949A9A82D1940"
                    }
                },
                "last_commit_hash": "7FE08F2D155922661247762A68A16C0439E894336FE3E87C07ED3ED0FDECE418",
                "data_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
                "validators_hash": "9EFBBA1CEA6B4CAE8F27C7F16E830FBBFEED6AB8D35245DE263D63FE4F7211B0",
                "next_validators_hash": "9EFBBA1CEA6B4CAE8F27C7F16E830FBBFEED6AB8D35245DE263D63FE4F7211B0",
                "consensus_hash": "717BE5422EFEECF5C48B9B5EA4AFD9C4C2E002A676CA2E9512CAE7802DA37D92",
                "app_hash": "EE32DBF923F4B63F0CCCBB7CABE3BFE58AD33B6749A2D2DC7130E683ED41FE91",
                "last_results_hash": "D0C1BDF3B1811F72A1DA190266D06CE950B465DE1681436AF826619D7DC92A79",
                "evidence_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
                "proposer_address": "B89CDCF017D80A946FBFFC41A2583C03190E8613"
            },
            "commit": {
                "height": "1000001",
                "round": 0,
                "block_id": {
                    "hash": "275AFF29FB91FCFC3E6581B3522502205F03758864CD1030D2A5E2212AA4FBE2",
                    "parts": {
                        "total": 1,
                        "hash": "8C36952ADE6331D2F711AF167879C60340F44DEDEF9E1ADB26DF9DC7549D9B06"
                    }
                },
                "signatures": [
                    {
                        "block_id_flag": 2,
                        "validator_address": "2E76AE6E453395F35D6C0728D44FB6147CE5B5A0",
                        "timestamp": "2021-11-23T16:07:43.581599777Z",
                        "signature": "rIt3m7ehMrIVRzzd3q6Ty6x3JGVutjKyEepb+VLVHmqzB76QgtbtHLRPm4Z5axTcUHf06hh8H2gCCiTN/jRYBg=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "D60D5EE59CF7B1F0D755FD1679661F42CC03CDDD",
                        "timestamp": "2021-11-23T16:07:43.606480752Z",
                        "signature": "S1eOSoZYbfvTsTPU4VosRKfYR1zRZnZ12NUV21jIOMyoBEQwZD7ipASxUGRzu7xHShDIMxJtNcqTiVRewvDGBA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "81EBCE2FFC29820351C086E9EDA6A220098FF41C",
                        "timestamp": "2021-11-23T16:07:43.738980926Z",
                        "signature": "83QVt8wbGoxKCRyXRhkQ6JXrJoHnsdDaaTeFCwE+062jMhGxO9lV8wAJ/egEro/AdpMuwAsRcMgoxy9d+2TWCg=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "237A513A407E33679C746E350B3307BAA5BCDEFD",
                        "timestamp": "2021-11-23T16:07:43.569268777Z",
                        "signature": "kwcpeveOEOObppqKl6E6YYIVgMfEXxxkzZ/rzIGDnd/LYBc8rxL/d6obRdb0milkpkdouisOqLAOug7DIdVOBQ=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "3199A17457ADAED098B8EB1DC932CC7DFBDC54E7",
                        "timestamp": "2021-11-23T16:07:43.591734474Z",
                        "signature": "/UfaiUCO9IaKB2l3ytUzFWmybknCwKXlGCY3hOTbnaKzH7eUaGgEEmQ1monavwS6vE/zCKxx1Q4dFr833v7bBQ=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "1B68882AB7CD6BC4CDDD742FC8F3D1FDE31C1A82",
                        "timestamp": "2021-11-23T16:07:43.524532088Z",
                        "signature": "PIY8y+EpaXjPpxYhpyMVnrL/MwyLmXXrcE0d2W8RE5vP5UNYqh87WDeIZH017pCGqh3CKL2sbthGmtQ3nfy8Cg=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "6C3166166510D49C2AD77A7B9F7308032CF01BE3",
                        "timestamp": "2021-11-23T16:07:43.556636701Z",
                        "signature": "6pkwc9XIKxE0U56LYGwVQdHFCr23K0gjyvb5wnQ88gLPybklLdR+KOpYVmu2+Ex3oYlGzt9HORE/0rUBxt6sAA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "0D5E166C2BEC1C462541D968C7994E45D0B3E51C",
                        "timestamp": "2021-11-23T16:07:43.495902838Z",
                        "signature": "El+2KVyDlA8iM6fKXeAK3BmOklVUaaULWuYmGZSG2me5bsWfC2YRqcW5fLOrsWgIyjdsDt0sWCXAVOJrkLlbAA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "2DD098C8ECAF04DFE31BBC59799C786AC09BF53F",
                        "timestamp": "2021-11-23T16:07:43.539076501Z",
                        "signature": "IzY2QESzvwlfwyrCKm2vkBa8Od3uGxgEkS4XbAfRSrO4impC7kSuWSX4yFxjDq3hbNwyfIqzm44MT6gjX0WHDw=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "73D9DDC9EBB5BDB44ADA9FF2051610B75CB31A8D",
                        "timestamp": "2021-11-23T16:07:43.505837198Z",
                        "signature": "tZVWkWhogLhNqBWm8/8KEABsHiIuOOt2pDeBaxbFy3oXiy0n0pcVUtYHnmQLMczrSQ5x2GNGI3TuPyDB1biYBw=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "45521282C12E0EC1691495FCA714947DCA072745",
                        "timestamp": "2021-11-23T16:07:43.543202285Z",
                        "signature": "YATAikmuLDY6McXakpOZr2zwY4TeeML+m/5IAgjExdD1+bj5nnXWZyQt4ZUAlA8taV4fGzjK+6dctmHB7jY1CA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "DB49D38076905669DD1F75DA868148157545FE0B",
                        "timestamp": "2021-11-23T16:07:43.610501418Z",
                        "signature": "kE6vFEeHl2ZBn4g6B70xC0KCqZHFXSy8jaSQgF7iIq7tgPOHV7qpBPBzS/eheTtGVrUR9DJ5gcUoDQpc26hYAg=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "B89CDCF017D80A946FBFFC41A2583C03190E8613",
                        "timestamp": "2021-11-23T16:07:43.557437271Z",
                        "signature": "O3xgzBsIeTvMPRqnA2OeSHR8fvv0MkMn2rZ7RmUBs4hpPQIi6PmqB+EyWoxMxSoFgD6V6fzzU31hL4tzI7XpDA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "0C085DC02FF7463D278D455B42D365E4D31D3DDD",
                        "timestamp": "2021-11-23T16:07:43.547088445Z",
                        "signature": "Pr7ESM5uNZlpQGIhGWDLFeQ8zGRAaKOVUL5wc5DfLxmE82/62kqPRUgu6f/PeavfF8DCoBqRglS52moNuXu6BQ=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "A009E408C25D73A033F36864FF78FF501626CA31",
                        "timestamp": "2021-11-23T16:07:43.571363982Z",
                        "signature": "HvJ9i+EimXKpCPoNI3JGlITMxrtrM8G7fHA7DA6//oTgSTNNzrjDnV8fQ8GsL2GqFIAGAD3Uh6cj8XTJHuDNAA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "C194AE33E0EFA5963D9951C690F72F8080574DCA",
                        "timestamp": "2021-11-23T16:07:43.546533391Z",
                        "signature": "shljDnKAil3lAHMhTIsqIyxzB16fld6gNJXQ+h4mCgz/VFsVc/Qjq1KPQl0Fsz9fgzivQo040Ip0OdNKu/ChDA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "A672CEA7267D152F6A284C2D1523FA852D7B7139",
                        "timestamp": "2021-11-23T16:07:43.526984674Z",
                        "signature": "YmavlSNq+5+5ivar8wkOPNZfpm4RwQ8R5/oYH3yhBOHfu4EAJaDj3mVnsM0CXpB3ufADa9OQs+ErYoDbo8ipAQ=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "606569CA92AE15476EB5CAE33F894E0B66E3139F",
                        "timestamp": "2021-11-23T16:07:43.543109359Z",
                        "signature": "ZKgKmrX/6SwBYsceMsoUgXTf4AGRtFPHNfwSsYkfwPNq5Y4Lw2X5ukt/UlK0FTvqcqi/ws5hVpe4K15hQM3lCA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "AF954724FCF00610DB810F0900BA7E3C0B6E036A",
                        "timestamp": "2021-11-23T16:07:43.62257973Z",
                        "signature": "NwckqxP7Wkibfw3zNK996+PtOSO7pkDky3J/Y6Ey+2fJfYgwk4fRPjzUOXghjIM3ZanBaqM4x+bSfc6g3ASJBg=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "4CCE562B1E2BC571751DB512222CED5A082470EA",
                        "timestamp": "2021-11-23T16:07:43.680154018Z",
                        "signature": "cH8hzURlek/EjN6fcnYDNjaZ/ZVFlVYfbRqCAHTeYrWAUR9nY+t9zKtw1e6PDnZZ46SqjOlcOvxrZ1XzrDVjDA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "FF4D500FAEC982345CF746F37A4C9D0E9CB71D2E",
                        "timestamp": "2021-11-23T16:07:43.554809534Z",
                        "signature": "MX2tYSJLuhOPHvpOelVelPTUlcMIKDwbupy7D6ldmqLsmH9y1IAvd38f3ggkwftvug7jmrSAo0cO63uvovniDg=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "18B444E801687196D48A075D3622BE1AEE070C11",
                        "timestamp": "2021-11-23T16:07:43.553314205Z",
                        "signature": "lIkjtOqcSt85VTtYgUdlHkUjoXUToCfJ0Ud0tOy04INx+4mWGR4L/AsHc3UdUNuRppGKLfe/NVo07/wg5qgqBw=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "CD052A549834C2B6D5FCB8079235AE6351817560",
                        "timestamp": "2021-11-23T16:07:43.52600024Z",
                        "signature": "UDr3afc+TwKP6S08EDCMNvojSEGddFvb5mv8dU53fsN5rHyC2i8Bc8PeOcGo455T6jeBqWZPZ1kVKuzRPrpUAg=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "0ED5ECB49B658B0F04F813C46064A4185AA02FCD",
                        "timestamp": "2021-11-23T16:07:43.532423778Z",
                        "signature": "z6SsYDdgwwveK33UcC+mv/30gk/SBEcWvdOrqAP1KyI4T2dDEZdyesWBWrql46PDHMy/Nc0/IH3WXZx6++vJDQ=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "035C0FDD9FBB94C2892D97BB1A6B0AE264BD3018",
                        "timestamp": "2021-11-23T16:07:43.569317092Z",
                        "signature": "cV82cPM/5dBEsrDpxriT+M6Hc+gPbL0nCX8cEy8ESwi30dwlXPu5FNZEPEOWS0JVX6DylG5zD5Gpa92+i943Aw=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "6AFCF9EB1AC264954C784274A6ABF012D50EB0B6",
                        "timestamp": "2021-11-23T16:07:43.569167873Z",
                        "signature": "WWMLMIyEOc5rRH9J8eaN8J7iwwekB+42fFAFzJMCZ50oPh7IvqI5KvAuV+vqh/KhImec406W0UFowIkKDri/DA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "C30A05B07434B65876E7AE5E29E257F409033707",
                        "timestamp": "2021-11-23T16:07:43.568778272Z",
                        "signature": "/Sv4VU0+WnK5MJS14iaquKevcecKpA+y/w/nVIwrV30ZEvxtoWzKX5LRwchBVCKqdkHU10Ev1s6eFptS9j4ICA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "32DA4555CC1DBAA54F608FA5F77ED05808ECEBB4",
                        "timestamp": "2021-11-23T16:07:43.568424128Z",
                        "signature": "T3xl5GUKPZgT5JmRBYo61tZOszUPmabjsMNmJ1prPxRhSIGp4BjyRQUN0f6texh0ss1OwZtzq1+yLLZNlwXrCg=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "A990AE301F4E8AB8E14D6965897303E30C7D3456",
                        "timestamp": "2021-11-23T16:07:43.542915104Z",
                        "signature": "So4zhTauJMfdflO8CP9i4wr4gpYa0XUYSz6poBt6JUDabmodnhfVDeFT9oQvHNV01xPnphD1qg2+Rbrab4a/BQ=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "FCA97223CE784E0A9EDBF0010B3BC8916BBCE4D3",
                        "timestamp": "2021-11-23T16:07:43.570321061Z",
                        "signature": "FyqCbB9ABCJoc1To/7RrJiFX6dgAAfX9j7m6z9DNuu+brwx7vAHd9WXO+0XZwxk0UOO9pS3OKomsRaKS2jzDDQ=="
                    },
                    {
                        "block_id_flag": 1,
                        "validator_address": "",
                        "timestamp": "0001-01-01T00:00:00Z",
                        "signature": null
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "6CA06185E170E49C09FA25EECDCE8A2450053C15",
                        "timestamp": "2021-11-23T16:07:43.529908987Z",
                        "signature": "weRDWM7nuiKpRiw2A9cEUW2o6DKXupzSHNT3BKNrFUliDGv3WHyEmOXbUsVlj5t/XQGj2uwxBeEOtdM+3NGzDQ=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "59281B2D5901BB87FD1DD0B36E6331F96BD6FDAB",
                        "timestamp": "2021-11-23T16:07:43.624746104Z",
                        "signature": "Qn/SBBJoeH2037erwkx+S3qt3MawvP+VGOqgtriHI4cGunZHUvjonmcNWMsw1275JKv1jcqP/Ifc5mNYRHNBDQ=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "CD95FE95B7296882E3B9FBF42B7F46961557F92E",
                        "timestamp": "2021-11-23T16:07:43.525079175Z",
                        "signature": "BCMMhQQ0xcKjVMF8gvAN/potHHT6evSlRA4jWSunFo1DpvCrJrLirPlTx71R7kEUoReUA7fnFXx091bVXKhQDQ=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "C410E318926699ADC67A6ECD1737D81115FE3CB0",
                        "timestamp": "2021-11-23T16:07:43.593584647Z",
                        "signature": "z5YlQ/jUMSMQri5mij5PkHS5dN7/GubqVHFPseFgrf39wp4fnr0p3K4T5SI5OdXKIwCj6D3/s6Vme3XWkpaXCA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "5CAE9959825009037C12D7B0F289D1B90E8B2181",
                        "timestamp": "2021-11-23T16:07:43.554659615Z",
                        "signature": "5O8NJC/7Rf+IX7W1TpsPe/WRb5xR/raU+9l9GjW+P6DDZwD0dd3vGEKHsbG59cuTQWw7/6Iney2bw2G2H5ZDDw=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "F3C7B9FA2D8F8A15E1E961EB1D0950D0F10A7B7F",
                        "timestamp": "2021-11-23T16:07:43.539204487Z",
                        "signature": "UMajk/9aoaVebCqICgPsnJeAvbbyIZq68WOv3HhlnagaFVgqovXXZcpSK5pCM5gsEusk11ACVXL331EQxF4GBg=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "84BC2C72491187FAB144F628166E10D592786616",
                        "timestamp": "2021-11-23T16:07:43.545670059Z",
                        "signature": "rVbVhrvZswFvkI3Mdwa45ZamMcZTvugG52nvdn1Zlh4uDHoM9IGtUkQUvoRW9GAH8101oRgCYPhI8pP6/GEyBA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "E855109B212B9EB65C982FD44EE13E77E9E33C4A",
                        "timestamp": "2021-11-23T16:07:43.559877221Z",
                        "signature": "DhKgtR4wyQFOWrM+SLXte28nWd0tr8MzlpQ2t73Yb5MOATujpIR1DmEfb0Mj++R/2zUltBOMTTTLNF6fcRhFCw=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "D588D700719B4D96FE53DFEA8A32BDAFCF13804B",
                        "timestamp": "2021-11-23T16:07:43.555840751Z",
                        "signature": "VIWY6eakAVOP6d7SPzke9La3GhlbOdFX5yOFILgPbZy/9rJnBE7RSsS4dESKWcBidRSxIJSLYljzNBibgWVxAg=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "C2A4856A7EC5E1AEE6712B5F84AD0ACDE44D1F8F",
                        "timestamp": "2021-11-23T16:07:43.503888108Z",
                        "signature": "Oxg0I/tv1YIDDW2cs/NT3ujcGUlGk0bo6D7VpfeNkNQiKrsuMxJfRK8/xauGUuYeviteAUfMt+WmoRHLZcRNBw=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "86BEC016E08B6C674AD5730CB9369C0E744AC09B",
                        "timestamp": "2021-11-23T16:07:43.563987643Z",
                        "signature": "58cH89c7JlIQOHKbTRYcxjGdszgATOOz4uaNpBaZD4YvcokNiwDM4Xs3kZXoG/wAjC6R4/Fv2+5KimsA0vV7Cg=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "E937690E72C1B4FDB08403B05EE820CF74C4D69D",
                        "timestamp": "2021-11-23T16:07:43.616548786Z",
                        "signature": "vNLGnweabsiel3/5SvsQFIrNWUftqH8QXs98NYVwz90+RBjp7i8ObTnO1f33M4RKNvnYvx3vtDTUV+viKVxUAw=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "F28CB422A38A6BBA98DA3344DDF8D6FBC3413319",
                        "timestamp": "2021-11-23T16:07:43.616705464Z",
                        "signature": "RnUC/9nT8ogVIPrtwXlSFTlha0vYK+tVqv3enA8XNPgvnEPfwigQvBU6ky7IrJasb7CBkzZni3h/ZEJY1rIHAg=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "84A43593046466D4BB0D75E03F273F6C9301F0FD",
                        "timestamp": "2021-11-23T16:07:43.532552198Z",
                        "signature": "Vd2bWl5ZIoLw984OfgoIX6AC9sOEYOrn55GZUd8/V0trLTTKNZPxPOM5mm6qJhLooE/PqXnYpzpYP/5/6PH6AA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "8A9EAEABB1A76476C82BAE2A7A50629E7948DFEC",
                        "timestamp": "2021-11-23T16:07:43.587614139Z",
                        "signature": "WZGQsmP28iqPIKI/hM9YPJVuOq6MYlptcmw/eK+hqaNQrGyebd0S14WsNBS8HtfhF1OaWx0y/5LHcbQUEn5qAg=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "836A3FB36DA340DFDE9A7527B171BFF757160E70",
                        "timestamp": "2021-11-23T16:07:43.528541762Z",
                        "signature": "8yJZavBDkUvlZQnSLtBYdoBP8JkgDZr4ImPZqfRrq4gbLwfePcyX9QaYO70GwupH9rThsoZw7Zi8lsypXC/9Bg=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "D164F5D602732836D35D35A7BE701D5706EAF33D",
                        "timestamp": "2021-11-23T16:07:43.594865845Z",
                        "signature": "pQr7AdkkXpr5Rmnq/C8zWFBOsomuhS77U8NCtI3jNNG7cjEVyIh6KN3CZf7u1iCsgqrF9iBYQTrCTC6fef51Cw=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "AE0226C471CD72E7472C4757B09749452F233337",
                        "timestamp": "2021-11-23T16:07:43.555997615Z",
                        "signature": "LcenqRpOn30jhdPXbeYZ+f36RRdYC3YiVq1cNIJyYXRh5fJCxkk0BBJZSfRXmMycuA37HV+K4Jjn7vFisac7DA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "530220D73A0A8A505026B2B0AE2801F172FC8B71",
                        "timestamp": "2021-11-23T16:07:43.50717642Z",
                        "signature": "LBIxt6HKKYui28KaxkH57OHyO7rq8sxfiJzZUiFyznr2m03vx1V6UAzi8+pr21M80qElxQYxXRd7LZhWCmEIDw=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "2A87CB7845C270F142457B7C9ABABC56A8FA0E43",
                        "timestamp": "2021-11-23T16:07:38.575128904Z",
                        "signature": "2DED61AlrowhuxdDaQJeNFytPx5CIS2abGwZPa6ujeECTMzBFs7k3kcn8W4ac/uXDztlgeOorncF6pltdhV5AQ=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "7AB535233EA96F0BA4AA74392714FADDC8B5DF91",
                        "timestamp": "2021-11-23T16:07:43.589498047Z",
                        "signature": "g1UR1LkLLJFkTns0ujc7MdQnrS6RXe6Jtceh17BkfeJRvaH9FBu4hbNPpeW8PNNnu5XD6xJdFQhG8ZPklAitAA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "556A18EE6D549DDEED1F69AF954A325179333754",
                        "timestamp": "2021-11-23T16:07:43.4927737Z",
                        "signature": "NHiqpcRBIaPCfRbHRuTg60TH0ZRKd161tYgLQqveVtmGHeESTySRaB9cUPer09xnhzXCtKqlemo55Smk8q83Dw=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "382F257221279696BA982D79F5E75089BFDDB150",
                        "timestamp": "2021-11-23T16:07:43.594761906Z",
                        "signature": "ItEnoGSevoi36RoA4q+NHppeu+gnI8N/5KmSc68ICcnmokXSrdlu1M/ovPHAAc/sPl1uIrofIkmIOcSCXF0tAw=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "082CDCAF49ED0BCFE3A655259CB4696372A4526F",
                        "timestamp": "2021-11-23T16:07:43.607624231Z",
                        "signature": "bvFiV0XUrhXmUGJjKLt6BOB0Ln3TnFyjjaOTh3g9+bKC37WAccAWLEUdOw5x3d+kvSZcBbxW7hnMtuh6plWGAA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "6FA359FFBE0BAD51E836EB947162CABA2D5821BC",
                        "timestamp": "2021-11-23T16:07:43.464434822Z",
                        "signature": "YGQ3BgJYVGbhz0BB3IL/jQ4DZs2sKm4hxrZABFeI0P1TGylJqQtaapAft6PE0uB2oRkV8wHlathe7lRU27CQCw=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "6541CD05575A924B6E35DCFD319AA70D3592FE31",
                        "timestamp": "2021-11-23T16:07:43.593127335Z",
                        "signature": "WFhE5NrMpK5tNqc0K9fmTqYy1xy8dBvMx9LvARjlcwIDdKqZeGDVL3LXSi6ZS1X/8kJnVCyxSiRtTMEpcEClDA=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "186ED967212C36E398521B2EFF12C510E71180F6",
                        "timestamp": "2021-11-23T16:07:43.483378315Z",
                        "signature": "R/gEd1hwFtbMYlG37ulxXswmOkT608v1C5yJMtm3LvH4m5cNYRiPjEJsahXmdXt8RHHAASj/IWVC1CoVunsPCg=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "10A163F613F952A848678462C6AD23798A80C565",
                        "timestamp": "2021-11-23T16:07:43.534651315Z",
                        "signature": "BDd5rF9BECTiqpgr4dBXroWlPyoMs6F7Pt8qYa90ZDdO8PE6i5YXYkM+ym3Dk81dj+1I7WOH/brMlICGgIazCQ=="
                    },
                    {
                        "block_id_flag": 2,
                        "validator_address": "9DE1234E27D05E9ED2A7E86952FFAF4C7C4CA60D",
                        "timestamp": "2021-11-23T16:07:43.527395666Z",
                        "signature": "4jIXs12swAWyzZaRjvcv6pGi7Pb9bDxbtavGTYzR2TAdPjlzmHt4xKTX09KUtcq3ALRpHZPHqMCRtvxs+pWABA=="
                    }
                ]
            }
        },
        "validators": [
            {
                "address": "2E76AE6E453395F35D6C0728D44FB6147CE5B5A0",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "fdzXH52SQvEkgykh5hYazKraLu07ppXBjPIyG2lz8Y0="
                },
                "voting_power": "10733763",
                "proposer_priority": "11122766"
            },
            {
                "address": "D60D5EE59CF7B1F0D755FD1679661F42CC03CDDD",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "pmk+WgRGOcYlc+J2Cklr63fjqXq+RByxkhlWtTYQocQ="
                },
                "voting_power": "7563935",
                "proposer_priority": "28051489"
            },
            {
                "address": "81EBCE2FFC29820351C086E9EDA6A220098FF41C",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "21xErkFKUyeMV9JcfyAjre2gMBBb/0GNSkvk+tp5J2Y="
                },
                "voting_power": "6917154",
                "proposer_priority": "-18584322"
            },
            {
                "address": "237A513A407E33679C746E350B3307BAA5BCDEFD",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "e42ZP1m+2e0b/eUPG8nfAdc/pbfzQtPk+aLkOL7RW0I="
                },
                "voting_power": "5411217",
                "proposer_priority": "48685722"
            },
            {
                "address": "3199A17457ADAED098B8EB1DC932CC7DFBDC54E7",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "ZtsQ/Ghz80NCbu8S/7wlAg8RHq4uE8ccECmobuA0c6I="
                },
                "voting_power": "4660572",
                "proposer_priority": "31401921"
            },
            {
                "address": "1B68882AB7CD6BC4CDDD742FC8F3D1FDE31C1A82",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "KgnRkdlLhDJT/9zxTl3YwUfXevNgYorFV7NjAflVkAg="
                },
                "voting_power": "4208969",
                "proposer_priority": "-22637039"
            },
            {
                "address": "6C3166166510D49C2AD77A7B9F7308032CF01BE3",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "IPfj4QhdDJ1CzEQc6qJgFoA2QkRrXVJwlpbiGsIChHE="
                },
                "voting_power": "3989864",
                "proposer_priority": "-4438335"
            },
            {
                "address": "0D5E166C2BEC1C462541D968C7994E45D0B3E51C",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "UmhQdJkwY00SqVR7Cx4r9Sqe0Ja+dDV1/csVZr4UKw8="
                },
                "voting_power": "3505500",
                "proposer_priority": "-9468395"
            },
            {
                "address": "2DD098C8ECAF04DFE31BBC59799C786AC09BF53F",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "FAwRvQKiSZ4fmGj/vxrDJmmIMv1ZD7hBb7VY6VeKIl0="
                },
                "voting_power": "3390576",
                "proposer_priority": "47432117"
            },
            {
                "address": "73D9DDC9EBB5BDB44ADA9FF2051610B75CB31A8D",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "OknyC8a6HMtoIkHHKCiag795xFcY7VOHk0W6MpG9/9k="
                },
                "voting_power": "3177484",
                "proposer_priority": "-8725528"
            },
            {
                "address": "45521282C12E0EC1691495FCA714947DCA072745",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "VkRa2IaDT/sDus5EwdKbRLgN1of0hSST7RBIkVnWhuQ="
                },
                "voting_power": "3114991",
                "proposer_priority": "-10440088"
            },
            {
                "address": "DB49D38076905669DD1F75DA868148157545FE0B",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "mSjB3iHfSYl2tCHxGZ5U6fjXktHAwsBRtAk8YFtGBq8="
                },
                "voting_power": "3066046",
                "proposer_priority": "-33667339"
            },
            {
                "address": "B89CDCF017D80A946FBFFC41A2583C03190E8613",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "bZm9aC0UrIKOs3TnUbwp6KnWgvFJR6Z/rF0sDMblIcw="
                },
                "voting_power": "3002666",
                "proposer_priority": "-44160010"
            },
            {
                "address": "0C085DC02FF7463D278D455B42D365E4D31D3DDD",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "/LVCbXPbXYt+FXxpLl69Au5u3UV1aBIsjhKcMJb48D4="
                },
                "voting_power": "2445457",
                "proposer_priority": "37315863"
            },
            {
                "address": "A009E408C25D73A033F36864FF78FF501626CA31",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "j+HBw3fSYfD8f8pxUhZNu7p2XbjPK993xNwqwYEPHpY="
                },
                "voting_power": "2268571",
                "proposer_priority": "46300305"
            },
            {
                "address": "C194AE33E0EFA5963D9951C690F72F8080574DCA",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "JvaRZ5WN1Wzg3xdEqCcZIceuyJQzk7tOiGJRCvasF88="
                },
                "voting_power": "2135509",
                "proposer_priority": "50198179"
            },
            {
                "address": "A672CEA7267D152F6A284C2D1523FA852D7B7139",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "tCBv6CzJBRbSunWqv1mbtA6TgxLPGn8uITIoXX9hfws="
                },
                "voting_power": "2114630",
                "proposer_priority": "-33044599"
            },
            {
                "address": "606569CA92AE15476EB5CAE33F894E0B66E3139F",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "qxCQtPo5y19+pPE+WjQDwLLxzJWAMoke8h+uu26G8pw="
                },
                "voting_power": "2062695",
                "proposer_priority": "-3791489"
            },
            {
                "address": "AF954724FCF00610DB810F0900BA7E3C0B6E036A",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "LCJgleu37ctMqUBunroX74N8rNSmBUbIqRqkhuZ6bWE="
                },
                "voting_power": "1820280",
                "proposer_priority": "-18893099"
            },
            {
                "address": "4CCE562B1E2BC571751DB512222CED5A082470EA",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "wZJMPYKc24dzlFK7GxhrbrrkFwSSIo53U3573Lqa7o8="
                },
                "voting_power": "1758917",
                "proposer_priority": "-12179741"
            },
            {
                "address": "FF4D500FAEC982345CF746F37A4C9D0E9CB71D2E",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "3tTrgzBxeuqhZHWIqIQQmMufKveh4m/avxQfBEyGrkQ="
                },
                "voting_power": "1758329",
                "proposer_priority": "30472079"
            },
            {
                "address": "18B444E801687196D48A075D3622BE1AEE070C11",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "UAgdCbph7gm4K9wh2wK7MPth6USwM5yZQBPwmtrxrdc="
                },
                "voting_power": "1695533",
                "proposer_priority": "-2256205"
            },
            {
                "address": "CD052A549834C2B6D5FCB8079235AE6351817560",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "O4nWlb3Uit2H8MQSlhoRfpGk+oqNbsgYqECmYrh4ttc="
                },
                "voting_power": "1584600",
                "proposer_priority": "39675052"
            },
            {
                "address": "0ED5ECB49B658B0F04F813C46064A4185AA02FCD",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "JckZVv0e1SRtt2NaIGIZsmxbq60k/p3ClaFAXQAu0rg="
                },
                "voting_power": "1494793",
                "proposer_priority": "8395862"
            },
            {
                "address": "035C0FDD9FBB94C2892D97BB1A6B0AE264BD3018",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "QUX6bTk70UalVSb2J3Gy9OkeIDPItKG01t4ANTWsgeA="
                },
                "voting_power": "1493219",
                "proposer_priority": "-41202094"
            },
            {
                "address": "6AFCF9EB1AC264954C784274A6ABF012D50EB0B6",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "b7WwtoywbUdwLDwVIjB10n4SeRKaPEc5afSqAAIdLjk="
                },
                "voting_power": "1447584",
                "proposer_priority": "-24735195"
            },
            {
                "address": "C30A05B07434B65876E7AE5E29E257F409033707",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "b3t6f1aJAH9j2f5pphjLNGAOmgFUt2uiGUG4H0wevlI="
                },
                "voting_power": "1307504",
                "proposer_priority": "11007234"
            },
            {
                "address": "32DA4555CC1DBAA54F608FA5F77ED05808ECEBB4",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "avWWPE0fIFWtYmVJgEnmIzJ/kCyHxVjceVr9I6JYfBI="
                },
                "voting_power": "1181642",
                "proposer_priority": "-18648052"
            },
            {
                "address": "A990AE301F4E8AB8E14D6965897303E30C7D3456",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "FOjiARfMsyE/piW0A6D1i4jhSyxNn2AWsSAxFyT7BRY="
                },
                "voting_power": "1047350",
                "proposer_priority": "18149168"
            },
            {
                "address": "FCA97223CE784E0A9EDBF0010B3BC8916BBCE4D3",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "HySlmwas41mdQxqhrfabyOgmQ826VdnHhnQXVJpRGQs="
                },
                "voting_power": "1028205",
                "proposer_priority": "13819102"
            },
            {
                "address": "14FED124F73CE68A579D4B6C3FCC5AA461BF3E07",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "l5+ckdzeD67c1zlEpCuQoSfYhgpkpn6mBLMTxq9Y7Qw="
                },
                "voting_power": "882996",
                "proposer_priority": "749125"
            },
            {
                "address": "6CA06185E170E49C09FA25EECDCE8A2450053C15",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "6CpAjjJ6i6qB3KA1sOt7PgD6nrIwJaUcfOJUr5dipAw="
                },
                "voting_power": "770569",
                "proposer_priority": "26859647"
            },
            {
                "address": "59281B2D5901BB87FD1DD0B36E6331F96BD6FDAB",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "nHebB956xRnjMNbHDNjZX0GEtDJJGuFJImMxkAIRwH8="
                },
                "voting_power": "736298",
                "proposer_priority": "-3216874"
            },
            {
                "address": "CD95FE95B7296882E3B9FBF42B7F46961557F92E",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "o3+aaISKGr2O2NTD15J3/0/zNVPgOfuqxu1t/NmickE="
                },
                "voting_power": "713623",
                "proposer_priority": "14521675"
            },
            {
                "address": "C410E318926699ADC67A6ECD1737D81115FE3CB0",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "wL3Uvi1Ori3mKhVyns/1PpRa/1VTxr0N4zkFyooLzJI="
                },
                "voting_power": "613105",
                "proposer_priority": "-24834558"
            },
            {
                "address": "5CAE9959825009037C12D7B0F289D1B90E8B2181",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "FSHE0kWApgbQK96kBbmXwpWt9eVpRSuaMCx/m/VKak0="
                },
                "voting_power": "493746",
                "proposer_priority": "53386908"
            },
            {
                "address": "F3C7B9FA2D8F8A15E1E961EB1D0950D0F10A7B7F",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "LCh8vJCJcTpi1X+uaOWWN/GsP73YrgySVeprqakdhyE="
                },
                "voting_power": "472810",
                "proposer_priority": "27237836"
            },
            {
                "address": "84BC2C72491187FAB144F628166E10D592786616",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "3eZa47Lruer80eZparaqL/WmbJ08UL2RhC1vT9zkGnE="
                },
                "voting_power": "458757",
                "proposer_priority": "-12686808"
            },
            {
                "address": "E855109B212B9EB65C982FD44EE13E77E9E33C4A",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "W53v9+QM1lS4uYbcfWw8uPAoQA1R6LtTqLMySTe24os="
                },
                "voting_power": "430270",
                "proposer_priority": "2117615"
            },
            {
                "address": "D588D700719B4D96FE53DFEA8A32BDAFCF13804B",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "P7uo7qvAFJtJMgHAG63/ziM4XBG28FBsQX+94tDcreg="
                },
                "voting_power": "366206",
                "proposer_priority": "26773562"
            },
            {
                "address": "C2A4856A7EC5E1AEE6712B5F84AD0ACDE44D1F8F",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "WexxXZt1GjvEcZOMNvmxRqmavrAX20JBMs+juS/3JkQ="
                },
                "voting_power": "323651",
                "proposer_priority": "-38353565"
            },
            {
                "address": "86BEC016E08B6C674AD5730CB9369C0E744AC09B",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "/9kVn9jYPB4KiRNXHHaENO9bIZJK84b7Nv+0lNIzZwo="
                },
                "voting_power": "294127",
                "proposer_priority": "25066267"
            },
            {
                "address": "E937690E72C1B4FDB08403B05EE820CF74C4D69D",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "c97pnpXA4WZQff/JoGc3S6emj5gSNUX9raNAIXl47OE="
                },
                "voting_power": "287848",
                "proposer_priority": "31352321"
            },
            {
                "address": "F28CB422A38A6BBA98DA3344DDF8D6FBC3413319",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "+95KA4cEQ8DS4dymcl+APnm9gEZnEyNpBrqxQLPKy3Q="
                },
                "voting_power": "267632",
                "proposer_priority": "-15284512"
            },
            {
                "address": "84A43593046466D4BB0D75E03F273F6C9301F0FD",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "MInR+cAUvmNwwTCAbC541qg7N28cliNRVgxrljbLM4U="
                },
                "voting_power": "228874",
                "proposer_priority": "-5270340"
            },
            {
                "address": "8A9EAEABB1A76476C82BAE2A7A50629E7948DFEC",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "uJH9TZLQMxk+bGG2zxzyZkL8NyPTUHI3ODPvUDSew2Y="
                },
                "voting_power": "195870",
                "proposer_priority": "40359104"
            },
            {
                "address": "836A3FB36DA340DFDE9A7527B171BFF757160E70",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "7L0pxIgiq+F5sMkxbZsWEBIR5jNPIbAoZUh3XXePiaI="
                },
                "voting_power": "163472",
                "proposer_priority": "-22771212"
            },
            {
                "address": "D164F5D602732836D35D35A7BE701D5706EAF33D",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "FPXg1eGvj/ssUseMD1zsd9uCBk/HT56dhhBC6vZfryk="
                },
                "voting_power": "123852",
                "proposer_priority": "49033670"
            },
            {
                "address": "AE0226C471CD72E7472C4757B09749452F233337",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "/KlrJDFI6aB23Re7TLb6yI1ojZBdDFYOkGvqMJWoanI="
                },
                "voting_power": "115248",
                "proposer_priority": "38827050"
            },
            {
                "address": "530220D73A0A8A505026B2B0AE2801F172FC8B71",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "ia1NW2yFhwu1xrAyzmz5VlRopNme1MPkADPjhyci4WA="
                },
                "voting_power": "108554",
                "proposer_priority": "22849273"
            },
            {
                "address": "2A87CB7845C270F142457B7C9ABABC56A8FA0E43",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "e4SoEbDwU1Icf6N9auC4BliMXgvoTVNll4e2zt6vHII="
                },
                "voting_power": "103843",
                "proposer_priority": "-34913617"
            },
            {
                "address": "7AB535233EA96F0BA4AA74392714FADDC8B5DF91",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "TgqjYmm7PAiIw2RnOFUwdHFwqJUoJ3wCgOda1F8zUns="
                },
                "voting_power": "40105",
                "proposer_priority": "11072756"
            },
            {
                "address": "556A18EE6D549DDEED1F69AF954A325179333754",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "7aW1aZ+XR6QdLMoxwYIkstxzu3G1gzH0pBK/llqVdGU="
                },
                "voting_power": "22565",
                "proposer_priority": "-22417599"
            },
            {
                "address": "382F257221279696BA982D79F5E75089BFDDB150",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "LIS4u7fFPx0OzDJ3OuaXKm3atKwDYJaS53zJs6QUtXc="
                },
                "voting_power": "21195",
                "proposer_priority": "9269065"
            },
            {
                "address": "082CDCAF49ED0BCFE3A655259CB4696372A4526F",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "jYQevmHVCNRRO8KnaM8pyPR6fQOnL1HZZnrzzFPvx6U="
                },
                "voting_power": "5503",
                "proposer_priority": "-535042"
            },
            {
                "address": "6FA359FFBE0BAD51E836EB947162CABA2D5821BC",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "XITiJSVtUuoJRScZwfGYSFenLDGv5+NC74+vsg9zXUw="
                },
                "voting_power": "4202",
                "proposer_priority": "-944833"
            },
            {
                "address": "6541CD05575A924B6E35DCFD319AA70D3592FE31",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "L4n3o3xRVLGTeImClUHzBv83SahNMlv3fqLIkriP81c="
                },
                "voting_power": "2829",
                "proposer_priority": "-5323147"
            },
            {
                "address": "186ED967212C36E398521B2EFF12C510E71180F6",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "WKNlQumQbcG9tf7lgqeRkP4dtPAKSyOTRHmLUavdUTs="
                },
                "voting_power": "1206",
                "proposer_priority": "-93256541"
            },
            {
                "address": "10A163F613F952A848678462C6AD23798A80C565",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "AMCL79oF8cN+7gf2zD/pywNsiCiwenRw7dY9YFcqoiM="
                },
                "voting_power": "50",
                "proposer_priority": "-110120866"
            },
            {
                "address": "9DE1234E27D05E9ED2A7E86952FFAF4C7C4CA60D",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "8z17XkDs93z6Joxbz7TfkQPF0ZWj++I2KUdFnc1bnMc="
                },
                "voting_power": "30",
                "proposer_priority": "-104701667"
            }
        ]
    }"#;

    #[test]
    fn test_veirfy_sanity() {
        let light_block: LightBlock = serde_json::from_str(LIGHT_BLOCK_JSON).unwrap();
        light_block.verify(&mut DalekEd25519Verifier {}).unwrap()
    }

    #[test]
    fn test_veirfy_invalid_header_hash() {
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
    fn test_veirfy_signature_for_different_header_hash() {
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
}
