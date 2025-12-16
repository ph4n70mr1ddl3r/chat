//! Shared types and utilities for the chat application
//!
//! This crate contains protocol definitions, error types, and shared data structures
//! used by both the backend server and frontend client.

pub mod errors;
pub mod protocol;

pub use errors::*;
pub use protocol::*;
