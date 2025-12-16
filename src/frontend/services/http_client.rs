//! HTTP client for communicating with the backend API
//!
//! Provides methods for authentication endpoints (signup, login)

use serde::{Deserialize, Serialize};

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
}

/// Error response from server
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    #[allow(dead_code)]
    pub error: String,
    pub message: String,
}

/// HTTP client for backend API
pub struct HttpClient {
    base_url: String,
    client: reqwest::Client,
}

impl HttpClient {
    /// Create a new HTTP client
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Sign up a new user
    #[allow(dead_code)]
    pub async fn signup(&self, username: String, password: String) -> Result<AuthResponse, String> {
        let url = format!("{}/auth/signup", self.base_url);
        let request = SignupRequest { username, password };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if response.status().is_success() {
            response
                .json::<AuthResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.message)
        }
    }

    /// Log in an existing user
    pub async fn login(&self, username: String, password: String) -> Result<AuthResponse, String> {
        let url = format!("{}/auth/login", self.base_url);
        let request = SignupRequest { username, password };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if response.status().is_success() {
            response
                .json::<AuthResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
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
}
