//! Connection lifecycle handlers
//!
//! Handles connection status changes and coordinates reconnection behaviors.
//! Responsible for:
//! - Tracking connection state changes
//! - Triggering delivery status sync on reconnection
//! - Managing connection recovery retries

use serde::{Deserialize, Serialize};

/// Connection status event from backend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatusEvent {
    pub is_connected: bool,
    pub timestamp: i64,
}

/// Client connection restored event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionRestoredEvent {
    pub timestamp: i64,
    pub offline_duration_ms: u64,
}

/// Client disconnected event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientDisconnectedEvent {
    pub timestamp: i64,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_restored_event_serializes() {
        let event = ConnectionRestoredEvent {
            timestamp: 1234567890,
            offline_duration_ms: 5000,
        };
        
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"offlineDurationMs\""));
        assert!(!json.contains("\"offline_duration_ms\""));
    }
}
