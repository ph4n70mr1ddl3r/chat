//! Shared test fixtures for database setup and common test data creation.
//!
//! This module provides reusable fixtures for integration and unit tests,
//! reducing duplication and improving consistency across the test suite.
//!
//! # Modules
//!
//! - `database` - Database initialization and schema setup
//! - `users` - User and conversation creation helpers

pub mod database;
pub mod users;

pub use database::setup_test_db;
pub use users::create_users_and_conversation;
