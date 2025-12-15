//! Integration tests for user signup endpoint

use warp::test::request;
use warp::Filter;

use crate::handlers::auth;
use serde_json;

/// Test successful user signup
#[tokio::test]
async fn test_signup_success() {
    // Create in-memory SQLite pool
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create test pool");
    
    // Run migrations
    let schema_sql = include_str!("../db/migrations/001_initial_schema.sql");
    for statement in schema_sql.split(';').filter(|s| !s.trim().is_empty()) {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .expect("Failed to run migration");
    }
    
    let jwt_secret = "test-secret".to_string();
    
    // Build the signup route
    let signup_route = warp::path!("auth" / "signup")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool.clone()))
        .and(warp::any().map(move || jwt_secret.clone()))
        .and_then(auth::signup_handler);
    
    // Make request
    let resp = request()
        .method("POST")
        .path("/auth/signup")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "TestPass123"
        }))
        .reply(&signup_route)
        .await;
    
    assert_eq!(resp.status(), 201, "Response: {:?}", resp.body());
    let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
    assert!(body.get("user_id").is_some());
    assert_eq!(body.get("username").unwrap(), "testuser");
    assert!(body.get("token").is_some());
    assert!(body.get("expires_in").is_some());
}

/// Test signup with duplicate username
#[tokio::test]
async fn test_signup_duplicate_username() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    
    let schema_sql = include_str!("../db/migrations/001_initial_schema.sql");
    for statement in schema_sql.split(';').filter(|s| !s.trim().is_empty()) {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    
    let jwt_secret = "test-secret".to_string();
    
    // First signup
    let signup_route = warp::path!("auth" / "signup")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool.clone()))
        .and(warp::any().map(move || jwt_secret.clone()))
        .and_then(auth::signup_handler);
    
    let resp = request()
        .method("POST")
        .path("/auth/signup")
        .json(&serde_json::json!({
            "username": "alice",
            "password": "TestPass123"
        }))
        .reply(&signup_route)
        .await;
    assert_eq!(resp.status(), 201);
    
    // Second signup with same username
    let resp2 = request()
        .method("POST")
        .path("/auth/signup")
        .json(&serde_json::json!({
            "username": "alice",
            "password": "AnotherPass456"
        }))
        .reply(&signup_route)
        .await;
    
    assert_eq!(resp2.status(), 409);
    let body: serde_json::Value = serde_json::from_slice(resp2.body()).unwrap();
    assert_eq!(body.get("error").unwrap().as_str().unwrap(), "CONFLICT");
    assert!(body.get("message").unwrap().as_str().unwrap().contains("Username already exists"));
}

/// Test signup with invalid password
#[tokio::test]
async fn test_signup_invalid_password() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    
    let schema_sql = include_str!("../db/migrations/001_initial_schema.sql");
    for statement in schema_sql.split(';').filter(|s| !s.trim().is_empty()) {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    
    let jwt_secret = "test-secret".to_string();
    
    let signup_route = warp::path!("auth" / "signup")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool.clone()))
        .and(warp::any().map(move || jwt_secret.clone()))
        .and_then(auth::signup_handler);
    
    // Password too short
    let resp = request()
        .method("POST")
        .path("/auth/signup")
        .json(&serde_json::json!({
            "username": "bob",
            "password": "short"
        }))
        .reply(&signup_route)
        .await;
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
    assert_eq!(body.get("error").unwrap().as_str().unwrap(), "VALIDATION_ERROR");
    
    // Missing uppercase
    let resp = request()
        .method("POST")
        .path("/auth/signup")
        .json(&serde_json::json!({
            "username": "charlie",
            "password": "lowercase123"
        }))
        .reply(&signup_route)
        .await;
    assert_eq!(resp.status(), 400);
    
    // Missing digit
    let resp = request()
        .method("POST")
        .path("/auth/signup")
        .json(&serde_json::json!({
            "username": "david",
            "password": "NoDigits"
        }))
        .reply(&signup_route)
        .await;
    assert_eq!(resp.status(), 400);
}

/// Test signup with invalid username
#[tokio::test]
async fn test_signup_invalid_username() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    
    let schema_sql = include_str!("../db/migrations/001_initial_schema.sql");
    for statement in schema_sql.split(';').filter(|s| !s.trim().is_empty()) {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    
    let jwt_secret = "test-secret".to_string();
    
    let signup_route = warp::path!("auth" / "signup")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool.clone()))
        .and(warp::any().map(move || jwt_secret.clone()))
        .and_then(auth::signup_handler);
    
    // Username too long
    let long_username = "a".repeat(51);
    let resp = request()
        .method("POST")
        .path("/auth/signup")
        .json(&serde_json::json!({
            "username": long_username,
            "password": "TestPass123"
        }))
        .reply(&signup_route)
        .await;
    assert_eq!(resp.status(), 400);
    
    // Invalid characters
    let resp = request()
        .method("POST")
        .path("/auth/signup")
        .json(&serde_json::json!({
            "username": "user-name",
            "password": "TestPass123"
        }))
        .reply(&signup_route)
        .await;
    assert_eq!(resp.status(), 400);
}
