//! CSRF protection service
//!
//! Provides CSRF token generation and validation for state-changing operations.
//! This is a security measure to prevent cross-site request forgery attacks.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

const CSRF_TOKEN_VALIDITY_SECS: i64 = 3600; // 1 hour
const CSRF_CLEANUP_INTERVAL_SECS: u64 = 300; // 5 minutes

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

    pub async fn generate_token(&self, user_id: &str) -> String {
        let token = Uuid::new_v4().to_string();
        let csrf_token = CsrfToken {
            token: token.clone(),
            user_id: user_id.to_string(),
            created_at: Utc::now(),
        };

        let mut tokens = self.tokens.lock().await;
        tokens.insert(token.clone(), csrf_token);
        token
    }

    pub async fn validate_token(&self, token: &str, user_id: &str) -> bool {
        let tokens = self.tokens.lock().await;
        if let Some(csrf_token) = tokens.get(token) {
            if csrf_token.user_id != user_id || csrf_token.is_expired() {
                return false;
            }
            true
        } else {
            false
        }
    }

    pub async fn invalidate_token(&self, token: &str) {
        let mut tokens = self.tokens.lock().await;
        tokens.remove(token);
    }

    pub async fn cleanup_expired(&self) {
        let mut tokens = self.tokens.lock().await;
        tokens.retain(|_, token| !token.is_expired());
    }

    pub fn start_periodic_cleanup(&self) -> tokio::task::JoinHandle<()> {
        let service = self.clone();
        let interval = std::time::Duration::from_secs(CSRF_CLEANUP_INTERVAL_SECS);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                interval.tick().await;
                service.cleanup_expired().await;
            }
        })
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
        let token = service.generate_token("user123").await;
        assert!(!token.is_empty());
    }

    #[tokio::test]
    async fn test_csrf_token_validation() {
        let service = CsrfService::new();
        let token = service.generate_token("user123").await;
        assert!(service.validate_token(&token, "user123").await);
        assert!(!service.validate_token(&token, "wrong_user").await);
    }

    #[tokio::test]
    async fn test_csrf_token_invalidation() {
        let service = CsrfService::new();
        let token = service.generate_token("user123").await;
        assert!(service.validate_token(&token, "user123").await);
        service.invalidate_token(&token).await;
        assert!(!service.validate_token(&token, "user123").await);
    }
}
