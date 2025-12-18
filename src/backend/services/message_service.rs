// ! Message service for handling message creation, validation, and delivery
//!
//! Implements message validation, status tracking, and offline delivery logic

use crate::db::queries;
use crate::models::Message;
use sqlx::SqlitePool;
use tracing::{info, warn};

/// Message status enum
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    Pending,
    Sent,
    Delivered,
    Failed,
}

impl MessageStatus {
    pub fn as_str(&self) -> &str {
        match self {
            MessageStatus::Pending => "pending",
            MessageStatus::Sent => "sent",
            MessageStatus::Delivered => "delivered",
            MessageStatus::Failed => "failed",
        }
    }
}

#[derive(Clone)]
/// Message service
pub struct MessageService {
    pool: SqlitePool,
}

impl MessageService {
    /// Create a new message service
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Send a message
    ///
    /// Validates message content and recipient, then stores with 'pending' status
    /// Returns the created message
    pub async fn send_message(
        &self,
        conversation_id: String,
        sender_id: String,
        recipient_id: String,
        content: String,
    ) -> Result<Message, String> {
        // Validate content length (1-5000 characters)
        if content.is_empty() || content.len() > 5000 {
            warn!(
                target: "message",
                event = "message.send",
                conversation_id = %conversation_id,
                sender_id = %sender_id,
                recipient_id = %recipient_id,
                outcome = "failed",
                reason = "invalid_length",
                content_length = content.len()
            );
            return Err("Message content must be between 1 and 5000 characters".to_string());
        }

        // Validate UTF-8 (Rust strings are already UTF-8, but check for validity)
        if !content.is_ascii() && content.chars().any(|c| !c.is_valid()) {
            warn!(
                target: "message",
                event = "message.send",
                conversation_id = %conversation_id,
                sender_id = %sender_id,
                recipient_id = %recipient_id,
                outcome = "failed",
                reason = "invalid_utf8"
            );
            return Err("Message content contains invalid UTF-8 characters".to_string());
        }

        // Verify recipient exists and is not deleted
        let recipient = queries::find_user_by_id(&self.pool, &recipient_id)
            .await?
            .ok_or("Recipient not found".to_string())?;

        if recipient.is_deleted() {
            return Err("Cannot send message to deleted user".to_string());
        }

        // Verify sender is not deleted
        let sender = queries::find_user_by_id(&self.pool, &sender_id)
            .await?
            .ok_or("Sender not found".to_string())?;

        if sender.is_deleted() {
            return Err("Cannot send message from deleted account".to_string());
        }

        // Create message with generated UUID
        let message = Message::new(
            conversation_id.clone(),
            sender_id.clone(),
            recipient_id.clone(),
            content,
        );

        // Insert into database
        let created_message = queries::insert_message(&self.pool, &message).await?;
        info!(
            target: "message",
            event = "message.send",
            conversation_id = %conversation_id,
            sender_id = %sender_id,
            recipient_id = %recipient_id,
            message_id = %created_message.id,
            status = %created_message.status,
            "Message persisted"
        );

        Ok(created_message)
    }

    /// Send message with client-provided UUID (idempotency)
    ///
    /// If message with same ID exists, returns existing message (prevents duplicates)
    pub async fn send_message_with_id(
        &self,
        message_id: String,
        conversation_id: String,
        sender_id: String,
        recipient_id: String,
        content: String,
    ) -> Result<(Message, bool), String> {
        // Check if message already exists (idempotency)
        if let Some(existing) = queries::find_message_by_id(&self.pool, &message_id).await? {
            info!(
                target: "message",
                event = "message.idempotent",
                conversation_id = %conversation_id,
                sender_id = %sender_id,
                recipient_id = %recipient_id,
                message_id = %existing.id,
                status = %existing.status,
                "Duplicate message detected; returning existing record"
            );
            return Ok((existing, false)); // Not created, already exists
        }

        // Validate and create new message
        let mut message = self
            .send_message(conversation_id, sender_id, recipient_id, content)
            .await?;

        // Update ID to client-provided one
        message.id = message_id;
        info!(
            target: "message",
            event = "message.send",
            conversation_id = %message.conversation_id,
            sender_id = %message.sender_id,
            recipient_id = %message.recipient_id,
            message_id = %message.id,
            status = %message.status,
            "Message persisted with client-supplied id"
        );

        Ok((message, true)) // Created new message
    }

    /// Get messages for a conversation
    ///
    /// Returns messages ordered by created_at DESC (newest first)
    /// Supports pagination via limit and offset
    pub async fn get_conversation_messages(
        &self,
        conversation_id: &str,
        user_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Message>, String> {
        // Verify user is participant in conversation
        let conversation = queries::get_conversation_by_id(&self.pool, conversation_id)
            .await?
            .ok_or("Conversation not found".to_string())?;

        if conversation.user1_id != user_id && conversation.user2_id != user_id {
            return Err("User is not a participant in this conversation".to_string());
        }

        // Get messages
        queries::get_messages_by_conversation(&self.pool, conversation_id, limit, offset).await
    }

    /// Search messages within a conversation
    ///
    /// Returns matching messages with context
    pub async fn search_messages_in_conversation(
        &self,
        conversation_id: &str,
        user_id: &str,
        query: &str,
        limit: u32,
    ) -> Result<Vec<Message>, String> {
        // Verify user is participant
        let conversation = queries::get_conversation_by_id(&self.pool, conversation_id)
            .await?
            .ok_or("Conversation not found".to_string())?;

        if conversation.user1_id != user_id && conversation.user2_id != user_id {
            return Err("User is not a participant in this conversation".to_string());
        }

        queries::search_messages_in_conversation(&self.pool, conversation_id, query, limit).await
    }

    /// Get pending messages (for offline delivery retry)
    ///
    /// Returns messages with 'pending' or 'failed' status
    pub async fn get_pending_messages(&self, recipient_id: &str) -> Result<Vec<Message>, String> {
        queries::get_pending_messages(&self.pool, recipient_id).await
    }

    /// Update message status
    ///
    /// Transitions: pending -> sent -> delivered (or failed)
    pub async fn update_message_status(
        &self,
        message_id: &str,
        status: MessageStatus,
    ) -> Result<(), String> {
        let result = queries::update_message_status(&self.pool, message_id, status.as_str()).await;

        match &result {
            Ok(_) => info!(
                target: "message",
                event = "message.status",
                message_id = %message_id,
                status = %status.as_str(),
                "Message status updated"
            ),
            Err(err) => warn!(
                target: "message",
                event = "message.status",
                message_id = %message_id,
                status = %status.as_str(),
                outcome = "failed",
                error = %err
            ),
        }

        result
    }

    /// Mark message as delivered
    ///
    /// Sets delivered_at timestamp and status to 'delivered'
    pub async fn mark_delivered(&self, message_id: &str) -> Result<(), String> {
        let result = queries::mark_message_delivered(&self.pool, message_id).await;

        match &result {
            Ok(_) => info!(
                target: "message",
                event = "message.delivered",
                message_id = %message_id,
                "Marked message delivered"
            ),
            Err(err) => warn!(
                target: "message",
                event = "message.delivered",
                message_id = %message_id,
                outcome = "failed",
                error = %err
            ),
        }

        result
    }

    /// Sync delivery status updates (idempotent)
    ///
    /// Batch updates delivery status for multiple messages with idempotent logic.
    /// Only upgrades status (pending < sent < delivered < read), never downgrades.
    /// 
    /// Returns list of updated messages for confirmation back to client.
    pub async fn sync_delivery_status(
        &self,
        user_id: &str,
        updates: Vec<(String, String)>, // (message_id, new_status)
    ) -> Result<Vec<Message>, String> {
        let mut updated_messages = Vec::new();

        // Status hierarchy for idempotent updates
        let status_weight = |s: &str| -> u32 {
            match s {
                "pending" => 0,
                "sent" => 1,
                "delivered" => 2,
                "read" => 3,
                "failed" => 99, // Failed is separate
                _ => 0,
            }
        };

        for (message_id, new_status) in updates {
            // Load current message
            let current = match queries::find_message_by_id(&self.pool, &message_id).await {
                Ok(Some(msg)) => msg,
                _ => continue, // Skip if not found
            };

            // Verify authorization
            if current.sender_id != user_id && current.recipient_id != user_id {
                continue; // Skip unauthorized updates
            }

            // Check if update is valid (idempotent - only upgrade)
            let current_weight = status_weight(&current.status);
            let new_weight = status_weight(&new_status);

            if new_weight >= current_weight {
                // Apply idempotent update
                let now = chrono::Utc::now().timestamp_millis();
                
                match new_status.as_str() {
                    "read" => {
                        sqlx::query("UPDATE messages SET status = ?, read_at = ? WHERE id = ?")
                            .bind("read")
                            .bind(now)
                            .bind(&message_id)
                            .execute(&self.pool)
                            .await
                            .map_err(|e| format!("Failed to update message: {}", e))?;
                    }
                    "delivered" => {
                        sqlx::query("UPDATE messages SET status = ?, delivered_at = ? WHERE id = ?")
                            .bind("delivered")
                            .bind(now)
                            .bind(&message_id)
                            .execute(&self.pool)
                            .await
                            .map_err(|e| format!("Failed to update message: {}", e))?;
                    }
                    _ => {
                        sqlx::query("UPDATE messages SET status = ? WHERE id = ?")
                            .bind(&new_status)
                            .bind(&message_id)
                            .execute(&self.pool)
                            .await
                            .map_err(|e| format!("Failed to update message: {}", e))?;
                    }
                }

                // Fetch updated message and add to response
                if let Ok(Some(updated)) = queries::find_message_by_id(&self.pool, &message_id).await {
                    updated_messages.push(updated);
                    
                    info!(
                        target: "message",
                        event = "delivery_status.synced",
                        message_id = %message_id,
                        status = %new_status,
                        "Synced delivery status"
                    );
                }
            } else {
                // Status would downgrade - skip idempotently
                updated_messages.push(current);
            }
        }

        Ok(updated_messages)
    }

    /// Validate message content
    ///
    /// Returns true if content is valid, false otherwise
    pub fn validate_content(content: &str) -> bool {
        !content.is_empty() && content.len() <= 5000 && content.chars().all(|c| c.is_valid())
    }
}

trait CharValidator {
    fn is_valid(&self) -> bool;
}

impl CharValidator for char {
    fn is_valid(&self) -> bool {
        // Check if character is valid UTF-8 and not a control character (except newline/tab)
        !self.is_control() || *self == '\n' || *self == '\t'
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::User;

    async fn setup_test_db() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let schema_sql = include_str!("../../backend/db/migrations/001_initial_schema.sql");
        for statement in schema_sql.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(statement).execute(&pool).await.unwrap();
        }

        pool
    }

    #[tokio::test]
    async fn test_send_message_valid() {
        let pool = setup_test_db().await;
        let service = MessageService::new(pool.clone());

        // Create users and conversation
        let user1 = User::new(
            "alice".to_string(),
            "hash1".to_string(),
            "salt1".to_string(),
        );
        let user2 = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());

        queries::insert_user(&pool, &user1).await.unwrap();
        queries::insert_user(&pool, &user2).await.unwrap();

        // Sort user IDs to satisfy database constraint
        let (user1_id, user2_id) = if user1.id < user2.id {
            (user1.id.clone(), user2.id.clone())
        } else {
            (user2.id.clone(), user1.id.clone())
        };
        let conv = crate::models::Conversation::new(user1_id, user2_id);
        queries::insert_conversation(&pool, &conv).await.unwrap();

        // Send message
        let message = service
            .send_message(
                conv.id.clone(),
                user1.id.clone(),
                user2.id.clone(),
                "Hello, Bob!".to_string(),
            )
            .await
            .unwrap();

        assert_eq!(message.content, "Hello, Bob!");
        assert_eq!(message.sender_id, user1.id);
    }

    #[tokio::test]
    async fn test_validate_content_length() {
        assert!(MessageService::validate_content("Valid message"));
        assert!(!MessageService::validate_content("")); // Too short
        assert!(!MessageService::validate_content(&"a".repeat(5001))); // Too long
    }

    #[tokio::test]
    async fn test_prevent_send_to_deleted_user() {
        let pool = setup_test_db().await;
        let service = MessageService::new(pool.clone());

        let user1 = User::new(
            "alice".to_string(),
            "hash1".to_string(),
            "salt1".to_string(),
        );
        let user2 = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());

        queries::insert_user(&pool, &user1).await.unwrap();
        queries::insert_user(&pool, &user2).await.unwrap();

        // Mark user2 as deleted
        queries::soft_delete_user(&pool, &user2.id).await.unwrap();

        // Sort user IDs to satisfy database constraint
        let (user1_id, user2_id) = if user1.id < user2.id {
            (user1.id.clone(), user2.id.clone())
        } else {
            (user2.id.clone(), user1.id.clone())
        };
        let conv = crate::models::Conversation::new(user1_id, user2_id);
        queries::insert_conversation(&pool, &conv).await.unwrap();

        // Try to send message
        let result = service
            .send_message(conv.id, user1.id, user2.id, "Hello".to_string())
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("deleted"));
    }
}
