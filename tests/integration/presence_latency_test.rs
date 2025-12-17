//! Presence latency SLA tests.
//!
//! Validates 1-second SLA for presence updates.
//! Requirement: T106 - Presence Latency SLA

use chat_backend::db;
use chat_backend::handlers::websocket::{ClientConnection, ConnectionManager};
use chat_backend::models::User;
use chat_backend::services::PresenceService;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::time::{Instant, timeout, Duration};
use crate::fixtures::{setup_test_db, create_users_and_conversation};

/// Test ID: T106-001
/// Given: A presence service with connected observers
/// When: Triggering 10 rapid online/offline status changes
/// Then: All broadcasts should arrive within 1 second SLA (< 100ms average in-memory)
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
