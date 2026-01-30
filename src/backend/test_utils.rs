//! Shared test utilities for backend modules

use sqlx::SqlitePool;

/// Initialize an in-memory SQLite database for testing
///
/// Creates a new connection pool and runs all migrations.
/// This is useful for tests that need a fresh database.
pub async fn setup_test_db() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create test pool");

    // Run initial schema migration
    let schema_sql = include_str!("db/migrations/001_initial_schema.sql");
    for statement in schema_sql.split(';').filter(|s| !s.trim().is_empty()) {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .expect("Failed to run schema statement");
    }

    // Run password salt removal migration
    let migration_sql = include_str!("db/migrations/002_remove_password_salt.sql");
    for statement in migration_sql.split(';').filter(|s| !s.trim().is_empty()) {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .expect("Failed to run migration statement");
    }

    pool
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_test_db_creates_tables() {
        let pool = setup_test_db().await;

        // Verify tables were created
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'",
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to query tables");

        assert!(result.0 >= 1, "users table should exist");
    }

    #[tokio::test]
    async fn test_setup_test_db_is_fresh() {
        let pool = setup_test_db().await;

        // Verify database is empty
        let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .expect("Failed to query users");

        assert_eq!(user_count.0, 0, "users table should be empty");
    }
}
