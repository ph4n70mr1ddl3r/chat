//! Database setup utilities for tests.
//!
//! Provides functions to initialize in-memory SQLite databases with the proper schema.

use sqlx::SqlitePool;

/// Sets up an in-memory SQLite database with the full schema.
///
/// Creates a new in-memory database and runs all migrations from the initial schema file.
/// This function is designed to be called at the start of each test to ensure a clean state.
///
/// # Errors
///
/// Returns an error if:
/// - Database pool creation fails
/// - Schema migration SQL parsing fails
/// - Any migration statement fails to execute
///
/// # Example
///
/// ```ignore
/// #[tokio::test]
/// async fn my_test() {
///     let pool = setup_test_db().await;
///     // Use pool in test
/// }
/// ```
pub async fn setup_test_db() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    let schema_sql = include_str!("../../src/backend/db/migrations/001_initial_schema.sql");
    for statement in schema_sql.split(';').filter(|s| !s.trim().is_empty()) {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }

    pool
}
