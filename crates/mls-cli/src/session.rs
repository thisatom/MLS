//! Session management for MLS CLI

use anyhow::Result;
use mls_core::models::{Item, ItemData, ItemId, ItemMetadata, ItemType, VaultId};
use mls_crypto::traits::{AesGcmBackend, CryptoBackend};
use mls_storage::repository::{ItemRepository, SyncStateRepository, VaultRepository};
use mls_storage::schema;
use rand::RngCore;
use rpassword::read_password;
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::config::{Config, DEFAULT_VAULT_FILE};
use crate::error::CliError;

/// Default session timeout (15 minutes)
const DEFAULT_SESSION_TIMEOUT: Duration = Duration::from_mins(15);

/// Session state
pub struct Session {
    /// Vault ID
    pub vault_id: VaultId,
    /// Vault name
    pub vault_name: String,
    /// Master key (derived from password)
    pub master_key: Zeroizing<[u8; 32]>,
    /// Authentication key
    pub auth_key: Zeroizing<[u8; 32]>,
    /// Encryption key
    pub encryption_key: Zeroizing<[u8; 32]>,
    /// Database connection
    pub conn: Connection,
    /// Vault repository
    pub vault_repo: VaultRepository,
    /// Item repository
    pub item_repo: ItemRepository,
    /// Sync state repository
    pub sync_repo: SyncStateRepository,
    /// Session start time
    pub started_at: Instant,
    /// Last activity time
    pub last_activity: Instant,
}

impl Clone for Session {
    fn clone(&self) -> Self {
        // Note: We create new database connections for the cloned session
        // since rusqlite::Connection doesn't implement Clone
        let conn_path = self.conn.path().expect("Failed to get connection path");
        let conn = Connection::open(conn_path).expect("Failed to clone connection");

        // Open separate connections for each repository
        let vault_conn = Connection::open(conn_path).expect("Failed to clone vault repo connection");
        let item_conn = Connection::open(conn_path).expect("Failed to clone item repo connection");
        let sync_conn = Connection::open(conn_path).expect("Failed to clone sync repo connection");

        Self {
            vault_id: self.vault_id,
            vault_name: self.vault_name.clone(),
            master_key: self.master_key.clone(),
            auth_key: self.auth_key.clone(),
            encryption_key: self.encryption_key.clone(),
            conn,
            vault_repo: VaultRepository::new(vault_conn),
            item_repo: ItemRepository::new(item_conn),
            sync_repo: SyncStateRepository::new(sync_conn),
            started_at: self.started_at,
            last_activity: self.last_activity,
        }
    }
}

impl Session {
    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        self.last_activity.elapsed() > DEFAULT_SESSION_TIMEOUT
    }

    /// Update last activity time
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Get vault ID as string
    pub fn vault_id_str(&self) -> String {
        self.vault_id.to_string()
    }
}

/// Session manager
#[derive(Clone)]
pub struct SessionManager {
    pub config: Config,
    inner: Arc<Mutex<SessionManagerInner>>,
}

#[derive(Default)]
struct SessionManagerInner {
    sessions: HashMap<VaultId, Session>,
    current_vault_id: Option<VaultId>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config: Config) -> Self {
        Self {
            config,
            inner: Arc::new(Mutex::new(SessionManagerInner::default())),
        }
    }

    /// Initialize a new vault
    ///
    /// # Errors
    /// Returns an error if initialization fails
    pub fn init_vault(
        &self,
        name: Option<String>,
        force: bool,
    ) -> Result<VaultId, CliError> {
        let crypto_backend = AesGcmBackend;

        // Generate vault ID
        let vault_id = Uuid::new_v4();
        let vault_dir = self.config.data_dir.join(vault_id.to_string());

        // Create vault directory
        std::fs::create_dir_all(&vault_dir).map_err(CliError::IoError)?;

        let vault_path = vault_dir.join(DEFAULT_VAULT_FILE);

        // Check if already initialized
        if vault_path.exists() && !force {
            return Err(CliError::AlreadyInitialized);
        }

        // Prompt for master password
        println!("Enter master password: ");
        let password = read_password().map_err(CliError::IoError)?;
        println!("Confirm master password: ");
        let password_confirm = read_password().map_err(CliError::IoError)?;

        if password != password_confirm {
            return Err(CliError::InvalidPassword);
        }

        // Generate salt
        let mut salt = vec![0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt);

        // Derive master key
        let master_key = crypto_backend
            .derive_master_key(&password, &salt)
            .map_err(|e| CliError::CryptoError(e.to_string()))?;

        // Derive encryption key
        let encryption_key = crypto_backend
            .derive_encryption_key(&master_key)
            .map_err(|e| CliError::CryptoError(e.to_string()))?;

        // Create vault metadata
        let vault_name = name.unwrap_or_else(|| "My Vault".to_string());

        // Initialize database
        let conn = Connection::open(&vault_path)
            .map_err(|e| CliError::StorageError(mls_storage::StorageError::SqliteError(e)))?;
        schema::init_schema(&conn)
            .map_err(|e| CliError::StorageError(mls_storage::StorageError::SqliteError(e)))?;

        let vault_repo = VaultRepository::new(conn);

        // Create vault in database
        vault_repo
            .create_vault(
                &vault_id.to_string(),
                &mls_core::models::VaultMetadata::new(vault_name),
                &encryption_key,
                &crypto_backend,
            )
            .map_err(CliError::StorageError)?;

        // Save salt to config
        let salt_hex = hex::encode(salt);
        let salt_path = vault_dir.join("salt.txt");
        std::fs::write(&salt_path, salt_hex).map_err(CliError::IoError)?;

        // Update default vault in config
        let mut config = self.config.clone();
        config.default_vault_id = Some(vault_id.to_string());
        config
            .save()
            .map_err(|e| CliError::IoError(std::io::Error::other(e)))?;

        // Auto-unlock the new vault
        let session = self.create_session(vault_id, &password, crypto_backend)?;
        self.set_current_session(vault_id, session);

        // Clear password from memory
        let _ = Zeroizing::new(password.into_bytes());

        Ok(vault_id)
    }

    /// Create a session for a vault
    fn create_session(
        &self,
        vault_id: VaultId,
        password: &str,
        crypto_backend: AesGcmBackend,
    ) -> Result<Session, CliError> {
        let vault_dir = self.config.data_dir.join(vault_id.to_string());
        let vault_path = vault_dir.join(DEFAULT_VAULT_FILE);

        // Read salt
        let salt_path = vault_dir.join("salt.txt");
        let salt_hex = std::fs::read_to_string(&salt_path)
            .map_err(|_| CliError::VaultNotInitialized)?;
        let salt = hex::decode(&salt_hex).map_err(|_| CliError::VaultNotInitialized)?;

        // Derive keys
        let master_key = crypto_backend
            .derive_master_key(password, &salt)
            .map_err(|e| CliError::CryptoError(e.to_string()))?;

        let auth_key = crypto_backend
            .derive_auth_key(&master_key)
            .map_err(|e| CliError::CryptoError(e.to_string()))?;

        let encryption_key = crypto_backend
            .derive_encryption_key(&master_key)
            .map_err(|e| CliError::CryptoError(e.to_string()))?;

        // Open database connections
        let vault_conn = Connection::open(&vault_path)
            .map_err(|e| CliError::StorageError(mls_storage::StorageError::SqliteError(e)))?;
        let item_conn = Connection::open(&vault_path)
            .map_err(|e| CliError::StorageError(mls_storage::StorageError::SqliteError(e)))?;
        let sync_conn = Connection::open(&vault_path)
            .map_err(|e| CliError::StorageError(mls_storage::StorageError::SqliteError(e)))?;
        let session_conn = Connection::open(&vault_path)
            .map_err(|e| CliError::StorageError(mls_storage::StorageError::SqliteError(e)))?;

        // Get vault metadata to get name
        let vault_repo = VaultRepository::new(vault_conn);
        let item_repo = ItemRepository::new(item_conn);
        let sync_repo = SyncStateRepository::new(sync_conn);

        let vault_metadata: mls_core::models::VaultMetadata = vault_repo
            .get_vault(&vault_id.to_string(), &encryption_key, &crypto_backend)
            .map_err(CliError::StorageError)?;

        Ok(Session {
            vault_id,
            vault_name: vault_metadata.name,
            master_key,
            auth_key,
            encryption_key,
            conn: session_conn,
            vault_repo,
            item_repo,
            sync_repo,
            started_at: Instant::now(),
            last_activity: Instant::now(),
        })
    }

    /// Set current session
    pub fn set_current_session(&self, vault_id: VaultId, session: Session) {
        let mut inner = self.inner.lock().unwrap();
        inner.sessions.insert(vault_id, session);
        inner.current_vault_id = Some(vault_id);
    }

    /// Get current session
    pub fn get_current_session(&self) -> Result<Session, CliError> {
        let inner = self.inner.lock().unwrap();
        let current_vault_id = inner.current_vault_id.ok_or(CliError::NotUnlocked)?;
        inner
            .sessions
            .get(&current_vault_id)
            .cloned()
            .ok_or(CliError::NotUnlocked)
    }

    /// Touch current session (update last activity)
    ///
    /// # Errors
    /// Returns an error if no session is unlocked
    pub fn touch_current_session(&self) -> Result<(), CliError> {
        let mut inner = self.inner.lock().unwrap();
        let current_vault_id = inner.current_vault_id.ok_or(CliError::NotUnlocked)?;
        inner
            .sessions
            .get_mut(&current_vault_id)
            .map_or_else(
                || Err(CliError::NotUnlocked),
                |session| {
                    session.touch();
                    Ok(())
                },
            )
    }

    /// Unlock a vault
    pub fn unlock_vault(
        &self,
        vault_id: Option<String>,
    ) -> Result<VaultId, CliError> {
        let crypto_backend = AesGcmBackend;

        // Determine which vault to unlock
        let vault_id = if let Some(id) = vault_id {
            Uuid::parse_str(&id).map_err(|_| CliError::InvalidPassword)?
        } else if let Some(default) = &self.config.default_vault_id {
            Uuid::parse_str(default).map_err(|_| CliError::InvalidPassword)?
        } else {
            return Err(CliError::VaultNotInitialized);
        };

        // Check if already unlocked
        {
            let inner = self.inner.lock().unwrap();
            if inner.sessions.contains_key(&vault_id) {
                return Err(CliError::AlreadyUnlocked);
            }
        }

        // Prompt for password
        println!("Enter master password: ");
        let password = read_password().map_err(CliError::IoError)?;

        // Create session
        let session = self.create_session(vault_id, &password, crypto_backend)?;

        // Set as current session
        self.set_current_session(vault_id, session);

        // Clear password from memory
        let _ = Zeroizing::new(password.into_bytes());

        Ok(vault_id)
    }

    /// Lock current session
    pub fn lock_session(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.current_vault_id = None;
        drop(inner);
    }

    /// Lock all sessions
    pub fn lock_all(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.sessions.clear();
        inner.current_vault_id = None;
        drop(inner);
    }

    /// Check if unlocked
    pub fn is_unlocked(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.current_vault_id.is_some()
    }

    /// Get current vault ID
    pub fn current_vault_id(&self) -> Option<VaultId> {
        let inner = self.inner.lock().unwrap();
        inner.current_vault_id
    }

    /// Add an item to the current vault
    pub fn add_item(
        &self,
        name: String,
        item_type: ItemType,
        service: Option<String>,
        username: Option<String>,
        url: Option<String>,
        content: Option<String>,
    ) -> Result<ItemId, CliError> {
        let session = self.get_current_session()?;
        let crypto_backend = AesGcmBackend;

        // Prepare item data based on type
        let data = match item_type {
            ItemType::Password => {
                let password = content.ok_or(CliError::UserCancelled)?;
                ItemData::Password(mls_core::models::PasswordData {
                    service: service.unwrap_or_default(),
                    username,
                    password,
                    url,
                    notes: None,
                })
            }
            ItemType::Note => {
                let note_content = content.unwrap_or_default();
                ItemData::Note(mls_core::models::NoteData {
                    title: Some(name.clone()),
                    content: note_content,
                    tags: Vec::new(),
                })
            }
            ItemType::Generic => {
                ItemData::Generic(mls_core::models::GenericData::new())
            }
        };

        // Create item
        let item_id = Uuid::new_v4();
        let item = Item::new(session.vault_id, name, item_type, data);

        session.item_repo.create_item(
            &session.vault_id_str(),
            &item_id.to_string(),
            &item,
            1,
            &session.encryption_key,
            &crypto_backend,
        ).map_err(CliError::StorageError)?;

        // Update session activity
        self.touch_current_session()?;

        Ok(item_id)
    }

    /// Get an item from the current vault
    pub fn get_item(&self, name_or_id: String) -> Result<Item, CliError> {
        let session = self.get_current_session()?;
        let crypto_backend = AesGcmBackend;

        // Try to parse as UUID first
        if let Ok(item_id) = Uuid::parse_str(&name_or_id) {
            // Get item by ID
            let (metadata, _version): (ItemMetadata, i64) = session
                .item_repo
                .get_item(&item_id.to_string(), &session.encryption_key, &crypto_backend)
                .map_err(CliError::StorageError)?;

            // For now, return with placeholder data
            return Ok(Item::new(
                session.vault_id,
                metadata.name.clone(),
                metadata.item_type,
                ItemData::Generic(mls_core::models::GenericData::new()),
            ));
        }

        // Otherwise, search by name
        let items = session.item_repo.list_items(&session.vault_id_str())
            .map_err(CliError::StorageError)?;

        for (found_id, _version) in items {
            let (metadata, _): (ItemMetadata, i64) = session
                .item_repo
                .get_item(&found_id, &session.encryption_key, &crypto_backend)
                .map_err(CliError::StorageError)?;

            if metadata.name == name_or_id {
                return Ok(Item::new(
                    session.vault_id,
                    metadata.name.clone(),
                    metadata.item_type,
                    ItemData::Generic(mls_core::models::GenericData::new()),
                ));
            }
        }

        Err(CliError::ItemNotFound(name_or_id))
    }

    /// List items in the current vault
    pub fn list_items(
        &self,
        item_type: Option<&ItemType>,
        search: Option<&String>,
        limit: Option<usize>,
    ) -> Result<Vec<Item>, CliError> {
        let session = self.get_current_session()?;
        let crypto_backend = AesGcmBackend;

        let items = session.item_repo.list_items(&session.vault_id_str())
            .map_err(CliError::StorageError)?;

        let mut results = Vec::new();
        for (item_id, _version) in &items {
            let (metadata, _): (ItemMetadata, i64) = session
                .item_repo
                .get_item(item_id, &session.encryption_key, &crypto_backend)
                .map_err(CliError::StorageError)?;

            // Filter by type if specified
            if let Some(filter_type) = item_type {
                if metadata.item_type != *filter_type {
                    continue;
                }
            }

            // Filter by search if specified
            if let Some(search_term) = search {
                if !metadata.name.contains(search_term) {
                    continue;
                }
            }

            results.push(Item::new(
                session.vault_id,
                metadata.name.clone(),
                metadata.item_type,
                ItemData::Generic(mls_core::models::GenericData::new()),
            ));

            if let Some(lim) = limit {
                if results.len() >= lim {
                    break;
                }
            }
        }

        Ok(results)
    }

    /// Remove an item from the current vault
    pub fn remove_item(&self, name_or_id: String, force: bool) -> Result<(), CliError> {
        if !force {
            println!("Remove item '{name_or_id}'? [y/N]: ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).map_err(CliError::IoError)?;
            let answer = input.trim().to_lowercase();

            if answer != "y" && answer != "yes" {
                return Err(CliError::UserCancelled);
            }
        }

        let session = self.get_current_session()?;

        // Try to parse as UUID first
        if let Ok(item_id) = Uuid::parse_str(&name_or_id) {
            session.item_repo.delete_item(&item_id.to_string())
                .map_err(CliError::StorageError)?;
            return Ok(());
        }

        // Otherwise, search by name
        let items = session.item_repo.list_items(&session.vault_id_str())
            .map_err(CliError::StorageError)?;

        for (found_id, _) in items {
            let (metadata, _): (ItemMetadata, i64) = session
                .item_repo
                .get_item(&found_id, &session.encryption_key, &AesGcmBackend)
                .map_err(CliError::StorageError)?;

            if metadata.name == name_or_id {
                session.item_repo.delete_item(&found_id)
                    .map_err(CliError::StorageError)?;
                return Ok(());
            }
        }

        Err(CliError::ItemNotFound(name_or_id))
    }

    /// Change master password (placeholder implementation)
    pub fn change_password() {
        // For now, this is a placeholder
        // In a full implementation, we'd:
        // 1. Verify current password
        // 2. Prompt for new password
        // 3. Re-encrypt all data with new key

        println!("Password change not implemented yet.");
        println!("This would require re-encrypting all data.");
    }
}
