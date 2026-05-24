pub mod client_connection;
pub mod crypto;
pub mod error;
pub mod messages;
pub mod network_protocol;
pub mod traits;

#[cfg(test)]
mod tests;

pub const DEFAULT_SERVER_ADDRESS: &str = "127.0.0.1:42677";
