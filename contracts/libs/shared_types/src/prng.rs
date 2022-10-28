use cosmwasm_std::{CanonicalAddr, Env, ReadonlyStorage, StdError, StdResult, Storage};
use rand::rngs::StdRng;
use rand::SeedableRng;
use secret_toolkit_crypto::sha_256;
use std::convert::TryInto;

type Seed = [u8; 32];

/// Update prng seed with the transaction sender address of the tx and additional entropy, then return a Rng generated from the new seed.
/// The New seed is sha256 hash of 'prev seed + sender address + entropy'.
/// In each txs, prng must be updated before used.
///
/// Seed updating generates New prng seed (S) from prev seed (PS), sender address(A), entropy(E(A)).
/// S = HASH(PS | A | E(A))
///
/// [Prev seed]
/// Prev seed makes seeds different between each transaction from one sender.
///
/// [Sender address]
/// Sender address makes seeds different between transaction senders (sender address is trusted parameter with signature which verified in TEE).
///
/// [Entropy]
/// Entropy makes seeds unpredictable on local, even by the first seed provider.
///
/// The first seed provider, which is the init transaction sender of the contract, can predict the first seed on local.
/// Anyone on network can predict a sequence of sender address generating new seeds.
///
/// If sender A gives entropy E(A) and generate a seed S = HASH(PS | A | E(A)),
/// the others cannot predict the same seed on their local without E(A) even if they know PS and A.
/// So it is important that senders who generated seeds before keep their entropies secret.
pub fn update_prng<S: Storage>(
    storage: &mut S,
    storage_key: &[u8],
    sender: &CanonicalAddr,
    entropy: &[u8],
) -> StdResult<StdRng> {
    let seed = update_prng_seed(storage, storage_key, sender, entropy)?;
    Ok(StdRng::from_seed(seed))
}

pub fn init_prng<S: Storage>(
    storage: &mut S,
    storage_key: &[u8],
    env: &Env,
    entropy: &[u8],
) -> StdResult<()> {
    let seed = gen_initial_prng_seed(env, entropy)?;
    write_prng_seed(storage, storage_key, &seed);
    Ok(())
}

/// Update prng seed and return the new seed.
fn update_prng_seed<S: Storage>(
    storage: &mut S,
    storage_key: &[u8],
    address: &CanonicalAddr,
    entropy: &[u8],
) -> StdResult<Seed> {
    let prev_seed = read_prng_seed(storage, storage_key)?;
    let new_seed = gen_next_prng_seed(&prev_seed, address, entropy);
    write_prng_seed(storage, storage_key, &new_seed);
    Ok(new_seed)
}

fn read_prng_seed<S: ReadonlyStorage>(storage: &S, storage_key: &[u8]) -> StdResult<Seed> {
    storage
        .get(storage_key)
        .map(|vec| vec.try_into().unwrap())
        .ok_or_else(|| StdError::generic_err("prng seed not stored"))
}

fn write_prng_seed<S: Storage>(storage: &mut S, storage_key: &[u8], seed: &Seed) {
    storage.set(storage_key, seed)
}

fn gen_initial_prng_seed(env: &Env, entropy: &[u8]) -> StdResult<Seed> {
    // 16 here represents the lengths in bytes of the block height and time.
    let entropy_len = 16 + env.message.sender.len() + entropy.len();
    let mut input = Vec::with_capacity(entropy_len);
    input.extend_from_slice(&env.block.height.to_be_bytes());
    input.extend_from_slice(&env.block.time.to_be_bytes());
    input.extend_from_slice(&env.message.sender.0.as_bytes());
    input.extend_from_slice(entropy);
    Ok(sha_256(&input))
}

fn gen_next_prng_seed(seed: &Seed, sender: &CanonicalAddr, entropy: &[u8]) -> Seed {
    let input_len = 32 + sender.len() + entropy.len();
    let mut input = Vec::with_capacity(input_len);
    input.extend_from_slice(seed);
    input.extend_from_slice(sender.as_slice());
    input.extend_from_slice(entropy);
    sha_256(&input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::*;
    use cosmwasm_std::*;

    #[test]
    fn test_gen_initial_prng_seed() {
        let mut env = Env {
            block: BlockInfo {
                height: 12_345,
                time: 12_345,
                chain_id: "chain-id".to_string(),
            },
            message: MessageInfo {
                sender: "address1".into(),
                sent_funds: vec![],
            },
            contract: ContractInfo {
                address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            },
            contract_key: Some("".to_string()),
            contract_code_hash: "".to_string(),
        };

        let mut entropy = vec![0, 1, 2];

        let seed = gen_initial_prng_seed(&env, &entropy).unwrap();
        assert_eq!(
            seed,
            [
                163, 219, 37, 161, 21, 203, 20, 172, 169, 48, 158, 146, 94, 235, 76, 75, 114, 236,
                114, 107, 72, 136, 53, 27, 26, 182, 111, 252, 19, 83, 45, 253
            ]
        );

        env.block.height += 1;
        let seed = gen_initial_prng_seed(&env, &entropy).unwrap();
        assert_eq!(
            seed,
            [
                165, 172, 8, 176, 29, 85, 82, 178, 39, 27, 58, 232, 41, 166, 145, 8, 224, 225, 29,
                97, 3, 72, 184, 229, 250, 172, 253, 31, 52, 239, 252, 211
            ]
        );

        env.block.time += 1;
        let seed = gen_initial_prng_seed(&env, &entropy).unwrap();
        assert_eq!(
            seed,
            [
                170, 17, 199, 84, 24, 84, 8, 209, 152, 245, 158, 17, 191, 166, 104, 73, 21, 109,
                85, 174, 191, 127, 66, 219, 102, 100, 161, 14, 155, 108, 82, 87
            ]
        );

        env.message.sender = "address2".into();
        let seed = gen_initial_prng_seed(&env, &entropy).unwrap();
        assert_eq!(
            seed,
            [
                45, 222, 37, 171, 224, 77, 119, 106, 209, 212, 249, 116, 113, 112, 126, 229, 95,
                82, 63, 52, 85, 180, 157, 215, 114, 160, 142, 144, 19, 161, 204, 156
            ]
        );

        entropy.push(3);
        let seed = gen_initial_prng_seed(&env, &entropy).unwrap();
        assert_eq!(
            seed,
            [
                43, 19, 186, 234, 158, 191, 50, 20, 160, 35, 59, 187, 253, 20, 127, 56, 104, 166,
                16, 115, 11, 178, 202, 240, 156, 49, 137, 164, 138, 158, 209, 211
            ]
        );
    }

    #[test]
    fn test_gen_next_prng_seed() {
        let seed = [0; 32];
        let new_seed = gen_next_prng_seed(&seed, &b"address1".to_vec().into(), b"entropy1");
        assert_eq!(
            new_seed,
            [
                42, 136, 48, 251, 249, 174, 176, 121, 38, 238, 102, 5, 57, 173, 140, 67, 221, 95,
                137, 14, 180, 182, 88, 134, 54, 196, 172, 156, 8, 6, 225, 113
            ]
        );

        let another_seed = [1; 32];
        let new_seed = gen_next_prng_seed(&another_seed, &b"address1".to_vec().into(), b"entropy1");
        assert_eq!(
            new_seed,
            [
                158, 101, 183, 85, 12, 72, 160, 149, 109, 172, 71, 158, 129, 170, 19, 146, 163, 77,
                223, 180, 162, 54, 250, 211, 242, 33, 146, 51, 217, 43, 179, 86
            ]
        );

        let new_seed = gen_next_prng_seed(&seed, &b"address2".to_vec().into(), b"entropy1");
        assert_eq!(
            new_seed,
            [
                114, 115, 52, 51, 10, 58, 82, 232, 184, 233, 198, 51, 170, 137, 108, 242, 208, 202,
                122, 25, 186, 24, 39, 161, 155, 181, 217, 222, 90, 150, 64, 128
            ]
        );

        let new_seed = gen_next_prng_seed(&seed, &b"address1".to_vec().into(), b"entropy2");
        assert_eq!(
            new_seed,
            [
                10, 130, 69, 56, 105, 190, 183, 0, 70, 213, 103, 171, 122, 193, 71, 243, 71, 45,
                100, 169, 95, 51, 32, 61, 237, 62, 191, 130, 73, 77, 130, 6
            ]
        );
    }
}
