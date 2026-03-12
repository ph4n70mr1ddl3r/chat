//! Conversation database queries

use crate::models::Conversation;
use sqlx::SqlitePool;

const SQL_SELECT_CONVERSATION_FIELDS: &str =
    "SELECT c.id, c.user1_id, c.user2_id, c.created_at, c.updated_at, c.last_message_at, c.message_count,
            (SELECT content FROM messages WHERE conversation_id = c.id ORDER BY created_at DESC LIMIT 1) as last_message";

/// Insert a new conversation
///
/// Returns the conversation if successful.
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn insert_conversation(
    pool: &SqlitePool,
    conversation: &Conversation,
) -> Result<Conversation, String> {
    sqlx::query(
        "INSERT INTO conversations (id, user1_id, user2_id, created_at, updated_at, last_message_at, message_count)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
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
    .map_err(|e| format!("Failed to insert conversation: {e}"))?;

    Ok(conversation.clone())
}

/// Get conversation by user pair (`user1_id` < `user2_id`)
///
/// Returns the conversation if found, None if not found.
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn get_conversation_by_users(
    pool: &SqlitePool,
    user1_id: &str,
    user2_id: &str,
) -> Result<Option<Conversation>, String> {
    sqlx::query_as::<_, Conversation>(&format!(
        "{SQL_SELECT_CONVERSATION_FIELDS} FROM conversations c WHERE c.user1_id = ? AND c.user2_id = ?"
    ))
    .bind(user1_id)
    .bind(user2_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to get conversation by users: {e}"))
}

/// Get conversation by ID.
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn get_conversation_by_id(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<Option<Conversation>, String> {
    sqlx::query_as::<_, Conversation>(&format!(
        "{SQL_SELECT_CONVERSATION_FIELDS} FROM conversations c WHERE c.id = ?"
    ))
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to get conversation by id: {e}"))
}

/// Get all conversations for a user.
///
/// Returns conversations sorted by `last_message_at` DESC (most recent first).
/// Supports pagination via limit and offset.
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn get_user_conversations(
    pool: &SqlitePool,
    user_id: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<Conversation>, String> {
    let limit = limit.min(100);
    let offset = offset.min(10_000);

    sqlx::query_as::<_, Conversation>(&format!(
        "{SQL_SELECT_CONVERSATION_FIELDS} FROM conversations c WHERE c.user1_id = ? OR c.user2_id = ? ORDER BY c.last_message_at DESC LIMIT ? OFFSET ?"
    ))
    .bind(user_id)
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get user conversations: {e}"))
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
