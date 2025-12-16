//! Presence latency SLA tests.
//!
//! Validates 1-second SLA for presence updates.

use crate::db;
use crate::handlers::websocket::{ClientConnection, ConnectionManager};
use crate::models::{Conversation, User};
use crate::services::PresenceService;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::time::{Instant, timeout, Duration};

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
async fn test_presence_latency_sla() {
    let pool = setup_test_db().await;
    let connection_manager = Arc::new(ConnectionManager::new());
    let presence_service = PresenceService::new(pool.clone(), connection_manager.clone());

    let (user1, user2, _conversation) = create_users_and_conversation(&pool).await;

    // Connect user2 (observer)
    let (tx, mut rx) = unbounded_channel();
    let connection = ClientConnection::new(user2.id.clone(), user2.username.clone());
    connection_manager.register(connection, tx).await;

    // Measure latency for 10 cycles
    let mut latencies = Vec::new();

    for _ in 0..10 {
        let start = Instant::now();
        presence_service.mark_online(&user1.id).await.unwrap();
        
        let msg = timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("broadcast timed out within 1s SLA");
        
        let elapsed = start.elapsed();
        latencies.push(elapsed);
        
        assert!(msg.is_some());
        
        // Reset for next cycle
        presence_service.mark_offline(&user1.id).await.unwrap();
        // Consume the offline message
        let _ = rx.recv().await; 
    }

    // Report stats
    let total: u128 = latencies.iter().map(|d| d.as_micros()).sum();
    let avg = total / latencies.len() as u128;
    println!("Average presence latency: {} µs", avg);

    // Assert SLA (under 1 second) - though timeout already asserts this, 
    // we can check average is well below 1s (e.g., < 100ms for in-memory)
    assert!(avg < 100_000, "Average latency should be under 100ms in integration test");
}
