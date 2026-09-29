//! Error types for mls-sync

use thiserror::Error;

/// Main error type for mls-sync
#[derive(Debug, Error)]
pub enum SyncError {
    /// CRDT error
    #[error("CRDT error: {0}")]
    CrdtError(String),
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
    /// Protocol error
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),
    /// Timeout error
    #[error("Timeout error")]
    TimeoutError,
}
