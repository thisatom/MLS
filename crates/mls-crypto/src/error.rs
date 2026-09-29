//! Error types for mls-crypto

use thiserror::Error;

/// Main error type for mls-crypto
#[derive(Debug, Error)]
pub enum CryptoError {
    /// Argon2 error
    #[error("Argon2 error: {0}")]
    Argon2Error(#[from] argon2::Error),
    /// AES-GCM error
    #[error("AES-GCM error: {0}")]
    AesGcmError(#[from] aes_gcm::Error),
    /// Invalid key length
    #[error("Invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },
    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    /// HKDF error
    #[error("HKDF error")]
    HkdfError,
}
