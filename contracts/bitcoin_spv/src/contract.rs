mod handle;
mod init;
mod query;

pub use handle::handle;
pub use init::init;
pub use query::query;

pub const CONTRACT_LABEL: &[u8] = b"bitcoin_spv";
pub const PREFIX_PRNG: &[u8] = b"prng";

#[cfg(test)]
mod tests;
