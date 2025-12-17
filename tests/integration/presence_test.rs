//! Integration tests for presence tracking and broadcasting.
//!
//! Covers T106 scenarios (online/offline broadcast, database updates).
//! Requirement: T105-T106 - Presence & Status

use chat_backend::db;
use chat_backend::handlers::websocket::{ClientConnection, ConnectionManager};
use chat_backend::models::User;
use chat_backend::services::PresenceService;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::time::{timeout, Duration};
use crate::fixtures::{setup_test_db, create_users_and_conversation};

/// Test ID: T105-001
/// Given: A user is online with an active WebSocket connection
/// When: Another user comes online
/// Then: An online presence broadcast should be sent to connected users
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

/// Test ID: T105-002
/// Given: A user is online with an active WebSocket connection
/// When: Another user goes offline
/// Then: An offline presence broadcast should be sent to connected users
#[tokio::test]
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

/// Test ID: T105-003
/// Given: Two unrelated users (no conversation between them)
/// When: One user's presence status changes
/// Then: The presence update should NOT be broadcast to unrelated users
#[tokio::test]
    let pool = setup_test_db().await;
    let connection_manager = Arc::new(ConnectionManager::new());
    let presence_service = PresenceService::new(pool.clone(), connection_manager.clone());

    let user1 = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    let user3 = User::new("charlie".to_string(), "hash3".to_string(), "salt3".to_string());
    db::queries::insert_user(pool.clone(), &user1).await.unwrap();
    db::queries::insert_user(pool.clone(), &user3).await.unwrap();

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
