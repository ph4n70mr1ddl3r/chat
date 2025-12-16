//! Chat backend library
//!
//! Core functionality for the chat server including models, services, handlers, and database.

pub mod db;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod server;
pub mod services;
pub mod validators;

use std::sync::OnceLock;
use tracing_subscriber::{fmt, EnvFilter};

/// Global tracing initializer to produce structured JSON logs across all binaries.
///
/// Idempotent: calling multiple times is safe and will only initialize once.
pub fn init_tracing(default_level: Option<&str>) {
    static TRACING: OnceLock<()> = OnceLock::new();

    let _ = TRACING.get_or_init(|| {
        let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            default_level
                .map(EnvFilter::new)
                .unwrap_or_else(|| EnvFilter::new("info"))
        });

        fmt()
            .json()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_level(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .init();
    });
}

#[cfg(test)]
mod tests;
