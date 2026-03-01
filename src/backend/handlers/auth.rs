//! Authentication HTTP handlers
//!
//! Implements POST /auth/signup and POST /auth/login endpoints

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{info, warn};
use warp::{reply, Rejection, Reply};

use crate::db::queries;
use crate::handlers::{websocket::ConnectionManager, ErrorBody};
use crate::services::{AuthService, CsrfService};
use crate::validators;
use std::sync::Arc;

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

/// Handle POST /auth/logout
pub async fn logout_handler(
    user_id: String,
    csrf_token: Option<String>,
    connection_manager: Arc<ConnectionManager>,
    csrf_service: CsrfService,
    pool: SqlitePool,
    ip_address: Option<&str>,
) -> Result<impl Reply, Rejection> {
    info!("Logout request for user: {}", user_id);

    if let Some(token) = csrf_token {
        if !csrf_service.validate_token(&token, &user_id).await {
            warn!("Invalid CSRF token for logout request");
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "FORBIDDEN".to_string(),
                    message: "Invalid CSRF token".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::FORBIDDEN,
            ));
        }
        csrf_service.invalidate_token(&token).await;
    }

    if let Err(e) = queries::insert_auth_log(
        &pool,
        ip_address.unwrap_or("unknown"),
        None,
        queries::AuthEventType::Logout,
        None,
        Some(&format!("User {} logged out", user_id)),
    )
    .await
    {
        warn!("Failed to log logout event: {}", e);
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
        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "VALIDATION_ERROR".to_string(),
                message: e,
                details: None,
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    if let Err(e) = validators::validate_password(&req.password) {
        warn!("Invalid password: {}", e);
        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "VALIDATION_ERROR".to_string(),
                message: e,
                details: None,
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    match queries::find_user_by_username(&pool, &req.username).await {
        Ok(Some(_)) => {
            warn!("Username already exists: {}", req.username);
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "CONFLICT".to_string(),
                    message: "Username already exists".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::CONFLICT,
            ));
        }
        Err(e) => {
            warn!(
                "Database error during user lookup for '{}': {}",
                req.username, e
            );
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "DATABASE_ERROR".to_string(),
                    message: "Failed to check username availability".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
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
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "AUTH_ERROR".to_string(),
                    message: e,
                    details: None,
                }),
                warp::http::StatusCode::BAD_REQUEST,
            ));
        }
    };

    if let Err(e) = queries::insert_user(&pool, &user).await {
        warn!("Failed to save user '{}' to database: {}", user.username, e);
        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "DATABASE_ERROR".to_string(),
                message: "Failed to create account".to_string(),
                details: None,
            }),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

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

    let csrf_token = csrf_service.generate_token(&user.id).await;

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
) -> Result<impl Reply, Rejection> {
    if let Err(e) = validators::validate_username(&req.username) {
        warn!("Login failed: invalid username ({}) - {}", req.username, e);
        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "VALIDATION_ERROR".to_string(),
                message: e,
                details: None,
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    let auth_service = AuthService::new(jwt_secret.clone());

    let user = match queries::find_user_by_username(&pool, &req.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("Login failed: user not found ({})", req.username);
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "AUTH_ERROR".to_string(),
                    message: "Invalid credentials".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
        Err(e) => {
            warn!(
                "Database error during login lookup for '{}': {}",
                req.username, e
            );
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

    if user.is_deleted() {
        warn!("Login failed: deleted account ({})", req.username);
        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "AUTH_ERROR".to_string(),
                message: "Invalid credentials".to_string(),
                details: None,
            }),
            warp::http::StatusCode::UNAUTHORIZED,
        ));
    }

    match auth_service.verify_login(&req.username, &req.password, &user.password_hash) {
        Ok(true) => {}
        Ok(false) => {
            warn!("Login failed: invalid password ({})", req.username);
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "AUTH_ERROR".to_string(),
                    message: "Invalid credentials".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
        Err(e) => {
            warn!(
                "Password verification error for user '{}': {}",
                req.username, e
            );
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "AUTH_ERROR".to_string(),
                    message: "Authentication failed".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }

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

    let csrf_token = csrf_service.generate_token(&user.id).await;

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
