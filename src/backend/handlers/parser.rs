//! WebSocket frame parser and JSON validation
//!
//! Parses incoming WebSocket frames into message envelopes with comprehensive error handling.
//! Validates JSON structure, message envelope format, and provides detailed error responses.

use crate::handlers::websocket::{ErrorResponse, MessageValidator};
use chat_shared::protocol::MessageEnvelope;
use serde_json::json;
use warp::ws::Message as WsMessage;

/// Result of parsing a WebSocket frame
#[derive(Debug, Clone)]
pub enum ParseResult {
    /// Successfully parsed message envelope
    Parsed {
        envelope: MessageEnvelope,
        frame_type: FrameType,
    },
    /// Frame requires protocol-level handling (ping/pong/close)
    Protocol {
        frame_type: FrameType,
        data: Option<Vec<u8>>,
    },
    /// Parse error with error message to send back
    Error { error_msg: WsMessage },
    /// Connection should be closed
    Close { code: u16, reason: String },
}

/// Type of WebSocket frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameType {
    Text,
    Binary,
    Ping,
    Pong,
    Close,
    Unknown,
}

/// WebSocket frame parser
pub struct FrameParser;

impl FrameParser {
    /// Parse incoming WebSocket frame
    pub fn parse(msg: &WsMessage) -> ParseResult {
        if msg.is_text() {
            if let Ok(text) = msg.to_str() {
                return Self::parse_text_frame(text);
            }
        }

        if msg.is_binary() {
            return ParseResult::Error {
                error_msg: ErrorResponse::server_error("Binary frames not supported"),
            };
        }

        if msg.is_ping() {
            return ParseResult::Protocol {
                frame_type: FrameType::Ping,
                data: None,
            };
        }

        if msg.is_pong() {
            return ParseResult::Protocol {
                frame_type: FrameType::Pong,
                data: None,
            };
        }

        if msg.is_close() {
            return ParseResult::Close {
                code: 1000,
                reason: "Normal closure".to_string(),
            };
        }

        ParseResult::Error {
            error_msg: ErrorResponse::server_error("Unsupported frame type"),
        }
    }

    /// Parse text frame into message envelope
    fn parse_text_frame(text: &str) -> ParseResult {
        // Parse JSON
        let envelope: MessageEnvelope = match serde_json::from_str(text) {
            Ok(env) => env,
            Err(_e) => {
                return ParseResult::Error {
                    error_msg: ErrorResponse::invalid_json(),
                };
            }
        };

        // Validate envelope structure
        if let Err(e) = MessageValidator::validate_envelope(&envelope) {
            return ParseResult::Error {
                error_msg: ErrorResponse::server_error(&format!("Invalid envelope: {}", e)),
            };
        }

        ParseResult::Parsed {
            envelope,
            frame_type: FrameType::Text,
        }
    }

    /// Parse raw JSON string into message envelope (for testing)
    pub fn parse_json(text: &str) -> Result<MessageEnvelope, String> {
        let envelope: MessageEnvelope =
            serde_json::from_str(text).map_err(|e| format!("Invalid JSON: {}", e))?;

        MessageValidator::validate_envelope(&envelope)
            .map_err(|e| format!("Invalid envelope: {}", e))?;

        Ok(envelope)
    }

    /// Extract message type from envelope
    pub fn extract_message_type(envelope: &MessageEnvelope) -> &str {
        &envelope.msg_type
    }

    /// Validate text message data (content and recipient)
    pub fn validate_text_message_data(
        envelope: &MessageEnvelope,
    ) -> Result<(String, String), String> {
        let data = &envelope.data;

        let recipient_id = data
            .get("recipientId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let content = data
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        MessageValidator::validate_text_message(&content, &recipient_id)
            .map(|_| (recipient_id, content))
            .map_err(|e| e.to_string())
    }

    /// Validate typing indicator data
    pub fn validate_typing_data(envelope: &MessageEnvelope) -> Result<String, String> {
        let data = &envelope.data;

        let recipient_id = data
            .get("recipientId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        MessageValidator::validate_typing(&recipient_id)
            .map(|_| recipient_id)
            .map_err(|e| e.to_string())
    }

    /// Create error response for invalid message
    pub fn create_error_response(
        code: &str,
        message: &str,
        details: Option<serde_json::Value>,
    ) -> WsMessage {
        let error = json!({
            "id": uuid::Uuid::new_v4().to_string(),
            "type": "error",
            "timestamp": chrono::Utc::now().timestamp_millis() as u64,
            "data": {
                "code": code,
                "message": message,
                "details": details
            }
        });

        WsMessage::text(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_text_message() {
        let json = json!({
            "id": "msg-123",
            "type": "message",
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "data": {
                "recipientId": "user-456",
                "content": "Hello"
            }
        });

        let msg = WsMessage::text(json.to_string());
        let result = FrameParser::parse(&msg);

        match result {
            ParseResult::Parsed {
                envelope,
                frame_type,
            } => {
                assert_eq!(frame_type, FrameType::Text);
                assert_eq!(envelope.id, "msg-123");
                assert_eq!(envelope.msg_type, "message");
            }
            _ => panic!("Expected Parsed result"),
        }
    }

    #[test]
    fn test_parse_invalid_json() {
        let msg = WsMessage::text("not valid json".to_string());
        let result = FrameParser::parse(&msg);

        match result {
            ParseResult::Error { .. } => {}
            _ => panic!("Expected Error result"),
        }
    }

    #[test]
    fn test_parse_binary_frame() {
        let msg = WsMessage::binary(vec![1, 2, 3]);
        let result = FrameParser::parse(&msg);

        match result {
            ParseResult::Error { .. } => {}
            _ => panic!("Expected Error result"),
        }
    }

    #[test]
    fn test_parse_ping() {
        let msg = WsMessage::ping(vec![1, 2, 3]);
        let result = FrameParser::parse(&msg);

        match result {
            ParseResult::Protocol { frame_type, data } => {
                assert_eq!(frame_type, FrameType::Ping);
                // Ping payload is not preserved in current implementation
                assert_eq!(data, None);
            }
            _ => panic!("Expected Protocol result"),
        }
    }

    #[test]
    fn test_parse_pong() {
        let msg = WsMessage::pong(vec![4, 5, 6]);
        let result = FrameParser::parse(&msg);

        match result {
            ParseResult::Protocol { frame_type, data } => {
                assert_eq!(frame_type, FrameType::Pong);
                // Pong payload is not preserved in current implementation
                assert_eq!(data, None);
            }
            _ => panic!("Expected Protocol result"),
        }
    }

    #[test]
    fn test_parse_close() {
        let msg = WsMessage::close();
        let result = FrameParser::parse(&msg);

        match result {
            ParseResult::Close { code, reason } => {
                assert_eq!(code, 1000);
                assert_eq!(reason, "Normal closure");
            }
            _ => panic!("Expected Close result"),
        }
    }

    #[test]
    fn test_parse_json_valid() {
        let json = json!({
            "id": "msg-123",
            "type": "message",
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "data": {}
        })
        .to_string();

        let result = FrameParser::parse_json(&json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_json_invalid() {
        let result = FrameParser::parse_json("invalid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_message_type() {
        let envelope = MessageEnvelope {
            id: "msg-123".to_string(),
            msg_type: "message".to_string(),
            timestamp: 1234567890,
            data: json!({}),
        };

        assert_eq!(FrameParser::extract_message_type(&envelope), "message");
    }

    #[test]
    fn test_validate_text_message_data_valid() {
        let envelope = MessageEnvelope {
            id: "msg-123".to_string(),
            msg_type: "message".to_string(),
            timestamp: 1234567890,
            data: json!({
                "recipientId": "user-456",
                "content": "Hello"
            }),
        };

        let result = FrameParser::validate_text_message_data(&envelope);
        assert!(result.is_ok());
        let (recipient_id, content) = result.unwrap();
        assert_eq!(recipient_id, "user-456");
        assert_eq!(content, "Hello");
    }

    #[test]
    fn test_validate_text_message_data_invalid() {
        let envelope = MessageEnvelope {
            id: "msg-123".to_string(),
            msg_type: "message".to_string(),
            timestamp: 1234567890,
            data: json!({
                "recipientId": "",
                "content": ""
            }),
        };

        let result = FrameParser::validate_text_message_data(&envelope);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_typing_data_valid() {
        let envelope = MessageEnvelope {
            id: "typing-123".to_string(),
            msg_type: "typing".to_string(),
            timestamp: 1234567890,
            data: json!({
                "recipientId": "user-456",
                "isTyping": true
            }),
        };

        let result = FrameParser::validate_typing_data(&envelope);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "user-456");
    }

    #[test]
    fn test_validate_typing_data_invalid() {
        let envelope = MessageEnvelope {
            id: "typing-123".to_string(),
            msg_type: "typing".to_string(),
            timestamp: 1234567890,
            data: json!({
                "recipientId": "",
                "isTyping": true
            }),
        };

        let result = FrameParser::validate_typing_data(&envelope);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_error_response() {
        let response = FrameParser::create_error_response(
            "TEST_ERROR",
            "Test message",
            Some(json!({"detail": "extra"})),
        );

        assert!(response.is_text(), "Expected text message");
        let text = response.to_str().unwrap_or_default();
        assert!(text.contains("TEST_ERROR"));
        assert!(text.contains("Test message"));
        assert!(text.contains("extra"));
    }
}
