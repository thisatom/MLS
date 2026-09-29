//! Tauri commands for MLS
//!
//! This module defines the Tauri commands that bridge between the frontend and backend.
//!
//! All commands are exposed to the TypeScript frontend via the `@tauri-apps/api` package.

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Manager;
use uuid::Uuid;

/// Session manager for the Tauri application
/// Manages vault sessions and provides data access
struct SessionManager {
    // In a real implementation, this would use mls-core, mls-crypto, mls-storage
    // For now, we use a simple in-memory store for demonstration
    current_vault: Option<Vault>,
    vaults: HashMap<Uuid, Vault>,
}

/// Vault representation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Vault {
    id: Uuid,
    name: String,
    items: HashMap<Uuid, Item>,
    created_at: String,
}

/// Item representation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Item {
    #[serde(rename = "password")]
    Password {
        id: Uuid,
        name: String,
        service: String,
        username: Option<String>,
        password: String,
        url: Option<String>,
        notes: Option<String>,
        updated_at: String,
    },
    #[serde(rename = "note")]
    Note {
        id: Uuid,
        name: String,
        title: Option<String>,
        content: String,
        updated_at: String,
    },
    #[serde(rename = "generic")]
    Generic {
        id: Uuid,
        name: String,
        data: String,
        updated_at: String,
    },
}

impl SessionManager {
    fn new() -> Self {
        Self {
            current_vault: None,
            vaults: HashMap::new(),
        }
    }

    /// Initialize a new vault
    fn init_vault(&mut self, name: Option<String>, _master_password: String) -> Result<VaultInfo, String> {
        // In production: derive keys from master_password using mls-crypto
        // For now, just create a vault
        
        let vault_id = Uuid::new_v4();
        let vault_name = name.unwrap_or_else(|| "My Vault".to_string());
        let created_at = chrono::Utc::now().to_rfc3339();

        let vault = Vault {
            id: vault_id,
            name: vault_name.clone(),
            items: HashMap::new(),
            created_at: created_at.clone(),
        };

        self.vaults.insert(vault_id, vault);
        self.current_vault = Some(vault_id);

        Ok(VaultInfo {
            vault_id: vault_id.to_string(),
            vault_name,
            created_at,
        })
    }

    /// Unlock a vault
    fn unlock(&mut self, vault_id: Option<Uuid>, _master_password: String) -> Result<VaultInfo, String> {
        // In production: verify password using mls-crypto
        // For now, just set the current vault
        
        let vault_id = vault_id.unwrap_or_else(|| {
            self.vaults.keys().next().copied().ok_or_else(|| {
                // No vaults exist
                Uuid::nil()
            })
        });

        let vault = self.vaults.get(&vault_id).ok_or("Vault not found")?;
        self.current_vault = Some(vault_id);

        Ok(VaultInfo {
            vault_id: vault.id.to_string(),
            vault_name: vault.name.clone(),
            created_at: vault.created_at.clone(),
        })
    }

    /// Lock the current vault
    fn lock(&mut self) -> Result<bool, String> {
        self.current_vault = None;
        Ok(true)
    }

    /// Get session status
    fn get_session_status(&self) -> SessionStatusResult {
        if let Some(vault_id) = self.current_vault {
            if let Some(vault) = self.vaults.get(&vault_id) {
                SessionStatusResult {
                    status: "unlocked".to_string(),
                    vault_id: Some(vault.id.to_string()),
                    vault_name: Some(vault.name.clone()),
                }
            } else {
                SessionStatusResult {
                    status: "unlocked".to_string(),
                    vault_id: None,
                    vault_name: None,
                }
            }
        } else {
            SessionStatusResult {
                status: "locked".to_string(),
                vault_id: None,
                vault_name: None,
            }
        }
    }

    /// List all vaults
    fn list_vaults(&self) -> ListVaultsResult {
        ListVaultsResult {
            vaults: self.vaults.values().map(|v| VaultInfo {
                vault_id: v.id.to_string(),
                vault_name: v.name.clone(),
                created_at: v.created_at.clone(),
            }).collect(),
        }
    }

    /// Add a new item
    fn add_item(&mut self, item_data: AddItemParams) -> Result<AddItemResult, String> {
        let vault_id = self.current_vault.ok_or("No vault unlocked")?;
        let vault = self.vaults.get_mut(&vault_id).ok_or("Vault not found")?;

        let item_id = Uuid::new_v4();
        let updated_at = chrono::Utc::now().to_rfc3339();

        let item = match item_data.item_type {
            "password" => Item::Password {
                id: item_id,
                name: item_data.name,
                service: item_data.service.unwrap_or_default(),
                username: item_data.username,
                password: item_data.password.unwrap_or_default(),
                url: item_data.url,
                notes: item_data.notes,
                updated_at: updated_at.clone(),
            },
            "note" => Item::Note {
                id: item_id,
                name: item_data.name,
                title: None,
                content: item_data.content.unwrap_or_default(),
                updated_at: updated_at.clone(),
            },
            "generic" => Item::Generic {
                id: item_id,
                name: item_data.name,
                data: item_data.content.unwrap_or_default(),
                updated_at: updated_at.clone(),
            },
            _ => return Err(format!("Unknown item type: {}", item_data.item_type)),
        };

        vault.items.insert(item_id, item);

        Ok(AddItemResult {
            item_id: item_id.to_string(),
        })
    }

    /// Get an item
    fn get_item(&self, item_id: Uuid) -> Result<GetItemResult, String> {
        let vault_id = self.current_vault.ok_or("No vault unlocked")?;
        let vault = self.vaults.get(&vault_id).ok_or("Vault not found")?;

        let item = vault.items.get(&item_id).ok_or("Item not found")?;

        Ok(GetItemResult { item: item.clone() })
    }

    /// List items
    fn list_items(&self, params: ListItemsParams) -> Result<ListItemsResult, String> {
        let vault_id = self.current_vault.ok_or("No vault unlocked")?;
        let vault = self.vaults.get(&vault_id).ok_or("Vault not found")?;

        let mut items: Vec<Item> = vault.items.values().cloned().collect();

        // Filter by type
        if let Some(item_type) = params.item_type {
            items.retain(|item| {
                match (item, item_type.as_str()) {
                    (Item::Password { .. }, "password") => true,
                    (Item::Note { .. }, "note") => true,
                    (Item::Generic { .. }, "generic") => true,
                    _ => false,
                }
            });
        }

        // Filter by search
        if let Some(search) = params.search {
            let search_lower = search.to_lowercase();
            items.retain(|item| {
                let name = match item {
                    Item::Password { name, .. } => name.to_lowercase(),
                    Item::Note { name, .. } => name.to_lowercase(),
                    Item::Generic { name, .. } => name.to_lowercase(),
                };
                name.contains(&search_lower)
            });
        }

        // Apply limit
        if let Some(limit) = params.limit {
            items.truncate(limit);
        }

        Ok(ListItemsResult { items })
    }

    /// Remove an item
    fn remove_item(&mut self, item_id: Uuid, _force: bool) -> Result<RemoveItemResult, String> {
        let vault_id = self.current_vault.ok_or("No vault unlocked")?;
        let vault = self.vaults.get_mut(&vault_id).ok_or("Vault not found")?;

        vault.items.remove(&item_id);

        Ok(RemoveItemResult { success: true })
    }

    /// Get sync status
    fn get_sync_status(&self) -> SyncStatusResult {
        SyncStatusResult {
            is_syncing: false,
            last_sync_time: None,
            connected_devices: 0,
        }
    }
}

// Tauri command input/output types

#[derive(Debug, Serialize, Deserialize)]
pub struct InitVaultParams {
    pub name: Option<String>,
    pub master_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultInfo {
    pub vault_id: String,
    pub vault_name: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitVaultResult {
    pub vault_id: String,
    pub vault_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnlockParams {
    pub vault_id: Option<Uuid>,
    pub master_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnlockResult {
    pub vault_id: String,
    pub vault_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockResult {
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStatusResult {
    pub status: String,
    pub vault_id: Option<String>,
    pub vault_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListVaultsResult {
    pub vaults: Vec<VaultInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddItemParams {
    pub item_type: String,
    pub name: String,
    pub service: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub content: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddItemResult {
    pub item_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetItemParams {
    pub item_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetItemResult {
    pub item: Item,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListItemsParams {
    pub item_type: Option<String>,
    pub search: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListItemsResult {
    pub items: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveItemParams {
    pub item_id: String,
    pub force: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveItemResult {
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncStatusResult {
    pub is_syncing: bool,
    pub last_sync_time: Option<String>,
    pub connected_devices: usize,
}

// Tauri commands

#[tauri::command]
fn init_vault(state: tauri::State<Arc<std::sync::Mutex<SessionManager>>>, params: InitVaultParams) -> Result<InitVaultResult, String> {
    let mut manager = state.lock().unwrap();
    manager.init_vault(params.name, params.master_password)
}

#[tauri::command]
fn unlock(state: tauri::State<Arc<std::sync::Mutex<SessionManager>>>, params: UnlockParams) -> Result<UnlockResult, String> {
    let mut manager = state.lock().unwrap();
    manager.unlock(params.vault_id, params.master_password)
}

#[tauri::command]
fn lock(state: tauri::State<Arc<std::sync::Mutex<SessionManager>>>) -> Result<LockResult, String> {
    let mut manager = state.lock().unwrap();
    manager.lock()
}

#[tauri::command]
fn get_session_status(state: tauri::State<Arc<std::sync::Mutex<SessionManager>>>) -> SessionStatusResult {
    let manager = state.lock().unwrap();
    manager.get_session_status()
}

#[tauri::command]
fn list_vaults(state: tauri::State<Arc<std::sync::Mutex<SessionManager>>>) -> ListVaultsResult {
    let manager = state.lock().unwrap();
    manager.list_vaults()
}

#[tauri::command]
fn add_item(state: tauri::State<Arc<std::sync::Mutex<SessionManager>>>, params: AddItemParams) -> Result<AddItemResult, String> {
    let mut manager = state.lock().unwrap();
    manager.add_item(params)
}

#[tauri::command]
fn get_item(state: tauri::State<Arc<std::sync::Mutex<SessionManager>>>, params: GetItemParams) -> Result<GetItemResult, String> {
    let manager = state.lock().unwrap();
    let item_id: Uuid = params.item_id.parse().map_err(|_| "Invalid item ID")?;
    manager.get_item(item_id)
}

#[tauri::command]
fn list_items(state: tauri::State<Arc<std::sync::Mutex<SessionManager>>>, params: ListItemsParams) -> Result<ListItemsResult, String> {
    let manager = state.lock().unwrap();
    manager.list_items(params)
}

#[tauri::command]
fn remove_item(state: tauri::State<Arc<std::sync::Mutex<SessionManager>>>, params: RemoveItemParams) -> Result<RemoveItemResult, String> {
    let mut manager = state.lock().unwrap();
    let item_id: Uuid = params.item_id.parse().map_err(|_| "Invalid item ID")?;
    manager.remove_item(item_id, params.force.unwrap_or(false))
}

#[tauri::command]
fn get_sync_status(state: tauri::State<Arc<std::sync::Mutex<SessionManager>>>) -> SyncStatusResult {
    let manager = state.lock().unwrap();
    manager.get_sync_status()
}

// Main Tauri application entry point
pub fn run() {
    let session_manager = Arc::new(std::sync::Mutex::new(SessionManager::new()));

    tauri::Builder::default()
        .manage(session_manager.clone())
        .invoke_handler(tauri::generate_handler![
            init_vault,
            unlock,
            lock,
            get_session_status,
            list_vaults,
            add_item,
            get_item,
            list_items,
            remove_item,
            get_sync_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
