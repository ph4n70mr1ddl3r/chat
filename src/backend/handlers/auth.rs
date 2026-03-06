//! Authentication HTTP handlers
//!
//! Implements POST /auth/signup and POST /auth/login endpoints

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{info, warn};
use warp::{reply, Rejection, Reply};

use crate::db::queries;
use crate::handlers::{websocket::ConnectionManager, ErrorBody};
use crate::services::{AuthService, CsrfService, LoginAttemptService};
use crate::validators;
use std::sync::Arc;

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

/// Handle POST /auth/logout
pub async fn logout_handler(
    user_id: String,
    ctx: LogoutContext,
    connection_manager: Arc<ConnectionManager>,
    auth_service: Arc<crate::services::AuthService>,
    csrf_service: CsrfService,
    pool: SqlitePool,
) -> Result<impl Reply, Rejection> {
    info!("Logout request for user: {}", user_id);

    let csrf_token = match ctx.csrf_token {
        Some(t) => t,
        None => {
            warn!("Missing CSRF token for logout request");
            return Ok(error_response!(
                "FORBIDDEN",
                "CSRF token required for logout",
                warp::http::StatusCode::FORBIDDEN
            ));
        }
    };

    if let Err(e) = csrf_service.validate_token(&csrf_token, &user_id) {
        let error_msg = match e {
            crate::services::csrf::CsrfValidationError::Expired => "CSRF token expired",
            crate::services::csrf::CsrfValidationError::UserMismatch => "CSRF token user mismatch",
            crate::services::csrf::CsrfValidationError::InvalidToken => "Invalid CSRF token",
        };
        warn!("{} for logout request", error_msg);
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
        Some(&format!("User {} logged out", user_id)),
    )
    .await
    {
        warn!("Failed to log logout event: {}", e);
    }

    if let Some(token) = ctx.auth_token {
        auth_service.revoke_token(&token).await;
        info!("Token revoked for user: {}", user_id);
    }

    connection_manager.disconnect_user(&user_id).await;

    Ok(reply::with_status(
        reply::json(&serde_json::json!({ "message": "Logged out successfully" })),
        warp::http::StatusCode::OK,
    ))
}

/// Handle POST /auth/signup
pub async fn signup_handler(
    req: SignupRequest,
    pool: SqlitePool,
    jwt_secret: String,
    csrf_service: CsrfService,
) -> Result<impl Reply, Rejection> {
    if let Err(e) = validators::validate_username(&req.username) {
        warn!("Invalid username: {}", e);
        return Ok(error_response!(
            "VALIDATION_ERROR",
            e,
            warp::http::StatusCode::BAD_REQUEST
        ));
    }

    if let Err(e) = validators::validate_password(&req.password) {
        warn!("Invalid password: {}", e);
        return Ok(error_response!(
            "VALIDATION_ERROR",
            e,
            warp::http::StatusCode::BAD_REQUEST
        ));
    }

    match queries::find_user_by_username(&pool, &req.username).await {
        Ok(Some(_)) => {
            warn!("Username already exists: {}", req.username);
            return Ok(error_response!(
                "CONFLICT",
                "Username already exists",
                warp::http::StatusCode::CONFLICT
            ));
        }
        Err(e) => {
            warn!(
                "Database error during user lookup for '{}': {}",
                req.username, e
            );
            return Ok(error_response!(
                "INTERNAL_ERROR",
                "An error occurred while processing your request",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            ));
        }
        Ok(None) => {}
    }

    let auth_service = AuthService::new(jwt_secret);
    let user = match auth_service
        .create_user(req.username.clone(), req.password)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            warn!("Failed to create user '{}': {}", req.username, e);
            return Ok(error_response!("AUTH_ERROR", e, warp::http::StatusCode::BAD_REQUEST));
        }
    };

    if let Err(e) = queries::insert_user(&pool, &user).await {
        warn!("Failed to save user '{}' to database: {}", user.username, e);
        return Ok(error_response!(
            "INTERNAL_ERROR",
            "An error occurred while processing your request",
            warp::http::StatusCode::INTERNAL_SERVER_ERROR
        ));
    }

    let (token, expires_at) = match auth_service.generate_token(user.id.clone()) {
        Ok((token, expires_at)) => (token, expires_at),
        Err(e) => {
            warn!("Failed to generate token: {}", e);
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
            warn!("Failed to generate CSRF token: {}", e);
            return Ok(error_response!(
                "AUTH_ERROR",
                "Failed to generate security token",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            ));
        }
    };

    info!("User signed up: {}", req.username);

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

/// Handle POST /auth/login
pub async fn login_handler(
    req: LoginRequest,
    pool: SqlitePool,
    jwt_secret: String,
    csrf_service: CsrfService,
    login_attempt_service: Arc<LoginAttemptService>,
) -> Result<impl Reply, Rejection> {
    if let Err(e) = validators::validate_username(&req.username) {
        warn!("Login failed: invalid username ({}) - {}", req.username, e);
        return Ok(error_response!(
            "VALIDATION_ERROR",
            e,
            warp::http::StatusCode::BAD_REQUEST
        ));
    }

    if login_attempt_service.is_locked(&req.username).await {
        warn!("Login attempt on locked account ({})", &req.username[..8.min(req.username.len())]);
        return Ok(error_response!(
            "AUTH_ERROR",
            "Account temporarily locked due to too many failed attempts. Please try again later.",
            warp::http::StatusCode::TOO_MANY_REQUESTS
        ));
    }

    let auth_service = AuthService::new(jwt_secret.clone());

    let user = match queries::find_user_by_username(&pool, &req.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("Login failed: user not found ({})", req.username);
            login_attempt_service.record_failed_attempt(&req.username).await;
            return Ok(error_response!(
                "AUTH_ERROR",
                "Invalid credentials",
                warp::http::StatusCode::UNAUTHORIZED
            ));
        }
        Err(e) => {
            warn!(
                "Database error during login lookup for '{}': {}",
                req.username, e
            );
            return Ok(error_response!(
                "INTERNAL_ERROR",
                "An error occurred while processing your request",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            ));
        }
    };

    if user.is_deleted() {
        warn!("Login failed: deleted account ({})", req.username);
        login_attempt_service.record_failed_attempt(&req.username).await;
        return Ok(error_response!(
            "AUTH_ERROR",
            "Invalid credentials",
            warp::http::StatusCode::UNAUTHORIZED
        ));
    }

    match auth_service.verify_login(&req.username, &req.password, &user.password_hash) {
        Ok(true) => {
            login_attempt_service.clear_attempts(&req.username).await;
        }
        Ok(false) => {
            warn!("Login failed: invalid password ({})", req.username);
            login_attempt_service.record_failed_attempt(&req.username).await;
            return Ok(error_response!(
                "AUTH_ERROR",
                "Invalid credentials",
                warp::http::StatusCode::UNAUTHORIZED
            ));
        }
        Err(e) => {
            warn!(
                "Password verification error for user '{}': {}",
                req.username, e
            );
            return Ok(error_response!(
                "AUTH_ERROR",
                "Authentication failed",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            ));
        }
    }

    let (token, expires_at) = match auth_service.generate_token(user.id.clone()) {
        Ok((token, expires_at)) => (token, expires_at),
        Err(e) => {
            warn!("Failed to generate token: {}", e);
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
            warn!("Failed to generate CSRF token: {}", e);
            return Ok(error_response!(
                "AUTH_ERROR",
                "Failed to generate security token",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
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
