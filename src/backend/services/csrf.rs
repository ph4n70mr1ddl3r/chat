//! CSRF protection service
//!
//! Provides CSRF token generation and validation for state-changing operations.
//! This is a security measure to prevent cross-site request forgery attacks.

use chrono::{DateTime, Utc};
use rand::Rng;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

const CSRF_TOKEN_VALIDITY_SECS: i64 = 3600; // 1 hour

#[derive(Clone)]
pub struct CsrfToken {
    pub token: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
}

impl CsrfToken {
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.created_at);
        elapsed.num_seconds() > CSRF_TOKEN_VALIDITY_SECS
    }
}

#[derive(Clone)]
pub struct CsrfService {
    tokens: Arc<Mutex<HashMap<String, CsrfToken>>>,
}

impl CsrfService {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn generate_token(&self, user_id: &str) -> String {
        let token = Uuid::new_v4().to_string() + &rand::thread_rng().gen::<u32>().to_string();
        let csrf_token = CsrfToken {
            token: token.clone(),
            user_id: user_id.to_string(),
            created_at: Utc::now(),
        };

        let mut tokens = self.tokens.lock().unwrap();
        tokens.insert(token.clone(), csrf_token);
        token
    }

    pub fn validate_token(&self, token: &str, user_id: &str) -> bool {
        let tokens = self.tokens.lock().unwrap();
        if let Some(csrf_token) = tokens.get(token) {
            if csrf_token.user_id != user_id || csrf_token.is_expired() {
                return false;
            }
            true
        } else {
            false
        }
    }

    pub fn invalidate_token(&self, token: &str) {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.remove(token);
    }

    pub fn cleanup_expired(&self) {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.retain(|_, token| !token.is_expired());
    }
}

impl Default for CsrfService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_csrf_token_generation() {
        let service = CsrfService::new();
        let token = service.generate_token("user123");
        assert!(!token.is_empty());
    }

    #[tokio::test]
    async fn test_csrf_token_validation() {
        let service = CsrfService::new();
        let token = service.generate_token("user123");
        assert!(service.validate_token(&token, "user123"));
        assert!(!service.validate_token(&token, "wrong_user"));
    }

    #[tokio::test]
    async fn test_csrf_token_invalidation() {
        let service = CsrfService::new();
        let token = service.generate_token("user123");
        assert!(service.validate_token(&token, "user123"));
        service.invalidate_token(&token);
        assert!(!service.validate_token(&token, "user123"));
    }
}
