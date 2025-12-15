//! User database queries
//!
//! Provides database operations for user management including insertion, lookup, and updates.

use sqlx::SqlitePool;
use crate::models::User;

/// Insert a new user into the database
///
/// Returns the user if successful
pub async fn insert_user(pool: &SqlitePool, user: &User) -> Result<User, String> {
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, password_salt, created_at, updated_at, is_online, deleted_at, last_seen_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&user.id)
    .bind(&user.username)
    .bind(&user.password_hash)
    .bind(&user.password_salt)
    .bind(user.created_at)
    .bind(user.updated_at)
    .bind(user.is_online)
    .bind(user.deleted_at)
    .bind(user.last_seen_at)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to insert user: {}", e))?;

    Ok(user.clone())
}

/// Find a user by username
///
/// Returns the user if found, None if not found
pub async fn find_user_by_username(pool: &SqlitePool, username: &str) -> Result<Option<User>, String> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, password_salt, created_at, updated_at, deleted_at, is_online, last_seen_at
         FROM users
         WHERE username = ?"
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to find user by username: {}", e))
}

/// Find a user by ID
///
/// Returns the user if found, None if not found
pub async fn find_user_by_id(pool: &SqlitePool, user_id: &str) -> Result<Option<User>, String> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, password_salt, created_at, updated_at, deleted_at, is_online, last_seen_at
         FROM users
         WHERE id = ?"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to find user by id: {}", e))
}

/// Update user online status
pub async fn update_online_status(pool: &SqlitePool, user_id: &str, is_online: bool) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();
    
    sqlx::query(
        "UPDATE users SET is_online = ?, last_seen_at = ?, updated_at = ? WHERE id = ?"
    )
    .bind(is_online)
    .bind(now)
    .bind(now)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update online status: {}", e))?;

    Ok(())
}

/// Update user last seen timestamp
pub async fn update_last_seen(pool: &SqlitePool, user_id: &str) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();
    
    sqlx::query(
        "UPDATE users SET last_seen_at = ?, updated_at = ? WHERE id = ?"
    )
    .bind(now)
    .bind(now)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update last seen: {}", e))?;

    Ok(())
}

/// Soft delete a user (mark deleted_at)
pub async fn delete_user(pool: &SqlitePool, user_id: &str) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();
    
    sqlx::query(
        "UPDATE users SET deleted_at = ?, updated_at = ? WHERE id = ?"
    )
    .bind(now)
    .bind(now)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to delete user: {}", e))?;

    Ok(())
}

/// Search users by username prefix (case-insensitive)
///
/// Excludes the current user and deleted users
pub async fn search_users_by_prefix(
    pool: &SqlitePool,
    query: &str,
    current_user_id: &str,
    limit: u32,
) -> Result<Vec<User>, String> {
    let search_pattern = format!("{}%", query);
    
    sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, password_salt, created_at, updated_at, deleted_at, is_online, last_seen_at
         FROM users
         WHERE username LIKE ? AND id != ? AND deleted_at IS NULL
         LIMIT ?"
    )
    .bind(search_pattern)
    .bind(current_user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to search users: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_and_find_user() -> Result<(), Box<dyn std::error::Error>> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await?;

        // Run migrations
        let schema_sql = include_str!("../migrations/001_initial_schema.sql");
        for statement in schema_sql.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(statement).execute(&pool).await?;
        }

        // Create and insert user
        let user = User::new(
            "alice".to_string(),
            "hash123".to_string(),
            "salt456".to_string(),
        );

        insert_user(&pool, &user).await?;

        // Find user
        let found = find_user_by_username(&pool, "alice").await?;
        assert!(found.is_some());
        assert_eq!(found.unwrap().username, "alice");

        Ok(())
    }
}
