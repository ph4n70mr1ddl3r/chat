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
const MAX_ENTRIES: usize = 50_000; // Maximum tracked usernames to prevent memory exhaustion

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
    cleanup_handle: Option<Arc<tokio::task::JoinHandle<()>>>,
}

impl Drop for LoginAttemptService {
    fn drop(&mut self) {
        if let Some(handle) = self.cleanup_handle.take() {
            if let Some(handle) = Arc::into_inner(handle) {
                handle.abort();
            }
        }
    }
}

impl LoginAttemptService {
    pub fn new() -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
            max_attempts: MAX_FAILED_ATTEMPTS,
            lockout_duration: Duration::from_secs(LOCKOUT_DURATION_SECS),
            attempt_window: Duration::from_secs(ATTEMPT_WINDOW_SECS),
            cleanup_handle: None,
        }
    }

    pub fn with_config(max_attempts: u32, lockout_duration_secs: u64, attempt_window_secs: u64) -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
            max_attempts,
            lockout_duration: Duration::from_secs(lockout_duration_secs),
            attempt_window: Duration::from_secs(attempt_window_secs),
            cleanup_handle: None,
        }
    }

    pub fn with_cleanup() -> Self {
        let attempts: Arc<RwLock<HashMap<String, LoginAttempt>>> = Arc::new(RwLock::new(HashMap::new()));
        let attempts_clone = attempts.clone();
        let attempt_window = Duration::from_secs(ATTEMPT_WINDOW_SECS);

        let cleanup_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300));
            loop {
                interval.tick().await;
                let mut attempts = attempts_clone.write().await;
                let now = Instant::now();
                let before_count = attempts.len();

                attempts.retain(|_, attempt| {
                    if let Some(locked_until) = attempt.locked_until {
                        if now < locked_until {
                            return true;
                        }
                    }
                    now.duration_since(attempt.last_attempt) <= attempt_window
                });

                let removed = before_count - attempts.len();
                if removed > 0 {
                    tracing::info!(
                        target: "login_attempt",
                        removed = removed,
                        remaining = attempts.len(),
                        "Cleaned up expired login attempts"
                    );
                }
            }
        });

        Self {
            attempts,
            max_attempts: MAX_FAILED_ATTEMPTS,
            lockout_duration: Duration::from_secs(LOCKOUT_DURATION_SECS),
            attempt_window: Duration::from_secs(ATTEMPT_WINDOW_SECS),
            cleanup_handle: Some(Arc::new(cleanup_handle)),
        }
    }

    pub async fn record_failed_attempt(&self, username: &str) {
        let mut attempts = self.attempts.write().await;
        let now = Instant::now();

        // Evict old entries if at capacity to prevent memory exhaustion
        if attempts.len() >= MAX_ENTRIES {
            let attempt_window = self.attempt_window;
            let before_count = attempts.len();
            
            // First, remove expired entries
            attempts.retain(|_, attempt| {
                if let Some(locked_until) = attempt.locked_until {
                    if now < locked_until {
                        return true;
                    }
                }
                now.duration_since(attempt.last_attempt) <= attempt_window
            });
            
            // If still over limit, evict oldest 10% of entries
            if attempts.len() >= MAX_ENTRIES {
                let eviction_count = (attempts.len() / 10).max(100);
                let mut entries_vec: Vec<_> = attempts.iter().collect();
                entries_vec.sort_by_key(|(_, a)| a.last_attempt);
                let keys_to_remove: Vec<String> = entries_vec
                    .into_iter()
                    .take(eviction_count)
                    .map(|(key, _)| key.clone())
                    .collect();
                for key in keys_to_remove {
                    attempts.remove(&key);
                }
                tracing::debug!(
                    target: "login_attempt",
                    evicted = eviction_count,
                    before = before_count,
                    after = attempts.len(),
                    "Evicted entries due to memory pressure"
                );
            }
        }
        
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
                }
                attempt.locked_until = None;
                attempt.count = 0;
                attempt.first_attempt = now;
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
        Self::with_cleanup()
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
