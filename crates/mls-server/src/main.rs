//! Relay server for MLS
//!
//! Relay server that stores encrypted blobs and retransmits sync messages.
//! 
//! Architecture:
//! - HTTP API for blob storage (GET, POST, DELETE /blobs/:id)
//! - WebSocket endpoint for real-time message relaying (/ws)
//! - Authentication via Auth Key header (X-MLS-Auth-Key)
//! - Zero-knowledge: server only sees encrypted data
//! - Rate limiting for DoS protection

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

mod error;
mod handlers;
mod state;

use anyhow::Result;
use handlers::create_router;
use state::AppState;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Default server port
const DEFAULT_PORT: u16 = 3000;

/// Default bind address
const DEFAULT_BIND_ADDR: &str = "0.0.0.0";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "mls_server=info");
    }

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("MLS Relay Server starting...");

    // Create application state
    let state = AppState::new();

    // Create router
    let app = create_router(state);

    // Bind address
    let bind_addr: SocketAddr = format!("{}:{}", DEFAULT_BIND_ADDR, DEFAULT_PORT)
        .parse()
        .expect("Failed to parse bind address");

    tracing::info!("Binding to {}", bind_addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    
    tracing::info!("MLS Relay Server running on http://{}", bind_addr);
    tracing::info!("WebSocket endpoint: ws://{}/ws", bind_addr);
    tracing::info!("Auth key header: X-MLS-Auth-Key");

    axum::serve(listener, app).await?;

    Ok(())
}
