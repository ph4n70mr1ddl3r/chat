//! Domain models for the chat application

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub password_salt: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub deleted_at: Option<u64>,
    pub is_online: bool,
    pub last_seen_at: Option<u64>,
}

impl User {
    pub fn new(username: String, password_hash: String, password_salt: String) -> Self {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        Self {
            id: Uuid::new_v4().to_string(),
            username,
            password_hash,
            password_salt,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            is_online: false,
            last_seen_at: None,
        }
    }
}

/// One-to-one conversation between two users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub user1_id: String,
    pub user2_id: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub last_message_at: Option<u64>,
    pub message_count: u32,
}

impl Conversation {
    pub fn new(user1_id: String, user2_id: String) -> Self {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        Self {
            id: Uuid::new_v4().to_string(),
            user1_id,
            user2_id,
            created_at: now,
            updated_at: now,
            last_message_at: None,
            message_count: 0,
        }
    }
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub content: String,
    pub created_at: u64,
    pub delivered_at: Option<u64>,
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
        let now = chrono::Utc::now().timestamp_millis() as u64;
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id,
            sender_id,
            recipient_id,
            content,
            created_at: now,
            delivered_at: None,
            status: "pending".to_string(),
            is_anonymized: false,
        }
    }
}
