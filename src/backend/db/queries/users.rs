//! User database queries

use crate::models::User;
use crate::utils::escape_like_pattern;
use sqlx::SqlitePool;
use tracing::warn;
use uuid::Uuid;

const SQL_SELECT_USER_FIELDS: &str =
    "SELECT id, username, password_hash, created_at, updated_at, deleted_at, is_online, last_seen_at";

/// Insert a new user into the database.
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn insert_user(pool: &SqlitePool, user: &User) -> Result<User, String> {
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, created_at, updated_at, is_online, deleted_at, last_seen_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&user.id)
    .bind(&user.username)
    .bind(&user.password_hash)
    .bind(user.created_at)
    .bind(user.updated_at)
    .bind(user.is_online)
    .bind(user.deleted_at)
    .bind(user.last_seen_at)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to insert user: {e}"))?;

    Ok(user.clone())
}

/// Find a user by username.
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn find_user_by_username(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<User>, String> {
    sqlx::query_as::<_, User>(&format!(
        "{SQL_SELECT_USER_FIELDS} FROM users WHERE username = ?"
    ))
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        warn!("Database error during user lookup: {e}");
        "Database operation failed".to_string()
    })
}

/// Find a user by ID.
///
/// # Errors
///
/// Returns an error string if the user ID format is invalid or the database operation fails.
pub async fn find_user_by_id(pool: &SqlitePool, user_id: &str) -> Result<Option<User>, String> {
    if Uuid::parse_str(user_id).is_err() {
        return Err("Invalid user ID format".to_string());
    }

    sqlx::query_as::<_, User>(&format!(
        "{SQL_SELECT_USER_FIELDS} FROM users WHERE id = ?"
    ))
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        warn!("Database error finding user by id: {e}");
        "Database operation failed".to_string()
    })
}


/// Find multiple users by their IDs.
///
/// # Errors
///
/// Returns an error string if too many IDs are provided, any ID format is invalid,
/// or the database operation fails.
pub async fn find_users_by_ids(
    pool: &SqlitePool,
    user_ids: &[String],
) -> Result<std::collections::HashMap<String, User>, String> {
    if user_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    if user_ids.len() > 1000 {
        return Err("Too many user IDs requested (max 1000)".to_string());
    }

    // SAFETY: Dynamic query construction is safe here because:
    // 1. Placeholders are fixed "?" strings, not user input
    // 2. All user_id values are bound as parameters via the query builder
    // 3. Each user_id is validated as a valid UUID before being bound
    // This prevents SQL injection while allowing efficient batch lookups.
    let placeholders: String = user_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let query = format!(
        "{SQL_SELECT_USER_FIELDS} FROM users WHERE id IN ({placeholders})"
    );

    let mut users = std::collections::HashMap::new();
    let mut query = sqlx::query_as::<_, User>(&query);

    for user_id in user_ids {
        if uuid::Uuid::parse_str(user_id).is_err() {
            return Err(format!("Invalid UUID format: {user_id}"));
        }
        query = query.bind(user_id);
    }

    let results = query
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to find users by ids: {e}"))?;

    for user in results {
        users.insert(user.id.clone(), user);
    }

    Ok(users)
}

/// Update user online status.
///
/// # Errors
///
/// Returns an error string if the user ID format is invalid or the database operation fails.
pub async fn update_online_status(
    pool: &SqlitePool,
    user_id: &str,
    is_online: bool,
) -> Result<(), String> {
    if Uuid::parse_str(user_id).is_err() {
        return Err("Invalid user ID format".to_string());
    }

    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query("UPDATE users SET is_online = ?, last_seen_at = ?, updated_at = ? WHERE id = ?")
        .bind(is_online)
        .bind(now)
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to update online status: {e}"))?;

    Ok(())
}

/// Update last seen timestamp for a user.
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn update_last_seen(pool: &SqlitePool, user_id: &str) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query("UPDATE users SET last_seen_at = ?, updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to update last seen: {e}"))?;

    Ok(())
}

/// Update user password.
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn update_password(
    pool: &SqlitePool,
    user_id: &str,
    new_hash: &str,
) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
        .bind(new_hash)
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to update password: {e}"))?;

    Ok(())
}

/// Delete a user (hard delete).
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn delete_user(pool: &SqlitePool, user_id: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to delete user: {e}"))?;

    sqlx::query("DELETE FROM conversations WHERE user1_id = ? OR user2_id = ?")
        .bind(user_id)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to delete user conversations: {e}"))?;

    Ok(())
}



/// Delete all conversations for a user.
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn delete_user_conversations(pool: &SqlitePool, user_id: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM conversations WHERE user1_id = ? OR user2_id = ?")
        .bind(user_id)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to delete user conversations: {e}"))?;

    Ok(())
}

/// Search users by username prefix (case-insensitive).
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn search_users_by_prefix(
    pool: &SqlitePool,
    query: &str,
    limit: u32,
) -> Result<Vec<User>, String> {
    let escaped = escape_like_pattern(&query.to_lowercase());
    let pattern = format!("{escaped}%");

    sqlx::query_as::<_, User>(&format!(
        "{SQL_SELECT_USER_FIELDS} FROM users WHERE LOWER(username) LIKE ? ESCAPE '\\' AND deleted_at IS NULL ORDER BY username LIMIT ?"
    ))
    .bind(pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to search users: {e}"))
}

/// Search users by username prefix, excluding the requester.
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn search_users_excluding_self(
    pool: &SqlitePool,
    query: &str,
    requester_id: &str,
    limit: u32,
) -> Result<Vec<User>, String> {
    let escaped = escape_like_pattern(&query.to_lowercase());
    let pattern = format!("{escaped}%");

    sqlx::query_as::<_, User>(&format!(
        "{SQL_SELECT_USER_FIELDS} FROM users WHERE LOWER(username) LIKE ? ESCAPE '\\' AND deleted_at IS NULL AND id != ? ORDER BY username LIMIT ?"
    ))
    .bind(pattern)
    .bind(requester_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to search users: {e}"))
}

/// Soft delete a user (sets `deleted_at` timestamp).
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn soft_delete_user(pool: &SqlitePool, user_id: &str) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query("UPDATE users SET deleted_at = ?, updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to soft delete user: {e}"))?;

    Ok(())
}
