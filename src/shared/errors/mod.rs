//! Error types for the chat application

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for chat operations
pub type Result<T> = std::result::Result<T, ChatError>;

/// Chat application errors
#[derive(Debug, Error, Clone)]
pub enum ChatError {
    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Message error: {0}")]
    MessageError(String),

    #[error("Database error")]
    DatabaseError {
        #[source]
        source: DatabaseErrorSource,
    },

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimited(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token: {0}")]
    TokenInvalid(String),

    #[error("Connection timeout")]
    Timeout,

    #[error("Internal server error")]
    InternalError { internal_message: String },
}

/// Source of database error (not serialized to clients)
#[derive(Debug, Clone)]
pub struct DatabaseErrorSource(pub String);

impl std::fmt::Display for DatabaseErrorSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DatabaseErrorSource {}

impl ChatError {
    /// Create a database error with internal context
    pub fn database(msg: impl Into<String>) -> Self {
        ChatError::DatabaseError {
            source: DatabaseErrorSource(msg.into()),
        }
    }

    /// Create an internal error with context
    pub fn internal(msg: impl Into<String>) -> Self {
        ChatError::InternalError {
            internal_message: msg.into(),
        }
    }

    /// Get error code for HTTP/WebSocket responses
    #[must_use]
    pub fn code(&self) -> &str {
        match self {
            ChatError::AuthError(_) => "AUTH_ERROR",
            ChatError::MessageError(_) => "MESSAGE_ERROR",
            ChatError::DatabaseError { .. } => "INTERNAL_ERROR",
            ChatError::ValidationError(_) => "VALIDATION_ERROR",
            ChatError::NotFound(_) => "NOT_FOUND",
            ChatError::Conflict(_) => "CONFLICT",
            ChatError::RateLimited(_) => "RATE_LIMITED",
            ChatError::TokenExpired => "TOKEN_EXPIRED",
            ChatError::TokenInvalid(_) => "TOKEN_INVALID",
            ChatError::Timeout => "TIMEOUT",
            ChatError::InternalError { .. } => "INTERNAL_ERROR",
        }
    }

    /// Get HTTP status code equivalent
    #[must_use]
    pub fn http_status(&self) -> u16 {
        match self {
            ChatError::AuthError(_) => 401,
            ChatError::TokenExpired | ChatError::TokenInvalid(_) => 401,
            ChatError::Timeout => 408,
            ChatError::MessageError(_) | ChatError::ValidationError(_) => 400,
            ChatError::DatabaseError { .. } | ChatError::InternalError { .. } => 500,
            ChatError::NotFound(_) => 404,
            ChatError::Conflict(_) => 409,
            ChatError::RateLimited(_) => 429,
        }
    }

    /// Get a client-safe message (hides internal details)
    #[must_use]
    pub fn client_message(&self) -> String {
        match self {
            ChatError::DatabaseError { .. } => "An internal error occurred".to_string(),
            ChatError::InternalError { .. } => "An internal error occurred".to_string(),
            ChatError::AuthError(msg) => msg.clone(),
            ChatError::MessageError(msg) => msg.clone(),
            ChatError::ValidationError(msg) => msg.clone(),
            ChatError::NotFound(msg) => msg.clone(),
            ChatError::Conflict(msg) => msg.clone(),
            ChatError::RateLimited(msg) => msg.clone(),
            ChatError::TokenExpired => "Token has expired".to_string(),
            ChatError::TokenInvalid(msg) => format!("Invalid token: {}", msg),
            ChatError::Timeout => "Request timed out".to_string(),
        }
    }
}

impl Serialize for ChatError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ChatError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.client_message())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ChatError {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;
        use std::marker::PhantomData;

        struct ChatErrorVisitor(PhantomData<ChatError>);

        impl<'de> Visitor<'de> for ChatErrorVisitor {
            type Value = ChatError;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a ChatError object with code and message fields")
            }

            fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut code: Option<String> = None;
                let mut message: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "code" => code = Some(map.next_value()?),
                        "message" => message = Some(map.next_value()?),
                        _ => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                let code = code.ok_or_else(|| de::Error::missing_field("code"))?;
                let message = message.ok_or_else(|| de::Error::missing_field("message"))?;

                match code.as_str() {
                    "AUTH_ERROR" => Ok(ChatError::AuthError(message)),
                    "MESSAGE_ERROR" => Ok(ChatError::MessageError(message)),
                    "VALIDATION_ERROR" => Ok(ChatError::ValidationError(message)),
                    "NOT_FOUND" => Ok(ChatError::NotFound(message)),
                    "CONFLICT" => Ok(ChatError::Conflict(message)),
                    "RATE_LIMITED" => Ok(ChatError::RateLimited(message)),
                    "TOKEN_EXPIRED" => Ok(ChatError::TokenExpired),
                    "TOKEN_INVALID" => Ok(ChatError::TokenInvalid(message)),
                    "TIMEOUT" => Ok(ChatError::Timeout),
                    _ => Ok(ChatError::internal(message)),
                }
            }
        }

        deserializer.deserialize_struct(
            "ChatError",
            &["code", "message"],
            ChatErrorVisitor(PhantomData),
        )
    }
}

/// Standard error response format
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub timestamp: i64,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    /// Create with explicit timestamp (for testing)
    pub fn with_timestamp(
        error: impl Into<String>,
        message: impl Into<String>,
        timestamp: i64,
    ) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: None,
            timestamp,
        }
    }

    #[must_use]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Create from ChatError
    pub fn from_error(err: &ChatError) -> Self {
        Self {
            error: err.code().to_string(),
            message: err.client_message(),
            details: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}
