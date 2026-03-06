//! WebSocket protocol message types and schemas

use serde::{Deserialize, Serialize};

/// Maximum nesting depth for JSON structures to prevent stack overflow attacks
pub const MAX_JSON_DEPTH: usize = 32;

/// Maximum number of items in batch operations to prevent memory exhaustion
pub const MAX_BATCH_SIZE: usize = 100;

/// Maximum size for the data payload in bytes (64KB)
pub const MAX_DATA_SIZE: usize = 64 * 1024;

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
    /// Message read by recipient
    Read,
    /// Message failed to deliver (recipient deleted, etc.)
    Failed,
}

impl MessageStatus {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            MessageStatus::Pending => "pending",
            MessageStatus::Sent => "sent",
            MessageStatus::Delivered => "delivered",
            MessageStatus::Read => "read",
            MessageStatus::Failed => "failed",
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
            "read" => Ok(MessageStatus::Read),
            "failed" => Ok(MessageStatus::Failed),
            _ => Err(format!("Unknown status: {s}")),
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
    /// Issuer
    #[serde(default = "default_issuer")]
    pub iss: String,
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

fn default_issuer() -> String {
    "chat-app".to_string()
}

impl TokenClaims {
    /// Check if the token has expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp() as u64;
        self.exp < now
    }

    /// Check if the token is valid (not expired and issued before now)
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let now = chrono::Utc::now().timestamp() as u64;
        self.iat <= now && self.exp > now
    }

    /// Check if the token has a specific scope
    #[must_use]
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }

    /// Check if the token has all of the specified scopes
    #[must_use]
    pub fn has_scopes(&self, required_scopes: &[&str]) -> bool {
        required_scopes.iter().all(|scope| self.has_scope(scope))
    }

    /// Validate issuer matches expected value
    #[must_use]
    pub fn has_issuer(&self, expected: &str) -> bool {
        self.iss == expected
    }

    /// Validate audience matches expected value
    #[must_use]
    pub fn has_audience(&self, expected: &str) -> bool {
        self.aud == expected
    }
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

/// Delivery status update from client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryStatusUpdate {
    pub message_id: String,
    pub status: String,
}

/// Sync delivery status command from client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncDeliveryStatusCommand {
    pub delivery_updates: Vec<DeliveryStatusUpdate>,
}

/// Delivery status updated event from backend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryStatusUpdatedEvent {
    pub message_id: String,
    pub status: String,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
}

/// Batch delivery status update event from backend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryStatusBatchUpdatedEvent {
    pub updates: Vec<DeliveryStatusUpdatedEvent>,
}

/// Sync delivery status completed event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncDeliveryStatusCompletedEvent {
    pub synced_count: u32,
    pub timestamp: i64,
}

/// Sync delivery status failed event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryStatusSyncFailedEvent {
    pub reason: String,
    pub retriable: bool,
}
