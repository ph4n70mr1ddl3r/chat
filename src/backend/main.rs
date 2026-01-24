//! Chat application backend server
//!
//! This is the main entry point for the chat server. It initializes the database,
//! sets up WebSocket listeners, and starts the HTTP API.

use chat_backend::{db, init_tracing, server};
use clap::Parser;
use std::path::PathBuf;
use tokio::signal;

#[derive(Parser, Debug)]
#[command(name = "chat-server")]
#[command(version = "0.1.0")]
#[command(about = "Private chat application server", long_about = None)]
struct Args {
    /// Server port
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Database file path
    #[arg(short, long, default_value = "chat.db")]
    db_path: PathBuf,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    init_tracing(Some(&args.log_level));

    tracing::info!("Starting chat server on port {}", args.port);
    tracing::info!("Database: {}", args.db_path.display());

    // Get database connection pool settings from environment variables
    let min_connections: u32 = std::env::var("DB_MIN_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);
    
    let max_connections: u32 = std::env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);

    tracing::info!(
        "Database connection pool settings: min={}, max={}",
        min_connections, max_connections
    );

    let pool = db::init_db_with_pool_settings(&args.db_path, min_connections, max_connections).await?;
    tracing::info!("Database initialized");

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    tokio::spawn(async move {
        if let Err(e) = signal::ctrl_c().await {
            tracing::error!("Failed to handle shutdown signal: {}", e);
        }
        tracing::info!("Received shutdown signal, initiating graceful shutdown...");
        let _ = shutdown_tx.send(true);
    });

    server::start_server(args.port, pool, None, shutdown_rx).await?;

    Ok(())
}
