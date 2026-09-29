//! Tests for mls-storage crate

use mls_crypto::traits::{AesGcmBackend};
use mls_storage::{
    repository::{ItemRepository, VaultRepository},
    schema,
};
use rusqlite::Connection;
use std::sync::Arc;
use tempfile::NamedTempFile;
use zeroize::Zeroizing;

#[test]
fn test_schema_initialization() {
    let temp_file = NamedTempFile::new().unwrap();
    let conn = Connection::open(temp_file.path()).unwrap();

    schema::init_schema(&conn).unwrap();

    // Verify tables were created
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")
        .unwrap();

    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    assert!(tables.contains(&"vaults".to_string()));
    assert!(tables.contains(&"items".to_string()));
    assert!(tables.contains(&"sync_state".to_string()));
}

#[test]
fn test_vault_repository_create_and_get() {
    let temp_file = NamedTempFile::new().unwrap();
    let conn = Connection::open(temp_file.path()).unwrap();
    schema::init_schema(&conn).unwrap();

    let vault_conn = Connection::open(temp_file.path()).unwrap();
    let vault_repo = VaultRepository::new(vault_conn);
    let crypto_backend: Arc<dyn mls_crypto::traits::CryptoBackend> = Arc::new(AesGcmBackend);

    let vault_id = "test-vault";
    let encryption_key = Zeroizing::new([0u8; 32]);

    // Test data
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestMetadata {
        name: String,
        description: String,
    }

    let metadata = TestMetadata {
        name: "Test Vault".to_string(),
        description: "A test vault".to_string(),
    };

    // Create vault
    vault_repo
        .create_vault(vault_id, &metadata, &encryption_key, &*crypto_backend)
        .unwrap();

    // Get vault
    let retrieved: TestMetadata = vault_repo
        .get_vault(vault_id, &encryption_key, &*crypto_backend)
        .unwrap();

    assert_eq!(retrieved.name, metadata.name);
    assert_eq!(retrieved.description, metadata.description);
}

#[test]
fn test_item_repository_create_and_list() {
    let temp_file = NamedTempFile::new().unwrap();
    let conn = Connection::open(temp_file.path()).unwrap();
    schema::init_schema(&conn).unwrap();

    let vault_conn = Connection::open(temp_file.path()).unwrap();
    let item_conn = Connection::open(temp_file.path()).unwrap();
    let vault_repo = VaultRepository::new(vault_conn);
    let item_repo = ItemRepository::new(item_conn);
    let crypto_backend: Arc<dyn mls_crypto::traits::CryptoBackend> = Arc::new(AesGcmBackend);

    let vault_id = "test-vault";
    let encryption_key = Zeroizing::new([0u8; 32]);

    // Create vault first
    vault_repo
        .create_vault(
            vault_id,
            &VaultMetadata { name: "Test".to_string() },
            &encryption_key,
            &*crypto_backend,
        )
        .unwrap();

    // Test data
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestItem {
        title: String,
        content: String,
    }

    let item_id = "test-item";
    let item_data = TestItem {
        title: "Test Item".to_string(),
        content: "Test content".to_string(),
    };

    // Create item
    item_repo
        .create_item(
            vault_id,
            item_id,
            &item_data,
            1,
            &encryption_key,
            &*crypto_backend,
        )
        .unwrap();

    // List items
    let items = item_repo.list_items(vault_id).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].0, item_id);
    assert_eq!(items[0].1, 1);

    // Get item
    let (retrieved, version): (TestItem, i64) = item_repo
        .get_item(item_id, &encryption_key, &*crypto_backend)
        .unwrap();

    assert_eq!(retrieved.title, item_data.title);
    assert_eq!(retrieved.content, item_data.content);
    assert_eq!(version, 1);
}

#[test]
fn test_round_trip_encryption() {
    let temp_file = NamedTempFile::new().unwrap();
    let conn = Connection::open(temp_file.path()).unwrap();
    schema::init_schema(&conn).unwrap();

    let vault_conn = Connection::open(temp_file.path()).unwrap();
    let item_conn = Connection::open(temp_file.path()).unwrap();
    let vault_repo = VaultRepository::new(vault_conn);
    let item_repo = ItemRepository::new(item_conn);
    let crypto_backend: Arc<dyn mls_crypto::traits::CryptoBackend> = Arc::new(AesGcmBackend);

    let encryption_key = Zeroizing::new([0u8; 32]);

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
    struct ComplexData {
        field1: String,
        field2: i32,
        field3: Vec<String>,
        field4: Option<String>,
    }

    let original_data = ComplexData {
        field1: "Hello World".to_string(),
        field2: 42,
        field3: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        field4: Some("optional".to_string()),
    };

    // Create vault and item with complex data
    let vault_id = "encryption-test-vault";
    vault_repo
        .create_vault(
            vault_id,
            &VaultMetadata { name: "Test".to_string() },
            &encryption_key,
            &*crypto_backend,
        )
        .unwrap();

    let item_id = "complex-item";
    item_repo
        .create_item(
            vault_id,
            item_id,
            &original_data,
            1,
            &encryption_key,
            &*crypto_backend,
        )
        .unwrap();

    // Retrieve and verify
    let (retrieved_data, _): (ComplexData, i64) = item_repo
        .get_item(item_id, &encryption_key, &*crypto_backend)
        .unwrap();

    assert_eq!(retrieved_data, original_data);
}

// Helper struct for tests
#[derive(serde::Serialize, serde::Deserialize)]
struct VaultMetadata {
    name: String,
}
