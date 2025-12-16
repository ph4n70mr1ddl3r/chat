//! Message handlers for WebSocket and HTTP endpoints

pub mod auth;
pub mod auth_with_rate_limit;
pub mod conversation;
pub mod dispatcher;
pub mod handshake;
pub mod heartbeat;
pub mod messages;
pub mod parser;
pub mod refresh;
pub mod router;
pub mod server;
pub mod user;
pub mod websocket;

use serde_json::Value;
use warp::http::StatusCode;
use warp::reject::Reject;

/// HTTP request/response types
#[derive(Debug, serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, serde::Serialize)]
pub struct LoginResponse {
    pub user_id: String,
    pub username: String,
    pub token: String,
    pub expires_in: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, serde::Serialize)]
pub struct SignupResponse {
    pub user_id: String,
    pub username: String,
    pub token: String,
    pub expires_in: u64,
}

/// Standard API error payload returned to clients.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

/// Unified application error with HTTP status mapping.
#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
    pub details: Option<Value>,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(
        status: StatusCode,
        code: &'static str,
        message: impl Into<String>,
        details: Value,
    ) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            details: Some(details),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "BAD_REQUEST", message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, "FORBIDDEN", message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "NOT_FOUND", message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, "CONFLICT", message)
    }

    pub fn too_many_requests(message: impl Into<String>) -> Self {
        Self::new(StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED", message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", message)
    }
}

impl Reject for ApiError {}

/// Create a Warp rejection carrying an ApiError.
pub fn rejection(error: ApiError) -> warp::reject::Rejection {
    warp::reject::custom(error)
}

/// Convenience helper to build a JSON error reply without going through rejection.
pub fn error_reply(status: StatusCode, code: &'static str, message: impl Into<String>) -> impl warp::Reply {
    warp::reply::with_status(
        warp::reply::json(&ErrorBody {
            code: code.to_string(),
            message: message.into(),
            details: None,
        }),
        status,
    )
}
