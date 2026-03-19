//! Enhanced authentication handlers with rate limiting
//!
//! Wraps the auth handlers with rate limiting and logging

use crate::db::queries::{self, AuthEventType};
use crate::handlers::auth::{AuthResponse, LoginRequest, DUMMY_BCRYPT_HASH_FOR_TIMING};
use crate::handlers::ErrorBody;
use crate::middleware::RateLimiter;
use crate::services::{AuthService, CsrfService};
use crate::utils::sanitize_for_log;
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::{info, warn};
use warp::{reply, Rejection, Reply};

/// Enhanced login handler with rate limiting
///
/// # Errors
///
/// Returns a warp rejection if:
/// - Rate limit is exceeded
/// - Database operations fail
/// - Token generation fails
#[allow(clippy::too_many_lines)]
pub async fn login_with_rate_limit(
    req: LoginRequest,
    pool: SqlitePool,
    jwt_secret: String,
    rate_limiter: Arc<RateLimiter>,
    ip_address: String,
    csrf_service: CsrfService,
) -> Result<impl Reply, Rejection> {
    if let Err(err) = rate_limiter.check_and_record(&ip_address).await {
        warn!("Rate limit exceeded for IP: {}", ip_address);

        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "RATE_LIMITED".to_string(),
                message: format!(
                    "Too many failed login attempts. Try again in {} seconds",
                    err.retry_after_secs
                ),
                details: None,
            }),
            warp::http::StatusCode::TOO_MANY_REQUESTS,
        ));
    }

    // Find user by username - use timing-attack resistant pattern
    // Always verify password even when user doesn't exist to prevent timing-based enumeration
    let (user, hash_to_verify) = match queries::find_user_by_username(&pool, &req.username).await {
        Ok(Some(user)) if user.is_deleted() => {
            // Return the hash for timing consistency but no user
            (None, user.password_hash)
        }
        Ok(Some(user)) => {
            let cloned = user.clone();
            (Some(cloned), user.password_hash)
        }
        Ok(None) => {
            // Use dummy hash to ensure password verification runs in constant time
            (None, DUMMY_BCRYPT_HASH_FOR_TIMING.to_string())
        }
        Err(e) => {
            warn!("Database error during login: {}", e);

            let _ = queries::insert_auth_log(
                &pool,
                &ip_address,
                Some(&req.username),
                AuthEventType::LoginFailed,
                None,
                Some("Database error"),
            )
            .await;

            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "DATABASE_ERROR".to_string(),
                    message: "Failed to authenticate".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Always verify password to maintain constant time (prevents user enumeration via timing)
    let password_valid = match AuthService::verify_password(&req.password, &hash_to_verify) {
        Ok(valid) => valid,
        Err(e) => {
            warn!("Password verification error for '{}': {}", sanitize_for_log(&req.username), e);

            let _ = queries::insert_auth_log(
                &pool,
                &ip_address,
                Some(&req.username),
                AuthEventType::LoginFailed,
                None,
                Some("Password verification error"),
            )
            .await;

            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "AUTH_ERROR".to_string(),
                    message: "Invalid credentials".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    // Check both password validity AND user existence
    let Some(user) = user.filter(|_: &crate::models::User| password_valid) else {
        warn!("Login failed: invalid credentials ({})", sanitize_for_log(&req.username));

        let _ = queries::insert_auth_log(
            &pool,
            &ip_address,
            Some(&req.username),
            AuthEventType::LoginFailed,
            None,
            Some("Invalid credentials"),
        )
        .await;

        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "AUTH_ERROR".to_string(),
                message: "Invalid credentials".to_string(),
                details: None,
            }),
            warp::http::StatusCode::UNAUTHORIZED,
        ));
    };

    // Generate token
    let auth_service = AuthService::new(jwt_secret);
    let (token, expires_at) = match auth_service.generate_token(user.id.clone()) {
        Ok((token, expires_at)) => (token, expires_at),
        Err(e) => {
            warn!("Failed to generate token: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "AUTH_ERROR".to_string(),
                    message: "Failed to generate authentication token".to_string(),
                    details: None,
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

    let _csrf_token = match csrf_service.generate_token(&user.id) {
        Ok(token) => token,
        Err(e) => {
            warn!("Failed to generate CSRF token: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "AUTH_ERROR".to_string(),
                    message: "Failed to generate security token".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    info!("User logged in: {}", req.username);

    Ok(reply::with_status(
        reply::json(&AuthResponse {
            user_id: user.id,
            username: user.username,
            token,
            expires_in: expires_at,
        }),
        warp::http::StatusCode::OK,
    ))
}
