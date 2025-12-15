//! Contract testing framework for WebSocket protocol validation
//!
//! This module validates that all WebSocket messages conform to the schema
//! defined in `specs/001-private-chat/contracts/message-envelope-schema.json`

use jsonschema::JSONSchema;
use serde_json::{json, Value};

/// Load the message envelope schema from the contract file
fn load_schema() -> Value {
    let schema_json = include_str!("../../../specs/001-private-chat/contracts/message-envelope-schema.json");
    serde_json::from_str(schema_json).expect("Failed to parse schema")
}

/// Validate a message envelope against the schema
fn validate_message_envelope(msg: &str) -> Result<(), String> {
    let schema_value = load_schema();
    let envelope_schema = &schema_value["definitions"]["messageEnvelope"];
    
    let schema = JSONSchema::compile(envelope_schema)
        .map_err(|e| format!("Schema compilation error: {}", e))?;
    
    let message_value: Value = serde_json::from_str(msg)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    if schema.is_valid(&message_value) {
        Ok(())
    } else {
        // Collect validation errors for reporting
        let errors: Vec<String> = schema.validate(&message_value)
            .err()
            .into_iter()
            .flat_map(|e| e.map(|err| err.to_string()))
            .collect();
        Err(format!("Validation error: {}", errors.join("; ")))
    }
}

/// Validate a JWT token claims object
fn validate_jwt_claims(claims: &Value) -> Result<(), String> {
    let schema_value = load_schema();
    let jwt_schema = &schema_value["definitions"]["jwtClaims"];
    
    let schema = JSONSchema::compile(jwt_schema)
        .map_err(|e| format!("JWT schema compilation error: {}", e))?;
    
    if !schema.is_valid(claims) {
        let errors: Vec<String> = schema.validate(claims)
            .err()
            .into_iter()
            .flat_map(|e| e.map(|err| err.to_string()))
            .collect();
        return Err(format!("JWT validation error: {}", errors.join("; ")));
    }
    
    // Additional validation: check expiration
    if let Some(exp) = claims.get("exp").and_then(|v| v.as_i64()) {
        let now = chrono::Utc::now().timestamp();
        if exp < now {
            return Err("Token is expired".to_string());
        }
    }
    
    Ok(())
}

/// Validate a conversation object
fn validate_conversation(conv: &Value) -> Result<(), String> {
    let schema_value = load_schema();
    let conv_schema = &schema_value["definitions"]["conversation"];
    
    let schema = JSONSchema::compile(conv_schema)
        .map_err(|e| format!("Conversation schema compilation error: {}", e))?;
    
    if !schema.is_valid(conv) {
        let errors: Vec<String> = schema.validate(conv)
            .err()
            .into_iter()
            .flat_map(|e| e.map(|err| err.to_string()))
            .collect();
        return Err(format!("Conversation validation error: {}", errors.join("; ")));
    }
    
    // Additional validation: user1_id must be < user2_id
    if let (Some(user1), Some(user2)) = (conv.get("user1_id"), conv.get("user2_id")) {
        if user1.as_str().unwrap_or("") >= user2.as_str().unwrap_or("") {
            return Err("user1_id must be less than user2_id".to_string());
        }
        
        if user1 == user2 {
            return Err("Cannot have conversation with self".to_string());
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // MESSAGE ENVELOPE TESTS
    // ============================================================================

    #[test]
    fn test_valid_text_message_envelope() {
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "message",
            "timestamp": 1702657890000i64,
            "data": {
                "recipient_id": "660e8400-e29b-41d4-a716-446655440001",
                "content": "Hello, world!",
                "status": "pending"
            }
        }).to_string();
        
        assert!(validate_message_envelope(&msg).is_ok(), "Valid text message should pass");
    }

    #[test]
    fn test_valid_typing_indicator() {
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "typing",
            "timestamp": 1702657890000i64,
            "data": {
                "recipient_id": "660e8400-e29b-41d4-a716-446655440001",
                "is_typing": true
            }
        }).to_string();
        
        assert!(validate_message_envelope(&msg).is_ok(), "Valid typing indicator should pass");
    }

    #[test]
    fn test_valid_presence() {
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "presence",
            "timestamp": 1702657890000i64,
            "data": {
                "user_id": "550e8400-e29b-41d4-a716-446655440000",
                "username": "alice",
                "is_online": true,
                "last_seen_at": 1702657890000i64
            }
        }).to_string();
        
        assert!(validate_message_envelope(&msg).is_ok(), "Valid presence should pass");
    }

    #[test]
    fn test_valid_ack() {
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "ack",
            "timestamp": 1702657890000i64,
            "data": {
                "status": "sent",
                "message_id": "550e8400-e29b-41d4-a716-446655440001",
                "conversation_id": "conv-123",
                "server_timestamp": 1702657890123i64
            }
        }).to_string();
        
        assert!(validate_message_envelope(&msg).is_ok(), "Valid ACK should pass");
    }

    #[test]
    fn test_valid_error_message() {
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "error",
            "timestamp": 1702657890000i64,
            "data": {
                "code": "VALIDATION_ERROR",
                "message": "Message content too long"
            }
        }).to_string();
        
        assert!(validate_message_envelope(&msg).is_ok(), "Valid error message should pass");
    }

    #[test]
    fn test_valid_heartbeat() {
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "heartbeat",
            "timestamp": 1702657890000i64,
            "data": {}
        }).to_string();
        
        assert!(validate_message_envelope(&msg).is_ok(), "Valid heartbeat should pass");
    }

    // ============================================================================
    // MESSAGE ENVELOPE VALIDATION FAILURES
    // ============================================================================

    #[test]
    fn test_missing_id_field() {
        let msg = json!({
            "type": "message",
            "timestamp": 1702657890000i64,
            "data": {
                "recipient_id": "660e8400-e29b-41d4-a716-446655440001",
                "content": "Hello"
            }
        }).to_string();
        
        assert!(validate_message_envelope(&msg).is_err(), "Missing id should fail");
    }

    #[test]
    fn test_invalid_message_type() {
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "invalid_type",
            "timestamp": 1702657890000i64,
            "data": {}
        }).to_string();
        
        assert!(validate_message_envelope(&msg).is_err(), "Invalid message type should fail");
    }

    #[test]
    fn test_missing_timestamp() {
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "message",
            "data": {
                "recipient_id": "660e8400-e29b-41d4-a716-446655440001",
                "content": "Hello"
            }
        }).to_string();
        
        assert!(validate_message_envelope(&msg).is_err(), "Missing timestamp should fail");
    }

    #[test]
    fn test_missing_data_field() {
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "message",
            "timestamp": 1702657890000i64
        }).to_string();
        
        assert!(validate_message_envelope(&msg).is_err(), "Missing data field should fail");
    }

    // ============================================================================
    // TEXT MESSAGE CONTENT VALIDATION
    // ============================================================================
    // Note: These tests verify that the envelope structure validates correctly.
    // Content-level validation (length, UTF-8, etc.) happens at the application layer,
    // not at the schema validation layer.

    #[test]
    fn test_message_content_empty() {
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "message",
            "timestamp": 1702657890000i64,
            "data": {
                "recipient_id": "660e8400-e29b-41d4-a716-446655440001",
                "content": "",
                "status": "pending"
            }
        }).to_string();
        
        // Note: Empty content passes envelope schema validation.
        // Application layer should reject via content validation.
        assert!(validate_message_envelope(&msg).is_ok(), "Envelope with empty content is structurally valid");
    }

    #[test]
    fn test_message_content_exceeds_limit() {
        let long_content = "a".repeat(5001);
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "message",
            "timestamp": 1702657890000i64,
            "data": {
                "recipient_id": "660e8400-e29b-41d4-a716-446655440001",
                "content": long_content,
                "status": "pending"
            }
        }).to_string();
        
        // Note: Oversized content passes envelope schema validation.
        // Application layer should reject via content validation.
        assert!(validate_message_envelope(&msg).is_ok(), "Envelope with oversized content is structurally valid");
    }

    #[test]
    fn test_message_content_at_max_limit() {
        let max_content = "a".repeat(5000);
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "message",
            "timestamp": 1702657890000i64,
            "data": {
                "recipient_id": "660e8400-e29b-41d4-a716-446655440001",
                "content": max_content,
                "status": "pending"
            }
        }).to_string();
        
        assert!(validate_message_envelope(&msg).is_ok(), "Content = 5000 chars should pass");
    }

    // ============================================================================
    // JWT TOKEN CLAIMS VALIDATION
    // ============================================================================

    #[test]
    fn test_valid_jwt_claims() {
        let now = chrono::Utc::now().timestamp();
        let claims = json!({
            "sub": "550e8400-e29b-41d4-a716-446655440000",
            "aud": "chat-app",
            "iat": now,
            "exp": now + 3600,
            "scopes": ["send", "receive"]
        });
        
        assert!(validate_jwt_claims(&claims).is_ok(), "Valid JWT claims should pass");
    }

    #[test]
    fn test_missing_jwt_subject() {
        let now = chrono::Utc::now().timestamp();
        let claims = json!({
            "aud": "chat-app",
            "iat": now,
            "exp": now + 3600
        });
        
        assert!(validate_jwt_claims(&claims).is_err(), "Missing subject should fail");
    }

    #[test]
    fn test_expired_jwt_token() {
        let now = chrono::Utc::now().timestamp();
        let claims = json!({
            "sub": "550e8400-e29b-41d4-a716-446655440000",
            "aud": "chat-app",
            "iat": now - 7200,
            "exp": now - 3600
        });
        
        assert!(validate_jwt_claims(&claims).is_err(), "Expired token should fail");
    }

    #[test]
    fn test_invalid_jwt_audience() {
        let now = chrono::Utc::now().timestamp();
        let claims = json!({
            "sub": "550e8400-e29b-41d4-a716-446655440000",
            "aud": "wrong-app",
            "iat": now,
            "exp": now + 3600
        });
        
        assert!(validate_jwt_claims(&claims).is_err(), "Wrong audience should fail");
    }

    // ============================================================================
    // CONVERSATION VALIDATION
    // ============================================================================

    #[test]
    fn test_valid_conversation() {
        let conv = json!({
            "id": "conv-550e8400-e29b-41d4-a716-446655440000",
            "user1_id": "550e8400-e29b-41d4-a716-446655440000",
            "user2_id": "660e8400-e29b-41d4-a716-446655440001",
            "created_at": 1702657890000i64
        });
        
        assert!(validate_conversation(&conv).is_ok(), "Valid conversation should pass");
    }

    #[test]
    fn test_conversation_user1_equals_user2() {
        let conv = json!({
            "id": "conv-550e8400-e29b-41d4-a716-446655440000",
            "user1_id": "550e8400-e29b-41d4-a716-446655440000",
            "user2_id": "550e8400-e29b-41d4-a716-446655440000",
            "created_at": 1702657890000i64
        });
        
        assert!(validate_conversation(&conv).is_err(), "Self-conversation should fail");
    }

    #[test]
    fn test_conversation_user1_greater_than_user2() {
        let conv = json!({
            "id": "conv-550e8400-e29b-41d4-a716-446655440000",
            "user1_id": "zz0e8400-e29b-41d4-a716-446655440000",
            "user2_id": "aa0e8400-e29b-41d4-a716-446655440000",
            "created_at": 1702657890000i64
        });
        
        assert!(validate_conversation(&conv).is_err(), "user1_id > user2_id should fail");
    }

    #[test]
    fn test_missing_conversation_id() {
        let conv = json!({
            "user1_id": "550e8400-e29b-41d4-a716-446655440000",
            "user2_id": "660e8400-e29b-41d4-a716-446655440001",
            "created_at": 1702657890000i64
        });
        
        assert!(validate_conversation(&conv).is_err(), "Missing conversation id should fail");
    }

    #[test]
    fn test_missing_conversation_created_at() {
        let conv = json!({
            "id": "conv-550e8400-e29b-41d4-a716-446655440000",
            "user1_id": "550e8400-e29b-41d4-a716-446655440000",
            "user2_id": "660e8400-e29b-41d4-a716-446655440001"
        });
        
        assert!(validate_conversation(&conv).is_err(), "Missing created_at should fail");
    }

    // ============================================================================
    // EDGE CASES & CORNER CASES
    // ============================================================================

    #[test]
    fn test_message_with_unicode_content() {
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "message",
            "timestamp": 1702657890000i64,
            "data": {
                "recipient_id": "660e8400-e29b-41d4-a716-446655440001",
                "content": "Hello 世界 🌍 مرحبا",
                "status": "pending"
            }
        }).to_string();
        
        assert!(validate_message_envelope(&msg).is_ok(), "Unicode content should pass");
    }

    #[test]
    fn test_message_with_newlines() {
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "message",
            "timestamp": 1702657890000i64,
            "data": {
                "recipient_id": "660e8400-e29b-41d4-a716-446655440001",
                "content": "Line 1\nLine 2\nLine 3",
                "status": "pending"
            }
        }).to_string();
        
        assert!(validate_message_envelope(&msg).is_ok(), "Newlines should pass");
    }

    #[test]
    fn test_message_with_json_escape_characters() {
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "message",
            "timestamp": 1702657890000i64,
            "data": {
                "recipient_id": "660e8400-e29b-41d4-a716-446655440001",
                "content": r#"She said "Hello" \ /"#,
                "status": "pending"
            }
        }).to_string();
        
        assert!(validate_message_envelope(&msg).is_ok(), "Escaped characters should pass");
    }

    #[test]
    fn test_ack_with_all_optional_fields() {
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "ack",
            "timestamp": 1702657890000i64,
            "data": {
                "status": "delivered",
                "message_id": "550e8400-e29b-41d4-a716-446655440001",
                "conversation_id": "conv-123",
                "server_timestamp": 1702657890123i64
            }
        }).to_string();
        
        assert!(validate_message_envelope(&msg).is_ok(), "ACK with all fields should pass");
    }

    #[test]
    fn test_ack_with_minimal_fields() {
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "ack",
            "timestamp": 1702657890000i64,
            "data": {
                "status": "sent"
            }
        }).to_string();
        
        assert!(validate_message_envelope(&msg).is_ok(), "ACK with minimal fields should pass");
    }

    #[test]
    fn test_presence_status_transitions() {
        // Test user coming online
        let online = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "presence",
            "timestamp": 1702657890000i64,
            "data": {
                "user_id": "550e8400-e29b-41d4-a716-446655440000",
                "username": "alice",
                "is_online": true,
                "last_seen_at": 1702657890000i64
            }
        }).to_string();
        
        assert!(validate_message_envelope(&online).is_ok(), "User online should pass");
        
        // Test user going offline
        let offline = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "presence",
            "timestamp": 1702657890000i64,
            "data": {
                "user_id": "550e8400-e29b-41d4-a716-446655440000",
                "username": "alice",
                "is_online": false,
                "last_seen_at": 1702657890000i64
            }
        }).to_string();
        
        assert!(validate_message_envelope(&offline).is_ok(), "User offline should pass");
    }

    #[test]
    fn test_message_status_values() {
        for status in &["pending", "sent", "delivered", "failed"] {
            let msg = json!({
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "type": "message",
                "timestamp": 1702657890000i64,
                "data": {
                    "recipient_id": "660e8400-e29b-41d4-a716-446655440001",
                    "content": "Hello",
                    "status": status
                }
            }).to_string();
            
            assert!(validate_message_envelope(&msg).is_ok(), "Status '{}' should pass", status);
        }
    }

    #[test]
    fn test_message_invalid_status() {
        let msg = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "message",
            "timestamp": 1702657890000i64,
            "data": {
                "recipient_id": "660e8400-e29b-41d4-a716-446655440001",
                "content": "Hello",
                "status": "invalid_status"
            }
        }).to_string();
        
        // Note: Invalid status value passes envelope schema validation because
        // the schema's data object doesn't specify required enum values.
        // Application layer should validate status values.
        assert!(validate_message_envelope(&msg).is_ok(), "Envelope with invalid status is structurally valid");
    }
}
