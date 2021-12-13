use crate::merkle::simple_hash_from_byte_vectors;
use ed25519_dalek::{PublicKey as Ed25519, Signature, Verifier};
use prost::{Message, Oneof};
use serde::Serializer;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Ed25519(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Ed25519(string) => write!(f, "ed25519 error: {}", string),
        }
    }
}

impl From<ed25519_dalek::ed25519::Error> for Error {
    fn from(e: ed25519_dalek::ed25519::Error) -> Self {
        Self::Ed25519(e.to_string())
    }
}

// Support Only Ed25519
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum PublicKey {
    #[schemars(with = "String")]
    #[serde(
        rename = "tendermint/PubKeyEd25519",
        deserialize_with = "deserialize_ed25519_base64",
        serialize_with = "serialize_ed25519_base64"
    )]
    Ed25519(Ed25519),
}

impl PublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Ok(Self::Ed25519(Ed25519::from_bytes(bytes)?))
    }

    pub fn verify(&self, msg: &[u8], signature: &Signature) -> Result<(), Error> {
        self.extract().verify(msg, signature)?;
        Ok(())
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.extract().as_bytes()
    }

    pub fn extract(&self) -> &Ed25519 {
        match self {
            Self::Ed25519(pk) => pk,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ValidatorInfo {
    pub address: String,
    pub pub_key: PublicKey,
    #[schemars(with = "String")]
    #[serde(with = "crate::serde::str")]
    pub voting_power: i64,
}

#[derive(Message)]
pub struct CanonicalValidator {
    #[prost(message, optional, tag = "1")]
    pub pub_key: Option<CanonicalPubkey>,

    #[prost(int64, tag = "2")]
    pub voting_power: i64,
}

#[derive(Message)]
pub struct CanonicalPubkey {
    #[prost(oneof = "CanonicalPubkeySum", tags = "1, 2")]
    pub sum: Option<CanonicalPubkeySum>,
}

#[derive(Oneof)]
pub enum CanonicalPubkeySum {
    #[prost(bytes, tag = "1")]
    Ed25519(Vec<u8>),
    #[prost(bytes, tag = "2")]
    Secp256k1(Vec<u8>),
}

fn deserialize_ed25519_base64<'de, D>(deserializer: D) -> Result<Ed25519, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    let bytes = base64::decode(&string).map_err(serde::de::Error::custom)?;
    Ed25519::from_bytes(&bytes).map_err(serde::de::Error::custom)
}

/// Serialize the bytes of an Ed25519 public key as Base64. Used for serializing JSON
fn serialize_ed25519_base64<S>(pk: &Ed25519, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    base64::encode(pk.as_bytes()).serialize(serializer)
}

impl ValidatorInfo {
    pub fn verify_signature(&self, msg: &[u8], signature: &Signature) -> Result<(), Error> {
        self.pub_key.verify(msg, signature)
    }
    fn hash_bytes(&self) -> Vec<u8> {
        let canonical: CanonicalValidator = self.into();
        canonical.encode_to_vec()
    }
}

impl From<&ValidatorInfo> for CanonicalValidator {
    fn from(info: &ValidatorInfo) -> CanonicalValidator {
        CanonicalValidator {
            pub_key: Some(CanonicalPubkey {
                sum: Some(CanonicalPubkeySum::Ed25519(
                    info.pub_key.as_bytes().to_vec(),
                )),
            }),
            voting_power: info.voting_power,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Validators(pub Vec<ValidatorInfo>);
impl Validators {
    pub fn total_voting_power(&self) -> i64 {
        self.0.iter().map(|validator| validator.voting_power).sum()
    }
    pub fn hash(&self) -> Vec<u8> {
        let leaves: Vec<Vec<u8>> = self
            .0
            .iter()
            .map(|validator| validator.hash_bytes())
            .collect();
        simple_hash_from_byte_vectors(leaves)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decode_validator_info() {
        let validator: ValidatorInfo = serde_json::from_str(r#"{"address":"035C0FDD9FBB94C2892D97BB1A6B0AE264BD3018","pub_key":{"type":"tendermint/PubKeyEd25519","value":"QUX6bTk70UalVSb2J3Gy9OkeIDPItKG01t4ANTWsgeA="},"voting_power":"835504","proposer_priority":"38083669"}"#).unwrap();
        assert_eq!(
            validator.address,
            "035C0FDD9FBB94C2892D97BB1A6B0AE264BD3018".to_string()
        );
        assert_eq!(
            validator.pub_key,
            PublicKey::from_bytes(
                &base64::decode("QUX6bTk70UalVSb2J3Gy9OkeIDPItKG01t4ANTWsgeA=").unwrap()
            )
            .unwrap(),
        );
        assert_eq!(validator.voting_power, 835504);
    }

    #[test]
    fn test_hash_validators() {
        //  validator set at 2812211
        let validators: Validators = serde_json::from_str(
            r#"[
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
            ]"#,
        )
        .unwrap();
        let hash = validators.hash();
        assert_eq!(
            hex::encode_upper(&hash),
            "9EFBBA1CEA6B4CAE8F27C7F16E830FBBFEED6AB8D35245DE263D63FE4F7211B0",
        );
    }
}
