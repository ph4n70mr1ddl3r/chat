//! HTTP client for communicating with the backend API
//!
//! Provides methods for authentication endpoints (signup, login)

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Signup request payload
#[derive(Debug, Serialize)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
}

/// Authentication response (signup and login)
#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub user_id: String,
    pub username: String,
    pub token: String,
    pub expires_in: u64,
    pub csrf_token: String,
}

/// Login request payload
#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Error response from server
#[derive(Debug, Deserialize)]
struct ErrorResponse {
    /// Error code from server (currently unused, only message field is used)
    #[allow(dead_code)]
    pub error: String,
    /// Human-readable error message
    pub message: String,
}

/// HTTP client for backend API
pub struct HttpClient {
    base_url: String,
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        Self { base_url, client }
    }

    /// Sign up a new user
    pub async fn signup(&self, username: String, password: String) -> Result<AuthResponse, String> {
        let url = format!("{}/auth/signup", self.base_url);
        let request = SignupRequest { username, password };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Network error during signup: {e}"))?;

        if response.status().is_success() {
            response
                .json::<AuthResponse>()
                .await
                .map_err(|e| format!("Failed to parse signup response: {e}"))
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse signup error: {e}"))?;
            Err(error.message)
        }
    }

    /// Log in an existing user
    pub async fn login(&self, username: String, password: String) -> Result<AuthResponse, String> {
        let url = format!("{}/auth/login", self.base_url);
        let request = LoginRequest { username, password };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Network error: {e}"))?;

        if response.status().is_success() {
            response
                .json::<AuthResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {e}"))
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {e}"))?;
            Err(error.message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signup_request_serialization() {
        let request = SignupRequest {
            username: "alice".to_string(),
            password: "TestPass123".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("alice"));
        assert!(json.contains("TestPass123"));
    }

    #[test]
    fn test_auth_response_deserialization() {
        let json = r#"{
            "user_id": "user123",
            "username": "alice",
            "token": "jwt-token-here",
            "expires_in": 3600,
            "csrf_token": "csrf-token-123"
        }"#;

        let response: AuthResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.user_id, "user123");
        assert_eq!(response.username, "alice");
        assert_eq!(response.csrf_token, "csrf-token-123");
    }
}
