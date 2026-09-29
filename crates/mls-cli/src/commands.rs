//! Command implementations for MLS CLI

use crate::config::Config;
use crate::error::CliError;
use crate::session::SessionManager;
use crate::{AddCommand, ChangePasswordCommand, GetCommand, InitCommand, ListCommand, LockCommand, RemoveCommand, StatusCommand, UnlockCommand};

impl InitCommand {
    /// Run the init command
    ///
    /// # Errors
    /// Returns an error if vault initialization fails
    pub fn run(
        cmd: Self,
        config: &Config,
        session_manager: &SessionManager,
    ) -> Result<(), CliError> {
        println!("Initializing MLS vault...");

        let vault_id = session_manager.init_vault(cmd.name, cmd.force)?;

        println!("Vault initialized successfully!");
        println!("Vault ID: {vault_id}");
        println!("Data directory: {}", config.data_dir.display());
        println!();
        println!("Run 'mls unlock' to start using your vault.");

        Ok(())
    }
}

impl UnlockCommand {
    /// Run the unlock command
    ///
    /// # Errors
    /// Returns an error if vault unlocking fails
    pub fn run(
        cmd: Self,
        _config: &Config,
        session_manager: &SessionManager,
    ) -> Result<(), CliError> {
        println!("Unlocking vault...");

        session_manager.unlock_vault(cmd.vault_id)?;

        let current_vault_id = session_manager.current_vault_id();
        println!("Vault unlocked successfully!");
        if let Some(vault_id) = current_vault_id {
            println!("Current vault: {vault_id}");
        }
        println!();
        println!("Run 'mls list' to see your items.");

        Ok(())
    }
}

impl LockCommand {
    /// Run the lock command
    ///
    /// # Errors
    /// Returns an error if locking fails
    pub fn run(
        _cmd: Self,
        session_manager: &SessionManager,
    ) -> Result<(), CliError> {
        println!("Locking session...");

        session_manager.lock_all();

        println!("Session locked successfully.");

        Ok(())
    }
}

impl AddCommand {
    /// Run the add command
    ///
    /// # Errors
    /// Returns an error if adding the item fails
    pub fn run(
        cmd: Self,
        session_manager: &SessionManager,
    ) -> Result<(), CliError> {
        if !session_manager.is_unlocked() {
            return Err(CliError::NotUnlocked);
        }

        println!("Adding item...");

        let item_id = session_manager.add_item(
            cmd.name,
            cmd.r#type,
            cmd.service,
            cmd.username,
            cmd.url,
            cmd.content,
        )?;

        println!("Item added successfully!");
        println!("Item ID: {item_id}");

        Ok(())
    }
}

impl GetCommand {
    /// Run the get command
    ///
    /// # Errors
    /// Returns an error if the item cannot be retrieved
    pub fn run(
        cmd: Self,
        session_manager: &SessionManager,
    ) -> Result<(), CliError> {
        if !session_manager.is_unlocked() {
            return Err(CliError::NotUnlocked);
        }

        let item = session_manager.get_item(cmd.name)?;

        match item.data {
            mls_core::models::ItemData::Password(pass) => {
                if cmd.password_only {
                    println!("{}", pass.password);
                } else {
                    println!("Service: {}", pass.service);
                    if let Some(ref username) = pass.username {
                        println!("Username: {username}");
                    }
                    println!("Password: {}", pass.password);
                    if let Some(ref url) = pass.url {
                        println!("URL: {url}");
                    }
                    if let Some(ref notes) = pass.notes {
                        println!("Notes: {notes}");
                    }
                }
            }
            mls_core::models::ItemData::Note(note) => {
                if let Some(ref title) = note.title {
                    println!("Title: {title}");
                }
                println!();
                println!("{}", note.content);
            }
            mls_core::models::ItemData::Generic(_) => {
                println!("Item: {}", item.metadata.name);
            }
        }

        Ok(())
    }
}

impl ListCommand {
    /// Run the list command
    ///
    /// # Errors
    /// Returns an error if listing items fails
    pub fn run(
        cmd: &Self,
        session_manager: &SessionManager,
    ) -> Result<(), CliError> {
        if !session_manager.is_unlocked() {
            return Err(CliError::NotUnlocked);
        }

        let items = session_manager.list_items(cmd.r#type.as_ref(), cmd.search.as_ref(), cmd.limit)?;

        if items.is_empty() {
            println!("No items found.");
            return Ok(());
        }

        println!("Items:");
        println!();

        for item in &items {
            let item_type = match &item.data {
                mls_core::models::ItemData::Password(_) => "password",
                mls_core::models::ItemData::Note(_) => "note",
                mls_core::models::ItemData::Generic(_) => "generic",
            };

            if cmd.show_details {
                println!("  {} [{}] - {}", item.metadata.name, item_type, item.id);
            } else {
                println!("  {} [{}]", item.metadata.name, item_type);
            }
        }

        println!();
        println!("Total: {} items", items.len());

        Ok(())
    }
}

impl RemoveCommand {
    /// Run the remove command
    ///
    /// # Errors
    /// Returns an error if removing the item fails
    pub fn run(
        cmd: Self,
        session_manager: &SessionManager,
    ) -> Result<(), CliError> {
        if !session_manager.is_unlocked() {
            return Err(CliError::NotUnlocked);
        }

        println!("Removing item...");

        session_manager.remove_item(cmd.name, cmd.force)?;

        println!("Item removed successfully.");

        Ok(())
    }
}

impl ChangePasswordCommand {
    /// Run the change password command
    ///
    /// # Errors
    /// Returns an error if changing password fails (placeholder implementation)
    pub fn run(
        _cmd: Self,
        _session_manager: &SessionManager,
    ) -> Result<(), CliError> {
        println!("Changing master password...");
        println!("Warning: This will re-encrypt all your data.");
        println!("This operation may take some time.");

        SessionManager::change_password();
        Ok(())
    }
}

impl StatusCommand {
    /// Run the status command
    ///
    /// # Errors
    /// Returns an error if checking status fails
    pub fn run(
        _cmd: Self,
        session_manager: &SessionManager,
    ) -> Result<(), CliError> {
        if session_manager.is_unlocked() {
            if let Some(vault_id) = session_manager.current_vault_id() {
                println!("Status: Unlocked");
                println!("Current vault: {vault_id}");
            } else {
                println!("Status: Unknown (no current vault)");
            }
        } else {
            println!("Status: Locked");
        }

        Ok(())
    }
}
