//! WebSocket connection handler and message dispatcher
//!
//! Manages WebSocket connections, message routing, and real-time delivery.
//! Handles authentication, message validation, and client-server communication.

use crate::models::MAX_MESSAGE_LENGTH;
use chat_shared::protocol::MessageEnvelope;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use warp::ws::Message as WsMessage;

/// WebSocket connection handle
pub type ConnectionId = String;

/// Client connection state
#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub user_id: String,
    pub username: String,
    pub connection_id: ConnectionId,
    pub connected_at: u64,
}

impl ClientConnection {
    pub fn new(user_id: String, username: String) -> Self {
        let connection_id = uuid::Uuid::new_v4().to_string();
        let connected_at = chrono::Utc::now().timestamp_millis() as u64;

        Self {
            user_id,
            username,
            connection_id,
            connected_at,
        }
    }
}

/// WebSocket connection manager
pub struct ConnectionManager {
    /// Map of user_id -> active connections
    connections: Arc<RwLock<HashMap<String, Vec<ManagedConnection>>>>,
}

#[derive(Clone)]
pub struct ManagedConnection {
    pub client: ClientConnection,
    pub sender: Sender<WsMessage>,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new connection for a user
    ///
    /// Adds a new WebSocket connection to the user's connection list.
    /// A single user can have multiple concurrent connections.
    ///
    /// # Returns
    /// The unique connection ID that can be used to unregister this connection.
    pub async fn register(
        &self,
        client: ClientConnection,
        sender: Sender<WsMessage>,
    ) -> ConnectionId {
        let mut conns = self.connections.write().await;
        let connection_id = client.connection_id.clone();

        conns
            .entry(client.user_id.clone())
            .or_insert_with(Vec::new)
            .push(ManagedConnection { client, sender });

        connection_id
    }

    /// Unregister a connection
    ///
    /// Removes a specific connection for a user. If this was the last
    /// connection for the user, the user is removed from the connection map.
    pub async fn unregister(&self, user_id: &str, connection_id: &str) {
        let mut conns = self.connections.write().await;

        if let Some(user_conns) = conns.get_mut(user_id) {
            user_conns.retain(|c| c.client.connection_id != connection_id);

            // Remove user entry if no connections remain
            if user_conns.is_empty() {
                conns.remove(user_id);
            }
        }
    }

    /// Disconnect all sessions for a user
    ///
    /// Removes all active WebSocket connections for the given user ID.
    /// This is called during logout or when a user account is deleted.
    pub async fn disconnect_user(&self, user_id: &str) {
        let mut conns = self.connections.write().await;
        conns.remove(user_id);
    }

    /// Get all connections for a user
    ///
    /// Returns a list of all active WebSocket connections for the specified user.
    /// This is useful for logging or when a user needs to be disconnected from all sessions.
    pub async fn get_user_connections(&self, user_id: &str) -> Vec<ClientConnection> {
        let conns = self.connections.read().await;
        conns
            .get(user_id)
            .map(|conns| conns.iter().map(|mc| mc.client.clone()).collect())
            .unwrap_or_default()
    }

    /// Check if user is online (has any active connections)
    ///
    /// Returns true if the user has at least one active WebSocket connection.
    pub async fn is_user_online(&self, user_id: &str) -> bool {
        let conns = self.connections.read().await;
        conns.contains_key(user_id)
    }

    /// Get all online users
    ///
    /// Returns a list of user IDs that have at least one active connection.
    pub async fn get_online_users(&self) -> Vec<String> {
        let conns = self.connections.read().await;
        conns.keys().cloned().collect()
    }

    /// Send a WebSocket message to all active connections for a user.
    ///
    /// # Returns
    /// The number of connections the message was successfully delivered to.
    pub async fn send_to_user(&self, user_id: &str, message: WsMessage) -> usize {
        let conns = self.connections.read().await;
        if let Some(entries) = conns.get(user_id) {
            let mut delivered = 0;
            for conn in entries {
                if conn.sender.try_send(message.clone()).is_ok() {
                    delivered += 1;
                }
            }
            delivered
        } else {
            0
        }
    }

    /// Broadcast a message to multiple user IDs.
    ///
    /// Sends the same message to all specified users. Each user receives
    /// the message on all their active connections.
    pub async fn broadcast_to_users<I>(&self, user_ids: I, message: WsMessage)
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        for uid in user_ids {
            let _ = self.send_to_user(uid.as_ref(), message.clone()).await;
        }
    }
}

/// Message validation and processing
pub struct MessageValidator;

impl MessageValidator {
    /// Validate message envelope structure
    pub fn validate_envelope(envelope: &MessageEnvelope) -> Result<(), String> {
        // Check ID is non-empty UUID
        if envelope.id.is_empty() {
            return Err("Message ID cannot be empty".to_string());
        }

        // Validate UUID format
        if uuid::Uuid::parse_str(&envelope.id).is_err() {
            return Err(format!("Message ID must be a valid UUID, got: {}", envelope.id));
        }

        // Check message type is valid
        match envelope.msg_type.as_str() {
            crate::models::msg_type::MESSAGE
            | crate::models::msg_type::TYPING
            | crate::models::msg_type::PRESENCE
            | crate::models::msg_type::ACK
            | crate::models::msg_type::ERROR
            | crate::models::msg_type::HEARTBEAT => {}
            _ => return Err(format!("Invalid message type: {}", envelope.msg_type)),
        }

        // Check timestamp is reasonable (not far in future/past).
        // Allow 5 minutes (300,000ms) tolerance to account for clock skew between clients and server.
        let now = chrono::Utc::now().timestamp_millis();
        let time_diff = (envelope.timestamp as i64).saturating_sub(now).abs();
        if time_diff > 300_000 {
            return Err("Timestamp out of reasonable range".to_string());
        }

        Ok(())
    }

    /// Validate text message data
    pub fn validate_text_message(content: &str, recipient_id: &str) -> Result<(), String> {
        if content.is_empty() || content.len() > MAX_MESSAGE_LENGTH {
            return Err(format!(
                "Message content must be 1-{} characters, got {}",
                MAX_MESSAGE_LENGTH,
                content.len()
            ));
        }

        if recipient_id.is_empty() {
            return Err("Recipient ID cannot be empty".to_string());
        }

        Ok(())
    }

    /// Validate typing indicator
    pub fn validate_typing(recipient_id: &str) -> Result<(), String> {
        if recipient_id.is_empty() {
            return Err("Recipient ID cannot be empty".to_string());
        }
        Ok(())
    }
}

/// Error response builder
pub struct ErrorResponse;

impl ErrorResponse {
    pub fn create_error_response(
        code: &str,
        message: &str,
        details: Option<serde_json::Value>,
    ) -> WsMessage {
        let mut data = serde_json::json!({
            "code": code,
            "message": message,
        });

        if let Some(d) = details {
            data["details"] = d;
        }

        let error = json!({
            "id": uuid::Uuid::new_v4().to_string(),
            "type": "error",
            "timestamp": chrono::Utc::now().timestamp_millis() as u64,
            "data": data,
        });

        WsMessage::text(error.to_string())
    }

    pub fn invalid_message_length(sent_length: usize, max_length: usize) -> WsMessage {
        Self::create_error_response(
            "INVALID_MESSAGE_LENGTH",
            &format!("Message content exceeds {} character limit", max_length),
            Some(serde_json::json!({
                "sentLength": sent_length,
                "maxLength": max_length
            })),
        )
    }

    pub fn recipient_not_found(recipient_id: &str) -> WsMessage {
        Self::create_error_response(
            "RECIPIENT_NOT_FOUND",
            "Recipient user not found",
            Some(serde_json::json!({
                "recipientId": recipient_id
            })),
        )
    }

    pub fn unauthorized(reason: &str) -> WsMessage {
        Self::create_error_response("UNAUTHORIZED", reason, None)
    }

    pub fn invalid_json() -> WsMessage {
        Self::create_error_response("INVALID_JSON", "Message is not valid JSON", None)
    }

    pub fn server_error(reason: &str) -> WsMessage {
        Self::create_error_response("SERVER_ERROR", reason, None)
    }

    pub fn authorization_failure() -> WsMessage {
        Self::create_error_response(
            "AUTHORIZATION_FAILURE",
            "You are not authorized to access this conversation",
            None,
        )
    }

    pub fn invalid_message(reason: &str) -> WsMessage {
        Self::create_error_response("INVALID_MESSAGE", reason, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[test]
    fn test_client_connection_new() {
        let client = ClientConnection::new("user123".to_string(), "alice".to_string());
        assert_eq!(client.user_id, "user123");
        assert_eq!(client.username, "alice");
        assert!(!client.connection_id.is_empty());
        assert!(client.connected_at > 0);
    }

    #[tokio::test]
    async fn test_connection_manager_register() {
        let manager = ConnectionManager::new();
        let client = ClientConnection::new("user123".to_string(), "alice".to_string());
        let connection_id = client.connection_id.clone();

        let (tx, _rx) = mpsc::channel(100);
        let registered_id = manager.register(client, tx).await;
        assert_eq!(registered_id, connection_id);

        assert!(manager.is_user_online("user123").await);
        let conns = manager.get_user_connections("user123").await;
        assert_eq!(conns.len(), 1);
        assert_eq!(conns[0].user_id, "user123");
    }

    #[tokio::test]
    async fn test_connection_manager_unregister() {
        let manager = ConnectionManager::new();
        let client = ClientConnection::new("user123".to_string(), "alice".to_string());
        let connection_id = client.connection_id.clone();

        let (tx, _rx) = mpsc::channel(100);
        manager.register(client, tx).await;
        assert!(manager.is_user_online("user123").await);

        manager.unregister("user123", &connection_id).await;
        assert!(!manager.is_user_online("user123").await);
    }

    #[tokio::test]
    async fn test_connection_manager_multiple_connections() {
        let manager = ConnectionManager::new();
        let client1 = ClientConnection::new("user123".to_string(), "alice".to_string());
        let client2 = ClientConnection::new("user123".to_string(), "alice".to_string());
        let conn2_id = client2.connection_id.clone();

        let (tx1, _rx1) = mpsc::channel(100);
        let (tx2, _rx2) = mpsc::channel(100);

        manager.register(client1, tx1).await;
        manager.register(client2, tx2).await;

        let conns = manager.get_user_connections("user123").await;
        assert_eq!(conns.len(), 2);

        manager.unregister("user123", &conn2_id).await;
        let conns = manager.get_user_connections("user123").await;
        assert_eq!(conns.len(), 1);

        manager.unregister("user123", &conns[0].connection_id).await;
        assert!(!manager.is_user_online("user123").await);
    }

    #[test]
    fn test_message_validator_valid_envelope() {
        let envelope = MessageEnvelope {
            id: uuid::Uuid::new_v4().to_string(),
            msg_type: "message".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            data: serde_json::json!({}),
        };

        assert!(MessageValidator::validate_envelope(&envelope).is_ok());
    }

    #[test]
    fn test_message_validator_empty_id() {
        let envelope = MessageEnvelope {
            id: "".to_string(),
            msg_type: "message".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            data: serde_json::json!({}),
        };

        assert!(MessageValidator::validate_envelope(&envelope).is_err());
    }

    #[test]
    fn test_message_validator_invalid_type() {
        let envelope = MessageEnvelope {
            id: "msg-123".to_string(),
            msg_type: "invalid_type".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            data: serde_json::json!({}),
        };

        assert!(MessageValidator::validate_envelope(&envelope).is_err());
    }

    #[test]
    fn test_message_validator_text_message_valid() {
        assert!(MessageValidator::validate_text_message("Hello", "recipient-456").is_ok());
    }

    #[test]
    fn test_message_validator_text_message_empty() {
        assert!(MessageValidator::validate_text_message("", "recipient-456").is_err());
    }

    #[test]
    fn test_message_validator_text_message_too_long() {
        let long_content = "a".repeat(MAX_MESSAGE_LENGTH + 1);
        assert!(MessageValidator::validate_text_message(&long_content, "recipient-456").is_err());
    }

    #[test]
    fn test_message_validator_text_message_no_recipient() {
        assert!(MessageValidator::validate_text_message("Hello", "").is_err());
    }

    #[test]
    fn test_error_response_invalid_message_length() {
        let response = ErrorResponse::invalid_message_length(5001, MAX_MESSAGE_LENGTH);
        if response.is_text() {
            let text = response.to_str().unwrap();
            assert!(text.contains("INVALID_MESSAGE_LENGTH"));
            assert!(text.contains("5001"));
            assert!(text.contains(&MAX_MESSAGE_LENGTH.to_string()));
        } else {
            panic!("Expected text message");
        }
    }

    #[test]
    fn test_error_response_unauthorized() {
        let response = ErrorResponse::unauthorized("Token expired");
        if response.is_text() {
            let text = response.to_str().unwrap();
            assert!(text.contains("UNAUTHORIZED"));
            assert!(text.contains("Token expired"));
        } else {
            panic!("Expected text message");
        }
    }
}
