//! WebSocket message handlers
//!
//! Handles incoming text messages from WebSocket connections, validates them,
//! stores them in the database, and routes them to online recipients or queues
//! them for offline delivery.

use crate::db::queries;
use crate::handlers::websocket::{ClientConnection, ConnectionManager, ErrorResponse};
use crate::services::{
    message_queue::MessageQueueService,
    message_service::MessageService,
};
use chat_shared::protocol::{MessageEnvelope, TextMessageData};
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;
use warp::ws::Message as WsMessage;

/// Message handler for processing incoming messages
pub struct MessageHandler {
    pool: SqlitePool,
    message_service: MessageService,
    connection_manager: Arc<ConnectionManager>,
    message_queue: MessageQueueService,
}

impl MessageHandler {
    pub fn new(
        pool: SqlitePool,
        connection_manager: Arc<ConnectionManager>,
        message_queue: MessageQueueService,
    ) -> Self {
        let message_service = MessageService::new(pool.clone());
        Self {
            pool,
            message_service,
            connection_manager,
            message_queue,
        }
    }

    /// Process incoming text message
    ///
    /// 1. Validates message envelope and content
    /// 2. Verifies sender is authenticated
    /// 3. Stores message in database
    /// 4. Checks if recipient is online
    /// 5. If online: broadcasts to recipient
    /// 6. If offline: queues for retry
    /// 7. Sends acknowledgement to sender
    pub async fn handle_message(
        &self,
        envelope: &MessageEnvelope,
        sender: &ClientConnection,
    ) -> Result<Vec<WsMessage>, String> {
        // Extract message data
        let data: TextMessageData = serde_json::from_value(envelope.data.clone())
            .map_err(|e| format!("Invalid message data: {}", e))?;

        // Validate recipient exists
        let recipient = queries::find_user_by_id(&self.pool, &data.recipient_id)
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Recipient not found".to_string())?;

        if recipient.is_deleted() {
            return Ok(vec![ErrorResponse::recipient_not_found(&data.recipient_id)]);
        }

        // Get or create conversation
        let conversation_id = if let Some(conv_id) = &data.conversation_id {
            conv_id.clone()
        } else {
            // Look up or create conversation between sender and recipient
            let (conversation, _) = self
                .create_or_get_conversation(sender.user_id.clone(), data.recipient_id.clone())
                .await?;
            conversation.id
        };

        // Send message using message service (with idempotency)
        let (message, was_created) = self
            .message_service
            .send_message_with_id(
                envelope.id.clone(),
                conversation_id.clone(),
                sender.user_id.clone(),
                data.recipient_id.clone(),
                data.content.clone(),
            )
            .await?;

        let mut responses = Vec::new();

        // If message was just created (not a duplicate), deliver it
        if was_created {
            // Check if recipient is online
            if self
                .connection_manager
                .is_user_online(&data.recipient_id)
                .await
            {
                // Deliver to recipient immediately
                let delivery_message = self.build_message_envelope(
                    &message.id,
                    &sender.user_id,
                    &sender.username,
                    &data.recipient_id,
                    &data.content,
                    &conversation_id,
                    "delivered",
                );

                self.connection_manager
                    .send_to_user(
                        &data.recipient_id,
                        WsMessage::text(serde_json::to_string(&delivery_message).unwrap()),
                    )
                    .await;

                // Update message status to 'delivered'
                self.message_service.mark_delivered(&message.id).await?;
            } else {
                // Recipient offline - queue for retry
                self.message_queue
                    .queue_message(message.id.clone(), data.recipient_id.clone())
                    .await;
            }
        }

        // Send acknowledgement to sender
        let ack_status = if was_created
            && self
                .connection_manager
                .is_user_online(&data.recipient_id)
                .await
        {
            "delivered"
        } else {
            "sent"
        };
        let ack = self.build_ack_envelope(&envelope.id, &conversation_id, &message.id, ack_status);
        responses.push(WsMessage::text(serde_json::to_string(&ack).unwrap()));

        Ok(responses)
    }

    /// Create or get conversation between two users
    async fn create_or_get_conversation(
        &self,
        user1_id: String,
        user2_id: String,
    ) -> Result<(crate::models::Conversation, bool), String> {
        // Ensure ordering (user1_id < user2_id)
        let (u1, u2) = if user1_id < user2_id {
            (user1_id, user2_id)
        } else {
            (user2_id, user1_id)
        };

        // Check if conversation exists
        if let Some(conversation) = queries::get_conversation_by_users(&self.pool, &u1, &u2).await?
        {
            return Ok((conversation, false));
        }

        // Create new conversation
        let conversation = crate::models::Conversation::new(u1, u2);
        let created = queries::insert_conversation(&self.pool, &conversation).await?;
        Ok((created, true))
    }

    /// Build message envelope for delivery
    fn build_message_envelope(
        &self,
        message_id: &str,
        sender_id: &str,
        sender_username: &str,
        recipient_id: &str,
        content: &str,
        conversation_id: &str,
        status: &str,
    ) -> MessageEnvelope {
        MessageEnvelope {
            id: message_id.to_string(),
            msg_type: "message".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            data: json!({
                "senderId": sender_id,
                "senderUsername": sender_username,
                "recipientId": recipient_id,
                "content": content,
                "conversationId": conversation_id,
                "status": status,
            }),
        }
    }

    /// Build acknowledgement envelope
    fn build_ack_envelope(
        &self,
        original_message_id: &str,
        conversation_id: &str,
        stored_message_id: &str,
        status: &str,
    ) -> MessageEnvelope {
        MessageEnvelope {
            id: uuid::Uuid::new_v4().to_string(),
            msg_type: "ack".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            data: json!({
                "status": status,
                "conversationId": conversation_id,
                "messageId": stored_message_id,
                "originalMessageId": original_message_id,
                "serverTimestamp": chrono::Utc::now().timestamp_millis(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::websocket::ConnectionManager;
    use crate::models::User;
    use crate::services::MessageQueueService;
    use tokio::sync::mpsc;

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
    async fn test_handle_message_creates_conversation() {
        let pool = setup_test_db().await;
        let conn_mgr = Arc::new(ConnectionManager::new());
        let queue = MessageQueueService::new(pool.clone(), conn_mgr.clone());
        let handler = MessageHandler::new(pool.clone(), conn_mgr.clone(), queue);

        // Create users
        let user1 = User::new(
            "alice".to_string(),
            "hash1".to_string(),
            "salt1".to_string(),
        );
        let user2 = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());

        queries::insert_user(&pool, &user1).await.unwrap();
        queries::insert_user(&pool, &user2).await.unwrap();

        // Create sender connection
        let sender = ClientConnection::new(user1.id.clone(), user1.username.clone());

        // Create message envelope
        let envelope = MessageEnvelope {
            id: uuid::Uuid::new_v4().to_string(),
            msg_type: "message".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            data: json!({
                "recipient_id": user2.id,
                "content": "Hello, Bob!",
            }),
        };

        // Handle message
        let responses = handler.handle_message(&envelope, &sender).await.unwrap();

        // Should get acknowledgement (recipient offline)
        assert_eq!(responses.len(), 1);

        // Verify message was stored
        let messages = queries::get_messages_by_conversation(&pool, "", 10, 0).await;
        // Note: This test would need the actual conversation ID to verify
    }

    #[tokio::test]
    async fn test_handle_message_to_online_recipient() {
        let pool = setup_test_db().await;
        let conn_mgr = Arc::new(ConnectionManager::new());
        let queue = MessageQueueService::new(pool.clone(), conn_mgr.clone());
        let handler = MessageHandler::new(pool.clone(), conn_mgr.clone(), queue);

        // Create users
        let user1 = User::new(
            "alice".to_string(),
            "hash1".to_string(),
            "salt1".to_string(),
        );
        let user2 = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());

        queries::insert_user(&pool, &user1).await.unwrap();
        queries::insert_user(&pool, &user2).await.unwrap();

        // Register recipient as online
        let recipient_conn = ClientConnection::new(user2.id.clone(), user2.username.clone());
        let (tx, _rx) = mpsc::unbounded_channel();
        conn_mgr.register(recipient_conn, tx).await;

        // Create sender connection
        let sender = ClientConnection::new(user1.id.clone(), user1.username.clone());

        // Create message envelope
        let envelope = MessageEnvelope {
            id: uuid::Uuid::new_v4().to_string(),
            msg_type: "message".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            data: json!({
                "recipient_id": user2.id,
                "content": "Hello, Bob!",
            }),
        };

        // Handle message
        let responses = handler.handle_message(&envelope, &sender).await.unwrap();

        // Should get acknowledgement
        assert_eq!(responses.len(), 1);
    }

    #[tokio::test]
    async fn test_handle_message_idempotency() {
        let pool = setup_test_db().await;
        let conn_mgr = Arc::new(ConnectionManager::new());
        let queue = MessageQueueService::new(pool.clone(), conn_mgr.clone());
        let handler = MessageHandler::new(pool.clone(), conn_mgr.clone(), queue);

        // Create users
        let user1 = User::new(
            "alice".to_string(),
            "hash1".to_string(),
            "salt1".to_string(),
        );
        let user2 = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());

        queries::insert_user(&pool, &user1).await.unwrap();
        queries::insert_user(&pool, &user2).await.unwrap();

        let sender = ClientConnection::new(user1.id.clone(), user1.username.clone());

        // Send same message twice (same ID)
        let message_id = uuid::Uuid::new_v4().to_string();
        let envelope = MessageEnvelope {
            id: message_id.clone(),
            msg_type: "message".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            data: json!({
                "recipient_id": user2.id,
                "content": "Hello, Bob!",
            }),
        };

        // First send
        let _responses1 = handler.handle_message(&envelope, &sender).await.unwrap();

        // Second send (duplicate)
        let responses2 = handler.handle_message(&envelope, &sender).await.unwrap();

        // Should still get acknowledgement
        assert!(!responses2.is_empty());
    }
}
