//! Domain models for the chat application

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Maximum length for message content in characters.
/// Note: The WebSocket frame size limit (configured in ServerConfig::max_message_size)
/// should be larger than this to accommodate JSON serialization overhead and metadata.
/// A 5000-character limit with ~10KB frame size allows for JSON structure overhead.
pub const MAX_MESSAGE_LENGTH: usize = 5000;

/// Message status constants
pub mod status {
    pub const PENDING: &str = "pending";
    pub const SENT: &str = "sent";
    pub const DELIVERED: &str = "delivered";
    pub const READ: &str = "read";
    pub const FAILED: &str = "failed";
}

/// Message type constants
pub mod msg_type {
    pub const MESSAGE: &str = "message";
    pub const TYPING: &str = "typing";
    pub const PRESENCE: &str = "presence";
    pub const ACK: &str = "ack";
    pub const ERROR: &str = "error";
    pub const HEARTBEAT: &str = "heartbeat";
}

/// User account
///
/// Represents a registered user in the chat system.
/// bcrypt includes salt internally in the password_hash.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    /// bcrypt hash which includes salt internally
    pub password_hash: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
    pub is_online: bool,
    pub last_seen_at: Option<i64>,
}

impl User {
    pub fn new(username: String, password_hash: String) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: Uuid::new_v4().to_string(),
            username,
            password_hash,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            is_online: false,
            last_seen_at: None,
        }
    }

    /// Check if user is deleted (soft delete with timestamp in the past)
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Check if user is active
    #[must_use]
    pub fn is_active(&self) -> bool {
        !self.is_deleted()
    }
}

/// One-to-one conversation between two users
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Conversation {
    pub id: String,
    pub user1_id: String,
    pub user2_id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_message_at: Option<i64>,
    pub last_message: Option<String>,
    pub message_count: i32,
}

impl Conversation {
    pub fn new(user1_id: String, user2_id: String) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: Uuid::new_v4().to_string(),
            user1_id,
            user2_id,
            created_at: now,
            updated_at: now,
            last_message_at: None,
            last_message: None,
            message_count: 0,
        }
    }

    /// Validate that conversation is between different users and ordered correctly
    pub fn validate(&self) -> Result<(), String> {
        if self.user1_id == self.user2_id {
            return Err("Cannot create conversation with self".to_string());
        }
        if self.user1_id >= self.user2_id {
            return Err("user1_id must be less than user2_id".to_string());
        }
        Ok(())
    }
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub content: String,
    pub created_at: i64,
    pub delivered_at: Option<i64>,
    pub read_at: Option<i64>,
    pub status: String,
    pub is_anonymized: bool,
}

impl Message {
    pub fn new(
        conversation_id: String,
        sender_id: String,
        recipient_id: String,
        content: String,
    ) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id,
            sender_id,
            recipient_id,
            content,
            created_at: now,
            delivered_at: None,
            read_at: None,
            status: status::PENDING.to_string(),
            is_anonymized: false,
        }
    }

    /// Validate message content
    pub fn validate(&self) -> Result<(), String> {
        let len = self.content.len();
        if len > MAX_MESSAGE_LENGTH {
            return Err(format!(
                "Message content exceeds {} character limit",
                MAX_MESSAGE_LENGTH
            ));
        }
        if self.sender_id == self.recipient_id {
            return Err("Cannot send message to yourself".to_string());
        }
        Ok(())
    }

    /// Check if message is pending delivery
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.status == status::PENDING
    }

    /// Check if message is delivered
    #[must_use]
    pub fn is_delivered(&self) -> bool {
        self.status == status::DELIVERED
    }

    /// Check if message is read
    #[must_use]
    pub fn is_read(&self) -> bool {
        self.status == status::READ
    }

    /// Check if message failed
    #[must_use]
    pub fn is_failed(&self) -> bool {
        self.status == status::FAILED
    }
}
