//! Authentication HTTP handlers
//!
//! Implements POST /auth/signup and POST /auth/login endpoints

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{info, warn};
use warp::{reply, Rejection, Reply};

use crate::db::queries;
use crate::handlers::{websocket::ConnectionManager, ErrorBody};
use crate::models::User;
use crate::services::{AuthService, CsrfService, LoginAttemptService};
use crate::utils::sanitize_for_log;
use crate::validators;
use std::sync::Arc;

/// Pre-computed dummy bcrypt hash for timing-attack resistant signup.
/// This ensures password hashing always runs even when username exists.
const DUMMY_BCRYPT_HASH_FOR_SIGNUP: &str = "$2b$12$LQv8wJ1ZQ7H8G8bS5h8QeO0o1iHtKrN6CWmYrfVrY1BuBbbfvVVW9";

macro_rules! error_response {
    ($code:expr, $message:expr, $status:expr) => {
        reply::with_status(
            reply::json(&ErrorBody {
                code: $code.to_string(),
                message: $message.to_string(),
                details: None,
            }),
            $status,
        )
    };
}

/// Signup request payload
#[derive(Debug, Serialize, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
}

/// Login request payload
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Authentication response (signup and login)
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user_id: String,
    pub username: String,
    pub token: String,
    pub expires_in: u64,
    pub csrf_token: String,
}

/// Context for logout operation
pub struct LogoutContext {
    pub csrf_token: Option<String>,
    pub auth_token: Option<String>,
    pub ip_address: Option<String>,
}

/// Handle POST /auth/logout.
///
/// # Errors
///
/// Returns a warp rejection if the CSRF token is missing, invalid, or expired.
#[allow(clippy::too_many_lines)]
pub async fn logout_handler(
    user_id: String,
    ctx: LogoutContext,
    connection_manager: Arc<ConnectionManager>,
    auth_service: Arc<crate::services::AuthService>,
    csrf_service: CsrfService,
    pool: SqlitePool,
) -> Result<impl Reply, Rejection> {
    info!("Logout request for user: {user_id}");

    let Some(csrf_token) = ctx.csrf_token else {
        warn!("Missing CSRF token for logout request");
        return Ok(error_response!(
            "FORBIDDEN",
            "CSRF token required for logout",
            warp::http::StatusCode::FORBIDDEN
        ));
    };

    if let Err(e) = csrf_service.validate_token(&csrf_token, &user_id) {
        let error_msg = match e {
            crate::services::csrf::CsrfValidationError::Expired => "CSRF token expired",
            crate::services::csrf::CsrfValidationError::UserMismatch => "CSRF token user mismatch",
            crate::services::csrf::CsrfValidationError::InvalidToken => "Invalid CSRF token",
        };
        warn!("{error_msg} for logout request");
        return Ok(error_response!(
            "FORBIDDEN",
            error_msg,
            warp::http::StatusCode::FORBIDDEN
        ));
    }

    if let Err(e) = queries::insert_auth_log(
        &pool,
        ctx.ip_address.as_deref().unwrap_or("unknown"),
        None,
        queries::AuthEventType::Logout,
        None,
        Some(&format!("User {user_id} logged out")),
    )
    .await
    {
        warn!("Failed to log logout event: {e}");
    }

    if let Some(token) = ctx.auth_token {
        // Try to decode token and revoke by JTI for better efficiency
        // Fall back to token hash revocation if decoding fails
        match auth_service.decode_token_without_verification(&token) {
            Ok(claims) if !claims.jti.is_empty() => {
                auth_service.revoke_token_by_jti(&claims.jti, &user_id).await;
                info!("Token revoked by JTI for user: {user_id}");
            }
            _ => {
                auth_service.revoke_token(&token, &user_id).await;
                info!("Token revoked by hash for user: {user_id}");
            }
        }
    }

    connection_manager.disconnect_user(&user_id).await;

    Ok(reply::with_status(
        reply::json(&serde_json::json!({ "message": "Logged out successfully" })),
        warp::http::StatusCode::OK,
    ))
}

/// Handle POST /auth/signup.
///
/// # Errors
///
/// Returns a warp rejection if validation fails, username is taken, or database operation fails.
#[allow(clippy::too_many_lines)]
pub async fn signup_handler(
    req: SignupRequest,
    pool: SqlitePool,
    jwt_secret: String,
    csrf_service: CsrfService,
) -> Result<impl Reply, Rejection> {
    if let Err(e) = validators::validate_username(&req.username) {
        warn!("Invalid username: {e}");
        return Ok(error_response!(
            "VALIDATION_ERROR",
            e,
            warp::http::StatusCode::BAD_REQUEST
        ));
    }

    if let Err(e) = validators::validate_password(&req.password) {
        warn!("Invalid password: {e}");
        return Ok(error_response!(
            "VALIDATION_ERROR",
            e,
            warp::http::StatusCode::BAD_REQUEST
        ));
    }

    let username_taken = match queries::find_user_by_username(&pool, &req.username).await {
        Ok(Some(_)) => true,
        Err(e) => {
            warn!(
                "Database error during user lookup for '{}': {e}",
                sanitize_for_log(&req.username)
            );
            return Ok(error_response!(
                "INTERNAL_ERROR",
                "An error occurred while processing your request",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            ));
        }
        Ok(None) => false,
    };

    // Always perform password hashing to prevent timing attacks
    // This ensures consistent response time whether username exists or not
    let password_hash = match AuthService::hash_password(&req.password) {
        Ok(hash) => hash,
        Err(e) => {
            warn!("Password hashing failed for '{}': {e}", sanitize_for_log(&req.username));
            // If username was taken, return generic error; otherwise return validation error
            return Ok(error_response!(
                if username_taken { "INTERNAL_ERROR" } else { "VALIDATION_ERROR" },
                if username_taken { "An error occurred while processing your request" } else { &e },
                if username_taken { warp::http::StatusCode::INTERNAL_SERVER_ERROR } else { warp::http::StatusCode::BAD_REQUEST }
            ));
        }
    };

    // Now check if username was taken (after password hashing to prevent timing attack)
    if username_taken {
        warn!("Signup failed: username taken ({})", sanitize_for_log(&req.username));
        return Ok(error_response!(
            "CONFLICT",
            "Unable to create account",
            warp::http::StatusCode::CONFLICT
        ));
    }

    let user = User::new(req.username.clone(), password_hash);

    let user = match queries::insert_user(&pool, &user).await {
        Ok(user) => user,
        Err(e) => {
            warn!("Failed to save user '{}' to database: {e}", sanitize_for_log(&req.username));
            return Ok(error_response!(
                "INTERNAL_ERROR",
                "An error occurred while processing your request",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            ));
        }
    };

    let auth_service = AuthService::new(jwt_secret);
    let (token, expires_at) = match auth_service.generate_token(user.id.clone()) {
        Ok((token, expires_at)) => (token, expires_at),
        Err(e) => {
            warn!("Failed to generate token: {e}");
            return Ok(error_response!(
                "AUTH_ERROR",
                "Failed to generate authentication token",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            ));
        }
    };

    let csrf_token = match csrf_service.generate_token(&user.id) {
        Ok(token) => token,
        Err(e) => {
            warn!("Failed to generate CSRF token: {e}");
            return Ok(error_response!(
                "AUTH_ERROR",
                "Failed to generate security token",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            ));
        }
    };

    info!("User signed up: {}", sanitize_for_log(&req.username));

    Ok(reply::with_status(
        reply::json(&AuthResponse {
            user_id: user.id,
            username: user.username,
            token,
            expires_in: expires_at,
            csrf_token,
        }),
        warp::http::StatusCode::CREATED,
    ))
}

/// Handle POST /auth/login.
///
/// # Errors
///
/// Returns a warp rejection if validation fails, credentials are invalid,
/// account is locked, or database operation fails.
#[allow(clippy::too_many_lines)]
pub async fn login_handler(
    req: LoginRequest,
    pool: SqlitePool,
    jwt_secret: String,
    csrf_service: CsrfService,
    login_attempt_service: Arc<LoginAttemptService>,
) -> Result<impl Reply, Rejection> {
    if let Err(e) = validators::validate_username(&req.username) {
        warn!("Login failed: invalid username ({}) - {e}", sanitize_for_log(&req.username));
        return Ok(error_response!(
            "VALIDATION_ERROR",
            e,
            warp::http::StatusCode::BAD_REQUEST
        ));
    }

    if login_attempt_service.is_locked(&req.username).await {
        warn!(
            target: "auth",
            event = "auth.login.locked",
            "Login attempt on locked account"
        );
        return Ok(error_response!(
            "AUTH_ERROR",
            "Account temporarily locked due to too many failed attempts. Please try again later.",
            warp::http::StatusCode::TOO_MANY_REQUESTS
        ));
    }

    let auth_service = AuthService::new(jwt_secret);

    let (user, hash_to_verify) = match queries::find_user_by_username(&pool, &req.username).await {
        Ok(Some(user)) if user.is_deleted() => {
            (None, user.password_hash)
        }
        Ok(Some(user)) => {
            let cloned = user.clone();
            (Some(cloned), user.password_hash)
        }
        Ok(None) => {
            (None, DUMMY_BCRYPT_HASH_FOR_SIGNUP.to_string())
        }
        Err(e) => {
            warn!(
                "Database error during login lookup for '{}': {e}",
                sanitize_for_log(&req.username)
            );
            return Ok(error_response!(
                "INTERNAL_ERROR",
                "An error occurred while processing your request",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            ));
        }
    };

    // Always verify password to maintain constant time (prevents user enumeration via timing)
    // Properly handle bcrypt errors instead of silently converting to "wrong password"
    let password_valid = match AuthService::verify_password(&req.password, &hash_to_verify) {
        Ok(valid) => valid,
        Err(e) => {
            warn!("Password verification error for '{}': {e}", sanitize_for_log(&req.username));
            login_attempt_service.record_failed_attempt(&req.username).await;
            return Ok(error_response!(
                "AUTH_ERROR",
                "Invalid credentials",
                warp::http::StatusCode::UNAUTHORIZED
            ));
        }
    };

    // Check both password validity AND user existence
    let Some(user) = user.filter(|_: &User| password_valid) else {
        warn!("Login failed: invalid credentials");
        login_attempt_service.record_failed_attempt(&req.username).await;
        return Ok(error_response!(
            "AUTH_ERROR",
            "Invalid credentials",
            warp::http::StatusCode::UNAUTHORIZED
        ));
    };

    let (token, expires_at) = match auth_service.generate_token(user.id.clone()) {
        Ok((token, expires_at)) => (token, expires_at),
        Err(e) => {
            warn!("Failed to generate token: {e}");
            return Ok(error_response!(
                "AUTH_ERROR",
                "Failed to generate authentication token",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            ));
        }
    };

    let csrf_token = match csrf_service.generate_token(&user.id) {
        Ok(token) => token,
        Err(e) => {
            warn!("Failed to generate CSRF token: {e}");
            return Ok(error_response!(
                "AUTH_ERROR",
                "Failed to generate security token",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            ));
        }
    };

    login_attempt_service.clear_attempts(&req.username).await;
    info!("User logged in: {}", sanitize_for_log(&req.username));

    Ok(reply::with_status(
        reply::json(&AuthResponse {
            user_id: user.id,
            username: user.username,
            token,
            expires_in: expires_at,
            csrf_token,
        }),
        warp::http::StatusCode::OK,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_response_serialization() {
        let response = AuthResponse {
            user_id: "user123".to_string(),
            username: "alice".to_string(),
            token: "eyJhbGc...".to_string(),
            expires_in: 3600,
            csrf_token: "csrf-token-123".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("user123"));
        assert!(json.contains("alice"));
        assert!(json.contains("csrf-token-123"));
    }
}
