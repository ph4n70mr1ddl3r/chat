// Integration tests for conversation functionality
//
// Tests conversation creation, retrieval, and constraints

use chat_backend::db;
use chat_backend::models::{User, Conversation};
use chat_backend::services::ConversationService;
use sqlx::SqlitePool;

async fn setup_test_db() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    // Run migrations
    let schema_sql = include_str!("../../src/backend/db/migrations/001_initial_schema.sql");
    for statement in schema_sql.split(';').filter(|s| !s.trim().is_empty()) {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }

    pool
}

#[tokio::test]
async fn test_start_conversation_creates_new() {
    let pool = setup_test_db().await;
    let service = ConversationService::new(pool.clone());
    
    // Create test users
    let user1 = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    let user2 = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());
    
    chat_backend::db::queries::insert_user(&pool, &user1).await.unwrap();
    chat_backend::db::queries::insert_user(&pool, &user2).await.unwrap();
    
    // Create conversation
    let (conversation, was_created) = service
        .create_or_get_conversation(user1.id.clone(), user2.id.clone())
        .await
        .unwrap();
    
    assert!(was_created);
    assert!(!conversation.id.is_empty());
}

#[tokio::test]
async fn test_start_conversation_prevents_self_chat() {
    let pool = setup_test_db().await;
    let service = ConversationService::new(pool.clone());
    
    let user1 = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    chat_backend::db::queries::insert_user(&pool, &user1).await.unwrap();
    
    // Try to create conversation with self
    let result = service
        .create_or_get_conversation(user1.id.clone(), user1.id.clone())
        .await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("self"));
}

#[tokio::test]
async fn test_get_existing_conversation() {
    let pool = setup_test_db().await;
    let service = ConversationService::new(pool.clone());
    
    let user1 = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    let user2 = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());
    
    chat_backend::db::queries::insert_user(&pool, &user1).await.unwrap();
    chat_backend::db::queries::insert_user(&pool, &user2).await.unwrap();
    
    // Create first conversation
    let (conv1, created1) = service
        .create_or_get_conversation(user1.id.clone(), user2.id.clone())
        .await
        .unwrap();
    
    assert!(created1);
    
    // Get existing conversation
    let (conv2, created2) = service
        .create_or_get_conversation(user1.id.clone(), user2.id.clone())
        .await
        .unwrap();
    
    assert!(!created2);
    assert_eq!(conv1.id, conv2.id);
}

#[tokio::test]
async fn test_list_conversations() {
    let pool = setup_test_db().await;
    let service = ConversationService::new(pool.clone());
    
    let user1 = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    let user2 = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());
    let user3 = User::new("charlie".to_string(), "hash3".to_string(), "salt3".to_string());
    
    chat_backend::db::queries::insert_user(&pool, &user1).await.unwrap();
    chat_backend::db::queries::insert_user(&pool, &user2).await.unwrap();
    chat_backend::db::queries::insert_user(&pool, &user3).await.unwrap();
    
    // Create conversations
    service.create_or_get_conversation(user1.id.clone(), user2.id.clone()).await.unwrap();
    service.create_or_get_conversation(user1.id.clone(), user3.id.clone()).await.unwrap();
    
    // Get user1's conversations
    let conversations = service
        .get_user_conversations(&user1.id, 20, 0)
        .await
        .unwrap();
    
    assert_eq!(conversations.len(), 2);
}

#[tokio::test]
async fn test_conversation_pagination() {
    let pool = setup_test_db().await;
    let service = ConversationService::new(pool.clone());
    
    let user1 = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    chat_backend::db::queries::insert_user(&pool, &user1).await.unwrap();
    
    // Create 5 conversations
    for i in 0..5 {
        let user = User::new(
            format!("user{}", i),
            format!("hash{}", i),
            format!("salt{}", i),
        );
        chat_backend::db::queries::insert_user(&pool, &user).await.unwrap();
        service.create_or_get_conversation(user1.id.clone(), user.id).await.unwrap();
    }
    
    // Get first 3 conversations
    let page1 = service.get_user_conversations(&user1.id, 3, 0).await.unwrap();
    assert_eq!(page1.len(), 3);
    
    // Get next 2 conversations
    let page2 = service.get_user_conversations(&user1.id, 3, 3).await.unwrap();
    assert_eq!(page2.len(), 2);
}
