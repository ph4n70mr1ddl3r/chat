//! Contract tests for WebSocket message JSON schema (message-envelope-schema.json)
//!
//! Covers T098: validates envelope and type-specific payloads for message, typing, and ack types.

use jsonschema::{Draft, JSONSchema};
use serde_json::json;

fn load_schema() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../specs/001-private-chat/contracts/message-envelope-schema.json"
    ))
    .expect("invalid JSON schema file")
}

fn compile_definition(name: &str) -> JSONSchema {
    let schema = load_schema();
    let subschema = schema
        .get("definitions")
        .and_then(|defs| defs.get(name))
        .cloned()
        .expect("definition missing");

    JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&subschema)
        .expect("schema should compile")
}

#[test]
fn text_message_payload_validates() {
    let validator = compile_definition("textMessage");
    let payload = json!({
        "recipient_id": "550e8400-e29b-41d4-a716-446655440000",
        "content": "Hello Bob!",
        "status": "pending"
    });

    assert!(validator.is_valid(&payload));
}

#[test]
fn text_message_requires_content() {
    let validator = compile_definition("textMessage");
    let payload = json!({
        "recipient_id": "550e8400-e29b-41d4-a716-446655440000",
        "status": "pending"
    });

    let result = validator.validate(&payload);
    assert!(result.is_err());
}

#[test]
fn typing_payload_validates() {
    let validator = compile_definition("typing");
    let payload = json!({
        "recipient_id": "550e8400-e29b-41d4-a716-446655440000",
        "is_typing": true
    });

    assert!(validator.is_valid(&payload));
}

#[test]
fn typing_payload_requires_recipient() {
    let validator = compile_definition("typing");
    let payload = json!({
        "is_typing": false
    });

    let result = validator.validate(&payload);
    assert!(result.is_err());
}

#[test]
fn ack_payload_requires_status() {
    let validator = compile_definition("ack");
    let payload = json!({
        "message_id": "550e8400-e29b-41d4-a716-446655440000",
        "conversation_id": "660e8400-e29b-41d4-a716-446655440000"
    });

    let result = validator.validate(&payload);
    assert!(result.is_err());
}

#[test]
fn envelope_requires_id_type_timestamp() {
    let validator = compile_definition("messageEnvelope");
    let payload = json!({
        "type": "message",
        "data": { "recipient_id": "550e8400-e29b-41d4-a716-446655440000" }
    });

    let result = validator.validate(&payload);
    assert!(result.is_err());
}

#[test]
fn envelope_accepts_valid_message_wrapper() {
    let validator = compile_definition("messageEnvelope");
    let payload = json!({
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "type": "message",
        "timestamp": 1700000000000u64,
        "data": {
            "recipient_id": "660e8400-e29b-41d4-a716-446655440000",
            "content": "hello"
        }
    });

    assert!(validator.is_valid(&payload));
}
