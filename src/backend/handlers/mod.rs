//! Message handlers for WebSocket and HTTP endpoints

use crate::models::{User, Conversation, Message};

/// WebSocket message handler
pub struct MessageHandler;

impl MessageHandler {
    pub fn new() -> Self {
        Self
    }

    /// Handle incoming text message
    pub async fn handle_message(&self, msg: Message) -> Result<Message, String> {
        // TODO: Validate message, store in DB, route to recipient
        msg.validate()?;
        Ok(msg)
    }
}

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
