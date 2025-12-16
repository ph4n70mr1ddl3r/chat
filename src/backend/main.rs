//! Chat application backend server
//!
//! This is the main entry point for the chat server. It initializes the database,
//! sets up WebSocket listeners, and starts the HTTP API.

use chat_backend::{db, init_tracing, server};
use clap::Parser;
use std::path::PathBuf;

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
    init_tracing(Some(&args.log_level));

    tracing::info!("Starting chat server on port {}", args.port);
    tracing::info!("Database: {}", args.db_path.display());

    // Initialize database
    let pool = db::init_db(&args.db_path).await?;
    tracing::info!("Database initialized");

    // Start HTTP server
    server::start_server(args.port, pool, None).await?;

    Ok(())
}
