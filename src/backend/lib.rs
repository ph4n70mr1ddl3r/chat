//! Chat backend library
//!
//! Core functionality for the chat server including models, services, handlers, and database.

pub mod db;
pub mod handlers;
pub mod models;
pub mod server;
pub mod services;
pub mod validators;

#[cfg(test)]
mod tests;
