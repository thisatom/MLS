//! Storage repository implementation

use crate::error::StorageError;
use mls_crypto::traits::CryptoBackend;
use rusqlite::Connection;
use secrecy::ExposeSecret;
use serde_json;
use zeroize::Zeroizing;

/// Vault repository for managing encrypted vaults
pub struct VaultRepository {
    conn: rusqlite::Connection,
}

impl VaultRepository {
    /// Create a new vault repository
    pub const fn new(conn: rusqlite::Connection) -> Self {
        Self { conn }
    }

    /// Create a new vault
    ///
    /// # Arguments
    /// * `vault_id` - Unique identifier for the vault
    /// * `metadata` - Vault metadata to encrypt and store (serializable)
    /// * `encryption_key` - 32-byte encryption key
    /// * `crypto_backend` - Cryptographic backend for encryption
    ///
    /// # Errors
    /// Returns an error if vault creation fails.
    pub fn create_vault<M: serde::Serialize>(
        &self,
        vault_id: &str,
        metadata: &M,
        encryption_key: &Zeroizing<[u8; 32]>,
        crypto_backend: &dyn CryptoBackend,
    ) -> Result<(), StorageError> {
        let serialized = serde_json::to_vec(metadata)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        let (nonce, ciphertext) = crypto_backend
            .encrypt(encryption_key, &serialized)?;

        let mut encrypted_metadata = Vec::with_capacity(nonce.len() + ciphertext.len());
        encrypted_metadata.extend_from_slice(&nonce);
        encrypted_metadata.extend_from_slice(&ciphertext);

        let created_at = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO vaults (id, encrypted_metadata, created_at) VALUES (?, ?, ?)",
            rusqlite::params![vault_id, encrypted_metadata, created_at],
        )?;

        Ok(())
    }

    /// Get vault metadata
    ///
    /// # Arguments
    /// * `vault_id` - Vault identifier
    /// * `encryption_key` - 32-byte encryption key
    /// * `crypto_backend` - Cryptographic backend for decryption
    ///
    /// # Returns
    /// Deserialized vault metadata
    ///
    /// # Errors
    /// Returns an error if vault is not found or decryption fails.
    pub fn get_vault<M: serde::de::DeserializeOwned>(
        &self,
        vault_id: &str,
        encryption_key: &Zeroizing<[u8; 32]>,
        crypto_backend: &dyn CryptoBackend,
    ) -> Result<M, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT encrypted_metadata FROM vaults WHERE id = ?")?;

        let encrypted_metadata: Vec<u8> = stmt
            .query_row(rusqlite::params![vault_id], |row| {
                row.get(0)
            })
            .map_err(|e| {
                if e == rusqlite::Error::QueryReturnedNoRows {
                    StorageError::VaultNotFound(vault_id.to_string())
                } else {
                    StorageError::SqliteError(e)
                }
            })?;

        if encrypted_metadata.len() < 12 {
            return Err(StorageError::DeserializationError(
                "Encrypted metadata too short".to_string(),
            ));
        }

        let nonce = &encrypted_metadata[..12];
        let ciphertext = &encrypted_metadata[12..];
        let decrypted = crypto_backend.decrypt(encryption_key, nonce, ciphertext)?;

        serde_json::from_slice(decrypted.expose_secret())
            .map_err(|e| StorageError::DeserializationError(e.to_string()))
    }

    /// List all vault IDs
    ///
    /// # Returns
    /// Vector of vault IDs
    ///
    /// # Errors
    /// Returns an error if listing fails.
    pub fn list_vaults(&self) -> Result<Vec<String>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM vaults ORDER BY created_at")?;

        let vault_ids: Result<Vec<String>, _> = stmt
            .query_map([], |row| row.get(0))?
            .collect();

        Ok(vault_ids?)
    }

    /// Check if a vault exists
    ///
    /// # Arguments
    /// * `vault_id` - Vault identifier
    ///
    /// # Returns
    /// True if vault exists
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    pub fn vault_exists(&self, vault_id: &str) -> Result<bool, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM vaults WHERE id = ?")?;

        let exists = stmt.exists(rusqlite::params![vault_id])?;
        Ok(exists)
    }

    /// Delete a vault and all its items
    ///
    /// # Arguments
    /// * `vault_id` - Vault identifier
    ///
    /// # Errors
    /// Returns an error if deletion fails.
    ///
    /// # Panics
    /// Panics if the connection path cannot be retrieved.
    pub fn delete_vault(&self, vault_id: &str) -> Result<(), StorageError> {
        // Note: We create a new connection for the transaction since we need mutable access
        // and Connection::transaction requires &mut self
        let conn_path = self.conn.path().expect("Failed to get connection path");
        let mut conn = Connection::open_with_flags(
            conn_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?;
        let tx = conn.transaction()?;

        // Delete sync state
        tx.execute("DELETE FROM sync_state WHERE vault_id = ?", [vault_id])?;

        // Delete items
        tx.execute("DELETE FROM items WHERE vault_id = ?", [vault_id])?;

        // Delete vault
        let rows_affected = tx.execute("DELETE FROM vaults WHERE id = ?", [vault_id])?;

        if rows_affected == 0 {
            return Err(StorageError::VaultNotFound(vault_id.to_string()));
        }

        tx.commit()?;
        Ok(())
    }
}

/// Item repository for managing encrypted items
pub struct ItemRepository {
    conn: rusqlite::Connection,
}

impl ItemRepository {
    /// Create a new item repository
    pub const fn new(conn: rusqlite::Connection) -> Self {
        Self { conn }
    }

    /// Create a new item
    ///
    /// # Arguments
    /// * `vault_id` - Parent vault identifier
    /// * `item_id` - Unique identifier for the item
    /// * `data` - Item data to encrypt and store (serializable)
    /// * `version` - Item version for CRDT
    /// * `encryption_key` - 32-byte encryption key
    /// * `crypto_backend` - Cryptographic backend for encryption
    ///
    /// # Errors
    /// Returns an error if item creation fails.
    pub fn create_item<D: serde::Serialize>(
        &self,
        vault_id: &str,
        item_id: &str,
        data: &D,
        version: i64,
        encryption_key: &Zeroizing<[u8; 32]>,
        crypto_backend: &dyn CryptoBackend,
    ) -> Result<(), StorageError> {
        let serialized = serde_json::to_vec(data)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        let (nonce, ciphertext) = crypto_backend
            .encrypt(encryption_key, &serialized)?;

        let mut encrypted_blob = Vec::with_capacity(nonce.len() + ciphertext.len());
        encrypted_blob.extend_from_slice(&nonce);
        encrypted_blob.extend_from_slice(&ciphertext);

        let updated_at = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO items (id, vault_id, encrypted_blob, version, updated_at) VALUES (?, ?, ?, ?, ?)",
            rusqlite::params![item_id, vault_id, encrypted_blob, version, updated_at],
        )?;

        Ok(())
    }

    /// Get item data
    ///
    /// # Arguments
    /// * `item_id` - Item identifier
    /// * `encryption_key` - 32-byte encryption key
    /// * `crypto_backend` - Cryptographic backend for decryption
    ///
    /// # Returns
    /// Tuple of (deserialized item data, version)
    ///
    /// # Errors
    /// Returns an error if item is not found or decryption fails.
    pub fn get_item<D: serde::de::DeserializeOwned>(
        &self,
        item_id: &str,
        encryption_key: &Zeroizing<[u8; 32]>,
        crypto_backend: &dyn CryptoBackend,
    ) -> Result<(D, i64), StorageError> {
        let mut stmt = self.conn.prepare(
            "SELECT encrypted_blob, version FROM items WHERE id = ?",
        )?;

        let (encrypted_blob, version): (Vec<u8>, i64) = stmt
            .query_row(rusqlite::params![item_id], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(|e| {
                if e == rusqlite::Error::QueryReturnedNoRows {
                    StorageError::ItemNotFound(item_id.to_string())
                } else {
                    StorageError::SqliteError(e)
                }
            })?;

        if encrypted_blob.len() < 12 {
            return Err(StorageError::DeserializationError(
                "Encrypted blob too short".to_string(),
            ));
        }

        let nonce = &encrypted_blob[..12];
        let ciphertext = &encrypted_blob[12..];
        let decrypted = crypto_backend.decrypt(encryption_key, nonce, ciphertext)?;

        let data = serde_json::from_slice(decrypted.expose_secret())
            .map_err(|e| StorageError::DeserializationError(e.to_string()))?;

        Ok((data, version))
    }

    /// List all items in a vault
    ///
    /// # Arguments
    /// * `vault_id` - Parent vault identifier
    ///
    /// # Returns
    /// Vector of (`item_id`, version) tuples
    ///
    /// # Errors
    /// Returns an error if listing fails.
    pub fn list_items(&self, vault_id: &str) -> Result<Vec<(String, i64)>, StorageError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, version FROM items WHERE vault_id = ? ORDER BY updated_at",
        )?;

        let items: Result<Vec<(String, i64)>, _> = stmt
            .query_map(rusqlite::params![vault_id], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect();

        Ok(items?)
    }

    /// Check if an item exists
    ///
    /// # Arguments
    /// * `item_id` - Item identifier
    ///
    /// # Returns
    /// True if item exists
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    pub fn item_exists(&self, item_id: &str) -> Result<bool, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM items WHERE id = ?")?;

        let exists = stmt.exists(rusqlite::params![item_id])?;
        Ok(exists)
    }

    /// Update an existing item
    ///
    /// # Arguments
    /// * `vault_id` - Parent vault identifier
    /// * `item_id` - Item identifier
    /// * `data` - New item data to encrypt and store (serializable)
    /// * `version` - New item version
    /// * `encryption_key` - 32-byte encryption key
    /// * `crypto_backend` - Cryptographic backend for encryption
    ///
    /// # Errors
    /// Returns an error if update fails.
    pub fn update_item<D: serde::Serialize>(
        &self,
        vault_id: &str,
        item_id: &str,
        data: &D,
        version: i64,
        encryption_key: &Zeroizing<[u8; 32]>,
        crypto_backend: &dyn CryptoBackend,
    ) -> Result<(), StorageError> {
        let serialized = serde_json::to_vec(data)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        let (nonce, ciphertext) = crypto_backend
            .encrypt(encryption_key, &serialized)?;

        let mut encrypted_blob = Vec::with_capacity(nonce.len() + ciphertext.len());
        encrypted_blob.extend_from_slice(&nonce);
        encrypted_blob.extend_from_slice(&ciphertext);

        let updated_at = chrono::Utc::now().to_rfc3339();

        let rows_affected = self.conn.execute(
            "UPDATE items SET encrypted_blob = ?, version = ?, updated_at = ? WHERE id = ? AND vault_id = ?",
            rusqlite::params![encrypted_blob, version, updated_at, item_id, vault_id],
        )?;

        if rows_affected == 0 {
            return Err(StorageError::ItemNotFound(item_id.to_string()));
        }

        Ok(())
    }

    /// Delete an item
    ///
    /// # Arguments
    /// * `item_id` - Item identifier
    ///
    /// # Errors
    /// Returns an error if deletion fails.
    pub fn delete_item(&self, item_id: &str) -> Result<(), StorageError> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM items WHERE id = ?", [item_id])?;

        if rows_affected == 0 {
            return Err(StorageError::ItemNotFound(item_id.to_string()));
        }

        Ok(())
    }
}

/// Sync state repository for CRDT state vectors
pub struct SyncStateRepository {
    conn: rusqlite::Connection,
}

impl SyncStateRepository {
    /// Create a new sync state repository
    pub const fn new(conn: rusqlite::Connection) -> Self {
        Self { conn }
    }

    /// Get sync state for a vault
    ///
    /// # Arguments
    /// * `vault_id` - Vault identifier
    ///
    /// # Returns
    /// Optional state vector bytes
    ///
    /// # Errors
    /// Returns an error if state retrieval fails.
    pub fn get_sync_state(&self, vault_id: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT state_vector FROM sync_state WHERE vault_id = ?")?;

        match stmt.query_row(rusqlite::params![vault_id], |row| row.get::<_, Vec<u8>>(0)) {
            Ok(state_vector) => Ok(Some(state_vector)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StorageError::SqliteError(e)),
        }
    }

    /// Update sync state for a vault
    ///
    /// # Arguments
    /// * `vault_id` - Vault identifier
    /// * `state_vector` - New state vector bytes
    ///
    /// # Errors
    /// Returns an error if state update fails.
    pub fn update_sync_state(&self, vault_id: &str, state_vector: &[u8]) -> Result<(), StorageError> {
        let updated_at = chrono::Utc::now().to_rfc3339();

        let rows_affected = self.conn.execute(
            "INSERT OR REPLACE INTO sync_state (id, vault_id, state_vector, updated_at) VALUES (?, ?, ?, ?)",
            rusqlite::params![
                format!("{}:sync", vault_id),
                vault_id,
                state_vector,
                updated_at
            ],
        )?;

        if rows_affected == 0 {
            return Err(StorageError::VaultNotFound(vault_id.to_string()));
        }

        Ok(())
    }
}
