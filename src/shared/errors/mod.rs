//! Error types for the chat application

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for chat operations
pub type Result<T> = std::result::Result<T, ChatError>;

/// Chat application errors
#[derive(Debug, Error, Serialize, Deserialize, Clone)]
pub enum ChatError {
    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Message error: {0}")]
    MessageError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimited(String),

    #[error("Internal server error")]
    InternalError,
}

impl ChatError {
    /// Get error code for HTTP/WebSocket responses
    pub fn code(&self) -> &str {
        match self {
            ChatError::AuthError(_) => "AUTH_ERROR",
            ChatError::MessageError(_) => "MESSAGE_ERROR",
            ChatError::DatabaseError(_) => "DATABASE_ERROR",
            ChatError::ValidationError(_) => "VALIDATION_ERROR",
            ChatError::NotFound(_) => "NOT_FOUND",
            ChatError::Conflict(_) => "CONFLICT",
            ChatError::RateLimited(_) => "RATE_LIMITED",
            ChatError::InternalError => "INTERNAL_ERROR",
        }
    }

    /// Get HTTP status code equivalent
    pub fn http_status(&self) -> u16 {
        match self {
            ChatError::AuthError(_) => 401,
            ChatError::MessageError(_) => 400,
            ChatError::DatabaseError(_) => 500,
            ChatError::ValidationError(_) => 400,
            ChatError::NotFound(_) => 404,
            ChatError::Conflict(_) => 409,
            ChatError::RateLimited(_) => 429,
            ChatError::InternalError => 500,
        }
    }
}

/// Standard error response format
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub timestamp: u64,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: None,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}
