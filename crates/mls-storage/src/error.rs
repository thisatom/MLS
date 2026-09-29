//! Error types for mls-storage

use mls_crypto::error::CryptoError;
use thiserror::Error;

/// Main error type for mls-storage
#[derive(Debug, Error)]
pub enum StorageError {
    /// SQLite error
    #[error("SQLite error: {0}")]
    SqliteError(#[from] rusqlite::Error),
    /// Crypto error
    #[error("Crypto error: {0}")]
    CryptoError(#[from] CryptoError),
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    /// Vault not found
    #[error("Vault not found: {0}")]
    VaultNotFound(String),
    /// Item not found
    #[error("Item not found: {0}")]
    ItemNotFound(String),
}
