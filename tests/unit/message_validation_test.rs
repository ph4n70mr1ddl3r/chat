//! Unit tests for message validation rules (content length, UTF-8 validity, recipient authorization)
//!
//! Covers T501: content length constraints, invalid characters, and recipient existence checks.
//! Requirement: T501 - Message Validation

use chat_backend::db;
use chat_backend::models::{Conversation, User};
use chat_backend::services::message_service::MessageService;
use crate::fixtures::setup_test_db;
use uuid::Uuid;

/// Test ID: T501-001
/// Given: Message content with valid and invalid characters
/// When: Message validation is performed
/// Then: Valid UTF-8 text should pass, control characters should be rejected
#[tokio::test]
async fn rejects_invalid_characters_in_content() {
    assert!(MessageService::validate_content("Hello, world!"));
    assert!(!MessageService::validate_content("bad\u{0007}chars")); // control character should be rejected
}

/// Test ID: T501-002
/// Given: A message is being sent to a non-existent recipient
/// When: Message validation checks recipient exists
/// Then: Validation should fail with "recipient not found" error
#[tokio::test]
    let pool = setup_test_db().await;
    let service = MessageService::new(pool.clone());

    let sender = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    db::queries::insert_user(&pool, &sender).await.unwrap();

    let missing_recipient_id = Uuid::new_v4().to_string();
    let mut ids = vec![sender.id.clone(), missing_recipient_id.clone()];
    ids.sort();
    let conversation = Conversation::new(ids[0].clone(), ids[1].clone());
    db::queries::insert_conversation(&pool, &conversation)
        .await
        .unwrap();

    let result = service
        .send_message(
            conversation.id.clone(),
            sender.id.clone(),
            missing_recipient_id.clone(),
            "hello there".to_string(),
        )
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_lowercase().contains("recipient"));
}

/// Test ID: T501-003
/// Given: A message with content exceeding 5000 characters
/// When: Message length validation is performed
/// Then: Validation should fail with "content must be between 1 and 5000 characters" error
#[tokio::test]
    let pool = setup_test_db().await;
    let service = MessageService::new(pool.clone());

    let sender = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    let recipient = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());
    db::queries::insert_user(&pool, &sender).await.unwrap();
    db::queries::insert_user(&pool, &recipient).await.unwrap();

    let mut ids = vec![sender.id.clone(), recipient.id.clone()];
    ids.sort();
    let conversation = Conversation::new(ids[0].clone(), ids[1].clone());
    db::queries::insert_conversation(&pool, &conversation)
        .await
        .unwrap();

    let long_content = "a".repeat(5001);
    let result = service
        .send_message(
            conversation.id.clone(),
            sender.id.clone(),
            recipient.id.clone(),
            long_content,
        )
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("1 and 5000"));
}
