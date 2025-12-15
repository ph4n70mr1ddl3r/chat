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
pub mod user;
pub mod websocket;

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
