//! CLI interface for MLS
//!
//! Command-line interface for My Life Storage.

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

mod commands;
mod config;
mod error;
mod interactive;
mod session;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::Config;
use mls_core::models::ItemType;
use session::SessionManager;
use std::path::PathBuf;
use std::sync::Arc;

/// MLS - My Life Storage CLI
///
/// Secure, local-first password and notes storage with end-to-end encryption.
#[derive(Debug, Parser)]
#[command(name = "mls")]
#[command(author = "MLS Team")]
#[command(version = "0.1.0")]
#[command(about = "My Life Storage - Secure personal data vault", long_about = None)]
pub struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Path to the data directory (default: ~/.mls)
    #[arg(short, long, global = true)]
    pub data_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available commands
#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum Commands {
    /// Initialize a new vault
    #[command(alias = "i")]
    Init(InitCommand),

    /// Unlock a vault (start session)
    #[command(alias = "u")]
    Unlock(UnlockCommand),

    /// Lock the current session
    #[command(alias = "l")]
    Lock(LockCommand),

    /// Add a new item to the vault
    #[command(alias = "a")]
    Add(AddCommand),

    /// Get an item from the vault
    #[command(alias = "g")]
    Get(GetCommand),

    /// List items in the vault
    #[command(alias = "ls")]
    List(ListCommand),

    /// Remove an item from the vault
    #[command(alias = "rm")]
    Remove(RemoveCommand),

    /// Change master password
    #[command(alias = "cp")]
    ChangePassword(ChangePasswordCommand),

    /// Show current session status
    #[command(alias = "s")]
    Status(StatusCommand),

    /// Interactive mode
    Interactive,
}

/// Initialize a new vault
#[derive(Debug, Parser, PartialEq, Eq)]
pub struct InitCommand {
    /// Name of the vault
    #[arg(short, long)]
    pub name: Option<String>,

    /// Force re-initialization if vault already exists
    #[arg(short, long)]
    pub force: bool,
}

/// Unlock a vault
#[derive(Debug, Parser, PartialEq, Eq)]
pub struct UnlockCommand {
    /// Vault ID to unlock (default: first vault)
    #[arg(long)]
    pub vault_id: Option<String>,
}

/// Lock the current session
#[derive(Debug, Parser, PartialEq, Eq)]
pub struct LockCommand {}

/// Add a new item
#[derive(Debug, Parser, PartialEq, Eq)]
pub struct AddCommand {
    /// Type of item: password, note, generic
    #[arg(short, long, value_parser = clap::value_parser!(ItemType))]
    pub r#type: ItemType,

    /// Name of the item
    #[arg(value_name = "NAME")]
    pub name: String,

    /// Service (for password type)
    #[arg(short, long, required = false)]
    pub service: Option<String>,

    /// Username (for password type)
    #[arg(short, long, required = false)]
    pub username: Option<String>,

    /// URL (for password type)
    #[arg(long, required = false)]
    pub url: Option<String>,

    /// Content (for note type, or password value for password type)
    #[arg(short, long, required = false)]
    pub content: Option<String>,

    /// Interactive mode (prompt for fields)
    #[arg(short, long)]
    pub interactive: bool,
}

/// Get an item
#[derive(Debug, Parser, PartialEq, Eq)]
pub struct GetCommand {
    /// Name or ID of the item
    #[arg(value_name = "NAME_OR_ID")]
    pub name: String,

    /// Show password only (for password type)
    #[arg(short, long)]
    pub password_only: bool,

    /// Copy to clipboard (if available)
    #[arg(short, long)]
    pub clipboard: bool,
}

/// List items
#[derive(Debug, Parser, PartialEq, Eq)]
pub struct ListCommand {
    /// Filter by type
    #[arg(short, long, value_parser = clap::value_parser!(ItemType))]
    pub r#type: Option<ItemType>,

    /// Search in name and content
    #[arg(short, long)]
    pub search: Option<String>,

    /// Show details
    #[arg(long)]
    pub show_details: bool,

    /// Limit number of results
    #[arg(short, long)]
    pub limit: Option<usize>,
}

/// Remove an item
#[derive(Debug, Parser, PartialEq, Eq)]
pub struct RemoveCommand {
    /// Name or ID of the item
    #[arg(value_name = "NAME_OR_ID")]
    pub name: String,

    /// Force removal without confirmation
    #[arg(short, long)]
    pub force: bool,
}

/// Change master password
#[derive(Debug, Parser, PartialEq, Eq)]
pub struct ChangePasswordCommand {}

/// Show session status
#[derive(Debug, Parser, PartialEq, Eq)]
pub struct StatusCommand {}

fn main() -> Result<()> {
    // Initialize tracing
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "mls=info");
    }
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Load or create config
    let config = Config::load_or_create(cli.data_dir.as_deref())?;

    // Create session manager
    let session_manager = Arc::new(SessionManager::new(config.clone()));

    // If no command specified or Interactive command, run interactive mode
    if cli.command.is_none() || cli.command == Some(Commands::Interactive) {
        return Ok(interactive::run_interactive(config)?);
    }

    // Execute command
    match cli.command.unwrap() {
        Commands::Init(cmd) => {
            let result = InitCommand::run(cmd, &config, &session_manager);
            if let Err(e) = result {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Unlock(cmd) => {
            let result = UnlockCommand::run(cmd, &config, &session_manager);
            if let Err(e) = result {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Lock(cmd) => {
            let result = LockCommand::run(cmd, &session_manager);
            if let Err(e) = result {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Add(cmd) => {
            let result = AddCommand::run(cmd, &session_manager);
            if let Err(e) = result {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Get(cmd) => {
            let result = GetCommand::run(cmd, &session_manager);
            if let Err(e) = result {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::List(cmd) => {
            let result = ListCommand::run(&cmd, &session_manager);
            if let Err(e) = result {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Remove(cmd) => {
            let result = RemoveCommand::run(cmd, &session_manager);
            if let Err(e) = result {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::ChangePassword(cmd) => {
            let result = ChangePasswordCommand::run(cmd, &session_manager);
            if let Err(e) = result {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Status(cmd) => {
            let result = StatusCommand::run(cmd, &session_manager);
            if let Err(e) = result {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Interactive => {
            interactive::run_interactive(config)?;
        }
    }

    Ok(())
}
