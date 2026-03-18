// Integration tests for account deletion
// Requirement: T004 - Account Deletion

use chat_backend::db::queries;
use chat_backend::models::{User, Message, Conversation};
use chat_backend::handlers::user::{delete_account, DeleteAccountRequest};
use chat_backend::handlers::websocket::ConnectionManager;
use chat_backend::services::AuthService;
use crate::fixtures::setup_test_db;
use std::sync::Arc;

/// Test ID: T004-001
/// Given: A user account with correct password
/// When: The account deletion request is made with correct password
/// Then: The user account should be marked as deleted
#[tokio::test]
async fn test_account_deletion_success() {
    let pool = setup_test_db().await;
    
    // Create user
    let hash = AuthService::hash_password("Password123!").unwrap();
    let user = User::new("delete_me".to_string(), hash);
    queries::insert_user(&pool, &user).await.unwrap();
    
    // Create request
    let req = DeleteAccountRequest {
        password: "Password123!".to_string(),
    };
    
    let auth_service = Arc::new(AuthService::new("test-secret".to_string()));
    let connection_manager = Arc::new(ConnectionManager::new());
    let csrf_service = chat_backend::services::CsrfService::new("csrf-secret".to_string());
    let csrf_token = csrf_service.generate_token(&user.id).unwrap();
    
    // Call handler
    let result = delete_account(
        user.id.clone(),
        req,
        Some(csrf_token),
        csrf_service,
        pool.clone(),
        auth_service,
        connection_manager,
    ).await;
    assert!(result.is_ok());
    
    // Verify user is deleted
    let db_user = queries::find_user_by_id(&pool, &user.id).await.unwrap().unwrap();
    assert!(db_user.is_deleted());
}

/// Test ID: T004-002
/// Given: A user account with correct password set
/// When: The account deletion is attempted with an incorrect password
/// Then: The account should NOT be deleted
#[tokio::test]
async fn test_account_deletion_wrong_password() {
    let pool = setup_test_db().await;
    
    let hash = AuthService::hash_password("Password123!").unwrap();
    let user = User::new("safe_user".to_string(), hash);
    queries::insert_user(&pool, &user).await.unwrap();
    
    let req = DeleteAccountRequest {
        password: "WrongPassword123!".to_string(),
    };
    
    let auth_service = Arc::new(AuthService::new("test-secret".to_string()));
    let connection_manager = Arc::new(ConnectionManager::new());
    let csrf_service = chat_backend::services::CsrfService::new("csrf-secret".to_string());
    let csrf_token = csrf_service.generate_token(&user.id).unwrap();
    
    // Call handler
    let _result = delete_account(
        user.id.clone(),
        req,
        Some(csrf_token),
        csrf_service,
        pool.clone(),
        auth_service,
        connection_manager,
    ).await;
    
    let db_user = queries::find_user_by_id(&pool, &user.id).await.unwrap().unwrap();
    assert!(!db_user.is_deleted());
}

/// Test ID: T004-003
/// Given: A user who has sent messages in conversations
/// When: The user's account is deleted
/// Then: The user's messages should be anonymized (marked with is_anonymized flag)
#[tokio::test]
async fn test_account_deletion_anonymizes_messages() {
    let pool = setup_test_db().await;
    
    let hash = AuthService::hash_password("Password123!").unwrap();
    let user = User::new("sender".to_string(), hash);
    queries::insert_user(&pool, &user).await.unwrap();
    
    let user2 = User::new("recipient".to_string(), "hash".to_string());
    queries::insert_user(&pool, &user2).await.unwrap();
    
    let (user1_id, user2_id) = if user.id < user2.id {
        (user.id.clone(), user2.id.clone())
    } else {
        (user2.id.clone(), user.id.clone())
    };
    let conv = Conversation::new(user1_id, user2_id);
    queries::insert_conversation(&pool, &conv).await.unwrap();
    
    let msg = Message::new(conv.id.clone(), user.id.clone(), user2.id.clone(), "Secret message".to_string());
    queries::insert_message(&pool, &msg).await.unwrap();
    
    let auth_service = Arc::new(AuthService::new("test-secret".to_string()));
    let connection_manager = Arc::new(ConnectionManager::new());
    let csrf_service = chat_backend::services::CsrfService::new("csrf-secret".to_string());
    let csrf_token = csrf_service.generate_token(&user.id).unwrap();
    
    // Delete account
    let req = DeleteAccountRequest { password: "Password123!".to_string() };
    delete_account(
        user.id.clone(),
        req,
        Some(csrf_token),
        csrf_service,
        pool.clone(),
        auth_service,
        connection_manager,
    ).await.unwrap();
    
    // Check message
    let msgs = queries::get_messages_by_conversation(&pool, &conv.id, 10, 0).await.unwrap();
    assert_eq!(msgs.len(), 1);
    assert!(msgs[0].is_anonymized);
}
