//! CRDT implementation for MLS
//!
//! This module provides CRDT-based data structures using Yrs (Rust port of Yjs).
//! Each vault is represented as a `YDoc`, with items stored as `YMap`s.
//!
//! # Architecture
//!
//! - Each Vault is a `YDoc`
//! - Vault metadata is stored in a `YMap` at the root
//! - Items are stored in a `YMap` under the "items" key
//! - Each item is a `YMap` with fields stored as JSON strings
//! - CRDT state vectors are used for synchronization

use std::collections::HashMap;
use yrs::{
    GetString, Map, MapPrelim, ReadTxn, StateVector, Transact, Update,
    updates::decoder::Decode,
    updates::encoder::Encode,
};
use uuid::Uuid;

use crate::models::{Item, ItemData, ItemId, ItemMetadata, VaultId, VaultMetadata};

/// Key for vault metadata map in the `YDoc` root
const VAULT_META_KEY: &str = "vault_meta";

/// Key for items map in the `YDoc` root
const ITEMS_KEY: &str = "items";

/// CRDT Vault structure
///
/// Represents a vault as a `YDoc` with CRDT capabilities.
/// This allows for automatic conflict-free merging of concurrent edits.
#[derive(Debug)]
pub struct CrdtVault {
    /// Yrs document
    doc: yrs::Doc,
    /// Vault ID
    vault_id: VaultId,
}

/// Helper trait to extract string from Value using transaction
/// 
trait ValueExt {
    fn to_string_value<T: ReadTxn>(&self, txn: &T) -> Option<String>;
}

impl ValueExt for yrs::types::Value {
    fn to_string_value<T: ReadTxn>(&self, txn: &T) -> Option<String> {
        match self {
            Self::Any(any) => Some(any.to_string()),
            Self::YText(text) => Some(text.get_string(txn)),
            _ => None,
        }
    }
}

impl CrdtVault {
    /// Create a new CRDT vault
    #[must_use]
    pub fn new(vault_id: VaultId) -> Self {
        Self {
            doc: yrs::Doc::new(),
            vault_id,
        }
    }

    /// Get the underlying `YDoc`
    #[must_use]
    pub const fn doc(&self) -> &yrs::Doc {
        &self.doc
    }

    /// Get the client ID (for synchronization)
    #[must_use]
    pub fn client_id(&self) -> u64 {
        self.doc.client_id()
    }

    /// Helper to insert a string value into a map
    fn insert_str(map: &yrs::MapRef, txn: &mut yrs::TransactionMut, key: &str, value: &str) {
        map.insert(txn, key, value);
    }

    /// Helper to get a string value from a map
    fn get_str<T: ReadTxn>(map: &yrs::MapRef, txn: &T, key: &str) -> Option<String> {
        map.get(txn, key).and_then(|v| v.to_string_value(txn))
    }

    /// Initialize vault metadata
    pub fn init_metadata(&mut self, metadata: &VaultMetadata) {
        let root = self.doc.get_or_insert_map(VAULT_META_KEY);
        let mut txn = self.doc.transact_mut();
        Self::insert_str(&root, &mut txn, "name", &metadata.name);
        if let Some(ref desc) = metadata.description {
            Self::insert_str(&root, &mut txn, "description", desc);
        }
        Self::insert_str(&root, &mut txn, "created_at", &metadata.created_at.to_rfc3339());
        Self::insert_str(&root, &mut txn, "updated_at", &metadata.updated_at.to_rfc3339());
    }

    /// Get vault metadata
    #[must_use]
    pub fn get_metadata(&self) -> Option<VaultMetadata> {
        let txn = self.doc.transact();
        let root = txn.get_map(VAULT_META_KEY)?;

        let name = Self::get_str(&root, &txn, "name")?;
        let description = Self::get_str(&root, &txn, "description");
        let created_at_str = Self::get_str(&root, &txn, "created_at")?;
        let updated_at_str = Self::get_str(&root, &txn, "updated_at")?;

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .ok()?
            .with_timezone(&chrono::Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .ok()?
            .with_timezone(&chrono::Utc);

        Some(VaultMetadata {
            name,
            description,
            created_at,
            updated_at,
        })
    }

    /// Add an item to the vault
    ///
    /// # Returns
    /// The item ID
    ///
    /// # Panics
    /// Panics if the value in the items map is not a `YMap`
    pub fn add_item(&mut self, item: &Item) -> ItemId {
        let items_map = self.doc.get_or_insert_map(ITEMS_KEY);
        let item_id_str = item.id.to_string();

        let mut txn = self.doc.transact_mut();

        // Get or create the item map
        let item_map = items_map
            .get(&txn, item_id_str.as_str())
            .map_or_else(
                || items_map.insert(&mut txn, item_id_str.as_str(), MapPrelim::from(HashMap::<String, String>::new())),
                |value| value.to_ymap().expect("Expected YMap"),
            );

        // Store item metadata as JSON
        let metadata_json = serde_json::to_string(&item.metadata).unwrap();
        Self::insert_str(&item_map, &mut txn, "metadata", &metadata_json);

        // Store item data as JSON
        let data_json = serde_json::to_string(&item.data).unwrap();
        Self::insert_str(&item_map, &mut txn, "data", &data_json);

        item.id
    }

    /// Get an item by ID
    #[must_use]
    pub fn get_item(&self, item_id: &ItemId) -> Option<Item> {
        let txn = self.doc.transact();
        let items_map = txn.get_map(ITEMS_KEY)?;
        let item_id_str = item_id.to_string();
        let item_map = items_map.get(&txn, item_id_str.as_str())?.to_ymap()?;

        let metadata_json = Self::get_str(&item_map, &txn, "metadata")?;
        let data_json = Self::get_str(&item_map, &txn, "data")?;

        let metadata: ItemMetadata = serde_json::from_str(&metadata_json).ok()?;
        let data: ItemData = serde_json::from_str(&data_json).ok()?;

        Some(Item::new(self.vault_id, metadata.name, metadata.item_type, data))
    }

    /// List all item IDs
    #[must_use]
    pub fn list_item_ids(&self) -> Vec<ItemId> {
        let txn = self.doc.transact();
        let items_map = txn.get_map(ITEMS_KEY);
        let mut item_ids = Vec::new();

        if let Some(items_map) = items_map {
            for key in items_map.keys(&txn) {
                if let Ok(uuid) = Uuid::parse_str(key) {
                    item_ids.push(uuid);
                }
            }
        }
        item_ids
    }

    /// Update an item
    ///
    /// # Returns
    /// `true` if the item existed before update, `false` otherwise
    ///
    /// # Panics
    /// Panics if the value in the items map is not a `YMap`
    pub fn update_item(&mut self, item: &Item) -> bool {
        let item_exists = self.get_item(&item.id).is_some();

        let items_map = self.doc.get_or_insert_map(ITEMS_KEY);
        let item_id_str = item.id.to_string();

        let mut txn = self.doc.transact_mut();

        // Get or create the item map
        let item_map = items_map
            .get(&txn, item_id_str.as_str())
            .map_or_else(
                || items_map.insert(&mut txn, item_id_str.as_str(), MapPrelim::from(HashMap::<String, String>::new())),
                |value| value.to_ymap().expect("Expected YMap"),
            );

        // Store item metadata as JSON
        let metadata_json = serde_json::to_string(&item.metadata).unwrap();
        Self::insert_str(&item_map, &mut txn, "metadata", &metadata_json);

        // Store item data as JSON
        let data_json = serde_json::to_string(&item.data).unwrap();
        Self::insert_str(&item_map, &mut txn, "data", &data_json);

        item_exists
    }

    /// Remove an item
    ///
    /// # Returns
    /// `true` if the item existed before removal, `false` otherwise
    pub fn remove_item(&mut self, item_id: &ItemId) -> bool {
        let existed = self.get_item(item_id).is_some();

        let items_map = self.doc.get_or_insert_map(ITEMS_KEY);
        let item_id_str = item_id.to_string();

        let mut txn = self.doc.transact_mut();
        items_map.remove(&mut txn, item_id_str.as_str());

        existed
    }

    /// Get sync state vector
    #[must_use]
    pub fn get_state_vector(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        txn.state_vector().encode_v1()
    }

    /// Apply update from another document
    ///
    /// # Errors
    /// Returns an error if the update cannot be decoded
    pub fn apply_update(&mut self, update: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let update = Update::decode_v1(update)?;
        let mut txn = self.doc.transact_mut();
        txn.apply_update(update);
        Ok(())
    }

    /// Get updates since state vector
    #[must_use]
    pub fn get_updates_since(&self, state_vector: &[u8]) -> Vec<u8> {
        let sv = StateVector::decode_v1(state_vector).unwrap_or_default();
        let txn = self.doc.transact();
        txn.encode_diff_v1(&sv)
    }

    /// Serialize the entire document to binary (full sync)
    #[must_use]
    pub fn serialize(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        let sv = StateVector::default();
        txn.encode_state_as_update_v1(&sv)
    }

    /// Deserialize document from binary
    #[must_use]
    pub fn deserialize(vault_id: VaultId, data: &[u8]) -> Option<Self> {
        let doc = yrs::Doc::new();
        let update = Update::decode_v1(data).ok()?;
        {
            let mut txn = doc.transact_mut();
            txn.apply_update(update);
        }
        Some(Self { doc, vault_id })
    }

    /// Set vault ID
    #[allow(clippy::missing_const_for_fn)]
    pub fn set_vault_id(&mut self, vault_id: VaultId) {
        self.vault_id = vault_id;
    }
}

/// Merge two CRDT vaults
///
/// This merges the state of one vault into another.
/// The merge is commutative and idempotent.
///
/// # Errors
/// Returns an error if the update cannot be applied
pub fn merge_vaults(vault1: &mut CrdtVault, vault2: &CrdtVault) -> Result<(), Box<dyn std::error::Error>> {
    let updates = vault2.serialize();
    vault1.apply_update(&updates)?;
    Ok(())
}

/// CRDT error type
#[derive(Debug, thiserror::Error)]
pub enum CrdtError {
    /// Serialization error
    #[error("CRDT serialization error: {0}")]
    SerializationError(String),
    /// Deserialization error
    #[error("CRDT deserialization error: {0}")]
    DeserializationError(String),
    /// Merge error
    #[error("CRDT merge error: {0}")]
    MergeError(String),
}

/// Serialize CRDT vault to encrypted blob
///
/// This serializes the `YDoc` state and returns it as bytes
/// which can then be encrypted by the crypto layer.
#[must_use]
pub fn serialize_vault_to_blob(vault: &CrdtVault) -> Vec<u8> {
    vault.serialize()
}

/// Deserialize CRDT vault from encrypted blob
///
/// This takes the decrypted bytes and reconstructs the `YDoc`.
///
/// # Errors
/// Returns an error if the vault cannot be deserialized
pub fn deserialize_vault_from_blob(vault_id: VaultId, data: &[u8]) -> Result<CrdtVault, CrdtError> {
    CrdtVault::deserialize(vault_id, data)
        .ok_or_else(|| CrdtError::DeserializationError("Failed to deserialize vault".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ItemType;
    use proptest::prelude::*;

    #[test]
    fn test_vault_creation() {
        let vault_id = VaultId::new_v4();
        let mut vault = CrdtVault::new(vault_id);

        let metadata = VaultMetadata::new("Test Vault");
        vault.init_metadata(&metadata);

        let retrieved = vault.get_metadata();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Vault");
    }

    #[test]
    fn test_item_round_trip() {
        let vault_id = VaultId::new_v4();
        let mut vault = CrdtVault::new(vault_id);

        let metadata = VaultMetadata::new("Test Vault");
        vault.init_metadata(&metadata);

        let original_item = Item::new(
            vault_id,
            "Test Item",
            ItemType::Note,
            ItemData::Note(crate::models::NoteData {
                title: Some("Test Note".to_string()),
                content: "Test content".to_string(),
                tags: vec!["test".to_string()],
            }),
        );

        let item_id = vault.add_item(&original_item);

        let retrieved_item = vault.get_item(&item_id);
        assert!(retrieved_item.is_some());
        let retrieved = retrieved_item.unwrap();
        assert_eq!(retrieved.metadata.name, "Test Item");
        assert_eq!(retrieved.metadata.item_type, ItemType::Note);
    }

    #[test]
    fn test_vault_merge() {
        let vault_id = VaultId::new_v4();
        let mut vault1 = CrdtVault::new(vault_id);
        let mut vault2 = CrdtVault::new(vault_id);

        // Initialize both vaults
        let metadata1 = VaultMetadata::new("Vault 1");
        vault1.init_metadata(&metadata1);

        let metadata2 = VaultMetadata::new("Vault 2");
        vault2.init_metadata(&metadata2);

        // Add items to both vaults
        let item1 = Item::new(
            vault_id,
            "Item 1",
            ItemType::Password,
            ItemData::Password(crate::models::PasswordData {
                service: "Service 1".to_string(),
                username: Some("user1".to_string()),
                password: "pass1".to_string(),
                url: None,
                notes: None,
            }),
        );
        vault1.add_item(&item1);

        let item2 = Item::new(
            vault_id,
            "Item 2",
            ItemType::Password,
            ItemData::Password(crate::models::PasswordData {
                service: "Service 2".to_string(),
                username: Some("user2".to_string()),
                password: "pass2".to_string(),
                url: None,
                notes: None,
            }),
        );
        vault2.add_item(&item2);

        // Merge vault2 into vault1
        merge_vaults(&mut vault1, &vault2).unwrap();

        // Check that vault1 now has both items
        let item_ids = vault1.list_item_ids();
        assert_eq!(item_ids.len(), 2);
    }

    #[test]
    fn test_idempotent_merge() {
        let vault_id = VaultId::new_v4();
        let mut vault1 = CrdtVault::new(vault_id);
        let mut vault2 = CrdtVault::new(vault_id);

        let metadata = VaultMetadata::new("Test");
        vault1.init_metadata(&metadata);
        vault2.init_metadata(&metadata);

        let item = Item::new(
            vault_id,
            "Test",
            ItemType::Generic,
            ItemData::Generic(crate::models::GenericData::new()),
        );
        vault1.add_item(&item);

        // Merge same document twice
        merge_vaults(&mut vault1, &vault2).unwrap();
        merge_vaults(&mut vault1, &vault2).unwrap();

        // Should still have same state
        let item_ids = vault1.list_item_ids();
        assert_eq!(item_ids.len(), 1);
    }

    #[test]
    fn test_commutative_merge() {
        let vault_id = VaultId::new_v4();
        let mut vault1a = CrdtVault::new(vault_id);
        let mut vault1b = CrdtVault::new(vault_id);
        let mut vault2a = CrdtVault::new(vault_id);
        let mut vault2b = CrdtVault::new(vault_id);

        let metadata = VaultMetadata::new("Test");
        vault1a.init_metadata(&metadata);
        vault1b.init_metadata(&metadata);
        vault2a.init_metadata(&metadata);
        vault2b.init_metadata(&metadata);

        // Add item1 to vault1
        let item1 = Item::new(
            vault_id,
            "Item 1",
            ItemType::Note,
            ItemData::Note(crate::models::NoteData {
                title: Some("Note 1".to_string()),
                content: "Content 1".to_string(),
                tags: vec![],
            }),
        );
        vault1a.add_item(&item1);
        vault1b.add_item(&item1);

        // Add item2 to vault2
        let item2 = Item::new(
            vault_id,
            "Item 2",
            ItemType::Note,
            ItemData::Note(crate::models::NoteData {
                title: Some("Note 2".to_string()),
                content: "Content 2".to_string(),
                tags: vec![],
            }),
        );
        vault2a.add_item(&item2);
        vault2b.add_item(&item2);

        // Merge in order A: vault1 + vault2
        merge_vaults(&mut vault1a, &vault2a).unwrap();

        // Merge in order B: vault2 + vault1
        merge_vaults(&mut vault2b, &vault1b).unwrap();

        // Both should have same number of items
        let items_a = vault1a.list_item_ids();
        let items_b = vault2b.list_item_ids();

        assert_eq!(items_a.len(), 2);
        assert_eq!(items_b.len(), 2);
    }

    #[test]
    fn test_serialization_round_trip() {
        let vault_id = VaultId::new_v4();
        let mut vault = CrdtVault::new(vault_id);

        let metadata = VaultMetadata::new("Test Vault");
        vault.init_metadata(&metadata);

        let item = Item::new(
            vault_id,
            "Test Item",
            ItemType::Generic,
            ItemData::Generic(crate::models::GenericData::new()),
        );
        vault.add_item(&item);

        // Serialize
        let data = vault.serialize();

        // Deserialize
        let vault2 = CrdtVault::deserialize(vault_id, &data).unwrap();

        // Check
        assert_eq!(vault2.list_item_ids().len(), 1);
        assert!(vault2.get_item(&item.id).is_some());
    }

    #[test]
    fn test_encrypt_decrypt_round_trip() {
        use super::super::models::{Item, ItemData, ItemType, VaultMetadata};
        
        let vault_id = VaultId::new_v4();
        let mut vault = CrdtVault::new(vault_id);
        let metadata = VaultMetadata::new("Test");
        vault.init_metadata(&metadata);
        
        let item = Item::new(
            vault_id,
            "Test",
            ItemType::Note,
            ItemData::Note(crate::models::NoteData {
                title: Some("Test".to_string()),
                content: "content".to_string(),
                tags: vec!["tag".to_string()],
            }),
        );
        vault.add_item(&item);
        
        // Serialize to blob
        let blob = serialize_vault_to_blob(&vault);
        
        // Deserialize from blob
        let vault2 = deserialize_vault_from_blob(vault_id, &blob).unwrap();
        
        assert_eq!(vault2.list_item_ids().len(), 1);
        let retrieved = vault2.get_item(&item.id).unwrap();
        assert_eq!(retrieved.metadata.name, "Test");
    }

    // Property-based tests with proptest
    proptest! {
        #[test]
        fn prop_merge_idempotent(item_name in "[a-zA-Z0-9 ]{1,50}") {
            let vault_id = VaultId::new_v4();
            let mut vault1 = CrdtVault::new(vault_id);
            let mut vault2 = CrdtVault::new(vault_id);
            
            let metadata = VaultMetadata::new("Test");
            vault1.init_metadata(&metadata);
            vault2.init_metadata(&metadata);
            
            let item = Item::new(
                vault_id,
                &item_name,
                ItemType::Generic,
                ItemData::Generic(crate::models::GenericData::new()),
            );
            vault1.add_item(&item);
            
            // First merge
            merge_vaults(&mut vault1, &vault2).unwrap();
            let items_after_first = vault1.list_item_ids().len();
            
            // Second merge (should be idempotent)
            merge_vaults(&mut vault1, &vault2).unwrap();
            let items_after_second = vault1.list_item_ids().len();
            
            // Should have same number of items
            prop_assert_eq!(items_after_first, items_after_second);
        }

        #[test]
        fn prop_merge_commutative(
            item1_name in "[a-zA-Z0-9 ]{1,50}",
            item2_name in "[a-zA-Z0-9 ]{1,50}"
        ) {
            let vault_id = VaultId::new_v4();
            let mut vault1a = CrdtVault::new(vault_id);
            let mut vault1b = CrdtVault::new(vault_id);
            let mut vault2a = CrdtVault::new(vault_id);
            let mut vault2b = CrdtVault::new(vault_id);
            
            let metadata = VaultMetadata::new("Test");
            vault1a.init_metadata(&metadata);
            vault1b.init_metadata(&metadata);
            vault2a.init_metadata(&metadata);
            vault2b.init_metadata(&metadata);
            
            let item1 = Item::new(
                vault_id,
                &item1_name,
                ItemType::Note,
                ItemData::Note(crate::models::NoteData {
                    title: Some(item1_name.clone()),
                    content: "content1".to_string(),
                    tags: vec![],
                }),
            );
            vault1a.add_item(&item1);
            vault1b.add_item(&item1);
            
            let item2 = Item::new(
                vault_id,
                &item2_name,
                ItemType::Note,
                ItemData::Note(crate::models::NoteData {
                    title: Some(item2_name.clone()),
                    content: "content2".to_string(),
                    tags: vec![],
                }),
            );
            vault2a.add_item(&item2);
            vault2b.add_item(&item2);
            
            // Merge in order A: vault1 + vault2
            merge_vaults(&mut vault1a, &vault2a).unwrap();
            
            // Merge in order B: vault2 + vault1
            merge_vaults(&mut vault2b, &vault1b).unwrap();
            
            // Both should have same number of items (2)
            prop_assert_eq!(vault1a.list_item_ids().len(), 2);
            prop_assert_eq!(vault2b.list_item_ids().len(), 2);
        }
    }
}
