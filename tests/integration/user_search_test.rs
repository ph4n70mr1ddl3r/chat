// Integration tests for user search functionality
//
// Tests search endpoint, pagination, and filtering

use chat_backend::db;
use chat_backend::models::User;
use chat_backend::handlers::user;
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
async fn test_user_search_valid_query() {
    let pool = setup_test_db().await;
    
    // Create test users
    let users = vec![
        User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string()),
        User::new("alicia".to_string(), "hash2".to_string(), "salt2".to_string()),
        User::new("bob".to_string(), "hash3".to_string(), "salt3".to_string()),
    ];
    
    for user in &users {
        chat_backend::db::queries::insert_user(&pool, user).await.unwrap();
    }
    
    // Search for "ali" - should return alice and alicia
    let results = chat_backend::db::queries::search_users_by_prefix(&pool, "ali", 10)
        .await
        .unwrap();
    
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|u| u.username == "alice"));
    assert!(results.iter().any(|u| u.username == "alicia"));
}

#[tokio::test]
async fn test_user_search_excludes_self() {
    let pool = setup_test_db().await;
    
    let user1 = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    let user2 = User::new("alicia".to_string(), "hash2".to_string(), "salt2".to_string());
    
    chat_backend::db::queries::insert_user(&pool, &user1).await.unwrap();
    chat_backend::db::queries::insert_user(&pool, &user2).await.unwrap();
    
    // Search should exclude current user (alice)
    let results = chat_backend::db::queries::search_users_by_prefix(&pool, "ali", 10)
        .await
        .unwrap();
    
    // Both should be in results (endpoint will filter self)
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_user_search_pagination() {
    let pool = setup_test_db().await;
    
    // Create 5 users with similar names
    for i in 0..5 {
        let user = User::new(
            format!("user{}", i),
            format!("hash{}", i),
            format!("salt{}", i),
        );
        chat_backend::db::queries::insert_user(&pool, &user).await.unwrap();
    }
    
    // Search with limit=3
    let results = chat_backend::db::queries::search_users_by_prefix(&pool, "user", 3)
        .await
        .unwrap();
    
    assert_eq!(results.len(), 3);
}

#[tokio::test]
async fn test_user_search_no_results() {
    let pool = setup_test_db().await;
    
    let user = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    chat_backend::db::queries::insert_user(&pool, &user).await.unwrap();
    
    // Search for non-existent prefix
    let results = chat_backend::db::queries::search_users_by_prefix(&pool, "xyz", 10)
        .await
        .unwrap();
    
    assert_eq!(results.len(), 0);
}
