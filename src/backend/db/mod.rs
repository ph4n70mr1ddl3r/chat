//! Database initialization and migration management

use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;
use tracing::info;

pub mod queries;

/// Initialize SQLite database and run migrations
pub async fn init_db(db_path: impl AsRef<Path>) -> Result<SqlitePool> {
    let db_path = db_path.as_ref();

    // Create parent directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    info!("Initializing database: {}", db_url);

    // Create connection options with WAL mode for better concurrency.
    // NOTE: MVP stores SQLite files in plaintext; production deployments should place
    // the database on an encrypted volume (LUKS/BitLocker) or enable SQLCipher.
    let connect_options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .pragma("journal_mode", "WAL")
        .pragma("foreign_keys", "ON")
        .pragma("synchronous", "NORMAL");

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .min_connections(5)
        .max_connections(20)
        .connect_with(connect_options)
        .await?;

    // Run migrations
    run_migrations(&pool).await?;

    info!("Database initialized successfully");
    Ok(pool)
}

/// Run all pending migrations
async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    info!("Running database migrations...");

    // Read and execute the initial schema
    let schema_sql = include_str!("migrations/001_initial_schema.sql");

    // Split schema into individual statements
    for statement in schema_sql.split(';').filter(|s| !s.trim().is_empty()) {
        sqlx::query(statement).execute(pool).await?;
    }

    info!("Migrations completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_db_init() -> Result<()> {
        let db_path = ":memory:";
        let pool = init_db(db_path).await?;

        // Verify tables were created
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'",
        )
        .fetch_one(&pool)
        .await?;

        assert_eq!(result.0, 1, "users table should exist");
        Ok(())
    }
}
