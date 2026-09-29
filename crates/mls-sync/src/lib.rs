//! CRDT-based synchronization for MLS
//!
//! This crate provides synchronization capabilities including:
//! - Yrs-based CRDT implementation
//! - LAN discovery via mDNS
//! - Relay server communication
//! - Sync protocol implementation

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

pub mod error;
pub mod lan;
pub mod protocol;
pub mod relay;
