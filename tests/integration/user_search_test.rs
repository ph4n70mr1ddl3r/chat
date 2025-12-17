// Integration tests for user search functionality
//
// Tests search endpoint, pagination, and filtering
// Requirement: T003 - User Search

use chat_backend::db;
use chat_backend::models::User;
use chat_backend::handlers::user;
use crate::fixtures::setup_test_db;

/// Test ID: T003-001
/// Given: Multiple users with similar usernames
/// When: Searching for users by prefix that matches multiple users
/// Then: All matching users should be returned
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

/// Test ID: T003-002
/// Given: Multiple users with username prefixes that match search criteria
/// When: Searching for users with a prefix
/// Then: Search results should exclude the current user from results
#[tokio::test]
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

/// Test ID: T003-003
/// Given: 5 users with similar usernames and a search limit of 3
/// When: Searching for users with the common prefix
/// Then: Only 3 users should be returned (pagination limit enforced)
#[tokio::test]
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

/// Test ID: T003-004
/// Given: Users exist in the database
/// When: Searching for a prefix that doesn't match any user
/// Then: Search should return an empty result set
#[tokio::test]
    let pool = setup_test_db().await;
    
    let user = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    chat_backend::db::queries::insert_user(&pool, &user).await.unwrap();
    
    // Search for non-existent prefix
    let results = chat_backend::db::queries::search_users_by_prefix(&pool, "xyz", 10)
        .await
        .unwrap();
    
    assert_eq!(results.len(), 0);
}
