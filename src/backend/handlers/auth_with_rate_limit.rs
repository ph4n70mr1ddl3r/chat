//! Enhanced authentication handlers with rate limiting
//!
//! Wraps the auth handlers with rate limiting and logging

use crate::db::queries::{self, AuthEventType};
use crate::handlers::auth::{AuthResponse, ErrorResponse, LoginRequest};
use crate::middleware::RateLimiter;
use crate::services::AuthService;
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::{info, warn};
use warp::{reply, Rejection, Reply};

/// Enhanced login handler with rate limiting
pub async fn login_with_rate_limit(
    req: LoginRequest,
    pool: SqlitePool,
    jwt_secret: String,
    rate_limiter: Arc<RateLimiter>,
    ip_address: String,
) -> Result<impl Reply, Rejection> {
    // Check rate limit
    if rate_limiter.is_rate_limited(&ip_address).await {
        let remaining = rate_limiter.get_remaining_attempts(&ip_address).await;
        warn!("Rate limit exceeded for IP: {}", ip_address);

        return Ok(reply::with_status(
            reply::json(&ErrorResponse {
                error: "RATE_LIMITED".to_string(),
                message: format!(
                    "Too many failed login attempts. Try again later. Remaining attempts: {}",
                    remaining
                ),
            }),
            warp::http::StatusCode::TOO_MANY_REQUESTS,
        ));
    }

    // Find user by username
    let user = match queries::find_user_by_username(&pool, &req.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("Login failed: user not found ({})", req.username);

            // Record failed attempt
            rate_limiter.record_attempt(&ip_address).await;
            let _ = queries::insert_auth_log(
                &pool,
                &ip_address,
                Some(&req.username),
                AuthEventType::LoginFailed,
                None,
                Some("User not found"),
            )
            .await;

            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "AUTH_ERROR".to_string(),
                    message: "Invalid credentials".to_string(),
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
        Err(e) => {
            warn!("Database error during login: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "DATABASE_ERROR".to_string(),
                    message: "Failed to authenticate".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Check if user is deleted
    if user.is_deleted() {
        warn!("Login failed: deleted account ({})", req.username);

        // Record failed attempt
        rate_limiter.record_attempt(&ip_address).await;
        let _ = queries::insert_auth_log(
            &pool,
            &ip_address,
            Some(&req.username),
            AuthEventType::LoginFailed,
            None,
            Some("Account deleted"),
        )
        .await;

        return Ok(reply::with_status(
            reply::json(&ErrorResponse {
                error: "ACCOUNT_DELETED".to_string(),
                message: "Account has been deleted".to_string(),
            }),
            warp::http::StatusCode::NOT_FOUND,
        ));
    }

    // Verify password
    match AuthService::verify_password(&req.password, &user.password_hash) {
        Ok(true) => {
            // Password is correct - proceed with token generation
        }
        Ok(false) => {
            warn!("Login failed: invalid password ({})", req.username);

            // Record failed attempt
            rate_limiter.record_attempt(&ip_address).await;
            let _ = queries::insert_auth_log(
                &pool,
                &ip_address,
                Some(&req.username),
                AuthEventType::LoginFailed,
                None,
                Some("Invalid password"),
            )
            .await;

            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "AUTH_ERROR".to_string(),
                    message: "Invalid credentials".to_string(),
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
        Err(e) => {
            warn!("Password verification error: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "AUTH_ERROR".to_string(),
                    message: "Authentication failed".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }

    // Generate token
    let auth_service = AuthService::new(jwt_secret);
    let (token, expires_at) = match auth_service.generate_token(user.id.clone()) {
        Ok((token, expires_at)) => (token, expires_at),
        Err(e) => {
            warn!("Failed to generate token: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "AUTH_ERROR".to_string(),
                    message: "Failed to generate authentication token".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Success! Reset rate limit and log success
    rate_limiter.reset(&ip_address).await;
    let _ = queries::insert_auth_log(
        &pool,
        &ip_address,
        Some(&req.username),
        AuthEventType::LoginSuccess,
        None,
        None,
    )
    .await;

    info!("User logged in: {}", req.username);

    Ok(reply::with_status(
        reply::json(&AuthResponse {
            user_id: user.id,
            username: user.username,
            token,
            expires_in: expires_at as u64,
        }),
        warp::http::StatusCode::OK,
    ))
}
