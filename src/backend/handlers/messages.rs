use crate::db::queries;
use crate::handlers::websocket::{ClientConnection, ConnectionManager, ErrorResponse};
use crate::models::MAX_MESSAGE_LENGTH;
use crate::services::{
    conversation_service::ConversationService, message_queue::MessageQueueService,
    message_service::MessageService,
};
use chat_shared::protocol::{MessageEnvelope, TextMessageData};
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::warn;
use unicode_normalization::UnicodeNormalization;
use warp::ws::Message as WsMessage;

const MAX_DELIVERY_STATUS_BATCH: usize = 100;

/// Basic HTML sanitization for message content
///
/// Performs HTML entity encoding to prevent XSS attacks by escaping:
/// - `&` -> `&amp;`
/// - `<` -> `&lt;`
/// - `>` -> `&gt;`
/// - `"` -> `&quot;`
/// - `'` -> `&#x27;`
/// - `/` -> `&#x2F;`
///
/// Also strips null bytes and control characters that could be used for
/// log injection or string truncation attacks.
///
/// # Limitations
/// This is a basic sanitization approach suitable for plain text messages.
/// For rich text or HTML content, consider using a dedicated sanitization library
/// like `ammonia` or `blake2` for more comprehensive protection.
///
/// # Security Note
/// This function escapes characters to prevent HTML injection but does not
/// validate or sanitize URL schemes, CSS, or JavaScript in more complex scenarios.
fn sanitize_html(input: &str) -> String {
    let mut result = String::with_capacity(input.len() + input.len() / 4);
    for c in input.chars() {
        if c == '\0' || (c.is_control() && c != '\n' && c != '\r' && c != '\t') {
            continue;
        }
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#x27;"),
            '/' => result.push_str("&#x2F;"),
            _ => result.push(c),
        }
    }
    result
}

/// Parameters for building a message envelope
#[derive(Debug, Clone)]
struct MessageParams<'a> {
    message_id: &'a str,
    sender_id: &'a str,
    sender_username: &'a str,
    recipient_id: &'a str,
    content: &'a str,
    conversation_id: &'a str,
    status: &'a str,
}

/// Message handler for processing incoming messages
pub struct MessageHandler {
    pool: SqlitePool,
    message_service: MessageService,
    connection_manager: Arc<ConnectionManager>,
    message_queue: MessageQueueService,
    conversation_service: ConversationService,
}

impl MessageHandler {
    pub fn new(
        pool: SqlitePool,
        connection_manager: Arc<ConnectionManager>,
        message_queue: MessageQueueService,
    ) -> Self {
        let message_service = MessageService::new(pool.clone());
        let conversation_service = ConversationService::new(pool.clone());
        Self {
            pool,
            message_service,
            connection_manager,
            message_queue,
            conversation_service,
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

        // Validate message content
        if data.content.trim().is_empty() {
            return Ok(vec![ErrorResponse::invalid_message(
                "Message content cannot be empty",
            )]);
        }
        if data.content.len() > MAX_MESSAGE_LENGTH {
            return Ok(vec![ErrorResponse::invalid_message(&format!(
                "Message content exceeds {} character limit",
                MAX_MESSAGE_LENGTH
            ))]);
        }
        
        // Validate characters in content
        let has_invalid_chars = data
            .content
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t');
        if has_invalid_chars {
            return Ok(vec![ErrorResponse::invalid_message(
                "Message contains invalid control characters",
            )]);
        }

        // Prevent self-messaging
        if data.recipient_id == sender.user_id {
            return Ok(vec![ErrorResponse::invalid_message(
                "Cannot send message to yourself",
            )]);
        }

        // Validate recipient_id is a valid UUID format before database query
        if uuid::Uuid::parse_str(&data.recipient_id).is_err() {
            return Ok(vec![ErrorResponse::invalid_message(
                "Invalid recipient ID format",
            )]);
        }

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
            let conversation_id = conv_id.clone();
            
            // Validate conversation_id is a valid UUID
            if uuid::Uuid::parse_str(&conversation_id).is_err() {
                return Ok(vec![ErrorResponse::invalid_message(
                    "Invalid conversation ID format",
                )]);
            }
            
            // Verify sender is a participant in the conversation
            let conversation = queries::get_conversation_by_id(&self.pool, &conversation_id)
                .await
                .map_err(|e| format!("Database error: {}", e))?
                .ok_or_else(|| "Conversation not found".to_string())?;

            // Verify sender is a participant and recipient is the other participant
            let (sender_is_user1, recipient_is_user2) = (
                conversation.user1_id == sender.user_id,
                conversation.user2_id == data.recipient_id,
            );
            let (sender_is_user2, recipient_is_user1) = (
                conversation.user2_id == sender.user_id,
                conversation.user1_id == data.recipient_id,
            );

            if !(sender_is_user1 && recipient_is_user2 || sender_is_user2 && recipient_is_user1) {
                return Ok(vec![ErrorResponse::authorization_failure()]);
            }
            conversation_id
        } else {
            // Look up or create conversation between sender and recipient
            let (conversation, _) = self
                .conversation_service
                .create_or_get_conversation(sender.user_id.clone(), data.recipient_id.clone())
                .await?;
            conversation.id
        };

        let normalized_content: String = data.content.nfc().collect();
        let sanitized_content = sanitize_html(&normalized_content);

        // Send message using message service (with idempotency)
        let (message, was_created) = self
            .message_service
            .send_message_with_id(
                envelope.id.clone(),
                conversation_id.clone(),
                sender.user_id.clone(),
                data.recipient_id.clone(),
                sanitized_content.clone(),
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
                let delivery_message = Self::build_message_envelope(MessageParams {
                    message_id: &message.id,
                    sender_id: &sender.user_id,
                    sender_username: &sender.username,
                    recipient_id: &data.recipient_id,
                    content: &sanitized_content,
                    conversation_id: &conversation_id,
                    status: "delivered",
                });

                let delivery_count = self
                    .connection_manager
                    .send_to_user(
                        &data.recipient_id,
                        WsMessage::text(
                            serde_json::to_string(&delivery_message)
                                .map_err(|e| format!("Failed to serialize message: {}", e))?,
                        ),
                    )
                    .await;

                if delivery_count > 0 {
                    // Update message status to 'delivered' only if delivery succeeded
                    self.message_service.mark_delivered(&message.id).await?;
                } else {
                    tracing::warn!(
                        "Failed to deliver message {} to recipient {}",
                        message.id,
                        data.recipient_id
                    );
                }
            } else {
                // Recipient offline - queue for retry
                if !self.message_queue
                    .queue_message(message.id.clone(), data.recipient_id.clone())
                    .await
                {
                    tracing::warn!(
                        message_id = %message.id,
                        recipient_id = %data.recipient_id,
                        "Message queue full for recipient - message will be delivered when they come online"
                    );
                }
            }
        }

        // Send acknowledgement to sender
        let ack_status = if was_created
            && self
                .connection_manager
                .is_user_online(&data.recipient_id)
                .await
        {
            crate::models::status::DELIVERED
        } else {
            crate::models::status::SENT
        };
        let ack = Self::build_ack_envelope(&envelope.id, &conversation_id, &message.id, ack_status);
        responses.push(WsMessage::text(
            serde_json::to_string(&ack).map_err(|e| format!("Failed to serialize ack: {}", e))?,
        ));

        Ok(responses)
    }

    /// Build message envelope for delivery
    ///
    /// # Arguments
    /// * `message_id` - Unique message identifier
    /// * `sender_id` - Sender's user ID
    /// * `sender_username` - Sender's username
    /// * `recipient_id` - Recipient's user ID
    /// * `content` - Message text content
    /// * `conversation_id` - Conversation identifier
    /// * `status` - Message delivery status (e.g., "delivered", "sent")
    fn build_message_envelope(params: MessageParams<'_>) -> MessageEnvelope {
        MessageEnvelope {
            id: params.message_id.to_string(),
            msg_type: "message".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            data: json!({
                "senderId": params.sender_id,
                "senderUsername": params.sender_username,
                "recipientId": params.recipient_id,
                "content": params.content,
                "conversationId": params.conversation_id,
                "status": params.status,
            }),
        }
    }

    fn build_ack_envelope(
        original_message_id: &str,
        conversation_id: &str,
        stored_message_id: &str,
        status: &str,
    ) -> MessageEnvelope {
        let now_ms = chrono::Utc::now().timestamp_millis().max(0) as u64;
        MessageEnvelope {
            id: uuid::Uuid::new_v4().to_string(),
            msg_type: "ack".to_string(),
            timestamp: now_ms,
            data: json!({
                "status": status,
                "conversationId": conversation_id,
                "messageId": stored_message_id,
                "originalMessageId": original_message_id,
                "serverTimestamp": now_ms,
            }),
        }
    }

    /// Sync delivery status updates from client
    ///
    /// Handles batch delivery status updates from a reconnected client.
    /// Implements idempotent status updates to prevent duplicate state changes.
    ///
    /// Status hierarchy: pending < sent < delivered < read
    /// Only upgrades to higher status, never downgrades.
    pub async fn handle_sync_delivery_status(
        &self,
        user_id: &str,
        updates: Vec<chat_shared::protocol::DeliveryStatusUpdate>,
    ) -> Result<Vec<WsMessage>, String> {
        if updates.is_empty() {
            return Ok(vec![]);
        }

        if updates.len() > MAX_DELIVERY_STATUS_BATCH {
            return Err(format!(
                "Too many updates in batch (max {})",
                MAX_DELIVERY_STATUS_BATCH
            ));
        }

        let mut responses = Vec::new();
        let mut synced_count = 0u32;

        for update in updates {
            let current = match queries::find_message_by_id(&self.pool, &update.message_id).await {
                Ok(Some(msg)) => msg,
                _ => continue,
            };

            // Only the recipient can update message status (e.g., mark as read/delivered)
            if current.recipient_id != user_id {
                continue;
            }

            let current_weight =
                crate::services::message_service::MessageService::status_weight(&current.status);
            let new_weight =
                crate::services::message_service::MessageService::status_weight(&update.status);

            if new_weight >= current_weight {
                self.message_service
                    .update_message_status_with_timestamp(&update.message_id, &update.status)
                    .await
                    .map_err(|e| {
                        warn!("Failed to update message status: {}", e);
                        e
                    })?;

                synced_count += 1;

                let conv_id = current.conversation_id.clone();
                let now_ms = chrono::Utc::now().timestamp_millis() as u64;
                let event = MessageEnvelope {
                    id: uuid::Uuid::new_v4().to_string(),
                    msg_type: "deliveryStatusUpdated".to_string(),
                    timestamp: now_ms,
                    data: json!({
                        "messageId": update.message_id,
                        "status": update.status,
                        "timestamp": now_ms,
                        "conversationId": conv_id,
                    }),
                };

                let event_json = serde_json::to_string(&event)
                    .map_err(|e| format!("Failed to serialize event: {}", e))?;
                self.connection_manager
                    .send_to_user(&current.sender_id, WsMessage::text(event_json.clone()))
                    .await;
                self.connection_manager
                    .send_to_user(&current.recipient_id, WsMessage::text(event_json))
                    .await;
            }
        }

        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        let completion = MessageEnvelope {
            id: uuid::Uuid::new_v4().to_string(),
            msg_type: "syncDeliveryStatusCompleted".to_string(),
            timestamp: now_ms,
            data: json!({
                "syncedCount": synced_count,
                "timestamp": now_ms,
            }),
        };
        responses.push(WsMessage::text(
            serde_json::to_string(&completion)
                .map_err(|e| format!("Failed to serialize completion: {}", e))?,
        ));

        Ok(responses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::websocket::ConnectionManager;
    use crate::models::User;
    use crate::services::MessageQueueService;
    use crate::test_utils;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_handle_message_creates_conversation() {
        let pool = test_utils::setup_test_db().await;
        let conn_mgr = Arc::new(ConnectionManager::new());
        let queue = MessageQueueService::new(pool.clone(), conn_mgr.clone());
        let handler = MessageHandler::new(pool.clone(), conn_mgr.clone(), queue);

        // Create users
        let user1 = User::new("alice".to_string(), "hash1".to_string());
        let user2 = User::new("bob".to_string(), "hash2".to_string());

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
    }

    #[tokio::test]
    async fn test_handle_message_to_online_recipient() {
        let pool = test_utils::setup_test_db().await;
        let conn_mgr = Arc::new(ConnectionManager::new());
        let queue = MessageQueueService::new(pool.clone(), conn_mgr.clone());
        let handler = MessageHandler::new(pool.clone(), conn_mgr.clone(), queue);

        // Create users
        let user1 = User::new("alice".to_string(), "hash1".to_string());
        let user2 = User::new("bob".to_string(), "hash2".to_string());

        queries::insert_user(&pool, &user1).await.unwrap();
        queries::insert_user(&pool, &user2).await.unwrap();

        // Register recipient as online
        let recipient_conn = ClientConnection::new(user2.id.clone(), user2.username.clone());
        let (tx, _rx) = mpsc::channel(100);
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
        let pool = test_utils::setup_test_db().await;
        let conn_mgr = Arc::new(ConnectionManager::new());
        let queue = MessageQueueService::new(pool.clone(), conn_mgr.clone());
        let handler = MessageHandler::new(pool.clone(), conn_mgr.clone(), queue);

        // Create users
        let user1 = User::new("alice".to_string(), "hash1".to_string());
        let user2 = User::new("bob".to_string(), "hash2".to_string());

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
