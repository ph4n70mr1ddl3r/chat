//! Login attempt tracking service
//!
//! Provides account-level lockout protection against credential stuffing attacks.
//! Tracks failed login attempts per username and implements progressive lockout.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

const MAX_FAILED_ATTEMPTS: u32 = 5;
const LOCKOUT_DURATION_SECS: u64 = 900; // 15 minutes
const ATTEMPT_WINDOW_SECS: u64 = 3600; // 1 hour

#[derive(Debug, Clone)]
struct LoginAttempt {
    count: u32,
    first_attempt: Instant,
    last_attempt: Instant,
    locked_until: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct LoginAttemptService {
    attempts: Arc<RwLock<HashMap<String, LoginAttempt>>>,
    max_attempts: u32,
    lockout_duration: Duration,
    attempt_window: Duration,
}

impl LoginAttemptService {
    pub fn new() -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
            max_attempts: MAX_FAILED_ATTEMPTS,
            lockout_duration: Duration::from_secs(LOCKOUT_DURATION_SECS),
            attempt_window: Duration::from_secs(ATTEMPT_WINDOW_SECS),
        }
    }

    pub fn with_config(max_attempts: u32, lockout_duration_secs: u64, attempt_window_secs: u64) -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
            max_attempts,
            lockout_duration: Duration::from_secs(lockout_duration_secs),
            attempt_window: Duration::from_secs(attempt_window_secs),
        }
    }

    pub async fn record_failed_attempt(&self, username: &str) {
        let mut attempts = self.attempts.write().await;
        let now = Instant::now();
        
        let entry = attempts.entry(username.to_lowercase()).or_insert(LoginAttempt {
            count: 0,
            first_attempt: now,
            last_attempt: now,
            locked_until: None,
        });

        if now.duration_since(entry.first_attempt) > self.attempt_window {
            entry.count = 0;
            entry.first_attempt = now;
        }

        entry.count += 1;
        entry.last_attempt = now;

        if entry.count >= self.max_attempts {
            entry.locked_until = Some(now + self.lockout_duration);
            tracing::warn!(
                username_prefix = &username[..8.min(username.len())],
                attempt_count = entry.count,
                "Account locked due to failed login attempts"
            );
        }
    }

    pub async fn is_locked(&self, username: &str) -> bool {
        let username_lower = username.to_lowercase();
        let mut attempts = self.attempts.write().await;
        let now = Instant::now();
        
        if let Some(attempt) = attempts.get_mut(&username_lower) {
            if let Some(locked_until) = attempt.locked_until {
                if now < locked_until {
                    return true;
                } else {
                    attempt.locked_until = None;
                    attempt.count = 0;
                    attempt.first_attempt = now;
                }
            }
        }
        
        false
    }

    pub async fn clear_attempts(&self, username: &str) {
        let mut attempts = self.attempts.write().await;
        attempts.remove(&username.to_lowercase());
    }

    pub async fn cleanup_expired(&self) {
        let mut attempts = self.attempts.write().await;
        let now = Instant::now();
        
        attempts.retain(|_, attempt| {
            if let Some(locked_until) = attempt.locked_until {
                if now < locked_until {
                    return true;
                }
            }
            
            now.duration_since(attempt.last_attempt) <= self.attempt_window
        });
    }
}

impl Default for LoginAttemptService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_failed_attempts() {
        let service = LoginAttemptService::with_config(3, 60, 3600);
        let username = "testuser";
        
        assert!(!service.is_locked(username).await);
        
        service.record_failed_attempt(username).await;
        assert!(!service.is_locked(username).await);
        
        service.record_failed_attempt(username).await;
        assert!(!service.is_locked(username).await);
        
        service.record_failed_attempt(username).await;
        assert!(service.is_locked(username).await);
    }

    #[tokio::test]
    async fn test_clear_attempts() {
        let service = LoginAttemptService::with_config(2, 60, 3600);
        let username = "testuser";
        
        service.record_failed_attempt(username).await;
        service.record_failed_attempt(username).await;
        assert!(service.is_locked(username).await);
        
        service.clear_attempts(username).await;
        assert!(!service.is_locked(username).await);
    }

    #[tokio::test]
    async fn test_successful_login_clears_attempts() {
        let service = LoginAttemptService::with_config(3, 60, 3600);
        let username = "testuser";
        
        service.record_failed_attempt(username).await;
        service.record_failed_attempt(username).await;
        
        service.clear_attempts(username).await;
        
        service.record_failed_attempt(username).await;
        assert!(!service.is_locked(username).await);
    }
}
