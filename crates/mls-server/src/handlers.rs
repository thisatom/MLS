//! HTTP and WebSocket handlers for MLS relay server

use axum::{
    body::Bytes,
    extract::{ConnectInfo, Path, State, WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::error::ServerError;
use crate::state::AppState;

/// Header name for authentication key
pub const AUTH_KEY_HEADER: &str = "x-mls-auth-key";

/// Hash the auth key (in production, use proper cryptographic hashing)
fn hash_auth_key(key: &str) -> String {
    // For now, simple hash. In production, use SHA-256 or similar
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Extract and validate auth key from headers
fn extract_auth_key(headers: &HeaderMap) -> Result<String, ServerError> {
    headers
        .get(AUTH_KEY_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or(ServerError::InvalidAuthKey)
}

/// Store a blob
pub async fn store_blob(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(blob_id): Path<Uuid>,
    body: Bytes,
) -> Result<impl IntoResponse, ServerError> {
    let auth_key = extract_auth_key(&headers)?;
    let auth_key_hash = hash_auth_key(&auth_key);

    state
        .blob_storage
        .store_blob(&auth_key_hash, blob_id, body.to_vec())
        .await
        .map_err(|e| ServerError::DatabaseError(e))?;

    Ok((StatusCode::OK, format!("Blob {} stored", blob_id)))
}

/// Retrieve a blob
pub async fn get_blob(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(blob_id): Path<Uuid>,
) -> Result<impl IntoResponse, ServerError> {
    let auth_key = extract_auth_key(&headers)?;
    let auth_key_hash = hash_auth_key(&auth_key);

    // Verify the blob belongs to this user
    let blob = state
        .blob_storage
        .get_blob(&auth_key_hash, blob_id)
        .await
        .ok_or(ServerError::BlobNotFound)?;

    Ok((StatusCode::OK, blob))
}

/// Delete a blob
pub async fn delete_blob(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(blob_id): Path<Uuid>,
) -> Result<impl IntoResponse, ServerError> {
    let auth_key = extract_auth_key(&headers)?;
    let auth_key_hash = hash_auth_key(&auth_key);

    let deleted = state
        .blob_storage
        .delete_blob(&auth_key_hash, blob_id)
        .await;

    if deleted {
        Ok((StatusCode::OK, "Blob deleted"))
    } else {
        Err(ServerError::BlobNotFound)
    }
}

/// List all blob IDs for a user
pub async fn list_blobs(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let auth_key = extract_auth_key(&headers)?;
    let auth_key_hash = hash_auth_key(&auth_key);

    let blob_ids = state.blob_storage.list_blobs(&auth_key_hash).await;

    Ok((StatusCode::OK, serde_json::to_string(&blob_ids).unwrap()))
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "MLS Relay Server - OK")
}

/// WebSocket handler for sync message relaying
pub async fn websocket_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
) -> Result<Response, ServerError> {
    let auth_key = extract_auth_key(&headers)?;
    let auth_key_hash = hash_auth_key(&auth_key);

    // Generate connection ID
    let conn_id = Uuid::new_v4();

    // Create channel for sending messages to this connection
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(100);

    // Add connection to store
    state
        .connection_store
        .add_connection(conn_id, auth_key_hash.clone(), tx)
        .await;

    // Accept the WebSocket upgrade
    Ok(ws.on_upgrade(move |socket| async move {
        // Split the socket into sender and receiver
        let (mut sender, mut receiver) = socket.split();
        
        // Clone state for use in tasks
        let state_for_receiver = state.clone();
        let state_for_cleanup = state.clone();

        // Spawn task to handle incoming messages
        let receiver_task = tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(axum::extract::ws::Message::Binary(data)) => {
                        // Relay message to all other connections of the same user
                        let _ = state_for_receiver
                            .connection_store
                            .broadcast_to_user(&auth_key_hash, data)
                            .await;
                    }
                    Ok(axum::extract::ws::Message::Close(_)) => {
                        // Connection closed by client
                        break;
                    }
                    Ok(_) => {
                        // Ignore text messages (we only use binary)
                    }
                    Err(_) => {
                        // Connection error
                        break;
                    }
                }
            }

            // Remove connection on disconnect
            state_for_cleanup.connection_store.remove_connection(&conn_id).await;
        });

        // Spawn task to forward messages from channel to WebSocket
        let sender_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if sender.send(axum::extract::ws::Message::Binary(msg)).await.is_err() {
                    // Connection closed or error
                    break;
                }
            }
        });

        // Wait for either task to complete
        tokio::select! {
            _ = receiver_task => {}
            _ = sender_task => {}
        }

        // Remove connection on disconnect
        state.connection_store.remove_connection(&conn_id).await;
    }))
}

/// Create router with all routes
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/blobs/:id", get(get_blob).post(store_blob).delete(delete_blob))
        .route("/blobs", get(list_blobs))
        .route("/ws", get(websocket_handler))
        .with_state(state)
}
