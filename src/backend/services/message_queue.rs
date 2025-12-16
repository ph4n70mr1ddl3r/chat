//! Message queue service for offline delivery with exponential backoff
//!
//! Handles retry logic for messages sent to offline recipients.
//! Implements exponential backoff: 0.5s, 1.5s, 3s, 7s, 15s, 30s, 60s (max)
//! Retries indefinitely until recipient comes online or is deleted.

use crate::db::queries;
use crate::handlers::websocket::ConnectionManager;
use crate::services::message_service::{MessageService, MessageStatus};
use chat_shared::protocol::MessageEnvelope;
use serde_json::json;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use warp::ws::Message as WsMessage;

/// Retry schedule in seconds
const RETRY_SCHEDULE: &[u64] = &[0, 1, 3, 7, 15, 30, 60];

/// Message delivery queue entry
#[derive(Debug, Clone)]
struct QueuedMessage {
    message_id: String,
    recipient_id: String,
    retry_count: usize,
    next_retry_at: u64, // Unix timestamp in seconds
}

/// Message queue service
#[derive(Clone)]
pub struct MessageQueueService {
    pool: SqlitePool,
    message_service: MessageService,
    connection_manager: Arc<ConnectionManager>,
    /// Queue of pending messages: recipient_id -> Vec<QueuedMessage>
    queue: Arc<RwLock<HashMap<String, Vec<QueuedMessage>>>>,
    /// Whether the background worker is running
    is_running: Arc<RwLock<bool>>,
}

impl MessageQueueService {
    /// Create a new message queue service
    pub fn new(pool: SqlitePool, connection_manager: Arc<ConnectionManager>) -> Self {
        let message_service = MessageService::new(pool.clone());
        Self {
            pool,
            message_service,
            connection_manager,
            queue: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the background worker for message delivery
    pub async fn start(&self) {
        let mut running = self.is_running.write().await;
        if *running {
            return; // Already running
        }
        *running = true;
        drop(running);

        let pool = self.pool.clone();
        let queue = self.queue.clone();
        let connection_manager = self.connection_manager.clone();
        let message_service = MessageService::new(pool.clone());
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            loop {
                // Check if we should stop
                if !*is_running.read().await {
                    break;
                }

                // Process queue every 500ms
                sleep(Duration::from_millis(500)).await;

                let now = chrono::Utc::now().timestamp() as u64;
                let mut queue_lock = queue.write().await;

                // Collect messages ready for retry
                let mut messages_to_retry = Vec::new();
                for (recipient_id, messages) in queue_lock.iter_mut() {
                    messages.retain(|msg| {
                        if msg.next_retry_at <= now {
                            messages_to_retry.push(msg.clone());
                            false // Remove from queue
                        } else {
                            true // Keep in queue
                        }
                    });
                }

                drop(queue_lock);

                // Attempt delivery for each message
                for queued_msg in messages_to_retry {
                    // Check if recipient is now online
                    if connection_manager
                        .is_user_online(&queued_msg.recipient_id)
                        .await
                    {
                        // Attempt delivery
                        match Self::deliver_message(
                            &pool,
                            &message_service,
                            connection_manager.as_ref(),
                            &queued_msg.message_id,
                        )
                        .await
                        {
                            Ok(_) => {
                                // Success - mark as sent
                                let _ = message_service
                                    .update_message_status(
                                        &queued_msg.message_id,
                                        MessageStatus::Sent,
                                    )
                                    .await;
                            }
                            Err(_) => {
                                // Failed - requeue with exponential backoff
                                Self::requeue_message(queue.clone(), queued_msg).await;
                            }
                        }
                    } else {
                        // Recipient still offline - requeue with exponential backoff
                        Self::requeue_message(queue.clone(), queued_msg).await;
                    }
                }
            }
        });
    }

    /// Stop the background worker
    pub async fn stop(&self) {
        let mut running = self.is_running.write().await;
        *running = false;
    }

    /// Queue a message for delivery
    pub async fn queue_message(&self, message_id: String, recipient_id: String) {
        let queued_msg = QueuedMessage {
            message_id,
            recipient_id: recipient_id.clone(),
            retry_count: 0,
            next_retry_at: chrono::Utc::now().timestamp() as u64,
        };

        let mut queue = self.queue.write().await;
        queue
            .entry(recipient_id)
            .or_insert_with(Vec::new)
            .push(queued_msg);
    }

    /// Deliver a message to online recipient
    async fn deliver_message(
        pool: &SqlitePool,
        message_service: &MessageService,
        connection_manager: &ConnectionManager,
        message_id: &str,
    ) -> Result<(), String> {
        // Load message from database
        let message = queries::find_message_by_id(pool, message_id)
            .await?
            .ok_or_else(|| "Message not found".to_string())?;

        // Verify recipient exists and is not deleted
        let recipient = queries::find_user_by_id(pool, &message.recipient_id)
            .await?
            .ok_or_else(|| "Recipient not found".to_string())?;

        if recipient.is_deleted() {
            // Mark message as failed
            message_service
                .update_message_status(message_id, MessageStatus::Failed)
                .await?;
            return Err("Recipient deleted".to_string());
        }

        // Build delivery payload
        let sender = queries::find_user_by_id(pool, &message.sender_id)
            .await?
            .ok_or_else(|| "Sender not found".to_string())?;

        let envelope = MessageEnvelope {
            id: message.id.clone(),
            msg_type: "message".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            data: json!({
                "senderId": sender.id,
                "senderUsername": sender.username,
                "recipientId": message.recipient_id,
                "content": message.content,
                "conversationId": message.conversation_id,
                "status": "delivered",
            }),
        };

        let outbound = WsMessage::text(
            serde_json::to_string(&envelope)
                .map_err(|e| format!("Failed to serialize message: {}", e))?,
        );

        // Attempt to send to recipient
        let delivered = connection_manager
            .send_to_user(&recipient.id, outbound.clone())
            .await;
        if delivered == 0 {
            return Err("Recipient offline".to_string());
        }

        // Mark delivered and send ack to sender if connected
        message_service.mark_delivered(&message.id).await?;

        let ack = MessageEnvelope {
            id: uuid::Uuid::new_v4().to_string(),
            msg_type: "ack".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            data: json!({
                "status": "delivered",
                "messageId": message.id,
                "conversationId": message.conversation_id,
                "serverTimestamp": chrono::Utc::now().timestamp_millis(),
            }),
        };
        let ack_msg = WsMessage::text(
            serde_json::to_string(&ack).map_err(|e| format!("Failed to serialize ack: {}", e))?,
        );
        let _ = connection_manager.send_to_user(&sender.id, ack_msg).await;

        Ok(())
    }

    /// Requeue a message with exponential backoff
    async fn requeue_message(
        queue: Arc<RwLock<HashMap<String, Vec<QueuedMessage>>>>,
        mut queued_msg: QueuedMessage,
    ) {
        // Calculate next retry time with exponential backoff
        let retry_index = queued_msg.retry_count.min(RETRY_SCHEDULE.len() - 1);
        let delay_seconds = RETRY_SCHEDULE[retry_index];

        queued_msg.retry_count += 1;
        queued_msg.next_retry_at = chrono::Utc::now().timestamp() as u64 + delay_seconds;

        let mut queue_lock = queue.write().await;
        queue_lock
            .entry(queued_msg.recipient_id.clone())
            .or_insert_with(Vec::new)
            .push(queued_msg);
    }

    /// Load pending messages from database on startup
    pub async fn load_pending_messages(&self) -> Result<(), String> {
        // Get all pending messages
        let pending_messages = queries::get_all_pending_messages(&self.pool).await?;

        let mut queue = self.queue.write().await;
        for message in pending_messages {
            let queued_msg = QueuedMessage {
                message_id: message.id,
                recipient_id: message.recipient_id.clone(),
                retry_count: 0,
                next_retry_at: chrono::Utc::now().timestamp() as u64,
            };

            queue
                .entry(message.recipient_id)
                .or_insert_with(Vec::new)
                .push(queued_msg);
        }

        Ok(())
    }

    /// Get queue statistics (for monitoring/debugging)
    pub async fn get_queue_stats(&self) -> HashMap<String, usize> {
        let queue = self.queue.read().await;
        queue
            .iter()
            .map(|(recipient_id, messages)| (recipient_id.clone(), messages.len()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::websocket::ClientConnection;
    use crate::models::{Conversation, User};

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
    async fn test_queue_message() {
        let pool = setup_test_db().await;
        let conn_mgr = Arc::new(ConnectionManager::new());
        let queue_service = MessageQueueService::new(pool, conn_mgr);

        queue_service
            .queue_message("msg-123".to_string(), "user-456".to_string())
            .await;

        let stats = queue_service.get_queue_stats().await;
        assert_eq!(stats.get("user-456"), Some(&1));
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        // Verify retry schedule increases exponentially
        assert_eq!(RETRY_SCHEDULE[0], 0);
        assert_eq!(RETRY_SCHEDULE[1], 1);
        assert_eq!(RETRY_SCHEDULE[2], 3);
        assert_eq!(RETRY_SCHEDULE[6], 60); // Max cap
    }

    #[tokio::test]
    async fn test_load_pending_messages() {
        let pool = setup_test_db().await;
        let conn_mgr = Arc::new(ConnectionManager::new());
        let queue_service = MessageQueueService::new(pool.clone(), conn_mgr);

        // Create users and conversation
        let user1 = User::new(
            "alice".to_string(),
            "hash1".to_string(),
            "salt1".to_string(),
        );
        let user2 = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());

        queries::insert_user(&pool, &user1).await.unwrap();
        queries::insert_user(&pool, &user2).await.unwrap();

        let conv = Conversation::new(user1.id.clone(), user2.id.clone());
        queries::insert_conversation(&pool, &conv).await.unwrap();

        // Create pending message
        let message = crate::models::Message::new(
            conv.id.clone(),
            user1.id.clone(),
            user2.id.clone(),
            "Hello".to_string(),
        );
        queries::insert_message(&pool, &message).await.unwrap();

        // Load pending messages
        queue_service.load_pending_messages().await.unwrap();

        let stats = queue_service.get_queue_stats().await;
        assert!(stats.get(&user2.id).is_some());
    }
}
