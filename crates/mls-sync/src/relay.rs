//! Relay server communication

use std::net::SocketAddr;

/// Relay client
#[allow(dead_code)]
pub struct RelayClient {
    server_addr: SocketAddr,
}

impl RelayClient {
    /// Create a new relay client
    #[must_use]
    pub const fn new(server_addr: SocketAddr) -> Self {
        Self { server_addr }
    }

    /// Connect to the relay server
    ///
    /// # Errors
    /// Returns an error if the connection fails.
    pub const fn connect(&self) -> Result<(), crate::error::SyncError> {
        Ok(())
    }
}
