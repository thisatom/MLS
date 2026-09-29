//! Core data models, CRDT, and crypto abstractions for MLS
//!
//! This crate provides the foundational types and traits for the My Life Storage application.

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

pub mod crdt;
pub mod error;
pub mod models;
