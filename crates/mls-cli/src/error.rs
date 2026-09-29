//! Error types for MLS CLI

use dialoguer;
use mls_storage::error::StorageError;
use thiserror::Error;

/// CLI error type
#[derive(Debug, Error)]
pub enum CliError {
    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),

    /// Crypto error
    #[error("Crypto error: {0}")]
    CryptoError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Vault not initialized
    #[error("Vault not initialized. Please run 'mls init' first.")]
    VaultNotInitialized,

    /// Already initialized
    #[error("Vault already initialized. Use --force to reinitialize.")]
    AlreadyInitialized,

    /// Not unlocked
    #[error("Vault is not unlocked. Please run 'mls unlock' first.")]
    NotUnlocked,

    /// Already unlocked
    #[error("Vault is already unlocked.")]
    AlreadyUnlocked,

    /// Item not found
    #[error("Item '{0}' not found.")]
    ItemNotFound(String),

    /// Invalid password
    #[error("Invalid password.")]
    InvalidPassword,

    /// Session expired
    #[error("Session expired. Please unlock again.")]
    SessionExpired,

    /// User cancelled
    #[error("Operation cancelled by user.")]
    UserCancelled,

    /// Confirmation required
    #[error("Confirmation required. Use --force to skip.")]
    ConfirmationRequired,

    /// Dialoguer error
    #[error("Input error: {0}")]
    DialoguerError(#[from] dialoguer::Error),
}
