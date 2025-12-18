//! Integration tests for delivery status synchronization
//!
//! Tests delivery status sync behavior:
//! - AC1: Delivery queue persists across disconnections
//! - AC2: All queued updates sent on reconnection in order
//! - AC3: Backend idempotently handles duplicate status updates
//! - AC4: UI reflects delivery status changes immediately
//! - AC5: Client/backend state consistent after sync
//! - AC6: Sync and UI update within 500ms

use chat_backend::db;
use chat_backend::handlers::websocket::{ClientConnection, ConnectionManager};
use chat_backend::models::Message;
use chat_backend::services::message_service::MessageService;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::unbounded_channel;
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use crate::helpers::polling::poll_until;
use crate::fixtures::{setup_test_db, create_users_and_conversation};

/// Test ID: US-013-001
/// Given: Multiple messages sent during disconnection
/// When: Client reconnects and syncs delivery status
/// Then: All messages are synced in order with correct status
#[tokio::test]
async fn syncs_delivery_status_on_reconnection() {
    let pool = setup_test_db().await;
    let service = MessageService::new(pool.clone());
    let (sender, recipient, conversation) = create_users_and_conversation(&pool).await;

    // GIVEN: Send multiple messages
    let mut message_ids = Vec::new();
    for i in 0..3 {
        let msg = service
            .send_message(
                conversation.id.clone(),
                sender.id.clone(),
                recipient.id.clone(),
                format!("message {}", i),
            )
            .await
            .unwrap();
        message_ids.push(msg.id.clone());
    }

    // WHEN: Sync delivery status with mix of statuses
    let updates = vec![
        (message_ids[0].clone(), "sent".to_string()),
        (message_ids[1].clone(), "delivered".to_string()),
        (message_ids[2].clone(), "sent".to_string()),
    ];

    let synced = service.sync_delivery_status(&sender.id, updates).await.unwrap();

    // THEN: All messages synced
    assert_eq!(synced.len(), 3);
    assert_eq!(synced[0].status, "sent");
    assert_eq!(synced[1].status, "delivered");
    assert_eq!(synced[2].status, "sent");
}

/// Test ID: US-013-002
/// Given: Multiple delivery status updates in batch
/// When: All updates are processed
/// Then: Single sync command processes all updates atomically
#[tokio::test]
async fn processes_batch_delivery_updates() {
    let pool = setup_test_db().await;
    let service = MessageService::new(pool.clone());
    let (sender, recipient, conversation) = create_users_and_conversation(&pool).await;

    // Send messages
    let mut ids = Vec::new();
    for i in 0..5 {
        let msg = service
            .send_message(
                conversation.id.clone(),
                sender.id.clone(),
                recipient.id.clone(),
                format!("msg-{}", i),
            )
            .await
            .unwrap();
        ids.push(msg.id);
    }

    // Batch update all 5 messages
    let updates: Vec<_> = ids
        .iter()
        .map(|id| (id.clone(), "delivered".to_string()))
        .collect();

    let synced = service.sync_delivery_status(&sender.id, updates).await.unwrap();

    // Verify all synced
    assert_eq!(synced.len(), 5);
    for msg in synced {
        assert_eq!(msg.status, "delivered");
    }
}

/// Test ID: US-013-003
/// Given: Duplicate delivery updates sent to backend
/// When: Same message status is updated multiple times
/// Then: Backend handles idempotently (no duplicate state changes)
#[tokio::test]
async fn handles_idempotent_delivery_updates() {
    let pool = setup_test_db().await;
    let service = MessageService::new(pool.clone());
    let (sender, recipient, conversation) = create_users_and_conversation(&pool).await;

    let msg = service
        .send_message(
            conversation.id.clone(),
            sender.id.clone(),
            recipient.id.clone(),
            "Test message".to_string(),
        )
        .await
        .unwrap();

    // First sync: mark delivered
    let updates1 = vec![(msg.id.clone(), "delivered".to_string())];
    service
        .sync_delivery_status(&sender.id, updates1)
        .await
        .unwrap();

    // Second sync: same message, same status (duplicate)
    let updates2 = vec![(msg.id.clone(), "delivered".to_string())];
    let synced = service
        .sync_delivery_status(&sender.id, updates2)
        .await
        .unwrap();

    // Should return message without error (idempotent)
    assert_eq!(synced.len(), 1);
    assert_eq!(synced[0].status, "delivered");

    // Verify no duplicates in database
    let stored = db::queries::find_message_by_id(&pool, &msg.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.status, "delivered");
}

/// Test ID: US-013-004
/// Given: Status update attempts to downgrade (delivered → sent)
/// When: Sync receives downgrade status
/// Then: Backend rejects downgrade, maintains current status
#[tokio::test]
async fn prevents_status_downgrade() {
    let pool = setup_test_db().await;
    let service = MessageService::new(pool.clone());
    let (sender, recipient, conversation) = create_users_and_conversation(&pool).await;

    let msg = service
        .send_message(
            conversation.id.clone(),
            sender.id.clone(),
            recipient.id.clone(),
            "Test".to_string(),
        )
        .await
        .unwrap();

    // First: upgrade to delivered
    let update1 = vec![(msg.id.clone(), "delivered".to_string())];
    service
        .sync_delivery_status(&sender.id, update1)
        .await
        .unwrap();

    // Second: try to downgrade to sent (should fail idempotently)
    let update2 = vec![(msg.id.clone(), "sent".to_string())];
    let synced = service
        .sync_delivery_status(&sender.id, update2)
        .await
        .unwrap();

    // Status should remain delivered (not downgraded)
    assert_eq!(synced[0].status, "delivered");

    // Verify in database
    let stored = db::queries::find_message_by_id(&pool, &msg.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.status, "delivered");
}

/// Test ID: US-013-005
/// Given: Delivery status sync on reconnection
/// When: Sync processes updates
/// Then: UI update completes within 500ms target (AC6)
#[tokio::test]
async fn sync_completes_within_performance_target() {
    let pool = setup_test_db().await;
    let service = MessageService::new(pool.clone());
    let (sender, recipient, conversation) = create_users_and_conversation(&pool).await;

    // Create 50 messages to stress test
    let mut ids = Vec::new();
    for i in 0..50 {
        let msg = service
            .send_message(
                conversation.id.clone(),
                sender.id.clone(),
                recipient.id.clone(),
                format!("msg-{}", i),
            )
            .await
            .unwrap();
        ids.push(msg.id);
    }

    // Prepare batch update
    let updates: Vec<_> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let status = if i % 3 == 0 {
                "delivered"
            } else {
                "sent"
            };
            (id.clone(), status.to_string())
        })
        .collect();

    // Time the sync operation
    let start = Instant::now();
    let _synced = service
        .sync_delivery_status(&sender.id, updates)
        .await
        .unwrap();
    let elapsed = start.elapsed();

    // AC6: Must complete within 500ms
    assert!(
        elapsed < Duration::from_millis(500),
        "Sync took {}ms, exceeds 500ms target",
        elapsed.as_millis()
    );
}

/// Test ID: US-013-006
/// Given: Unauthorized user tries to sync message status
/// When: User is neither sender nor recipient
/// Then: Update is silently skipped (idempotent)
#[tokio::test]
async fn prevents_unauthorized_status_updates() {
    let pool = setup_test_db().await;
    let service = MessageService::new(pool.clone());
    let (sender, recipient, conversation) = create_users_and_conversation(&pool).await;

    let msg = service
        .send_message(
            conversation.id.clone(),
            sender.id.clone(),
            recipient.id.clone(),
            "Test".to_string(),
        )
        .await
        .unwrap();

    // Create unauthorized user
    let unauthorized_user = chat_backend::models::User::new(
        "hacker".to_string(),
        "hash".to_string(),
        "salt".to_string(),
    );
    db::queries::insert_user(&pool, &unauthorized_user)
        .await
        .unwrap();

    // Try to update status as unauthorized user
    let updates = vec![(msg.id.clone(), "read".to_string())];
    let synced = service
        .sync_delivery_status(&unauthorized_user.id, updates)
        .await
        .unwrap();

    // No messages should be synced (empty)
    assert_eq!(synced.len(), 0);

    // Original status unchanged
    let stored = db::queries::find_message_by_id(&pool, &msg.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.status, "pending");
}

/// Test ID: US-013-007
/// Given: Read status update
/// When: Message status changed to "read"
/// Then: read_at timestamp is set correctly
#[tokio::test]
async fn read_status_sets_timestamp() {
    let pool = setup_test_db().await;
    let service = MessageService::new(pool.clone());
    let (sender, recipient, conversation) = create_users_and_conversation(&pool).await;

    let msg = service
        .send_message(
            conversation.id.clone(),
            sender.id.clone(),
            recipient.id.clone(),
            "Test".to_string(),
        )
        .await
        .unwrap();

    let before = chrono::Utc::now().timestamp_millis();

    // Sync read status
    let updates = vec![(msg.id.clone(), "read".to_string())];
    service
        .sync_delivery_status(&recipient.id, updates)
        .await
        .unwrap();

    let after = chrono::Utc::now().timestamp_millis();

    // Verify read_at timestamp is set
    let stored = db::queries::find_message_by_id(&pool, &msg.id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(stored.status, "read");
    assert!(stored.read_at.is_some());
    
    let read_at = stored.read_at.unwrap();
    assert!(read_at >= before && read_at <= after);
}
