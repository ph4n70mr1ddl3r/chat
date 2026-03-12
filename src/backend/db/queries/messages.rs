//! Message database queries

use crate::models::Message;
use sqlx::SqlitePool;

const SQL_SELECT_MESSAGE_FIELDS: &str =
    "SELECT id, conversation_id, sender_id, recipient_id, content, created_at, delivered_at, read_at, status, is_anonymized";

/// Valid message status values
pub const VALID_STATUSES: &[&str] = &["pending", "sent", "delivered", "read", "failed"];

/// Insert a new message (returns false if duplicate ID).
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn insert_message_or_ignore(
    pool: &SqlitePool,
    message: &Message,
) -> Result<bool, String> {
    let result = sqlx::query(
        "INSERT OR IGNORE INTO messages (id, conversation_id, sender_id, recipient_id, content, created_at, delivered_at, read_at, status, is_anonymized)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&message.id)
    .bind(&message.conversation_id)
    .bind(&message.sender_id)
    .bind(&message.recipient_id)
    .bind(&message.content)
    .bind(message.created_at)
    .bind(message.delivered_at)
    .bind(message.read_at)
    .bind(&message.status)
    .bind(message.is_anonymized)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to insert message: {e}"))?;

    Ok(result.rows_affected() > 0)
}

/// Insert a new message.
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn insert_message(pool: &SqlitePool, message: &Message) -> Result<Message, String> {
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, sender_id, recipient_id, content, created_at, delivered_at, read_at, status, is_anonymized)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&message.id)
    .bind(&message.conversation_id)
    .bind(&message.sender_id)
    .bind(&message.recipient_id)
    .bind(&message.content)
    .bind(message.created_at)
    .bind(message.delivered_at)
    .bind(message.read_at)
    .bind(&message.status)
    .bind(message.is_anonymized)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to insert message: {e}"))?;

    Ok(message.clone())
}

/// Find message by ID.
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn find_message_by_id(
    pool: &SqlitePool,
    message_id: &str,
) -> Result<Option<Message>, String> {
    sqlx::query_as::<_, Message>(&format!(
        "{SQL_SELECT_MESSAGE_FIELDS} FROM messages WHERE id = ?"
    ))
    .bind(message_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to find message by id: {e}"))
}

/// Get messages by conversation (sorted by `created_at` DESC).
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn get_messages_by_conversation(
    pool: &SqlitePool,
    conversation_id: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<Message>, String> {
    let limit = limit.min(100);
    let offset = offset.min(10_000);

    sqlx::query_as::<_, Message>(&format!(
        "{SQL_SELECT_MESSAGE_FIELDS} FROM messages WHERE conversation_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
    ))
    .bind(conversation_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get messages by conversation: {e}"))
}

/// Get pending messages for a recipient (status = 'pending' or 'failed').
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn get_pending_messages(
    pool: &SqlitePool,
    recipient_id: &str,
) -> Result<Vec<Message>, String> {
    sqlx::query_as::<_, Message>(&format!(
        "{SQL_SELECT_MESSAGE_FIELDS} FROM messages WHERE recipient_id = ? AND (status = 'pending' OR status = 'failed') ORDER BY created_at ASC"
    ))
    .bind(recipient_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get pending messages: {e}"))
}

/// Get all pending messages (for queue initialization).
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn get_all_pending_messages(pool: &SqlitePool) -> Result<Vec<Message>, String> {
    sqlx::query_as::<_, Message>(&format!(
        "{SQL_SELECT_MESSAGE_FIELDS} FROM messages WHERE status = 'pending' OR status = 'failed' ORDER BY created_at ASC"
    ))
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get all pending messages: {e}"))
}

/// Update message status.
///
/// # Errors
///
/// Returns an error string if the status is invalid or the database operation fails.
pub async fn update_message_status(
    pool: &SqlitePool,
    message_id: &str,
    status: &str,
) -> Result<(), String> {
    if !VALID_STATUSES.contains(&status) {
        return Err(format!("Invalid message status: {status}"));
    }

    sqlx::query("UPDATE messages SET status = ? WHERE id = ?")
        .bind(status)
        .bind(message_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to update message status: {e}"))?;

    Ok(())
}

/// Mark message as delivered.
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn mark_message_delivered(pool: &SqlitePool, message_id: &str) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query("UPDATE messages SET status = 'delivered', delivered_at = ? WHERE id = ?")
        .bind(now)
        .bind(message_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to mark message delivered: {e}"))?;

    Ok(())
}

/// Anonymize all messages for a user (GDPR compliance).
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn anonymize_user_messages(pool: &SqlitePool, user_id: &str) -> Result<(), String> {
    sqlx::query(
        "UPDATE messages SET content = '[Message deleted]', is_anonymized = 1 WHERE sender_id = ?",
    )
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to anonymize user messages: {e}"))?;

    Ok(())
}

/// Search messages within a conversation.
///
/// # Errors
///
/// Returns an error string if the database operation fails.
pub async fn search_messages_in_conversation(
    pool: &SqlitePool,
    conversation_id: &str,
    query: &str,
    limit: u32,
) -> Result<Vec<Message>, String> {
    let pattern = format!("%{query}%");
    let limit = limit.min(100);

    sqlx::query_as::<_, Message>(&format!(
        "{SQL_SELECT_MESSAGE_FIELDS} FROM messages WHERE conversation_id = ? AND content LIKE ? ORDER BY created_at DESC LIMIT ?"
    ))
    .bind(conversation_id)
    .bind(pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to search messages: {e}"))
}
