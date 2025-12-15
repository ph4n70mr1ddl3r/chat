use anyhow::anyhow;
use chrono::Utc;
use clap::{Parser, Subcommand};
use serde::Serialize;
use serde_json::json;
use sqlx::SqlitePool;
use std::fs;
use std::path::PathBuf;

use chat_backend::db;

#[derive(Parser)]
#[command(name = "admin_cli")]
#[command(about = "Admin CLI for chat server", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, default_value = "chat.db")]
    db_path: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// User management
    Users {
        #[command(subcommand)]
        subcommand: UsersSubcommand,
    },
    /// Inspect messages in a conversation
    Inspect {
        conversation_id: String,
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// Server health
    Health,
    /// Server stats
    Stats,
}

#[derive(Subcommand)]
enum UsersSubcommand {
    /// List users
    List {
        #[arg(long, default_value = "false")]
        deleted: bool,
    },
    /// Delete a user
    Delete { username: String },
}

#[derive(Debug, Serialize)]
struct UserView {
    id: String,
    username: String,
    created_at: i64,
    updated_at: i64,
    deleted_at: Option<i64>,
    is_online: bool,
    last_seen_at: Option<i64>,
}

impl From<chat_backend::models::User> for UserView {
    fn from(user: chat_backend::models::User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            created_at: user.created_at,
            updated_at: user.updated_at,
            deleted_at: user.deleted_at,
            is_online: user.is_online,
            last_seen_at: user.last_seen_at,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let pool = db::init_db(&args.db_path).await?;

    match args.command {
        Commands::Users { subcommand } => match subcommand {
            UsersSubcommand::List { deleted } => {
                let query = if deleted {
                    "SELECT id, username, password_hash, password_salt, created_at, updated_at, deleted_at, is_online, last_seen_at FROM users"
                } else {
                    "SELECT id, username, password_hash, password_salt, created_at, updated_at, deleted_at, is_online, last_seen_at FROM users WHERE deleted_at IS NULL"
                };
                let users: Vec<chat_backend::models::User> = sqlx::query_as(query)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to list users: {}", e))?;

                let user_views: Vec<UserView> = users.into_iter().map(|u| u.into()).collect();
                let output = json!(user_views);
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
            UsersSubcommand::Delete { username } => {
                // Find user by username
                let user: Option<chat_backend::models::User> = sqlx::query_as(
                    "SELECT id, username, password_hash, password_salt, created_at, updated_at, deleted_at, is_online, last_seen_at FROM users WHERE username = ?"
                )
                    .bind(&username)
                    .fetch_optional(&pool)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to find user: {}", e))?;

                match user {
                    Some(user) => {
                        let now = chrono::Utc::now().timestamp_millis();
                        sqlx::query("UPDATE users SET deleted_at = ?, updated_at = ? WHERE id = ?")
                            .bind(now)
                            .bind(now)
                            .bind(&user.id)
                            .execute(&pool)
                            .await
                            .map_err(|e| anyhow::anyhow!("Failed to delete user: {}", e))?;

                        println!("User '{}' soft-deleted", username);
                    }
                    None => {
                        eprintln!("User '{}' not found", username);
                        std::process::exit(1);
                    }
                }
            }
        },
        Commands::Inspect {
            conversation_id,
            limit,
        } => {
            // Fetch messages in conversation
            let messages: Vec<chat_backend::models::Message> = sqlx::query_as(
                "SELECT id, conversation_id, sender_id, recipient_id, content, created_at, delivered_at, status, is_anonymized FROM messages WHERE conversation_id = ? ORDER BY created_at DESC LIMIT ?"
            )
                .bind(&conversation_id)
                .bind(limit)
                .fetch_all(&pool)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to fetch messages: {}", e))?;

            let output = json!(messages);
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        Commands::Health => {
            // Simple health check: database connection and table counts
            let user_count: (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to query users: {}", e))?;
            let message_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages")
                .fetch_one(&pool)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to query messages: {}", e))?;
            let conversation_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM conversations")
                .fetch_one(&pool)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to query conversations: {}", e))?;

            let health = json!({
                "status": "healthy",
                "database": args.db_path.display().to_string(),
                "timestamp": chrono::Utc::now().timestamp_millis(),
                "user_count": user_count.0,
                "message_count": message_count.0,
                "conversation_count": conversation_count.0,
            });
            println!("{}", serde_json::to_string_pretty(&health)?);
        }
        Commands::Stats => {
            // Basic stats: active connections (not stored), throughput (not stored)
            // For now, just show table sizes
            let user_count: (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to query users: {}", e))?;
            let message_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages")
                .fetch_one(&pool)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to query messages: {}", e))?;
            let pending_messages: (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM messages WHERE status = 'pending'")
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to query pending messages: {}", e))?;

            let stats = json!({
                "timestamp": chrono::Utc::now().timestamp_millis(),
                "active_users": user_count.0,
                "total_messages": message_count.0,
                "pending_messages": pending_messages.0,
                "database_size_bytes": fs::metadata(&args.db_path)?.len(),
            });
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
    }
    Ok(())
}
