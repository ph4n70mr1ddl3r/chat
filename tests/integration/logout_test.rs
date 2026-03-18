// Integration tests for logout functionality
// Requirement: T005 - Logout

use chat_backend::db::queries;
use chat_backend::models::User;
use crate::fixtures::setup_test_db;

/// Test ID: T005-001
/// Given: A valid user ID and connection manager
/// When: The logout handler is called for that user
/// Then: The logout should complete successfully
#[tokio::test]
async fn test_logout_endpoint() {
    use chat_backend::handlers::auth::logout_handler;
    use chat_backend::handlers::websocket::ConnectionManager;
    use chat_backend::services::{AuthService, CsrfService};
    use std::sync::Arc;

    let pool = setup_test_db().await;
    let cm = Arc::new(ConnectionManager::new());
    let auth_service = Arc::new(AuthService::new("test-secret".to_string()));
    let csrf_service = CsrfService::new("csrf-secret");

    let (token, _) = auth_service.generate_token("user123".to_string()).unwrap();
    let claims = auth_service.verify_token(&token).await.unwrap();

    let ctx = chat_backend::handlers::auth::LogoutContext {
        csrf_token: Some("test-csrf".to_string()),
        auth_token: Some(token),
        ip_address: Some("127.0.0.1".to_string()),
    };

    let result = logout_handler(claims, ctx, cm, auth_service, csrf_service, pool).await;

    assert!(result.is_ok());
}
