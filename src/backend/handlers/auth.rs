//! Authentication HTTP handlers
//!
//! Implements POST /auth/signup and POST /auth/login endpoints

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{info, warn};
use warp::{reply, Rejection, Reply};

use crate::db::queries;
use crate::services::AuthService;
use crate::validators;
use crate::handlers::websocket::ConnectionManager;
use std::sync::Arc;

/// Signup request payload
#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
}

/// Login request payload
#[derive(Debug, Deserialize)]
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
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// Unified HTTP response that can be either success or error
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

impl Reply for HttpResponse {
    fn into_response(self) -> warp::reply::Response {
        reply::with_status(
            self.body,
            warp::http::StatusCode::from_u16(self.status).unwrap(),
        )
        .into_response()
    }
}

/// Handle POST /auth/logout
pub async fn logout_handler(
    user_id: String,
    connection_manager: Arc<ConnectionManager>,
    pool: SqlitePool,
) -> Result<impl Reply, Rejection> {
    info!("Logout request for user: {}", user_id);

    // Log the event
    if let Err(e) = queries::insert_auth_log(
        &pool,
        "unknown", // IP address not available in this context easily without extraction
        None, // username not strictly needed if we have user_id but log takes username
        queries::AuthEventType::Logout,
        None,
        Some(&format!("User {} logged out", user_id)),
    ).await {
        warn!("Failed to log logout event: {}", e);
    }

    // Disconnect active WebSocket connections
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
) -> Result<impl Reply, Rejection> {
    // Validate username
    if let Err(e) = validators::validate_username(&req.username) {
        warn!("Invalid username: {}", e);
        return Ok(reply::with_status(
            reply::json(&ErrorResponse {
                error: "VALIDATION_ERROR".to_string(),
                message: e,
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Validate password
    if let Err(e) = validators::validate_password(&req.password) {
        warn!("Invalid password: {}", e);
        return Ok(reply::with_status(
            reply::json(&ErrorResponse {
                error: "VALIDATION_ERROR".to_string(),
                message: e,
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Check if username already exists
    match queries::find_user_by_username(&pool, &req.username).await {
        Ok(Some(_)) => {
            warn!("Username already exists: {}", req.username);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "CONFLICT".to_string(),
                    message: "Username already exists".to_string(),
                }),
                warp::http::StatusCode::CONFLICT,
            ));
        }
        Err(e) => {
            warn!("Database error during user lookup: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "DATABASE_ERROR".to_string(),
                    message: "Failed to check username availability".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
        Ok(None) => {
            // Username is available, continue
        }
    }

    // Create user
    let auth_service = AuthService::new(jwt_secret);
    let user = match auth_service
        .create_user(req.username.clone(), req.password)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            warn!("Failed to create user: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "AUTH_ERROR".to_string(),
                    message: e,
                }),
                warp::http::StatusCode::BAD_REQUEST,
            ));
        }
    };

    // Save user to database
    if let Err(e) = queries::insert_user(&pool, &user).await {
        warn!("Failed to save user to database: {}", e);
        return Ok(reply::with_status(
            reply::json(&ErrorResponse {
                error: "DATABASE_ERROR".to_string(),
                message: "Failed to create account".to_string(),
            }),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    // Generate JWT token
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

    info!("User signed up: {}", req.username);

    Ok(reply::with_status(
        reply::json(&AuthResponse {
            user_id: user.id,
            username: user.username,
            token,
            expires_in: expires_at as u64,
        }),
        warp::http::StatusCode::CREATED,
    ))
}

/// Handle POST /auth/login
pub async fn login_handler(
    req: LoginRequest,
    pool: SqlitePool,
    jwt_secret: String,
) -> Result<impl Reply, Rejection> {
    // Find user by username
    let user = match queries::find_user_by_username(&pool, &req.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("Login failed: user not found ({})", req.username);
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
            // Password is correct
        }
        Ok(false) => {
            warn!("Login failed: invalid password ({})", req.username);
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
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("user123"));
        assert!(json.contains("alice"));
    }
}
