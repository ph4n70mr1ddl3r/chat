//! Message queue service for offline delivery with exponential backoff
//!
//! Handles retry logic for messages sent to offline recipients.
//! Implements exponential backoff: 0s, 1s, 3s, 7s, 15s, 30s, 60s (max)
//! Retries indefinitely until recipient comes online or is deleted.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use warp::ws::Message as WsMessage;

use crate::db::queries;
use crate::handlers::websocket::ConnectionManager;
use crate::services::message_service::MessageService;
use chat_shared::protocol::{MessageEnvelope, MessageStatus};
use serde_json::json;
use sqlx::SqlitePool;

/// Retry schedule in seconds
const RETRY_SCHEDULE: &[u64] = &[0, 1, 3, 7, 15, 30, 60];

/// Maximum queued messages per recipient to prevent memory exhaustion
const MAX_QUEUED_MESSAGES_PER_USER: usize = 100;

/// Maximum total queued messages across all recipients
const MAX_TOTAL_QUEUED_MESSAGES: usize = 50_000;

/// Message delivery queue entry
#[derive(Debug, Clone)]
struct QueuedMessage {
    message_id: String,
    recipient_id: String,
    retry_count: usize,
    next_retry_at: u64,
}

/// Message queue service
#[derive(Clone)]
pub struct MessageQueueService {
    pool: SqlitePool,
    connection_manager: Arc<ConnectionManager>,
    queue: Arc<RwLock<HashMap<String, Vec<QueuedMessage>>>>,
    total_queued: Arc<AtomicUsize>,
    is_running: Arc<AtomicBool>,
}

impl MessageQueueService {
    /// Create a new message queue service
    pub fn new(pool: SqlitePool, connection_manager: Arc<ConnectionManager>) -> Self {
        Self {
            pool,
            connection_manager,
            queue: Arc::new(RwLock::new(HashMap::new())),
            total_queued: Arc::new(AtomicUsize::new(0)),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the background worker for message delivery
    pub async fn start(&self) {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return;
        }

        let pool = self.pool.clone();
        let queue = self.queue.clone();
        let connection_manager = self.connection_manager.clone();
        let message_service = MessageService::new(pool.clone());
        let is_running = self.is_running.clone();
        let total_queued = self.total_queued.clone();

        tokio::spawn(async move {
            loop {
                if !is_running.load(Ordering::SeqCst) {
                    tracing::info!("Message queue service shutting down gracefully");
                    break;
                }

                tokio::select! {
                    () = sleep(Duration::from_millis(500)) => {
                        let now = chrono::Utc::now().timestamp().max(0).cast_unsigned();
                        let mut queue_lock = queue.write().await;

                        let mut ready_by_recipient: HashMap<String, Vec<QueuedMessage>> = HashMap::new();
                        for (_recipient_id, messages) in queue_lock.iter_mut() {
                            messages.retain(|msg| {
                                if msg.next_retry_at <= now {
                                    ready_by_recipient
                                        .entry(msg.recipient_id.clone())
                                        .or_default()
                                        .push(msg.clone());
                                    false
                                } else {
                                    true
                                }
                            });
                        }

                        drop(queue_lock);

                        for (recipient_id, queued_messages) in ready_by_recipient {
                            if connection_manager.is_user_online(&recipient_id).await {
                                Self::deliver_batch(
                                    &pool,
                                    &message_service,
                                    connection_manager.as_ref(),
                                    queue.clone(),
                                    queued_messages,
                                    total_queued.clone(),
                                )
                                .await;
                            } else {
                                for msg in queued_messages {
                                    if !Self::requeue_message(queue.clone(), msg, total_queued.clone()).await {
                                        tracing::warn!(
                                            "Dropped message for recipient {} due to queue overflow",
                                            recipient_id
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    /// Queue a message for delivery
    ///
    /// Returns `true` if message was queued successfully.
    /// Returns `false` if the per-user queue or the global queue is full (message dropped).
    pub async fn queue_message(&self, message_id: String, recipient_id: String) -> bool {
        let current_total = self.total_queued.load(Ordering::SeqCst);
        if current_total >= MAX_TOTAL_QUEUED_MESSAGES {
            tracing::warn!(
                "Global message queue full ({} messages), cannot queue new message for user {}",
                current_total,
                recipient_id
            );
            return false;
        }

        let queued_msg = QueuedMessage {
            message_id,
            recipient_id: recipient_id.clone(),
            retry_count: 0,
            next_retry_at: chrono::Utc::now().timestamp().max(0).cast_unsigned(),
        };

        let mut queue = self.queue.write().await;
        let user_queue = queue.entry(recipient_id.clone()).or_insert_with(Vec::new);

        if user_queue.len() >= MAX_QUEUED_MESSAGES_PER_USER {
            tracing::warn!(
                "Message queue full for user {}, cannot queue new message",
                recipient_id
            );
            return false;
        }
        user_queue.push(queued_msg);
        self.total_queued.fetch_add(1, Ordering::SeqCst);
        true
    }

    /// Deliver a message to online recipient
    async fn deliver_message(
        pool: &SqlitePool,
        message_service: &MessageService,
        connection_manager: &ConnectionManager,
        message_id: &str,
    ) -> Result<(), String> {
        let message = queries::find_message_by_id(pool, message_id)
            .await?
            .ok_or_else(|| "Message not found".to_string())?;

        let recipient = queries::find_user_by_id(pool, &message.recipient_id)
            .await?
            .ok_or_else(|| "Recipient not found".to_string())?;

        if recipient.is_deleted() {
            message_service
                .update_message_status(message_id, MessageStatus::Failed)
                .await?;
            return Err("Recipient deleted".to_string());
        }

        let sender = queries::find_user_by_id(pool, &message.sender_id)
            .await?
            .ok_or_else(|| "Sender not found".to_string())?;

        let envelope = MessageEnvelope {
            id: message.id.clone(),
            msg_type: "message".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis().max(0).cast_unsigned(),
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
                .map_err(|e| format!("Failed to serialize message: {e}"))?,
        );

        let delivered = connection_manager
            .send_to_user(&recipient.id, outbound.clone())
            .await;
        if delivered == 0 {
            return Err("Recipient offline".to_string());
        }

        message_service.mark_delivered(&message.id).await?;

        let ack = MessageEnvelope {
            id: uuid::Uuid::new_v4().to_string(),
            msg_type: "ack".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis().max(0).cast_unsigned(),
            data: json!({
                "status": "delivered",
                "messageId": message.id,
                "conversationId": message.conversation_id,
                "serverTimestamp": chrono::Utc::now().timestamp_millis(),
            }),
        };
        let ack_msg = WsMessage::text(
            serde_json::to_string(&ack).map_err(|e| format!("Failed to serialize ack: {e}"))?,
        );
        let _ = connection_manager.send_to_user(&sender.id, ack_msg).await;

        Ok(())
    }

    /// Deliver a batch of messages to a single recipient
    async fn deliver_batch(
        pool: &SqlitePool,
        message_service: &MessageService,
        connection_manager: &ConnectionManager,
        queue: Arc<RwLock<HashMap<String, Vec<QueuedMessage>>>>,
        messages: Vec<QueuedMessage>,
        total_queued: Arc<AtomicUsize>,
    ) {
        for queued_msg in messages {
            match Self::deliver_message(
                pool,
                message_service,
                connection_manager,
                &queued_msg.message_id,
            )
            .await
            {
                Ok(()) => {
                    total_queued.fetch_sub(1, Ordering::SeqCst);
                }
                Err(reason) => {
                    if reason != "Recipient deleted" && reason != "Recipient not found" {
                        Self::requeue_message(queue.clone(), queued_msg, total_queued.clone()).await;
                    } else {
                        total_queued.fetch_sub(1, Ordering::SeqCst);
                    }
                }
            }
        }
    }

    /// Requeue a message with exponential backoff
    async fn requeue_message(
        queue: Arc<RwLock<HashMap<String, Vec<QueuedMessage>>>>,
        mut queued_msg: QueuedMessage,
        total_queued: Arc<AtomicUsize>,
    ) -> bool {
        let retry_index = queued_msg.retry_count.min(RETRY_SCHEDULE.len() - 1);
        let delay_seconds = RETRY_SCHEDULE[retry_index];

        queued_msg.retry_count += 1;
        queued_msg.next_retry_at = chrono::Utc::now().timestamp().max(0).cast_unsigned() + delay_seconds;

        let mut queue_lock = queue.write().await;
        let user_queue = queue_lock
            .entry(queued_msg.recipient_id.clone())
            .or_insert_with(Vec::new);

        if user_queue.len() >= MAX_QUEUED_MESSAGES_PER_USER {
            total_queued.fetch_sub(1, Ordering::SeqCst);
            return false;
        }
        user_queue.push(queued_msg);
        true
    }

    /// Load pending messages from database on startup
    ///
    /// # Errors
    /// Returns an error string if database access fails.
    pub async fn load_pending_messages(&self) -> Result<(), String> {
        let pending_messages = queries::get_all_pending_messages(&self.pool).await?;

        let mut queue = self.queue.write().await;
        for message in pending_messages {
            let user_queue = queue
                .entry(message.recipient_id.clone())
                .or_insert_with(Vec::new);

            if user_queue.len() >= MAX_QUEUED_MESSAGES_PER_USER {
                continue;
            }

            let queued_msg = QueuedMessage {
                message_id: message.id,
                recipient_id: message.recipient_id.clone(),
                retry_count: 0,
                next_retry_at: chrono::Utc::now().timestamp().max(0).cast_unsigned(),
            };

            user_queue.push(queued_msg);
            self.total_queued.fetch_add(1, Ordering::SeqCst);
        }

        Ok(())
    }

    /// Get queue statistics (for monitoring/debugging)
    pub async fn get_queue_stats(&self) -> HashMap<String, usize> {
        let queue = self.queue.read().await;
        let mut stats: HashMap<String, usize> = HashMap::new();
        for (recipient_id, messages) in queue.iter() {
            stats.insert(recipient_id.clone(), messages.len());
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Conversation, User};

    async fn setup_test_db() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let schema_sql = include_str!("../db/migrations/001_initial_schema.sql");
        for statement in schema_sql.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(statement).execute(&pool).await.unwrap();
        }

        let migration_sql = include_str!("../db/migrations/002_remove_password_salt.sql");
        for statement in migration_sql.split(';').filter(|s| !s.trim().is_empty()) {
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
        assert_eq!(RETRY_SCHEDULE[0], 0);
        assert_eq!(RETRY_SCHEDULE[1], 1);
        assert_eq!(RETRY_SCHEDULE[2], 3);
        assert_eq!(RETRY_SCHEDULE[6], 60);
    }

    #[tokio::test]
    async fn test_load_pending_messages() {
        let pool = setup_test_db().await;
        let conn_mgr = Arc::new(ConnectionManager::new());
        let queue_service = MessageQueueService::new(pool.clone(), conn_mgr);

        let user1 = User::new("alice".to_string(), "hash1".to_string());
        let user2 = User::new("bob".to_string(), "hash2".to_string());

        queries::insert_user(&pool, &user1).await.unwrap();
        queries::insert_user(&pool, &user2).await.unwrap();

        let (user1_id, user2_id) = if user1.id < user2.id {
            (user1.id.clone(), user2.id.clone())
        } else {
            (user2.id.clone(), user1.id.clone())
        };
        let conv = Conversation::new(user1_id, user2_id);
        queries::insert_conversation(&pool, &conv).await.unwrap();

        let message = crate::models::Message::new(
            conv.id.clone(),
            user1.id.clone(),
            user2.id.clone(),
            "Hello".to_string(),
        );
        queries::insert_message(&pool, &message).await.unwrap();

        queue_service.load_pending_messages().await.unwrap();

        let stats = queue_service.get_queue_stats().await;
        assert!(stats.contains_key(&user2.id));
    }
}
