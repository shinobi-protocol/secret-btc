use crate::{
    bitcoin_spv, sfps, state_proxy, viewing_key, CanonicalContractReference, Canonicalize,
    ContractReference, BLOCK_SIZE,
};
use cosmwasm_std::{Api, Binary, CanonicalAddr, HumanAddr, StdResult};
use schemars::JsonSchema;
use secret_toolkit::utils::HandleCallback;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InitMsg {
    pub seed: state_proxy::client::Seed,
    pub config: Config,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct Config {
    /// [Bitcoin]
    /// Unit of utxo value that the contrat accepts
    pub btc_tx_values: HashSet<u64>,

    /// [Contract References]
    pub bitcoin_spv: ContractReference,
    pub sfps: ContractReference,
    pub sbtc: ContractReference,
    pub log: ContractReference,
    pub state_proxy: ContractReference,

    /// [Owner]
    pub owner: HumanAddr,
}

#[derive(Serialize, Deserialize)]
pub struct CanonicalConfig {
    pub btc_tx_values: HashSet<u64>,
    pub bitcoin_spv: CanonicalContractReference,
    pub sfps: CanonicalContractReference,
    pub sbtc: CanonicalContractReference,
    pub log: CanonicalContractReference,
    pub state_proxy: CanonicalContractReference,
    pub owner: CanonicalAddr,
}

impl Canonicalize for Config {
    type Canonicalized = CanonicalConfig;

    fn into_canonical<A: Api>(self, api: &A) -> StdResult<Self::Canonicalized> {
        Ok(Self::Canonicalized {
            btc_tx_values: self.btc_tx_values,
            bitcoin_spv: self.bitcoin_spv.into_canonical(api)?,
            sfps: self.sfps.into_canonical(api)?,
            sbtc: self.sbtc.into_canonical(api)?,
            log: self.log.into_canonical(api)?,
            state_proxy: self.state_proxy.into_canonical(api)?,
            owner: self.owner.into_canonical(api)?,
        })
    }

    fn from_canonical<A: Api>(canonical: Self::Canonicalized, api: &A) -> StdResult<Self> {
        Ok(Self {
            btc_tx_values: canonical.btc_tx_values,
            bitcoin_spv: ContractReference::from_canonical(canonical.bitcoin_spv, api)?,
            sfps: ContractReference::from_canonical(canonical.sfps, api)?,
            sbtc: ContractReference::from_canonical(canonical.sbtc, api)?,
            log: ContractReference::from_canonical(canonical.log, api)?,
            state_proxy: ContractReference::from_canonical(canonical.state_proxy, api)?,
            owner: HumanAddr::from_canonical(canonical.owner, api)?,
        })
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    CreateViewingKey {
        entropy: String,
    },
    SetViewingKey {
        key: viewing_key::ViewingKey,
    },

    RequestMintAddress {
        entropy: Binary,
    },
    VerifyMintTx {
        height: u32,
        tx: Binary,
        merkle_proof: bitcoin_spv::MerkleProofMsg,
    },
    ReleaseIncorrectAmountBTC {
        height: u32,
        tx: Binary,
        merkle_proof: bitcoin_spv::MerkleProofMsg,
        recipient_address: String,
        fee_per_vb: u64,
    },
    RequestReleaseBtc {
        entropy: Binary,
        amount: u64,
    },
    ClaimReleasedBtc {
        merkle_proof: sfps::MerkleProof,
        #[schemars(with = "Vec<String>")]
        #[serde(with = "sfps::serde_proto_message_array")]
        headers: Vec<sfps::Header>,
        block_hash_index: u64,
        encryption_key: Binary,
        recipient_address: String,
        fee_per_vb: u64,
    },
    ChangeOwner {
        new_owner: HumanAddr,
    },
    SetSuspensionSwitch {
        suspension_switch: SuspensionSwitch,
    },
    ReleaseBtcByOwner {
        tx_value: u64,
        max_input_length: u64,
        recipient_address: String,
        fee_per_vb: u64,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    CreateViewingKey { key: viewing_key::ViewingKey },
    RequestMintAddress { mint_address: String },
    ReleaseIncorrectAmountBTC { tx: Binary },
    ClaimReleasedBtc { tx: Binary },
    RequestReleaseBtc { request_key: RequestKey },
    ReleaseBtcByOwner { tx: Binary },
}

#[derive(Serialize, Deserialize, Clone, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    MintAddress {
        address: HumanAddr,
        key: viewing_key::ViewingKey,
    },
    SuspensionSwitch {},
    Config {},
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    MintAddress { address: Option<String> },
    Config(Config),
    SuspensionSwitch(SuspensionSwitch),
    ViewingKeyError { msg: String },
}

/// Bitcoin withdrawal request key.
/// It is sha256 hash of 'requester address + utxo + pseudorandom bytes'.
///
/// [IMPORTANT]
/// It must be unpredictable.
/// It must not leak any information about the used pseudorandom bytes and utxo at generation process.
///
/// The request key is provided to the requseter as the proof of the request, in the form of the response of the request transaction.
/// Therefore, the request key is published to the out of the contract.
/// At the claim phase, the requester send the request key to the contract so that the contract can verify the request.
#[derive(Debug, PartialEq, Eq, Copy, Clone, JsonSchema, Serialize, Deserialize)]
pub struct RequestKey([u8; 32]);

impl RequestKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
// true => supsend
pub struct SuspensionSwitch {
    pub request_mint_address: bool,
    pub verify_mint_tx: bool,
    pub release_incorrect_amount_btc: bool,
    pub request_release_btc: bool,
    pub claim_release_btc: bool,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_deserialize_claim_release_btc_msg() {
        let json = r#"
        {
            "claim_released_btc": {
                "recipient_address": "bcrt1qqww9y2xewqr679k6fectv74lkjq64498f4xvml",
                "fee_per_vb": 24,
                "merkle_proof": {
                "total": 1,
                "index": 0,
                "leaf": "129a030a97030a2a2f7365637265742e636f6d707574652e763162657461312e4d736745786563757465436f6e747261637412e8024773fc1286787623ed4bd8346698e047b12e803a7281abbc1433eaf9bf4728008da38415eb8160316b63d28979a1241a537ec1c16da7e25704eadda4a45a68637e3ddee8b1a868e4586c9e7bd2a663e235c937a233024fd9a4429b0568fe6db94b10657e6c3a0c8a5fd4e0ca50f773a9e6686db4052ecc34efea12a8ad7e267b9cbddb418d658a9e5b5f0339c6107784c2d46a5e1918220759769da20873465c8db820dfc30069b6a935196b9106b2816c8930c3e19e74092559a76f281b04fcf96ab7065cf75375f666dec028e38d7cdacaa55c61293f6668fa47b02b3705a920621074d900d72158c7118aaf1586c21615bdec4e27c9c6bd362c604e9648a0dcb6bcd568016ee01f6ab67fd60e2854d65ceedf341d599c18fa927bc48344a946c25af9dfaf27aa9e33ddc49b386039f49930f87bece0b53524d380f46336e37843df732cb679176b3017768f22c6292b05826f81788febeb5cdc9257e639851d06f54dcf62556f28c0cf2430f0bc08",
                "aunts": []
                },
                "headers": [
                "CgIICxILc2VjcmV0ZGV2LTEYqgMiCwjo0NCWBhDb39MCKkgKILCxAcHRYXi1uJvUfkdO1O43IiHbxLuiorroorEOMnnVEiQIARIgpdAWN+UnhdPDrFtBPL4ZHN2bSJBBVubw1uO+Ohz+zesyIATm5CfG7D68p6lhJZQkQtJYh/1efvkKhKggI1z8/5jGOiDjsMRCmPwcFJr79MiZb7kkJ65B5GSbk0yklZkbeFK4VUIghsxqKC81tk1qrG7YbMpAoAwEIZt92Zcc2pq+QedHjoZKIIbMaigvNbZNaqxu2GzKQKAMBCGbfdmXHNqavkHnR46GUiAEgJG8fdwoP3e/v5HXPETaWMPfipy8hnQF2Lfz2q2iL1ogapYfobSfmOe3+soNokyd0PVtP2qaoboC9W30wiJNNfViIIZuyU/hhjjiPrFY8w97TZneMypeGqFL2XeH2nFvAnl1aiDjsMRCmPwcFJr79MiZb7kkJ65B5GSbk0yklZkbeFK4VXIULxRYhmxZuyGFvvpTEeSR5j7g8/c=",
                "CgIICxILc2VjcmV0ZGV2LTEYqwMiCwjt0NCWBhC5zbgFKkgKILqfctc21Rurzq/J78ztltatljfG0eFWyFMRC69IbEDrEiQIARIg5ULX4ygObOyzd9edUaqTxtTbeuWmwggd+rzESrnhZDsyIKdW0Gpldf8DMaFYSsgfg3PZVpcdVJaeNleKTpkdQUpwOiBcVvkSKDa/sCRlMT2cjeofssNANZC4f9fvUdoyDlMFx0IghsxqKC81tk1qrG7YbMpAoAwEIZt92Zcc2pq+QedHjoZKIIbMaigvNbZNaqxu2GzKQKAMBCGbfdmXHNqavkHnR46GUiAEgJG8fdwoP3e/v5HXPETaWMPfipy8hnQF2Lfz2q2iL1ogyDYtUBk/eEsF62t0SETqKO5m5+t5WcoMxT2e0TV8mnJiIOOwxEKY/BwUmvv0yJlvuSQnrkHkZJuTTKSVmRt4UrhVaiDjsMRCmPwcFJr79MiZb7kkJ65B5GSbk0yklZkbeFK4VXIULxRYhmxZuyGFvvpTEeSR5j7g8/c=",
                "CgIICxILc2VjcmV0ZGV2LTEYrAMiCwjy0NCWBhCot7oIKkgKIGYEHVGkjSTmaOYjKlp381lHt7XyOU/wtfVPUdLKDDbnEiQIARIgt7qNLrHMECvrni699+xTAK9L2NxI1YmnZbaKQetSH44yIKUIK6GPc2vxQaPGAlH0mu+Z8PLp0wJZ7/WdhIdeQHGbOiDjsMRCmPwcFJr79MiZb7kkJ65B5GSbk0yklZkbeFK4VUIghsxqKC81tk1qrG7YbMpAoAwEIZt92Zcc2pq+QedHjoZKIIbMaigvNbZNaqxu2GzKQKAMBCGbfdmXHNqavkHnR46GUiAEgJG8fdwoP3e/v5HXPETaWMPfipy8hnQF2Lfz2q2iL1ogHYQq8z3QIBonWAeHP4Zo1qeiOripQJNcCuxhAZ4JHT9iILhDN8QF6Q7pk+VmAI/Fv6N1nqcT3X7VGorCVrNJuYZDaiDjsMRCmPwcFJr79MiZb7kkJ65B5GSbk0yklZkbeFK4VXIULxRYhmxZuyGFvvpTEeSR5j7g8/c=",
                "CgIICxILc2VjcmV0ZGV2LTEYrQMiCwj30NCWBhC70eULKkgKIGH1WI4EPtx2eyiqngcVb1/X2Be+od8xifz7Jb8AQzl0EiQIARIg0+XBOqSo8VAROOeKSuewK/4blyng1kQG+HZEBp23el4yIOUYce+ofOUfVkjUhRCbO0UajjVQbVh2CP0IUxsnNEhdOiDjsMRCmPwcFJr79MiZb7kkJ65B5GSbk0yklZkbeFK4VUIghsxqKC81tk1qrG7YbMpAoAwEIZt92Zcc2pq+QedHjoZKIIbMaigvNbZNaqxu2GzKQKAMBCGbfdmXHNqavkHnR46GUiAEgJG8fdwoP3e/v5HXPETaWMPfipy8hnQF2Lfz2q2iL1ogl4MbrPFAAk21Kp82nljmVZae3Dr5JRPgYsvYEi8Ogu1iIOOwxEKY/BwUmvv0yJlvuSQnrkHkZJuTTKSVmRt4UrhVaiDjsMRCmPwcFJr79MiZb7kkJ65B5GSbk0yklZkbeFK4VXIULxRYhmxZuyGFvvpTEeSR5j7g8/c=",
                "CgIICxILc2VjcmV0ZGV2LTEYrgMiCwj80NCWBhD4hbAPKkgKIPZI3h5ndk1+t2h2/vc/AITp2fe2AfoSitghKhBWKoUqEiQIARIgiFaBwlvNignOrpJCs6V4Uvp2Fv9/f4AT6Z2YxSXw2PcyIPIt6r1I2lrkGXTv9vzZ+vIX+nio6ZjmKvM7eJ3NlohsOiDjsMRCmPwcFJr79MiZb7kkJ65B5GSbk0yklZkbeFK4VUIghsxqKC81tk1qrG7YbMpAoAwEIZt92Zcc2pq+QedHjoZKIIbMaigvNbZNaqxu2GzKQKAMBCGbfdmXHNqavkHnR46GUiAEgJG8fdwoP3e/v5HXPETaWMPfipy8hnQF2Lfz2q2iL1ogQ5mbBKoYu2GBURhfvlJNYiqVax0FaffsOCGfaR5Y6DRiIOOwxEKY/BwUmvv0yJlvuSQnrkHkZJuTTKSVmRt4UrhVaiDjsMRCmPwcFJr79MiZb7kkJ65B5GSbk0yklZkbeFK4VXIULxRYhmxZuyGFvvpTEeSR5j7g8/c=",
                "CgIICxILc2VjcmV0ZGV2LTEYrwMiCwiB0dCWBhCdqp4VKkgKIDEleN4mWq2vS9WB5SFbtLYDDLjxXBslh2HBVNLO0YFaEiQIARIgUB5WTuZG5+cHRKjslP547AOIShLFfNSzhnWBtviImZIyIHgOKLNYl3PbMvehI4tztSuqW/6hEkmwhiR4G65fkFQ3OiDjsMRCmPwcFJr79MiZb7kkJ65B5GSbk0yklZkbeFK4VUIghsxqKC81tk1qrG7YbMpAoAwEIZt92Zcc2pq+QedHjoZKIIbMaigvNbZNaqxu2GzKQKAMBCGbfdmXHNqavkHnR46GUiAEgJG8fdwoP3e/v5HXPETaWMPfipy8hnQF2Lfz2q2iL1ogjyhp/dIRp1GiMkSsXJQVxbHnu16pjmkX9vwmOawOnMViIOOwxEKY/BwUmvv0yJlvuSQnrkHkZJuTTKSVmRt4UrhVaiDjsMRCmPwcFJr79MiZb7kkJ65B5GSbk0yklZkbeFK4VXIULxRYhmxZuyGFvvpTEeSR5j7g8/c=",
                "CgIICxILc2VjcmV0ZGV2LTEYsAMiCwiG0dCWBhC/yb0bKkgKIM11aE+/shfFQNfaGc3mnEptCqrOcCanSG8/DqVOAw92EiQIARIg1YnTPX1yDtagJz3VoeLD0B2d6Nc9+6q68F+muv4MReQyINNtESWd7m8pw5sqgjgP8i5Snf/1QPVpsChUGdqTdiDeOiDjsMRCmPwcFJr79MiZb7kkJ65B5GSbk0yklZkbeFK4VUIghsxqKC81tk1qrG7YbMpAoAwEIZt92Zcc2pq+QedHjoZKIIbMaigvNbZNaqxu2GzKQKAMBCGbfdmXHNqavkHnR46GUiAEgJG8fdwoP3e/v5HXPETaWMPfipy8hnQF2Lfz2q2iL1ogRrutNowm+VAXK5f3Ias1hE2evrC4UWQ3blA3aBRHppxiIOOwxEKY/BwUmvv0yJlvuSQnrkHkZJuTTKSVmRt4UrhVaiDjsMRCmPwcFJr79MiZb7kkJ65B5GSbk0yklZkbeFK4VXIULxRYhmxZuyGFvvpTEeSR5j7g8/c="
                ],
                "block_hash_index": 42,
                "encryption_key": "LlWet5oKuhFkEJSskLy2JOO/lKfnoyUyeAKHeIcw53s="
            }
        }"#;
        serde_json::from_str::<HandleMsg>(&json).unwrap();
    }
}
