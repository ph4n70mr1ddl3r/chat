//! Conversation database queries

use crate::models::Conversation;
use sqlx::SqlitePool;
use uuid::Uuid;

const SQL_SELECT_CONVERSATION_FIELDS: &str =
    "SELECT c.id, c.user1_id, c.user2_id, c.created_at, c.updated_at, c.last_message_at, c.message_count,
            (SELECT content FROM messages WHERE conversation_id = c.id ORDER BY created_at DESC LIMIT 1) as last_message";

/// Insert a new conversation
///
/// Returns `(conversation, was_inserted: bool)`.
/// If a conversation with the same (user1_id, user2_id) already exists,
/// returns `was_inserted = false` and the existing conversation.
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn insert_conversation(
    pool: &SqlitePool,
    conversation: &Conversation,
) -> Result<(Conversation, bool), String> {
    let result = sqlx::query(
        "INSERT OR IGNORE INTO conversations (id, user1_id, user2_id, created_at, updated_at, last_message_at, message_count)
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

    if result.rows_affected() > 0 {
        Ok((conversation.clone(), true))
    } else {
        let existing = get_conversation_by_users(pool, &conversation.user1_id, &conversation.user2_id)
            .await?
            .ok_or_else(|| "Conversation should exist after INSERT OR IGNORE failed".to_string())?;
        Ok((existing, false))
    }
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
/// Returns an error string if the conversation ID format is invalid or the database operation fails.
pub async fn get_conversation_by_id(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<Option<Conversation>, String> {
    if Uuid::parse_str(conversation_id).is_err() {
        return Err("Invalid conversation ID format".to_string());
    }

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

/// Update conversation stats after a new message.
///
/// Updates `last_message_at`, `message_count`, and `updated_at`.
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn update_conversation_stats(
    pool: &SqlitePool,
    conversation_id: &str,
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE conversations SET last_message_at = ?, message_count = message_count + 1, updated_at = ? WHERE id = ?"
    )
    .bind(now)
    .bind(now)
    .bind(conversation_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update conversation stats: {e}"))?;

    Ok(())
}
