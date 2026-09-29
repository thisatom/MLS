//! Configuration management for MLS CLI

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Default data directory name
pub const DEFAULT_DATA_DIR: &str = ".mls";

/// Default vault file name
pub const DEFAULT_VAULT_FILE: &str = "vault.db";

/// Default config file name
pub const CONFIG_FILE: &str = "config.json";

/// MLS CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to the data directory
    pub data_dir: PathBuf,
    /// Path to the vault database file
    pub vault_path: PathBuf,
    /// Session timeout in seconds (default: 900 = 15 minutes)
    pub session_timeout: u64,
    /// Default vault ID (for single-vault mode)
    pub default_vault_id: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = dirs::home_dir()
            .expect("Could not determine home directory")
            .join(DEFAULT_DATA_DIR);
        let vault_path = data_dir.join(DEFAULT_VAULT_FILE);

        Self {
            data_dir,
            vault_path,
            session_timeout: 900, // 15 minutes
            default_vault_id: None,
        }
    }
}

impl Config {
    /// Load config from file or create default
    pub fn load_or_create(data_dir: Option<&Path>) -> Result<Self> {
        // Determine data directory
        let data_dir = data_dir.map_or_else(
            || dirs::home_dir().expect("Could not determine home directory").join(DEFAULT_DATA_DIR),
            std::path::Path::to_path_buf,
        );

        // Create data directory if it doesn't exist
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)?;
        }

        let config_path = data_dir.join(CONFIG_FILE);

        // Try to load existing config
        if config_path.exists() {
            let config = fs::read_to_string(&config_path)?;
            let mut config: Self = serde_json::from_str(&config)?;

            // Override paths if custom data_dir was provided
            if data_dir != config.data_dir {
                config.data_dir.clone_from(&data_dir);
                config.vault_path = data_dir.join(format!("{DEFAULT_VAULT_FILE}.db"));
            }

            return Ok(config);
        }

        // Create default config with data_dir
        let config = Self {
            data_dir: data_dir.clone(),
            vault_path: data_dir.join(DEFAULT_VAULT_FILE),
            ..Self::default()
        };

        // Save default config
        config.save()?;

        Ok(config)
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_path = self.data_dir.join(CONFIG_FILE);
        let config_json = serde_json::to_string_pretty(self)?;
        fs::write(config_path, config_json)?;
        Ok(())
    }

    /// Get vault path for a specific vault ID
    pub fn vault_path_for(&self, vault_id: &str) -> PathBuf {
        self.data_dir.join(format!("{vault_id}.db"))
    }
}
