//! Integration tests for presence tracking and broadcasting.
//!
//! Covers T106 scenarios (online/offline broadcast, database updates).

use crate::db;
use crate::handlers::websocket::{ClientConnection, ConnectionManager};
use crate::models::{Conversation, User};
use crate::services::PresenceService;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::time::{timeout, Duration};

async fn setup_test_db() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    let schema_sql = include_str!("../db/migrations/001_initial_schema.sql");
    for statement in schema_sql.split(';').filter(|s| !s.trim().is_empty()) {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }

    pool
}

async fn create_users_and_conversation(pool: &SqlitePool) -> (User, User, Conversation) {
    let user1 = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    let user2 = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());
    db::queries::insert_user(pool, &user1).await.unwrap();
    db::queries::insert_user(pool, &user2).await.unwrap();

    let mut ids = vec![user1.id.clone(), user2.id.clone()];
    ids.sort();
    let conversation = Conversation::new(ids[0].clone(), ids[1].clone());
    db::queries::insert_conversation(pool, &conversation)
        .await
        .unwrap();

    (user1, user2, conversation)
}

#[tokio::test]
async fn broadcasts_presence_when_user_goes_online() {
    let pool = setup_test_db().await;
    let connection_manager = Arc::new(ConnectionManager::new());
    let presence_service = PresenceService::new(pool.clone(), connection_manager.clone());

    let (user1, user2, _conversation) = create_users_and_conversation(&pool).await;

    // Connect user2 (observer)
    let (tx, mut rx) = unbounded_channel();
    let connection = ClientConnection::new(user2.id.clone(), user2.username.clone());
    connection_manager.register(connection, tx).await;

    // Mark user1 online
    presence_service.mark_online(&user1.id).await.unwrap();

    // Verify broadcast received
    let msg = timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("broadcast timed out");
    
    assert!(msg.is_some());
    let msg = msg.unwrap();
    let text = msg.to_str().unwrap();
    assert!(text.contains("presence"));
    assert!(text.contains(&user1.id));
    assert!(text.contains("true")); // is_online: true

    // Verify DB update
    let user = db::queries::find_user_by_id(&pool, &user1.id).await.unwrap().unwrap();
    assert!(user.is_online);
}

#[tokio::test]
async fn broadcasts_presence_when_user_goes_offline() {
    let pool = setup_test_db().await;
    let connection_manager = Arc::new(ConnectionManager::new());
    let presence_service = PresenceService::new(pool.clone(), connection_manager.clone());

    let (user1, user2, _conversation) = create_users_and_conversation(&pool).await;

    // Connect user2 (observer)
    let (tx, mut rx) = unbounded_channel();
    let connection = ClientConnection::new(user2.id.clone(), user2.username.clone());
    connection_manager.register(connection, tx).await;

    // Mark user1 offline
    presence_service.mark_offline(&user1.id).await.unwrap();

    // Verify broadcast received
    let msg = timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("broadcast timed out");
    
    assert!(msg.is_some());
    let msg = msg.unwrap();
    let text = msg.to_str().unwrap();
    assert!(text.contains("presence"));
    assert!(text.contains(&user1.id));
    assert!(text.contains("false")); // is_online: false

    // Verify DB update
    let user = db::queries::find_user_by_id(&pool, &user1.id).await.unwrap().unwrap();
    assert!(!user.is_online);
}

#[tokio::test]
async fn presence_not_broadcast_to_unrelated_users() {
    let pool = setup_test_db().await;
    let connection_manager = Arc::new(ConnectionManager::new());
    let presence_service = PresenceService::new(pool.clone(), connection_manager.clone());

    let user1 = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    let user3 = User::new("charlie".to_string(), "hash3".to_string(), "salt3".to_string());
    db::queries::insert_user(&pool, &user1).await.unwrap();
    db::queries::insert_user(&pool, &user3).await.unwrap();

    // Connect user3 (unrelated)
    let (tx, mut rx) = unbounded_channel();
    let connection = ClientConnection::new(user3.id.clone(), user3.username.clone());
    connection_manager.register(connection, tx).await;

    // Mark user1 online
    presence_service.mark_online(&user1.id).await.unwrap();

    // Verify NO broadcast received
    let result = timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(result.is_err(), "should timeout (no message expected)");
}
