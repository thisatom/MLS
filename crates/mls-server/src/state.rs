//! Application state for MLS relay server

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// In-memory storage for encrypted blobs
/// In production, this should be persisted to a database
#[derive(Debug, Default)]
pub struct BlobStorage {
    /// Maps user auth key hash to their blobs
    /// Key: auth_key_hash (String), Value: Map of blob_id to blob data
    data: RwLock<HashMap<String, HashMap<Uuid, Vec<u8>>>>,
    /// Maps blob_id to user auth key hash for reverse lookup
    blob_owners: RwLock<HashMap<Uuid, String>>,
}

impl BlobStorage {
    /// Create a new blob storage
    pub fn new() -> Self {
        Self::default()
    }

    /// Store a blob for a user
    pub async fn store_blob(
        &self,
        auth_key_hash: &str,
        blob_id: Uuid,
        data: Vec<u8>,
    ) -> Result<(), String> {
        let mut data_lock = self.data.write().await;
        let mut owners_lock = self.blob_owners.write().await;

        data_lock
            .entry(auth_key_hash.to_string())
            .or_default()
            .insert(blob_id, data);
        owners_lock.insert(blob_id, auth_key_hash.to_string());

        Ok(())
    }

    /// Retrieve a blob for a user
    pub async fn get_blob(
        &self,
        auth_key_hash: &str,
        blob_id: Uuid,
    ) -> Option<Vec<u8>> {
        let data_lock = self.data.read().await;

        data_lock
            .get(auth_key_hash)
            .and_then(|user_blobs| user_blobs.get(&blob_id).cloned())
    }

    /// Delete a blob
    pub async fn delete_blob(&self, auth_key_hash: &str, blob_id: Uuid) -> bool {
        let mut data_lock = self.data.write().await;
        let mut owners_lock = self.blob_owners.write().await;

        if let Some(user_blobs) = data_lock.get_mut(auth_key_hash) {
            if user_blobs.remove(&blob_id).is_some() {
                owners_lock.remove(&blob_id);
                return true;
            }
        }
        false
    }

    /// List all blob IDs for a user
    pub async fn list_blobs(&self, auth_key_hash: &str) -> Vec<Uuid> {
        let data_lock = self.data.read().await;

        data_lock
            .get(auth_key_hash)
            .map(|user_blobs| user_blobs.keys().cloned().collect())
            .unwrap_or_default()
    }
}

/// Connection tracking for WebSocket sessions
#[derive(Debug, Default)]
pub struct ConnectionStore {
    /// Maps connection ID to (user auth key hash, sender channel)
    connections: RwLock<HashMap<Uuid, (String, tokio::sync::mpsc::Sender<Vec<u8>>)>>,
}

impl ConnectionStore {
    /// Create a new connection store
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new connection
    pub async fn add_connection(
        &self,
        conn_id: Uuid,
        auth_key_hash: String,
        sender: tokio::sync::mpsc::Sender<Vec<u8>>,
    ) {
        let mut conn_lock = self.connections.write().await;
        conn_lock.insert(conn_id, (auth_key_hash, sender));
    }

    /// Remove a connection
    pub async fn remove_connection(&self, conn_id: &Uuid) {
        let mut conn_lock = self.connections.write().await;
        conn_lock.remove(conn_id);
    }

    /// Get all connections for a user
    pub async fn get_user_connections(&self, auth_key_hash: &str) -> Vec<Uuid> {
        let conn_lock = self.connections.read().await;

        conn_lock
            .iter()
            .filter(|(_, (hash, _))| hash == auth_key_hash)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Send message to all connections of a user
    pub async fn broadcast_to_user(
        &self,
        auth_key_hash: &str,
        message: Vec<u8>,
    ) -> usize {
        let conn_lock = self.connections.read().await;
        let mut count = 0;

        for (_, (hash, sender)) in conn_lock.iter() {
            if hash == auth_key_hash {
                if sender.send(message.clone()).await.is_ok() {
                    count += 1;
                }
            }
        }

        count
    }
}

/// Rate limiting state
#[derive(Debug)]
pub struct RateLimiter {
    /// Maps IP address to request count and timestamp
    requests: RwLock<HashMap<String, (u32, DateTime<Utc>)>>,
    /// Max requests per window
    max_requests: u32,
    /// Window duration in seconds
    window_seconds: i64,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_requests: u32, window_seconds: i64) -> Self {
        Self {
            requests: RwLock::new(HashMap::new()),
            max_requests,
            window_seconds,
        }
    }

    /// Check if request is allowed
    pub async fn check(&self, ip: &str) -> bool {
        let mut req_lock = self.requests.write().await;
        let now = Utc::now();

        if let Some((count, timestamp)) = req_lock.get_mut(ip) {
            // Reset if window has passed
            if now.signed_duration_since(*timestamp).num_seconds() > self.window_seconds {
                *count = 0;
                *timestamp = now;
            }

            if *count < self.max_requests {
                *count += 1;
                true
            } else {
                false
            }
        } else {
            req_lock.insert(ip.to_string(), (1, now));
            true
        }
    }
}

/// Main application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Blob storage
    pub blob_storage: Arc<BlobStorage>,
    /// Connection store for WebSocket sessions
    pub connection_store: Arc<ConnectionStore>,
    /// Rate limiter
    pub rate_limiter: Arc<RateLimiter>,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> Self {
        Self {
            blob_storage: Arc::new(BlobStorage::new()),
            connection_store: Arc::new(ConnectionStore::new()),
            rate_limiter: Arc::new(RateLimiter::new(100, 60)), // 100 requests/minute
        }
    }
}
