//! Integration tests for user login endpoint

use warp::test::request;
use warp::Filter;

use crate::handlers::auth_with_rate_limit;
use crate::middleware::rate_limit::RateLimiter;
use serde_json;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;

/// Helper function to create a test database with a user
async fn setup_test_db() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
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
    
    pool
}

/// Test successful login
#[tokio::test]
async fn test_login_success() {
    let pool = setup_test_db().await;
    let jwt_secret = "test-secret".to_string();
    let rate_limiter = Arc::new(RateLimiter::new(5, 900)); // 5 attempts per 15 minutes
    
    // First create a user via signup
    let pool_clone = pool.clone();
    let jwt_secret_clone = jwt_secret.clone();
    let signup_route = warp::path!("auth" / "signup")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool_clone.clone()))
        .and(warp::any().map(move || jwt_secret_clone.clone()))
        .and_then(crate::handlers::auth::signup_handler);
    
    request()
        .method("POST")
        .path("/auth/signup")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "TestPass123"
        }))
        .reply(&signup_route)
        .await;
    
    // Now test login
    let pool_clone = pool.clone();
    let jwt_secret_clone = jwt_secret.clone();
    let rate_limiter_clone = rate_limiter.clone();
    
    let login_route = warp::path!("auth" / "login")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool_clone.clone()))
        .and(warp::any().map(move || jwt_secret_clone.clone()))
        .and(warp::any().map(move || rate_limiter_clone.clone()))
        .and(warp::any().map(|| "127.0.0.1".to_string()))
        .and_then(auth_with_rate_limit::login_with_rate_limit);
    
    let resp = request()
        .method("POST")
        .path("/auth/login")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "TestPass123"
        }))
        .reply(&login_route)
        .await;
    
    assert_eq!(resp.status(), 200, "Response: {:?}", resp.body());
    let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
    assert_eq!(body.get("username").unwrap(), "testuser");
    assert!(body.get("token").is_some());
    assert!(body.get("expires_in").is_some());
}

/// Test login with invalid password
#[tokio::test]
async fn test_login_invalid_password() {
    let pool = setup_test_db().await;
    let jwt_secret = "test-secret".to_string();
    let rate_limiter = Arc::new(RateLimiter::new(5, 900));
    
    // Create a user
    let pool_clone = pool.clone();
    let jwt_secret_clone = jwt_secret.clone();
    let signup_route = warp::path!("auth" / "signup")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool_clone.clone()))
        .and(warp::any().map(move || jwt_secret_clone.clone()))
        .and_then(crate::handlers::auth::signup_handler);
    
    request()
        .method("POST")
        .path("/auth/signup")
        .json(&serde_json::json!({
            "username": "alice",
            "password": "TestPass123"
        }))
        .reply(&signup_route)
        .await;
    
    // Attempt login with wrong password
    let pool_clone = pool.clone();
    let jwt_secret_clone = jwt_secret.clone();
    let rate_limiter_clone = rate_limiter.clone();
    
    let login_route = warp::path!("auth" / "login")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool_clone.clone()))
        .and(warp::any().map(move || jwt_secret_clone.clone()))
        .and(warp::any().map(move || rate_limiter_clone.clone()))
        .and(warp::any().map(|| "127.0.0.1".to_string()))
        .and_then(auth_with_rate_limit::login_with_rate_limit);
    
    let resp = request()
        .method("POST")
        .path("/auth/login")
        .json(&serde_json::json!({
            "username": "alice",
            "password": "WrongPassword123"
        }))
        .reply(&login_route)
        .await;
    
    assert_eq!(resp.status(), 401);
    let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
    assert_eq!(body.get("error").unwrap().as_str().unwrap(), "AUTH_ERROR");
}

/// Test login with non-existent user
#[tokio::test]
async fn test_login_user_not_found() {
    let pool = setup_test_db().await;
    let jwt_secret = "test-secret".to_string();
    let rate_limiter = Arc::new(RateLimiter::new(5, 900));
    
    let login_route = warp::path!("auth" / "login")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool.clone()))
        .and(warp::any().map(move || jwt_secret.clone()))
        .and(warp::any().map(move || rate_limiter.clone()))
        .and(warp::any().map(|| "127.0.0.1".to_string()))
        .and_then(auth_with_rate_limit::login_with_rate_limit);
    
    let resp = request()
        .method("POST")
        .path("/auth/login")
        .json(&serde_json::json!({
            "username": "nonexistent",
            "password": "TestPass123"
        }))
        .reply(&login_route)
        .await;
    
    assert_eq!(resp.status(), 401);
}

/// Test rate limiting after multiple failed login attempts
#[tokio::test]
async fn test_login_rate_limiting() {
    let pool = setup_test_db().await;
    let jwt_secret = "test-secret".to_string();
    let rate_limiter = Arc::new(RateLimiter::new(5, 900)); // 5 attempts per 15 minutes
    
    // Create a user
    let pool_clone = pool.clone();
    let jwt_secret_clone = jwt_secret.clone();
    let signup_route = warp::path!("auth" / "signup")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool_clone.clone()))
        .and(warp::any().map(move || jwt_secret_clone.clone()))
        .and_then(crate::handlers::auth::signup_handler);
    
    request()
        .method("POST")
        .path("/auth/signup")
        .json(&serde_json::json!({
            "username": "bob",
            "password": "TestPass123"
        }))
        .reply(&signup_route)
        .await;
    
    // Attempt login 5 times with wrong password
    for i in 1..=5 {
        let pool_clone = pool.clone();
        let jwt_secret_clone = jwt_secret.clone();
        let rate_limiter_clone = rate_limiter.clone();
        
        let login_route = warp::path!("auth" / "login")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || pool_clone.clone()))
            .and(warp::any().map(move || jwt_secret_clone.clone()))
            .and(warp::any().map(move || rate_limiter_clone.clone()))
            .and(warp::any().map(|| "127.0.0.1".to_string()))
            .and_then(auth_with_rate_limit::login_with_rate_limit);
        
        let resp = request()
            .method("POST")
            .path("/auth/login")
            .json(&serde_json::json!({
                "username": "bob",
                "password": "WrongPass456"
            }))
            .reply(&login_route)
            .await;
        
        if i < 5 {
            assert_eq!(resp.status(), 401, "Attempt {}", i);
        }
    }
    
    // 6th attempt should be rate limited
    let pool_clone = pool.clone();
    let jwt_secret_clone = jwt_secret.clone();
    let rate_limiter_clone = rate_limiter.clone();
    
    let login_route = warp::path!("auth" / "login")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool_clone.clone()))
        .and(warp::any().map(move || jwt_secret_clone.clone()))
        .and(warp::any().map(move || rate_limiter_clone.clone()))
        .and(warp::any().map(|| "127.0.0.1".to_string()))
        .and_then(auth_with_rate_limit::login_with_rate_limit);
    
    let resp = request()
        .method("POST")
        .path("/auth/login")
        .json(&serde_json::json!({
            "username": "bob",
            "password": "WrongPass456"
        }))
        .reply(&login_route)
        .await;
    
    assert_eq!(resp.status(), 429); // Too Many Requests
    let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
    assert_eq!(body.get("error").unwrap().as_str().unwrap(), "RATE_LIMITED");
}

/// Test login with deleted account
#[tokio::test]
async fn test_login_deleted_account() {
    let pool = setup_test_db().await;
    let jwt_secret = "test-secret".to_string();
    let rate_limiter = Arc::new(RateLimiter::new(5, 900));
    
    // Create a user
    let pool_clone = pool.clone();
    let jwt_secret_clone = jwt_secret.clone();
    let signup_route = warp::path!("auth" / "signup")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool_clone.clone()))
        .and(warp::any().map(move || jwt_secret_clone.clone()))
        .and_then(crate::handlers::auth::signup_handler);
    
    request()
        .method("POST")
        .path("/auth/signup")
        .json(&serde_json::json!({
            "username": "charlie",
            "password": "TestPass123"
        }))
        .reply(&signup_route)
        .await;
    
    // Soft delete the user (use Unix timestamp)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    sqlx::query("UPDATE users SET deleted_at = ? WHERE username = ?")
        .bind(now)
        .bind("charlie")
        .execute(&pool)
        .await
        .unwrap();
    
    // Attempt login with deleted account
    let pool_clone = pool.clone();
    let jwt_secret_clone = jwt_secret.clone();
    let rate_limiter_clone = rate_limiter.clone();
    
    let login_route = warp::path!("auth" / "login")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool_clone.clone()))
        .and(warp::any().map(move || jwt_secret_clone.clone()))
        .and(warp::any().map(move || rate_limiter_clone.clone()))
        .and(warp::any().map(|| "127.0.0.1".to_string()))
        .and_then(auth_with_rate_limit::login_with_rate_limit);
    
    let resp = request()
        .method("POST")
        .path("/auth/login")
        .json(&serde_json::json!({
            "username": "charlie",
            "password": "TestPass123"
        }))
        .reply(&login_route)
        .await;
    
    // Debug output
    if resp.status() != 404 {
        let body_str = std::str::from_utf8(resp.body()).unwrap();
        eprintln!("Unexpected status: {}, body: {}", resp.status(), body_str);
    }
    
    assert_eq!(resp.status(), 404);
    let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
    assert!(body.get("message").unwrap().as_str().unwrap().contains("deleted"));
}
