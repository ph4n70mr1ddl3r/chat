//! User database queries
//!
//! Provides database operations for user management including insertion, lookup, and updates.

use crate::models::{Conversation, Message, User};
use sqlx::SqlitePool;
use uuid::Uuid;

/// Auth event types
#[derive(Debug, Clone)]
pub enum AuthEventType {
    LoginSuccess,
    LoginFailed,
    Signup,
    Logout,
}

impl AuthEventType {
    pub fn as_str(&self) -> &str {
        match self {
            AuthEventType::LoginSuccess => "login_success",
            AuthEventType::LoginFailed => "login_failed",
            AuthEventType::Signup => "signup",
            AuthEventType::Logout => "logout",
        }
    }
}

/// Insert an auth log entry
pub async fn insert_auth_log(
    pool: &SqlitePool,
    ip_address: &str,
    username: Option<&str>,
    event_type: AuthEventType,
    user_agent: Option<&str>,
    details: Option<&str>,
) -> Result<(), String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query(
        "INSERT INTO auth_logs (id, ip_address, username, event_type, created_at, user_agent, details)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(id)
    .bind(ip_address)
    .bind(username)
    .bind(event_type.as_str())
    .bind(now)
    .bind(user_agent)
    .bind(details)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to insert auth log: {}", e))?;

    Ok(())
}

/// Get failed login attempts for an IP address within a time window
pub async fn get_failed_attempts(
    pool: &SqlitePool,
    ip_address: &str,
    window_seconds: i64,
) -> Result<u32, String> {
    let now = chrono::Utc::now().timestamp_millis();
    let window_start = now - (window_seconds * 1000);

    let result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM auth_logs 
         WHERE ip_address = ? AND event_type = 'login_failed' AND created_at > ?"
    )
    .bind(ip_address)
    .bind(window_start)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to get failed attempts: {}", e))?;

    Ok(result as u32)
}

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
pub async fn find_user_by_username(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<User>, String> {
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
pub async fn update_online_status(
    pool: &SqlitePool,
    user_id: &str,
    is_online: bool,
) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query("UPDATE users SET is_online = ?, last_seen_at = ?, updated_at = ? WHERE id = ?")
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

    sqlx::query("UPDATE users SET last_seen_at = ?, updated_at = ? WHERE id = ?")
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

    sqlx::query("UPDATE users SET deleted_at = ?, updated_at = ? WHERE id = ?")
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
/// Excludes deleted users; limit specifies max results
pub async fn search_users_by_prefix(
    pool: &SqlitePool,
    query: &str,
    limit: u32,
) -> Result<Vec<User>, String> {
    let search_pattern = format!("{}%", query);

    sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, password_salt, created_at, updated_at, deleted_at, is_online, last_seen_at
         FROM users
         WHERE username LIKE ? AND deleted_at IS NULL
         LIMIT ?"
    )
    .bind(search_pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to search users: {}", e))
}

/// Search users excluding self
pub async fn search_users_excluding_self(
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

/// Insert a new conversation
pub async fn insert_conversation(
    pool: &SqlitePool,
    conversation: &Conversation,
) -> Result<Conversation, String> {
    sqlx::query(
        "INSERT INTO conversations (id, user1_id, user2_id, created_at, updated_at, last_message_at, message_count)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&conversation.id)
    .bind(&conversation.user1_id)
    .bind(&conversation.user2_id)
    .bind(conversation.created_at)
    .bind(conversation.updated_at)
    .bind(conversation.last_message_at)
    .bind(conversation.message_count)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to insert conversation: {}", e))?;

    Ok(conversation.clone())
}

/// Get conversation by user pair (user1_id < user2_id)
pub async fn get_conversation_by_users(
    pool: &SqlitePool,
    user1_id: &str,
    user2_id: &str,
) -> Result<Option<Conversation>, String> {
    sqlx::query_as::<_, Conversation>(
        "SELECT id, user1_id, user2_id, created_at, updated_at, last_message_at, message_count
         FROM conversations
         WHERE user1_id = ? AND user2_id = ?"
    )
    .bind(user1_id)
    .bind(user2_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to get conversation by users: {}", e))
}

/// Get conversation by ID
pub async fn get_conversation_by_id(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<Option<Conversation>, String> {
    sqlx::query_as::<_, Conversation>(
        "SELECT id, user1_id, user2_id, created_at, updated_at, last_message_at, message_count
         FROM conversations
         WHERE id = ?"
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to get conversation by id: {}", e))
}

/// Get all conversations for a user (sorted by last_message_at DESC)
pub async fn get_user_conversations(
    pool: &SqlitePool,
    user_id: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<Conversation>, String> {
    sqlx::query_as::<_, Conversation>(
        "SELECT id, user1_id, user2_id, created_at, updated_at, last_message_at, message_count
         FROM conversations
         WHERE user1_id = ? OR user2_id = ?
         ORDER BY updated_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(user_id)
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get user conversations: {}", e))
}

// ============================================================================
// Message Queries
// ============================================================================

/// Insert a new message
pub async fn insert_message(pool: &SqlitePool, message: &Message) -> Result<Message, String> {
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, sender_id, recipient_id, content, created_at, delivered_at, status, is_anonymized)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&message.id)
    .bind(&message.conversation_id)
    .bind(&message.sender_id)
    .bind(&message.recipient_id)
    .bind(&message.content)
    .bind(message.created_at)
    .bind(message.delivered_at)
    .bind(&message.status)
    .bind(message.is_anonymized)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to insert message: {}", e))?;

    Ok(message.clone())
}

/// Find message by ID
pub async fn find_message_by_id(
    pool: &SqlitePool,
    message_id: &str,
) -> Result<Option<Message>, String> {
    sqlx::query_as::<_, Message>(
        "SELECT id, conversation_id, sender_id, recipient_id, content, created_at, delivered_at, status, is_anonymized
         FROM messages
         WHERE id = ?"
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to find message by id: {}", e))
}

/// Get messages by conversation (sorted by created_at DESC)
pub async fn get_messages_by_conversation(
    pool: &SqlitePool,
    conversation_id: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<Message>, String> {
    sqlx::query_as::<_, Message>(
        "SELECT id, conversation_id, sender_id, recipient_id, content, created_at, delivered_at, status, is_anonymized
         FROM messages
         WHERE conversation_id = ?
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(conversation_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get messages by conversation: {}", e))
}

/// Get pending messages for a recipient (status = 'pending' or 'failed')
pub async fn get_pending_messages(
    pool: &SqlitePool,
    recipient_id: &str,
) -> Result<Vec<Message>, String> {
    sqlx::query_as::<_, Message>(
        "SELECT id, conversation_id, sender_id, recipient_id, content, created_at, delivered_at, status, is_anonymized
         FROM messages
         WHERE recipient_id = ? AND (status = 'pending' OR status = 'failed')
         ORDER BY created_at ASC"
    )
    .bind(recipient_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get pending messages: {}", e))
}

/// Get all pending messages (status = 'pending' or 'failed') for queue initialization
pub async fn get_all_pending_messages(pool: &SqlitePool) -> Result<Vec<Message>, String> {
    sqlx::query_as::<_, Message>(
        "SELECT id, conversation_id, sender_id, recipient_id, content, created_at, delivered_at, status, is_anonymized
         FROM messages
         WHERE status = 'pending' OR status = 'failed'
         ORDER BY created_at ASC"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get all pending messages: {}", e))
}

/// Update message status
pub async fn update_message_status(
    pool: &SqlitePool,
    message_id: &str,
    status: &str,
) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query("UPDATE messages SET status = ? WHERE id = ?")
        .bind(status)
        .bind(message_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to update message status: {}", e))?;

    Ok(())
}

/// Mark message as delivered (sets delivered_at and status = 'delivered')
pub async fn mark_message_delivered(pool: &SqlitePool, message_id: &str) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query("UPDATE messages SET status = 'delivered', delivered_at = ? WHERE id = ?")
        .bind(now)
        .bind(message_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to mark message delivered: {}", e))?;

    Ok(())
}

/// Anonymize messages from a deleted user
pub async fn anonymize_user_messages(pool: &SqlitePool, user_id: &str) -> Result<(), String> {
    sqlx::query("UPDATE messages SET is_anonymized = TRUE WHERE sender_id = ?")
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to anonymize messages: {}", e))?;

    Ok(())
}

/// Search messages in conversation by content
pub async fn search_messages_in_conversation(
    pool: &SqlitePool,
    conversation_id: &str,
    search_query: &str,
    limit: u32,
) -> Result<Vec<Message>, String> {
    let search_pattern = format!("%{}%", search_query);

    sqlx::query_as::<_, Message>(
        "SELECT id, conversation_id, sender_id, recipient_id, content, created_at, delivered_at, status, is_anonymized
         FROM messages
         WHERE conversation_id = ? AND content LIKE ?
         ORDER BY created_at DESC
         LIMIT ?"
    )
    .bind(conversation_id)
    .bind(search_pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to search messages: {}", e))
}

/// Soft delete user helper
pub async fn soft_delete_user(pool: &SqlitePool, user_id: &str) -> Result<(), String> {
    delete_user(pool, user_id).await?;
    anonymize_user_messages(pool, user_id).await?;
    Ok(())
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
