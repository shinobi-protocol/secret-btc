mod handle;
mod init;
mod query;
mod query_bitcoin_network;

pub use handle::handle;
pub use init::init;
pub use query::query;

pub const CONTRACT_LABEL: &[u8] = b"gateway";

#[cfg(test)]
mod tests;
