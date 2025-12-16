//! Chat application backend server
//!
//! This is the main entry point for the chat server. It initializes the database,
//! sets up WebSocket listeners, and starts the HTTP API.

use chat_backend::db;
use chat_backend::server;
use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "chat-server")]
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

    // Initialize logging
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&args.log_level));

    tracing_subscriber::fmt()
        .json()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_thread_ids(true)
        .init();

    tracing::info!("Starting chat server on port {}", args.port);
    tracing::info!("Database: {}", args.db_path.display());

    // Initialize database
    let pool = db::init_db(&args.db_path).await?;
    tracing::info!("Database initialized");

    // Start HTTP server
    server::start_server(args.port, pool, None).await?;

    Ok(())
}
