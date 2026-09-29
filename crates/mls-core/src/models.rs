//! Data models for MLS
//!
//! This module provides the core data structures for the My Life Storage application.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a vault
pub type VaultId = Uuid;

/// Unique identifier for an item
pub type ItemId = Uuid;

/// Item type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemType {
    /// Password entry
    Password,
    /// Note entry
    Note,
    /// Generic entry (for future extensions)
    Generic,
}

impl std::fmt::Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Password => write!(f, "password"),
            Self::Note => write!(f, "note"),
            Self::Generic => write!(f, "generic"),
        }
    }
}

impl std::str::FromStr for ItemType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "password" | "pass" | "pwd" => Ok(Self::Password),
            "note" | "text" => Ok(Self::Note),
            "generic" | "item" => Ok(Self::Generic),
            _ => Err(format!("Unknown item type: {s}")),
        }
    }
}

/// Vault metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultMetadata {
    /// Vault name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl VaultMetadata {
    /// Create new vault metadata
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            name: name.into(),
            description: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update vault metadata
    pub fn update(&mut self, name: Option<String>, description: Option<String>) {
        if let Some(name) = name {
            self.name = name;
        }
        if let Some(description) = description {
            self.description = Some(description);
        }
        self.updated_at = Utc::now();
    }
}

/// Password item data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PasswordData {
    /// Service or website name
    pub service: String,
    /// Username or login
    pub username: Option<String>,
    /// Password value
    pub password: String,
    /// Optional URL
    pub url: Option<String>,
    /// Optional notes
    pub notes: Option<String>,
}

impl PasswordData {
    /// Create new password data
    pub fn new(password: impl Into<String>) -> Self {
        Self {
            service: String::new(),
            username: None,
            password: password.into(),
            url: None,
            notes: None,
        }
    }
}

/// Note item data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteData {
    /// Note title
    pub title: Option<String>,
    /// Note content (markdown supported)
    pub content: String,
    /// Optional tags
    pub tags: Vec<String>,
}

impl NoteData {
    /// Create new note data
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            title: None,
            content: content.into(),
            tags: Vec::new(),
        }
    }
}

/// Generic item data (for future extensions)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GenericData {
    /// Key-value pairs
    pub fields: std::collections::HashMap<String, String>,
}

impl GenericData {
    /// Create new generic data
    #[must_use]
    pub fn new() -> Self {
        Self {
            fields: std::collections::HashMap::new(),
        }
    }
}

/// Item data enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemData {
    /// Password item
    Password(PasswordData),
    /// Note item
    Note(NoteData),
    /// Generic item
    Generic(GenericData),
}

impl ItemData {
    /// Get item type
    #[must_use]
    pub const fn item_type(&self) -> ItemType {
        match self {
            Self::Password(_) => ItemType::Password,
            Self::Note(_) => ItemType::Note,
            Self::Generic(_) => ItemType::Generic,
        }
    }

    /// Get title/name for display
    #[must_use]
    pub fn title(&self) -> String {
        match self {
            Self::Password(p) => p.service.clone(),
            Self::Note(n) => n.title.clone().unwrap_or_default(),
            Self::Generic(_) => "Generic".to_string(),
        }
    }
}

/// Item metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemMetadata {
    /// Item name (unique within vault)
    pub name: String,
    /// Item type
    pub item_type: ItemType,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// CRDT version
    pub version: i64,
}

impl ItemMetadata {
    /// Create new item metadata
    pub fn new(name: impl Into<String>, item_type: ItemType) -> Self {
        let now = Utc::now();
        Self {
            name: name.into(),
            item_type,
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    /// Increment version
    pub fn increment_version(&mut self) {
        self.version += 1;
        self.updated_at = Utc::now();
    }
}

/// Vault structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    /// Vault identifier
    pub id: VaultId,
    /// Vault metadata
    pub metadata: VaultMetadata,
}

impl Vault {
    /// Create new vault
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            metadata: VaultMetadata::new(name),
        }
    }
}

/// Item structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// Item identifier
    pub id: ItemId,
    /// Parent vault identifier
    pub vault_id: VaultId,
    /// Item metadata
    pub metadata: ItemMetadata,
    /// Item data
    pub data: ItemData,
}

impl Item {
    /// Create new item
    pub fn new(vault_id: VaultId, name: impl Into<String>, item_type: ItemType, data: ItemData) -> Self {
        Self {
            id: Uuid::new_v4(),
            vault_id,
            metadata: ItemMetadata::new(name, item_type),
            data,
        }
    }
}

/// Filter for listing items
#[derive(Debug, Clone, Default)]
pub struct ItemFilter {
    /// Filter by vault ID
    pub vault_id: Option<VaultId>,
    /// Filter by item type
    pub item_type: Option<ItemType>,
    /// Search in name/content
    pub search: Option<String>,
    /// Limit results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Sort order for items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    /// Sort by creation date (newest first)
    CreatedDesc,
    /// Sort by creation date (oldest first)
    CreatedAsc,
    /// Sort by update date (newest first)
    #[default]
    UpdatedDesc,
    /// Sort by update date (oldest first)
    UpdatedAsc,
    /// Sort by name (A-Z)
    NameAsc,
    /// Sort by name (Z-A)
    NameDesc,
}
