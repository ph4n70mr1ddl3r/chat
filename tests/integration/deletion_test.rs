// Integration tests for account deletion

use chat_backend::db::queries;
use chat_backend::models::{User, Message, Conversation};
use chat_backend::handlers::user::{delete_account, DeleteAccountRequest};
use chat_backend::services::AuthService;
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
async fn test_account_deletion_success() {
    let pool = setup_test_db().await;
    
    // Create user
    let (hash, salt) = AuthService::hash_password("Password123").unwrap();
    let user = User::new("delete_me".to_string(), hash, salt);
    queries::insert_user(&pool, &user).await.unwrap();
    
    // Create request
    let req = DeleteAccountRequest {
        password: "Password123".to_string(),
    };
    
    // Call handler
    let result = delete_account(user.id.clone(), req, pool.clone()).await;
    assert!(result.is_ok());
    
    // Verify user is deleted
    let db_user = queries::find_user_by_id(&pool, &user.id).await.unwrap().unwrap();
    assert!(db_user.is_deleted());
}

#[tokio::test]
async fn test_account_deletion_wrong_password() {
    let pool = setup_test_db().await;
    
    let (hash, salt) = AuthService::hash_password("Password123").unwrap();
    let user = User::new("safe_user".to_string(), hash, salt);
    queries::insert_user(&pool, &user).await.unwrap();
    
    let req = DeleteAccountRequest {
        password: "WrongPassword123".to_string(),
    };
    
    // Call handler
    let result = delete_account(user.id.clone(), req, pool.clone()).await;
    // Handler returns Ok(Reply) even for errors (wrapped in JSON response with status code)
    // To verify failure, we check DB
    
    let db_user = queries::find_user_by_id(&pool, &user.id).await.unwrap().unwrap();
    assert!(!db_user.is_deleted());
}

#[tokio::test]
async fn test_account_deletion_anonymizes_messages() {
    let pool = setup_test_db().await;
    
    let (hash, salt) = AuthService::hash_password("Password123").unwrap();
    let user = User::new("sender".to_string(), hash, salt);
    queries::insert_user(&pool, &user).await.unwrap();
    
    let user2 = User::new("recipient".to_string(), "hash".to_string(), "salt".to_string());
    queries::insert_user(&pool, &user2).await.unwrap();
    
    let conv = Conversation::new(user.id.clone(), user2.id.clone());
    queries::insert_conversation(&pool, &conv).await.unwrap();
    
    let msg = Message::new(conv.id.clone(), user.id.clone(), user2.id.clone(), "Secret message".to_string());
    queries::insert_message(&pool, &msg).await.unwrap();
    
    // Delete account
    let req = DeleteAccountRequest { password: "Password123".to_string() };
    delete_account(user.id.clone(), req, pool.clone()).await.unwrap();
    
    // Check message
    let msgs = queries::get_messages_by_conversation(&pool, &conv.id, 10, 0).await.unwrap();
    assert_eq!(msgs.len(), 1);
    assert!(msgs[0].is_anonymized);
    // Note: Content is NOT cleared in DB per current `anonymize_user_messages` implementation (only `is_anonymized` flag set).
    // The UI/API layer is responsible for hiding content if `is_anonymized` is true.
}
