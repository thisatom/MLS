//! Storage backend trait and implementation

use crate::error::StorageError;
use mls_crypto::traits::CryptoBackend;
use secrecy::ExposeSecret;
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;
use std::sync::Arc;
use zeroize::Zeroizing;

/// Trait for storage backend implementations
///
/// This trait allows swapping storage implementations (e.g., for testing or
/// alternative storage engines). The default implementation uses SQLite.
pub trait StorageBackend {
    /// Type of the vault identifier
    type VaultId: Send + Sync + 'static;
    /// Type of the item identifier
    type ItemId: Send + Sync + 'static;

    /// Initialize the storage
    ///
    /// # Errors
    /// Returns an error if initialization fails.
    fn init(&self) -> Result<(), StorageError>;

    /// Create a new vault
    ///
    /// # Arguments
    /// * `vault_id` - Unique identifier for the vault
    /// * `metadata` - Vault metadata to encrypt and store
    /// * `encryption_key` - Key for encrypting the vault metadata
    ///
    /// # Returns
    /// The created vault
    ///
    /// # Errors
    /// Returns an error if vault creation fails.
    fn create_vault(
        &self,
        vault_id: &str,
        metadata: &impl Serialize,
        encryption_key: &Zeroizing<[u8; 32]>,
    ) -> Result<(), StorageError>;

    /// Get vault metadata
    ///
    /// # Arguments
    /// * `vault_id` - Vault identifier
    /// * `encryption_key` - Key for decrypting the vault metadata
    ///
    /// # Returns
    /// Deserialized vault metadata
    ///
    /// # Errors
    /// Returns an error if vault is not found or decryption fails.
    fn get_vault<T: DeserializeOwned>(
        &self,
        vault_id: &str,
        encryption_key: &Zeroizing<[u8; 32]>,
    ) -> Result<T, StorageError>;

    /// List all vault IDs
    ///
    /// # Returns
    /// Vector of vault IDs
    ///
    /// # Errors
    /// Returns an error if listing fails.
    fn list_vaults(&self) -> Result<Vec<String>, StorageError>;

    /// Delete a vault
    ///
    /// # Arguments
    /// * `vault_id` - Vault identifier
    ///
    /// # Errors
    /// Returns an error if deletion fails.
    fn delete_vault(&self, vault_id: &str) -> Result<(), StorageError>;

    /// Create a new item in a vault
    ///
    /// # Arguments
    /// * `vault_id` - Parent vault identifier
    /// * `item_id` - Unique identifier for the item
    /// * `data` - Item data to encrypt and store
    /// * `version` - Item version for CRDT
    /// * `encryption_key` - Key for encrypting the item data
    ///
    /// # Returns
    /// The created item
    ///
    /// # Errors
    /// Returns an error if item creation fails.
    fn create_item(
        &self,
        vault_id: &str,
        item_id: &str,
        data: &impl Serialize,
        version: i64,
        encryption_key: &Zeroizing<[u8; 32]>,
    ) -> Result<(), StorageError>;

    /// Get item data
    ///
    /// # Arguments
    /// * `item_id` - Item identifier
    /// * `encryption_key` - Key for decrypting the item data
    ///
    /// # Returns
    /// Deserialized item data and version
    ///
    /// # Errors
    /// Returns an error if item is not found or decryption fails.
    fn get_item<T: DeserializeOwned>(
        &self,
        item_id: &str,
        encryption_key: &Zeroizing<[u8; 32]>,
    ) -> Result<(T, i64), StorageError>;

    /// List all item IDs in a vault
    ///
    /// # Arguments
    /// * `vault_id` - Parent vault identifier
    ///
    /// # Returns
    /// Vector of item IDs and versions
    ///
    /// # Errors
    /// Returns an error if listing fails.
    fn list_items(&self, vault_id: &str) -> Result<Vec<(String, i64)>, StorageError>;

    /// Update an existing item
    ///
    /// # Arguments
    /// * `vault_id` - Parent vault identifier
    /// * `item_id` - Item identifier
    /// * `data` - New item data to encrypt and store
    /// * `version` - New item version
    /// * `encryption_key` - Key for encrypting the item data
    ///
    /// # Errors
    /// Returns an error if update fails.
    fn update_item(
        &self,
        vault_id: &str,
        item_id: &str,
        data: &impl Serialize,
        version: i64,
        encryption_key: &Zeroizing<[u8; 32]>,
    ) -> Result<(), StorageError>;

    /// Delete an item
    ///
    /// # Arguments
    /// * `item_id` - Item identifier
    ///
    /// # Errors
    /// Returns an error if deletion fails.
    fn delete_item(&self, item_id: &str) -> Result<(), StorageError>;

    /// Get sync state for a vault
    ///
    /// # Arguments
    /// * `vault_id` - Vault identifier
    ///
    /// # Returns
    /// State vector for CRDT synchronization
    ///
    /// # Errors
    /// Returns an error if state retrieval fails.
    fn get_sync_state(&self, vault_id: &str) -> Result<Option<Vec<u8>>, StorageError>;

    /// Update sync state for a vault
    ///
    /// # Arguments
    /// * `vault_id` - Vault identifier
    /// * `state_vector` - New state vector
    ///
    /// # Errors
    /// Returns an error if state update fails.
    fn update_sync_state(
        &self,
        vault_id: &str,
        state_vector: &[u8],
    ) -> Result<(), StorageError>;
}

/// SQLite storage backend implementation
#[derive(Clone)]
pub struct SqliteStorage {
    db_path: String,
    #[allow(dead_code)]
    crypto_backend: Arc<dyn CryptoBackend>,
}

impl std::fmt::Debug for SqliteStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteStorage")
            .field("db_path", &self.db_path)
            .field("crypto_backend", &"Arc<dyn CryptoBackend>")
            .finish()
    }
}

impl SqliteStorage {
    /// Create a new SQLite storage backend
    ///
    /// # Arguments
    /// * `db_path` - Path to the SQLite database file
    /// * `crypto_backend` - Cryptographic backend for encryption/decryption
    pub fn new(db_path: impl AsRef<Path>, crypto_backend: Arc<dyn CryptoBackend>) -> Self {
        let db_path = db_path.as_ref().to_string_lossy().into_owned();
        Self {
            db_path,
            crypto_backend,
        }
    }

    /// Get a database connection
    ///
    /// # Errors
    /// Returns an error if connection fails.
    #[allow(dead_code)]
    fn get_conn(&self) -> Result<rusqlite::Connection, StorageError> {
        let conn = rusqlite::Connection::open_with_flags(
            &self.db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        Ok(conn)
    }

    /// Encrypt data using the crypto backend
    #[allow(dead_code)]
    fn encrypt_data<D: Serialize>(
        &self,
        data: &D,
        key: &Zeroizing<[u8; 32]>,
    ) -> Result<Vec<u8>, StorageError> {
        let serialized = serde_json::to_vec(data)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        let (nonce, ciphertext) = self.crypto_backend.encrypt(key, &serialized)?;
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypt data using the crypto backend
    #[allow(dead_code)]
    fn decrypt_data<D: DeserializeOwned>(
        &self,
        encrypted: &[u8],
        key: &Zeroizing<[u8; 32]>,
    ) -> Result<D, StorageError> {
        if encrypted.len() < 12 {
            return Err(StorageError::DeserializationError(
                "Encrypted data too short".to_string(),
            ));
        }
        let nonce = &encrypted[..12];
        let ciphertext = &encrypted[12..];
        let decrypted = self.crypto_backend.decrypt(key, nonce, ciphertext)?;
        let data = serde_json::from_slice(decrypted.expose_secret())
            .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
        Ok(data)
    }
}
