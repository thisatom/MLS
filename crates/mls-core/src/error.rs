//! Error types for mls-core

use thiserror::Error;

/// Main error type for mls-core
#[derive(Debug, Error)]
pub enum CoreError {
    /// Placeholder error variant
    #[error("Core error: {0}")]
    Generic(String),
}
