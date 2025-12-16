//! Integration tests for message delivery, offline queue, history, and duplicate prevention.
//!
//! Covers T096 scenarios (online delivery, offline queue + retry, message history pagination, duplicate prevention).

use chat_backend::db;
use chat_backend::handlers::websocket::{ClientConnection, ConnectionManager};
use chat_backend::models::{Conversation, User};
use chat_backend::services::message_queue::MessageQueueService;
use chat_backend::services::message_service::MessageService;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::time::{sleep, timeout, Duration};
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

async fn create_users_and_conversation(pool: &SqlitePool) -> (User, User, Conversation) {
    let sender = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    let recipient = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());
    db::queries::insert_user(pool, &sender).await.unwrap();
    db::queries::insert_user(pool, &recipient).await.unwrap();

    let mut ids = vec![sender.id.clone(), recipient.id.clone()];
    ids.sort();
    let conversation = Conversation::new(ids[0].clone(), ids[1].clone());
    db::queries::insert_conversation(pool, &conversation)
        .await
        .unwrap();

    (sender, recipient, conversation)
}

#[tokio::test]
async fn delivers_message_when_recipient_online() {
    let pool = setup_test_db().await;
    let connection_manager = Arc::new(ConnectionManager::new());
    let queue = MessageQueueService::new(pool.clone(), connection_manager.clone());
    queue.start().await;

    let (sender, recipient, conversation) = create_users_and_conversation(&pool).await;

    let (tx, mut rx) = unbounded_channel();
    let connection = ClientConnection::new(recipient.id.clone(), recipient.username.clone());
    connection_manager.register(connection, tx).await;

    let service = MessageService::new(pool.clone());
    let message = service
        .send_message(
            conversation.id.clone(),
            sender.id.clone(),
            recipient.id.clone(),
            "Hi Bob".to_string(),
        )
        .await
        .unwrap();

    queue
        .queue_message(message.id.clone(), recipient.id.clone())
        .await;

    let delivered = timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("delivery timed out");
    assert!(delivered.is_some(), "expected delivery to online user");

    let stored = db::queries::find_message_by_id(&pool, &message.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.status, "sent");

    queue.stop().await;
}

#[tokio::test]
async fn queues_and_delivers_when_recipient_comes_online() {
    let pool = setup_test_db().await;
    let connection_manager = Arc::new(ConnectionManager::new());
    let queue = MessageQueueService::new(pool.clone(), connection_manager.clone());
    queue.start().await;

    let (sender, recipient, conversation) = create_users_and_conversation(&pool).await;
    let (tx, mut rx) = unbounded_channel();
    let service = MessageService::new(pool.clone());

    let message = service
        .send_message(
            conversation.id.clone(),
            sender.id.clone(),
            recipient.id.clone(),
            "Queued hello".to_string(),
        )
        .await
        .unwrap();

    queue
        .queue_message(message.id.clone(), recipient.id.clone())
        .await;

    // Recipient offline initially; bring them online after short delay.
    sleep(Duration::from_millis(600)).await;
    let connection = ClientConnection::new(recipient.id.clone(), recipient.username.clone());
    connection_manager.register(connection, tx).await;

    let delivered = timeout(Duration::from_secs(4), rx.recv())
        .await
        .expect("delivery timed out");
    assert!(delivered.is_some(), "expected queued message delivery after reconnect");

    let stored = db::queries::find_message_by_id(&pool, &message.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.status, "sent");

    queue.stop().await;
}

#[tokio::test]
async fn loads_message_history_with_pagination() {
    let pool = setup_test_db().await;
    let service = MessageService::new(pool.clone());
    let (sender, recipient, conversation) = create_users_and_conversation(&pool).await;

    for i in 0..3 {
        service
            .send_message(
                conversation.id.clone(),
                if i % 2 == 0 { sender.id.clone() } else { recipient.id.clone() },
                if i % 2 == 0 { recipient.id.clone() } else { sender.id.clone() },
                format!("msg-{}", i),
            )
            .await
            .unwrap();
        sleep(Duration::from_millis(5)).await; // ensure ordering by timestamp
    }

    let first_page = service
        .get_conversation_messages(&conversation.id, &sender.id, 2, 0)
        .await
        .unwrap();
    assert_eq!(first_page.len(), 2);
    assert_eq!(first_page[0].content, "msg-2");
    assert_eq!(first_page[1].content, "msg-1");
}

#[tokio::test]
async fn prevents_duplicate_message_ids() {
    let pool = setup_test_db().await;
    let service = MessageService::new(pool.clone());
    let (sender, recipient, conversation) = create_users_and_conversation(&pool).await;

    let message_id = Uuid::new_v4().to_string();
    let (first, created_first) = service
        .send_message_with_id(
            message_id.clone(),
            conversation.id.clone(),
            sender.id.clone(),
            recipient.id.clone(),
            "First send".to_string(),
        )
        .await
        .unwrap();
    assert!(created_first);

    let (second, created_second) = service
        .send_message_with_id(
            message_id.clone(),
            conversation.id.clone(),
            sender.id.clone(),
            recipient.id.clone(),
            "Duplicate send".to_string(),
        )
        .await
        .unwrap();
    assert!(!created_second);
    assert_eq!(first.id, second.id);

    let messages = db::queries::get_messages_by_conversation(&pool, &conversation.id, 10, 0)
        .await
        .unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "First send");
}
