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
use thiserror::Error;
use warp::http::StatusCode;
use warp::reject::Reject;

use chat_shared::errors::ChatError;

#[derive(Error, Debug, Clone)]
#[error("{message}")]
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

impl From<ChatError> for ApiError {
    fn from(err: ChatError) -> Self {
        match err {
            ChatError::AuthError(msg) => Self::unauthorized(msg),
            ChatError::MessageError(msg) | ChatError::ValidationError(msg) => {
                Self::bad_request(msg)
            }
            ChatError::DatabaseError { .. } => Self::internal("An internal error occurred"),
            ChatError::InternalError { .. } => Self::internal("Internal server error"),
            ChatError::NotFound(msg) => Self::not_found(msg),
            ChatError::Conflict(msg) => Self::conflict(msg),
            ChatError::RateLimited(msg) => Self::too_many_requests(msg),
            ChatError::TokenExpired => Self::unauthorized("Token has expired"),
            ChatError::TokenInvalid(msg) => Self::unauthorized(msg),
            ChatError::Timeout => Self::new(StatusCode::REQUEST_TIMEOUT, "TIMEOUT", "Request timed out"),
            _ => Self::internal("An error occurred"),
        }
    }
}

impl Reject for ApiError {}

/// Standard API error payload returned to clients.
#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

/// Create a Warp rejection carrying an ApiError.
pub fn rejection(error: ApiError) -> warp::reject::Rejection {
    warp::reject::custom(error)
}

/// Convenience helper to build a JSON error reply without going through rejection.
pub fn error_reply(
    status: StatusCode,
    code: &'static str,
    message: impl Into<String>,
) -> impl warp::Reply {
    warp::reply::with_status(
        warp::reply::json(&ErrorBody {
            code: code.to_string(),
            message: message.into(),
            details: None,
        }),
        status,
    )
}
