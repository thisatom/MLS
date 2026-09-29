//! LAN discovery and communication

use std::net::SocketAddr;

/// LAN sync manager
#[derive(Default)]
pub struct LanSyncManager;

impl LanSyncManager {
    /// Create a new LAN sync manager
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Discover devices on the local network
    #[must_use]
    pub const fn discover_devices() -> Vec<SocketAddr> {
        Vec::new()
    }
}
