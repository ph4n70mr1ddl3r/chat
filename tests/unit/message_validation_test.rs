//! Unit tests for message validation rules (content length, UTF-8 validity, recipient authorization)
//!
//! Covers T097: content length constraints, invalid characters, and recipient existence checks.

use chat_backend::db;
use chat_backend::models::{Conversation, User};
use chat_backend::services::message_service::MessageService;
use sqlx::SqlitePool;
use uuid::Uuid;

async fn setup_test_db() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    let schema_sql = include_str!("../../src/backend/db/migrations/001_initial_schema.sql");
    for statement in schema_sql.split(';').filter(|s| !s.trim().is_empty()) {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }

    pool
}

#[tokio::test]
async fn rejects_invalid_characters_in_content() {
    assert!(MessageService::validate_content("Hello, world!"));
    assert!(!MessageService::validate_content("bad\u{0007}chars")); // control character should be rejected
}

#[tokio::test]
async fn rejects_missing_recipient() {
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

#[tokio::test]
async fn rejects_message_too_long() {
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
