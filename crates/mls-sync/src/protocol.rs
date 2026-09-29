//! Sync protocol implementation

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Sync message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Handshake message
    Handshake(HandshakeMessage),
    /// State vector exchange
    StateVector(StateVectorMessage),
    /// Update message
    Update(UpdateMessage),
}

/// Handshake message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeMessage {
    pub device_id: Uuid,
    pub device_name: String,
    pub public_key: Vec<u8>,
}

/// State vector message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVectorMessage {
    pub vault_id: Uuid,
    pub state_vector: Vec<u8>,
}

/// Update message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMessage {
    pub vault_id: Uuid,
    pub update: Vec<u8>,
}
