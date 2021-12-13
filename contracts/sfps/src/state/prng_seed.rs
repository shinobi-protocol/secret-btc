use super::PRNG_SEED_KEY;
use crate::rng::Seed;
use cosmwasm_std::{ReadonlyStorage, Storage};
use std::convert::TryInto;

pub fn read_prng_seed<S: ReadonlyStorage>(storage: &S) -> Seed {
    storage.get(PRNG_SEED_KEY).unwrap().try_into().unwrap()
}

pub fn write_prng_seed<S: Storage>(storage: &mut S, seed: &Seed) {
    storage.set(PRNG_SEED_KEY, seed)
}
