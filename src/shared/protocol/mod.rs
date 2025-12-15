//! WebSocket protocol message types and schemas

use serde::{Deserialize, Serialize};

/// Message status lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    /// Message accepted by server, queued for delivery
    Pending,
    /// Message successfully transmitted to recipient (online) or queued (offline)
    Sent,
    /// Message received and acknowledged by recipient
    Delivered,
    /// Message failed to deliver (recipient deleted, etc.)
    Failed,
}

impl std::fmt::Display for MessageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageStatus::Pending => write!(f, "pending"),
            MessageStatus::Sent => write!(f, "sent"),
            MessageStatus::Delivered => write!(f, "delivered"),
            MessageStatus::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for MessageStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(MessageStatus::Pending),
            "sent" => Ok(MessageStatus::Sent),
            "delivered" => Ok(MessageStatus::Delivered),
            "failed" => Ok(MessageStatus::Failed),
            _ => Err(format!("Unknown status: {}", s)),
        }
    }
}

/// WebSocket message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// Unique message ID (UUID v4)
    pub id: String,
    /// Message type
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Timestamp in milliseconds
    pub timestamp: u64,
    /// Type-specific payload
    pub data: serde_json::Value,
}

/// Text message data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMessageData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_username: Option<String>,
    pub recipient_id: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Message acknowledgement data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckData {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_timestamp: Option<u64>,
}

/// Typing indicator data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_username: Option<String>,
    pub recipient_id: String,
    pub is_typing: bool,
}

/// Presence status data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceData {
    pub user_id: String,
    pub username: String,
    pub is_online: bool,
    pub last_seen_at: u64,
}

/// Error message data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorData {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// JWT token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Audience
    pub aud: String,
    /// Issued at
    pub iat: u64,
    /// Expires at
    pub exp: u64,
    /// Scopes
    #[serde(default)]
    pub scopes: Vec<String>,
}

/// User DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub user_id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_online: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_at: Option<u64>,
}

/// Conversation DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationDto {
    pub conversation_id: String,
    pub participant_id: String,
    pub participant_username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_is_online: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_count: Option<u32>,
}

/// Message DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDto {
    pub id: String,
    pub sender_id: String,
    pub sender_username: String,
    pub recipient_id: String,
    pub content: String,
    pub created_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivered_at: Option<u64>,
    pub status: String,
}
