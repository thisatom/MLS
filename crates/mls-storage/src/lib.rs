//! SQLite-based encrypted storage for MLS
//!
//! This crate provides the storage backend using SQLite with:
//! - Encrypted data storage
//! - Vault and item management
//! - CRDT state vector storage
//!
//! All data is encrypted before being stored in the database using the
//! cryptographic backend from `mls-crypto`.

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

pub mod backend;
pub mod error;
pub mod repository;
pub mod schema;

pub use backend::StorageBackend;
pub use error::StorageError;
pub use repository::{ItemRepository, SyncStateRepository, VaultRepository};
