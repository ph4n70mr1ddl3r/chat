//! WebSocket message frame parser and message dispatcher
//!
//! Parses incoming WebSocket frames and routes messages to appropriate handlers.
//! Validates message format, extracts message types, and dispatches to service layer.

use crate::handlers::websocket::{ErrorResponse, MessageValidator};
use chat_shared::protocol::MessageEnvelope;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message as WsMessage;

/// Message dispatcher routes incoming WebSocket messages to appropriate handlers
pub struct MessageDispatcher;

/// Result of parsing and dispatching a message
#[derive(Debug, Clone)]
pub enum DispatchResult {
    /// Message parsed and dispatched successfully
    Success {
        msg_type: String,
        envelope: MessageEnvelope,
    },
    /// Message requires response/acknowledgement
    RequiresAck {
        message_id: String,
        msg_type: String,
    },
    /// Error occurred during parsing or dispatching
    Error { error_msg: WsMessage },
    /// Connection should be closed
    Close { code: u16, reason: String },
}

impl MessageDispatcher {
    /// Parse and validate incoming WebSocket message frame
    pub fn parse_message(msg: &WsMessage) -> DispatchResult {
        // Only process text messages
        match msg {
            WsMessage::Text(text) => Self::parse_text_frame(text),
            WsMessage::Binary(_) => DispatchResult::Error {
                error_msg: ErrorResponse::server_error("Binary frames not supported"),
            },
            WsMessage::Close(frame) => {
                let (code, reason) = frame
                    .as_ref()
                    .map(|f| (f.code.into(), f.reason.to_string()))
                    .unwrap_or((1000, "Normal closure".to_string()));

                DispatchResult::Close { code, reason }
            }
            WsMessage::Ping(data) => {
                // Pings are handled at protocol level, but we acknowledge explicitly
                DispatchResult::Success {
                    msg_type: "ping".to_string(),
                    envelope: MessageEnvelope {
                        id: uuid::Uuid::new_v4().to_string(),
                        msg_type: "ping".to_string(),
                        timestamp: chrono::Utc::now().timestamp_millis() as u64,
                        data: json!({"data": String::from_utf8_lossy(data)}),
                    },
                }
            }
            WsMessage::Pong(_) => {
                // Pongs are handled at protocol level
                DispatchResult::Success {
                    msg_type: "pong".to_string(),
                    envelope: MessageEnvelope {
                        id: uuid::Uuid::new_v4().to_string(),
                        msg_type: "pong".to_string(),
                        timestamp: chrono::Utc::now().timestamp_millis() as u64,
                        data: json!({}),
                    },
                }
            }
            WsMessage::Frame(_) => DispatchResult::Error {
                error_msg: ErrorResponse::server_error("Raw frames not supported"),
            },
        }
    }

    /// Parse text frame into message envelope
    fn parse_text_frame(text: &str) -> DispatchResult {
        // Parse JSON
        let envelope: MessageEnvelope = match serde_json::from_str(text) {
            Ok(env) => env,
            Err(_) => {
                return DispatchResult::Error {
                    error_msg: ErrorResponse::invalid_json(),
                };
            }
        };

        // Validate envelope structure
        if let Err(e) = MessageValidator::validate_envelope(&envelope) {
            return DispatchResult::Error {
                error_msg: ErrorResponse::server_error(&format!("Invalid envelope: {}", e)),
            };
        }

        // Dispatch based on message type
        match envelope.msg_type.as_str() {
            "message" => Self::dispatch_text_message(&envelope),
            "typing" => Self::dispatch_typing(&envelope),
            "heartbeat" => DispatchResult::Success {
                msg_type: "heartbeat".to_string(),
                envelope,
            },
            "ack" | "presence" | "error" => {
                // These are typically server-sent, but could be received
                DispatchResult::Success {
                    msg_type: envelope.msg_type.clone(),
                    envelope,
                }
            }
            _ => DispatchResult::Error {
                error_msg: ErrorResponse::server_error(&format!(
                    "Unknown message type: {}",
                    envelope.msg_type
                )),
            },
        }
    }

    /// Dispatch text message with validation
    fn dispatch_text_message(envelope: &MessageEnvelope) -> DispatchResult {
        // Extract required fields from data
        let data = &envelope.data;

        let recipient_id = data
            .get("recipientId")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let content = data.get("content").and_then(|v| v.as_str()).unwrap_or("");

        // Validate message data
        if let Err(e) = MessageValidator::validate_text_message(content, recipient_id) {
            return DispatchResult::Error {
                error_msg: if e.contains("character") {
                    ErrorResponse::invalid_message_length(content.len(), 5000)
                } else {
                    ErrorResponse::server_error(&e)
                },
            };
        }

        DispatchResult::RequiresAck {
            message_id: envelope.id.clone(),
            msg_type: "message".to_string(),
        }
    }

    /// Dispatch typing indicator with validation
    fn dispatch_typing(envelope: &MessageEnvelope) -> DispatchResult {
        let data = &envelope.data;

        let recipient_id = data
            .get("recipientId")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Validate typing data
        if let Err(e) = MessageValidator::validate_typing(recipient_id) {
            return DispatchResult::Error {
                error_msg: ErrorResponse::server_error(&e),
            };
        }

        DispatchResult::Success {
            msg_type: "typing".to_string(),
            envelope: envelope.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_parse_text_message() {
        let json = json!({
            "id": "msg-123",
            "type": "message",
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "data": {
                "recipientId": "user-456",
                "content": "Hello"
            }
        });

        let msg = WsMessage::Text(json.to_string());
        let result = MessageDispatcher::parse_message(&msg);

        match result {
            DispatchResult::RequiresAck {
                message_id,
                msg_type,
            } => {
                assert_eq!(message_id, "msg-123");
                assert_eq!(msg_type, "message");
            }
            _ => panic!("Expected RequiresAck"),
        }
    }

    #[test]
    fn test_dispatcher_parse_typing() {
        let json = json!({
            "id": "typing-123",
            "type": "typing",
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "data": {
                "recipientId": "user-456",
                "isTyping": true
            }
        });

        let msg = WsMessage::Text(json.to_string());
        let result = MessageDispatcher::parse_message(&msg);

        match result {
            DispatchResult::Success { msg_type, .. } => {
                assert_eq!(msg_type, "typing");
            }
            _ => panic!("Expected Success"),
        }
    }

    #[test]
    fn test_dispatcher_invalid_json() {
        let msg = WsMessage::Text("not valid json".to_string());
        let result = MessageDispatcher::parse_message(&msg);

        match result {
            DispatchResult::Error { .. } => {}
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_dispatcher_message_too_long() {
        let long_content = "a".repeat(5001);
        let json = json!({
            "id": "msg-123",
            "type": "message",
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "data": {
                "recipientId": "user-456",
                "content": long_content
            }
        });

        let msg = WsMessage::Text(json.to_string());
        let result = MessageDispatcher::parse_message(&msg);

        match result {
            DispatchResult::Error { error_msg } => {
                if let WsMessage::Text(text) = error_msg {
                    assert!(text.contains("INVALID_MESSAGE_LENGTH"));
                }
            }
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_dispatcher_missing_recipient() {
        let json = json!({
            "id": "msg-123",
            "type": "message",
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "data": {
                "content": "Hello"
            }
        });

        let msg = WsMessage::Text(json.to_string());
        let result = MessageDispatcher::parse_message(&msg);

        match result {
            DispatchResult::Error { .. } => {}
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_dispatcher_empty_message() {
        let json = json!({
            "id": "msg-123",
            "type": "message",
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "data": {
                "recipientId": "user-456",
                "content": ""
            }
        });

        let msg = WsMessage::Text(json.to_string());
        let result = MessageDispatcher::parse_message(&msg);

        match result {
            DispatchResult::Error { .. } => {}
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_dispatcher_heartbeat() {
        let json = json!({
            "id": "hb-123",
            "type": "heartbeat",
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "data": {}
        });

        let msg = WsMessage::Text(json.to_string());
        let result = MessageDispatcher::parse_message(&msg);

        match result {
            DispatchResult::Success { msg_type, .. } => {
                assert_eq!(msg_type, "heartbeat");
            }
            _ => panic!("Expected Success"),
        }
    }

    #[test]
    fn test_dispatcher_binary_frame() {
        let msg = WsMessage::Binary(vec![1, 2, 3]);
        let result = MessageDispatcher::parse_message(&msg);

        match result {
            DispatchResult::Error { .. } => {}
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_dispatcher_ping() {
        let msg = WsMessage::Ping(vec![]);
        let result = MessageDispatcher::parse_message(&msg);

        match result {
            DispatchResult::Success { msg_type, .. } => {
                assert_eq!(msg_type, "ping");
            }
            _ => panic!("Expected Success"),
        }
    }

    #[test]
    fn test_dispatcher_pong() {
        let msg = WsMessage::Pong(vec![]);
        let result = MessageDispatcher::parse_message(&msg);

        match result {
            DispatchResult::Success { msg_type, .. } => {
                assert_eq!(msg_type, "pong");
            }
            _ => panic!("Expected Success"),
        }
    }
}
