use cosmwasm_std::Env;
use secret_toolkit::crypto::sha_256;
use sfps_lib::ed25519_dalek::rand::{rngs::StdRng, SeedableRng};

pub type Seed = [u8; 32];

pub fn rng(seed: Seed) -> StdRng {
    StdRng::from_seed(seed)
}

pub fn gen_seed(prev_seed: Seed, env: &Env, entropy: &[u8]) -> Seed {
    // 16 here represents the lengths in bytes of the block height and time.
    let entropy_len = 32 + 16 + env.message.sender.len() + entropy.len();
    let mut rng_entropy = Vec::with_capacity(entropy_len);
    rng_entropy.extend_from_slice(&prev_seed);
    rng_entropy.extend_from_slice(&env.block.height.to_be_bytes());
    rng_entropy.extend_from_slice(&env.block.time.to_be_bytes());
    rng_entropy.extend_from_slice(&env.message.sender.0.as_bytes());
    rng_entropy.extend_from_slice(entropy);
    sha_256(&rng_entropy)
}
