//! Interactive mode for MLS CLI
//!
//! Provides a REPL-like interface for managing vaults and items.

use crate::config::Config;
use crate::error::CliError;
use crate::session::SessionManager;
use dialoguer::{Input, Password, Select};
use mls_core::models::ItemType;
use std::sync::Arc;

/// Run interactive mode
pub fn run_interactive(config: Config) -> Result<(), CliError> {
    let session_manager = Arc::new(SessionManager::new(config));

    println!("MLS - My Life Storage");
    println!("====================");
    println!();

    loop {
        println!();
        
        let options = if session_manager.is_unlocked() {
            vec![
                "List items",
                "Add item",
                "Get item",
                "Remove item",
                "Lock vault",
                "Status",
                "Exit",
            ]
        } else {
            vec![
                "Init vault",
                "Unlock vault",
                "Status",
                "Exit",
            ]
        };

        let selection = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .items(&options)
            .interact()?;

        match selection {
            // Common commands
            idx if options[idx] == "Status" => {
                print_status(&session_manager);
            }
            idx if options[idx] == "Exit" => {
                println!("Goodbye!");
                break;
            }
            
            // Locked state commands
            idx if options[idx] == "Init vault" => {
                cmd_init(&session_manager, &session_manager.config)?;
            }
            idx if options[idx] == "Unlock vault" => {
                cmd_unlock(&session_manager)?;
            }
            
            // Unlocked state commands
            idx if options[idx] == "List items" => {
                cmd_list(&session_manager)?;
            }
            idx if options[idx] == "Add item" => {
                cmd_add(&session_manager)?;
            }
            idx if options[idx] == "Get item" => {
                cmd_get(&session_manager)?;
            }
            idx if options[idx] == "Remove item" => {
                cmd_remove(&session_manager)?;
            }
            idx if options[idx] == "Lock vault" => {
                cmd_lock(&session_manager);
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn print_status(session_manager: &SessionManager) {
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
}

fn cmd_init(session_manager: &SessionManager, config: &Config) -> Result<(), CliError> {
    println!("\n=== Initialize Vault ===");
    
    let name_input: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Vault name (optional):")
        .allow_empty(true)
        .interact_text()?;
    
    let name = if name_input.is_empty() { None } else { Some(name_input) };
    
    let force = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Force re-initialization if exists?")
        .default(false)
        .interact()?;
    
    let vault_id = session_manager.init_vault(name, force)?;
    
    println!();
    println!("Vault initialized successfully!");
    println!("Vault ID: {vault_id}");
    println!("Data directory: {}", config.data_dir.display());
    println!();
    println!("Run 'unlock' to start using your vault.");
    
    Ok(())
}

fn cmd_unlock(session_manager: &SessionManager) -> Result<(), CliError> {
    println!("\n=== Unlock Vault ===");
    
    let vault_id_input: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Vault ID (leave empty for default):")
        .allow_empty(true)
        .interact_text()?;
    
    let vault_id = if vault_id_input.is_empty() { None } else { Some(vault_id_input) };
    
    session_manager.unlock_vault(vault_id)?;
    
    let current_vault_id = session_manager.current_vault_id();
    println!();
    println!("Vault unlocked successfully!");
    if let Some(vault_id) = current_vault_id {
        println!("Current vault: {vault_id}");
    }
    
    Ok(())
}

fn cmd_lock(session_manager: &SessionManager) {
    println!("\nLocking session...");
    session_manager.lock_all();
    println!("Session locked successfully.");
}

fn cmd_list(session_manager: &SessionManager) -> Result<(), CliError> {
    if !session_manager.is_unlocked() {
        return Err(CliError::NotUnlocked);
    }

    println!("\n=== Your Items ===");
    
    let items = session_manager.list_items(None, None, None)?;
    
    if items.is_empty() {
        println!("No items found.");
        return Ok(());
    }

    for (i, item) in items.iter().enumerate() {
        let item_type = match &item.data {
            mls_core::models::ItemData::Password(_) => "password",
            mls_core::models::ItemData::Note(_) => "note",
            mls_core::models::ItemData::Generic(_) => "generic",
        };
        println!("{}. {} [{}] - {}", i + 1, item.metadata.name, item_type, item.id);
    }
    
    println!();
    println!("Total: {} items", items.len());
    
    Ok(())
}

fn cmd_add(session_manager: &SessionManager) -> Result<(), CliError> {
    if !session_manager.is_unlocked() {
        return Err(CliError::NotUnlocked);
    }

    println!("\n=== Add Item ===");
    
    let name: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Item name:")
        .interact_text()?;
    
    let item_type = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Item type:")
        .items(&["Password", "Note", "Generic"])
        .interact()?;
    
    let item_type = match item_type {
        0 => ItemType::Password,
        1 => ItemType::Note,
        _ => ItemType::Generic,
    };
    
    let (service, username, url, content) = match item_type {
        ItemType::Password => {
            let service: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt("Service:")
                .interact_text()?;
            let username_input: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt("Username (optional):")
                .allow_empty(true)
                .interact_text()?;
            let username = if username_input.is_empty() { None } else { Some(username_input) };
            let url_input: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt("URL (optional):")
                .allow_empty(true)
                .interact_text()?;
            let url = if url_input.is_empty() { None } else { Some(url_input) };
            let password: String = Password::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt("Password:")
                .interact()?;
            (Some(service), username, url, Some(password))
        }
        ItemType::Note => {
            let content: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt("Content:")
                .interact_text()?;
            (None, None, None, Some(content))
        }
        ItemType::Generic => {
            (None, None, None, None)
        }
    };
    
    let item_id = session_manager.add_item(
        name,
        item_type,
        service,
        username,
        url,
        content,
    )?;
    
    println!();
    println!("Item added successfully!");
    println!("Item ID: {item_id}");
    
    Ok(())
}

fn cmd_get(session_manager: &SessionManager) -> Result<(), CliError> {
    if !session_manager.is_unlocked() {
        return Err(CliError::NotUnlocked);
    }

    println!("\n=== Get Item ===");
    
    let name: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Item name or ID:")
        .interact_text()?;
    
    let item = session_manager.get_item(name)?;
    
    println!();
    match item.data {
        mls_core::models::ItemData::Password(pass) => {
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

fn cmd_remove(session_manager: &SessionManager) -> Result<(), CliError> {
    if !session_manager.is_unlocked() {
        return Err(CliError::NotUnlocked);
    }

    println!("\n=== Remove Item ===");
    
    let name: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Item name or ID:")
        .interact_text()?;
    
    let force = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Are you sure?")
        .default(false)
        .interact()?;
    
    session_manager.remove_item(name, force)?;
    
    println!();
    println!("Item removed successfully.");
    
    Ok(())
}
