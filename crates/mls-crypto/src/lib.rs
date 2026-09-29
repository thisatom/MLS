//! Zero-knowledge encryption backend for MLS
//!
//! This crate provides cryptographic primitives including:
//! - Master key derivation from password using Argon2id
//! - Key derivation using HKDF
//! - Encryption/decryption using AES-256-GCM
//! - `CryptoBackend` trait for future PQC support

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

pub mod error;
pub mod key_derivation;
pub mod symmetric;
pub mod traits;

// Re-export main types for convenience
pub use traits::{AesGcmBackend, CryptoBackend};


