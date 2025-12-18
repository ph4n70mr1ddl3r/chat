//! Delivery status event handlers
//!
//! Handles incoming delivery status updates and manages the client-side delivery queue.
//! Responsible for:
//! - Processing delivery status updates from backend
//! - Managing pending delivery queue persistence
//! - Triggering delivery sync on reconnection
//! - Handling retry logic with exponential backoff

use serde::{Deserialize, Serialize};

/// Pending delivery update queued for sync
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingDeliveryUpdate {
    pub message_id: String,
    pub status: String, // "sent", "delivered", "read"
    pub queued_at: i64, // timestamp for ordering
    #[serde(default)]
    pub retry_count: u32,
}

/// Delivery status update event from backend
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

/// Sync delivery status command to send to backend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncDeliveryStatusCommand {
    pub delivery_updates: Vec<DeliveryStatusUpdate>,
}

/// Individual delivery status update in sync command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryStatusUpdate {
    pub message_id: String,
    pub status: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_delivery_update_serializes() {
        let update = PendingDeliveryUpdate {
            message_id: "msg-123".to_string(),
            status: "sent".to_string(),
            queued_at: 1234567890,
            retry_count: 0,
        };
        
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"messageId\""));
        assert!(json.contains("\"msg-123\""));
        assert!(json.contains("\"queuedAt\""));
    }

    #[test]
    fn sync_command_serializes_with_camel_case() {
        let cmd = SyncDeliveryStatusCommand {
            delivery_updates: vec![
                DeliveryStatusUpdate {
                    message_id: "msg-1".to_string(),
                    status: "sent".to_string(),
                },
            ],
        };
        
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"deliveryUpdates\""));
        assert!(!json.contains("\"delivery_updates\""));
    }
}
