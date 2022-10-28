mod api_ed25519_verifier;
mod handle;
mod init;
mod query;

pub use handle::handle;
pub use init::init;
pub use query::query;

pub const CONTRACT_LABEL: &[u8] = b"sfps";

#[cfg(test)]
mod test;
