// Integration tests for logout functionality

use chat_backend::db::queries;
use chat_backend::models::User;
use sqlx::SqlitePool;

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
async fn test_logout_endpoint() {
    // Note: Since JWTs are stateless and we don't have a blacklist yet,
    // the logout endpoint mainly disconnects WebSockets (which is hard to test in API test)
    // and logs the event.
    // We verify the endpoint returns 200 OK.

    // This test is minimal as connection manager state is internal to the running server.
    // Ideally we would mock ConnectionManager but it's part of the server binary.
    
    // We can test that the auth_log is created?
    // The handlers need `pool`.
    // But `logout_handler` is in `backend::handlers::auth`.
    // It takes `ConnectionManager`.
    
    // As an integration test for the handler logic:
    use chat_backend::handlers::auth::logout_handler;
    use chat_backend::handlers::websocket::ConnectionManager;
    use std::sync::Arc;
    
    let pool = setup_test_db().await;
    let cm = Arc::new(ConnectionManager::new());
    
    let result = logout_handler("user123".to_string(), cm, pool).await;
    
    assert!(result.is_ok());
    // In a real warp test we'd check status code, but here we just check Result
}
