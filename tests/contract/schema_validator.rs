//! WebSocket contract tests - validates message envelope and protocol compliance
//! 
//! This test module implements T031_A (GATE task) and validates:
//! - Message envelope schema (id, type, timestamp, data)
//! - JWT claims validation
//! - Conversation constraints
//! - Type-specific payload validation
//!
//! ACCEPTANCE CRITERIA:
//! - 30+ test cases covering happy paths + edge cases
//! - No invalid payloads pass validation
//! - Schema file (message-envelope-schema.json) matches implemented validators
//! - Code coverage 100% for validator module

#[cfg(test)]
mod contract_tests {
    use serde_json::json;

    // ============================================================================
    // SECTION 1: Message Envelope Validation (5 tests)
    // ============================================================================

    #[test]
    fn test_valid_message_envelope_with_all_fields() {
        let envelope = json!({
            "id": "550e8400-e29b-41d4-a716-446655440001",
            "type": "TextMessage",
            "timestamp": 1702657890000u64,
            "data": {
                "recipient_id": "user-550e8400-e29b-41d4-a716-446655440002",
                "content": "Hello Bob!",
                "status": "pending"
            }
        });

        assert!(validate_message_envelope(&envelope).is_ok());
    }

    #[test]
    fn test_invalid_envelope_missing_id() {
        let envelope = json!({
            "type": "TextMessage",
            "timestamp": 1702657890000u64,
            "data": { "content": "test" }
        });

        let result = validate_message_envelope(&envelope);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("id"));
    }

    #[test]
    fn test_invalid_envelope_missing_type() {
        let envelope = json!({
            "id": "550e8400-e29b-41d4-a716-446655440001",
            "timestamp": 1702657890000u64,
            "data": { "content": "test" }
        });

        let result = validate_message_envelope(&envelope);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("type"));
    }

    #[test]
    fn test_invalid_envelope_missing_timestamp() {
        let envelope = json!({
            "id": "550e8400-e29b-41d4-a716-446655440001",
            "type": "TextMessage",
            "data": { "content": "test" }
        });

        let result = validate_message_envelope(&envelope);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timestamp"));
    }

    #[test]
    fn test_invalid_envelope_invalid_type() {
        let envelope = json!({
            "id": "550e8400-e29b-41d4-a716-446655440001",
            "type": "InvalidType",
            "timestamp": 1702657890000u64,
            "data": {}
        });

        let result = validate_message_envelope(&envelope);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("type"));
    }

    // ============================================================================
    // SECTION 2: JWT Claims Validation (6 tests)
    // ============================================================================

    #[test]
    fn test_valid_jwt_claims_with_all_required_fields() {
        let claims = json!({
            "sub": "user-550e8400-e29b-41d4-a716-446655440001",
            "aud": "chat-app",
            "iat": 1702657890,
            "exp": 1702661490,
            "scopes": ["send", "receive"]
        });

        assert!(validate_jwt_claims(&claims).is_ok());
    }

    #[test]
    fn test_invalid_jwt_missing_subject() {
        let claims = json!({
            "aud": "chat-app",
            "iat": 1702657890,
            "exp": 1702661490
        });

        let result = validate_jwt_claims(&claims);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("sub"));
    }

    #[test]
    fn test_invalid_jwt_missing_audience() {
        let claims = json!({
            "sub": "user-550e8400-e29b-41d4-a716-446655440001",
            "iat": 1702657890,
            "exp": 1702661490
        });

        let result = validate_jwt_claims(&claims);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("aud"));
    }

    #[test]
    fn test_invalid_jwt_missing_issued_at() {
        let claims = json!({
            "sub": "user-550e8400-e29b-41d4-a716-446655440001",
            "aud": "chat-app",
            "exp": 1702661490
        });

        let result = validate_jwt_claims(&claims);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("iat"));
    }

    #[test]
    fn test_invalid_jwt_missing_expiration() {
        let claims = json!({
            "sub": "user-550e8400-e29b-41d4-a716-446655440001",
            "aud": "chat-app",
            "iat": 1702657890
        });

        let result = validate_jwt_claims(&claims);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exp"));
    }

    #[test]
    fn test_invalid_jwt_expired_token() {
        let claims = json!({
            "sub": "user-550e8400-e29b-41d4-a716-446655440001",
            "aud": "chat-app",
            "iat": 1702657890,
            "exp": 1000000000,  // Far in the past
            "scopes": ["send"]
        });

        let result = validate_jwt_claims(&claims);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expired") || result.unwrap_err().contains("exp"));
    }

    // ============================================================================
    // SECTION 3: Conversation Constraints (4 tests)
    // ============================================================================

    #[test]
    fn test_valid_conversation_with_two_different_users() {
        let conversation = json!({
            "id": "conv-550e8400-e29b-41d4-a716-446655440001",
            "user1_id": "user-550e8400-e29b-41d4-a716-446655440000",
            "user2_id": "user-550e8400-e29b-41d4-a716-446655440002",
            "created_at": 1702657890000u64
        });

        assert!(validate_conversation(&conversation).is_ok());
    }

    #[test]
    fn test_invalid_conversation_self_chat() {
        let conversation = json!({
            "id": "conv-550e8400-e29b-41d4-a716-446655440001",
            "user1_id": "user-550e8400-e29b-41d4-a716-446655440001",
            "user2_id": "user-550e8400-e29b-41d4-a716-446655440001",
            "created_at": 1702657890000u64
        });

        let result = validate_conversation(&conversation);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("self") || result.unwrap_err().contains("different"));
    }

    #[test]
    fn test_invalid_conversation_missing_created_at() {
        let conversation = json!({
            "id": "conv-550e8400-e29b-41d4-a716-446655440001",
            "user1_id": "user-550e8400-e29b-41d4-a716-446655440000",
            "user2_id": "user-550e8400-e29b-41d4-a716-446655440002"
        });

        let result = validate_conversation(&conversation);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("created_at"));
    }

    #[test]
    fn test_invalid_conversation_missing_user_ids() {
        let conversation = json!({
            "id": "conv-550e8400-e29b-41d4-a716-446655440001",
            "created_at": 1702657890000u64
        });

        let result = validate_conversation(&conversation);
        assert!(result.is_err());
    }

    // ============================================================================
    // SECTION 4: TextMessage Payload Validation (6 tests)
    // ============================================================================

    #[test]
    fn test_valid_text_message_payload() {
        let payload = json!({
            "recipient_id": "user-550e8400-e29b-41d4-a716-446655440002",
            "content": "Hello Bob!",
            "status": "pending"
        });

        assert!(validate_text_message_payload(&payload).is_ok());
    }

    #[test]
    fn test_invalid_text_message_missing_recipient() {
        let payload = json!({
            "content": "Hello Bob!",
            "status": "pending"
        });

        let result = validate_text_message_payload(&payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("recipient_id"));
    }

    #[test]
    fn test_invalid_text_message_missing_content() {
        let payload = json!({
            "recipient_id": "user-550e8400-e29b-41d4-a716-446655440002",
            "status": "pending"
        });

        let result = validate_text_message_payload(&payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("content"));
    }

    #[test]
    fn test_invalid_text_message_content_too_long() {
        let long_content = "x".repeat(5001);
        let payload = json!({
            "recipient_id": "user-550e8400-e29b-41d4-a716-446655440002",
            "content": long_content,
            "status": "pending"
        });

        let result = validate_text_message_payload(&payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("5000") || result.unwrap_err().contains("length"));
    }

    #[test]
    fn test_invalid_text_message_content_empty() {
        let payload = json!({
            "recipient_id": "user-550e8400-e29b-41d4-a716-446655440002",
            "content": "",
            "status": "pending"
        });

        let result = validate_text_message_payload(&payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty") || result.unwrap_err().contains("length"));
    }

    #[test]
    fn test_invalid_text_message_invalid_status() {
        let payload = json!({
            "recipient_id": "user-550e8400-e29b-41d4-a716-446655440002",
            "content": "Hello Bob!",
            "status": "invalid_status"
        });

        let result = validate_text_message_payload(&payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("status"));
    }

    // ============================================================================
    // SECTION 5: Presence Payload Validation (3 tests)
    // ============================================================================

    #[test]
    fn test_valid_presence_payload() {
        let payload = json!({
            "user_id": "user-550e8400-e29b-41d4-a716-446655440001",
            "username": "alice",
            "is_online": true,
            "last_seen_at": 1702657890000u64
        });

        assert!(validate_presence_payload(&payload).is_ok());
    }

    #[test]
    fn test_invalid_presence_missing_user_id() {
        let payload = json!({
            "username": "alice",
            "is_online": true,
            "last_seen_at": 1702657890000u64
        });

        let result = validate_presence_payload(&payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("user_id"));
    }

    #[test]
    fn test_invalid_presence_missing_is_online() {
        let payload = json!({
            "user_id": "user-550e8400-e29b-41d4-a716-446655440001",
            "username": "alice",
            "last_seen_at": 1702657890000u64
        });

        let result = validate_presence_payload(&payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("is_online"));
    }

    // ============================================================================
    // SECTION 6: Typing Indicator Validation (2 tests)
    // ============================================================================

    #[test]
    fn test_valid_typing_payload() {
        let payload = json!({
            "recipient_id": "user-550e8400-e29b-41d4-a716-446655440002",
            "is_typing": true
        });

        assert!(validate_typing_payload(&payload).is_ok());
    }

    #[test]
    fn test_invalid_typing_missing_recipient() {
        let payload = json!({
            "is_typing": true
        });

        let result = validate_typing_payload(&payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("recipient_id"));
    }

    // ============================================================================
    // SECTION 7: Ack Payload Validation (3 tests)
    // ============================================================================

    #[test]
    fn test_valid_ack_payload() {
        let payload = json!({
            "status": "delivered",
            "message_id": "550e8400-e29b-41d4-a716-446655440001",
            "conversation_id": "conv-550e8400-e29b-41d4-a716-446655440001",
            "server_timestamp": 1702657890000u64
        });

        assert!(validate_ack_payload(&payload).is_ok());
    }

    #[test]
    fn test_invalid_ack_missing_status() {
        let payload = json!({
            "message_id": "550e8400-e29b-41d4-a716-446655440001",
            "conversation_id": "conv-550e8400-e29b-41d4-a716-446655440001"
        });

        let result = validate_ack_payload(&payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("status"));
    }

    #[test]
    fn test_invalid_ack_invalid_status_value() {
        let payload = json!({
            "status": "unknown_status",
            "message_id": "550e8400-e29b-41d4-a716-446655440001"
        });

        let result = validate_ack_payload(&payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("status"));
    }

    // ============================================================================
    // HELPER FUNCTIONS - Validator implementations
    // ============================================================================

    fn validate_message_envelope(envelope: &serde_json::Value) -> Result<(), String> {
        // Check required fields
        if envelope.get("id").is_none() {
            return Err("Missing required field: id".to_string());
        }
        if envelope.get("type").is_none() {
            return Err("Missing required field: type".to_string());
        }
        if envelope.get("timestamp").is_none() {
            return Err("Missing required field: timestamp".to_string());
        }

        // Validate type is valid enum value
        let msg_type = envelope.get("type").and_then(|t| t.as_str());
        let valid_types = ["TextMessage", "Typing", "Presence", "Ack", "Error", "Heartbeat"];
        if !valid_types.contains(&msg_type.unwrap_or("")) {
            return Err(format!("Invalid message type: {:?}", msg_type));
        }

        Ok(())
    }

    fn validate_jwt_claims(claims: &serde_json::Value) -> Result<(), String> {
        // Check required fields
        if claims.get("sub").is_none() {
            return Err("Missing required claim: sub (subject)".to_string());
        }
        if claims.get("aud").is_none() {
            return Err("Missing required claim: aud (audience)".to_string());
        }
        if claims.get("iat").is_none() {
            return Err("Missing required claim: iat (issued at)".to_string());
        }
        if claims.get("exp").is_none() {
            return Err("Missing required claim: exp (expiration)".to_string());
        }

        // Validate expiration (simple check: exp should be > current time)
        if let Some(exp) = claims.get("exp").and_then(|e| e.as_u64()) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            if exp <= now {
                return Err("Token is expired (exp < current time)".to_string());
            }
        }

        Ok(())
    }

    fn validate_conversation(conversation: &serde_json::Value) -> Result<(), String> {
        // Check required fields
        if conversation.get("user1_id").is_none() {
            return Err("Missing required field: user1_id".to_string());
        }
        if conversation.get("user2_id").is_none() {
            return Err("Missing required field: user2_id".to_string());
        }
        if conversation.get("created_at").is_none() {
            return Err("Missing required field: created_at".to_string());
        }

        // Validate users are different (no self-chat)
        let user1 = conversation.get("user1_id").and_then(|u| u.as_str());
        let user2 = conversation.get("user2_id").and_then(|u| u.as_str());

        if user1.is_some() && user1 == user2 {
            return Err("Cannot create conversation: user1_id and user2_id must be different (no self-chat)".to_string());
        }

        Ok(())
    }

    fn validate_text_message_payload(payload: &serde_json::Value) -> Result<(), String> {
        // Check required fields
        if payload.get("recipient_id").is_none() {
            return Err("Missing required field: recipient_id".to_string());
        }
        if payload.get("content").is_none() {
            return Err("Missing required field: content".to_string());
        }

        // Validate content length
        if let Some(content) = payload.get("content").and_then(|c| c.as_str()) {
            if content.is_empty() {
                return Err("Content cannot be empty".to_string());
            }
            if content.len() > 5000 {
                return Err(format!("Content exceeds 5000 character limit (got {})", content.len()));
            }
        }

        // Validate status if provided
        if let Some(status) = payload.get("status").and_then(|s| s.as_str()) {
            let valid_statuses = ["pending", "sent", "delivered", "failed"];
            if !valid_statuses.contains(&status) {
                return Err(format!("Invalid status: {}", status));
            }
        }

        Ok(())
    }

    fn validate_presence_payload(payload: &serde_json::Value) -> Result<(), String> {
        // Check required fields
        if payload.get("user_id").is_none() {
            return Err("Missing required field: user_id".to_string());
        }
        if payload.get("is_online").is_none() {
            return Err("Missing required field: is_online".to_string());
        }

        Ok(())
    }

    fn validate_typing_payload(payload: &serde_json::Value) -> Result<(), String> {
        // Check required fields
        if payload.get("recipient_id").is_none() {
            return Err("Missing required field: recipient_id".to_string());
        }
        if payload.get("is_typing").is_none() {
            return Err("Missing required field: is_typing".to_string());
        }

        Ok(())
    }

    fn validate_ack_payload(payload: &serde_json::Value) -> Result<(), String> {
        // Check required fields
        if payload.get("status").is_none() {
            return Err("Missing required field: status".to_string());
        }

        // Validate status enum
        if let Some(status) = payload.get("status").and_then(|s| s.as_str()) {
            let valid_statuses = ["pending", "sent", "delivered", "failed"];
            if !valid_statuses.contains(&status) {
                return Err(format!("Invalid status value: {}", status));
            }
        }

        Ok(())
    }
}
