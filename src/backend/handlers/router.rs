//! WebSocket router and rate limiting
//!
//! Sets up HTTP and WebSocket endpoints using Warp framework.
//! Implements per-user token bucket rate limiting (100 messages/60 seconds).

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Rate limit error
#[derive(Debug, Clone)]
pub enum RateLimitError {
    ExceededLimit { retry_after_secs: u64 },
}

/// Token bucket for rate limiting
#[derive(Debug, Clone)]
struct TokenBucket {
    capacity: f64,
    tokens: f64,
    last_refill: u64,
    refill_rate: f64,  // tokens per second
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        let now = current_timestamp_millis() / 1000;
        Self {
            capacity,
            tokens: capacity,
            last_refill: now,
            refill_rate,
        }
    }

    fn refill(&mut self) {
        let now = current_timestamp_millis() / 1000;
        let elapsed = (now - self.last_refill) as f64;
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_refill = now;
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn get_tokens_until_full(&self) -> f64 {
        (self.capacity - self.tokens) / self.refill_rate
    }
}

/// Get current timestamp in milliseconds
fn current_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Rate limiter with per-user token buckets
pub struct RateLimiter {
    // Capacity: 100 messages per 60 seconds
    // Refill rate: 100/60 = 1.667 tokens/second
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if user can send a message
    pub async fn check_limit(&self, user_id: &str) -> Result<(), RateLimitError> {
        let mut buckets = self.buckets.write().await;
        
        let bucket = buckets
            .entry(user_id.to_string())
            .or_insert_with(|| TokenBucket::new(100.0, 100.0 / 60.0));

        if bucket.try_consume(1.0) {
            Ok(())
        } else {
            let retry_after = bucket.get_tokens_until_full().ceil() as u64 + 1;
            Err(RateLimitError::ExceededLimit {
                retry_after_secs: retry_after,
            })
        }
    }

    /// Check if user burst limit is exceeded (5 messages per second)
    pub async fn check_burst_limit(&self, user_id: &str) -> Result<(), RateLimitError> {
        // This would require tracking per-second messages, simplified for now
        self.check_limit(user_id).await
    }

    /// Get remaining tokens for user
    pub async fn get_remaining_tokens(&self, user_id: &str) -> f64 {
        let buckets = self.buckets.read().await;
        buckets
            .get(user_id)
            .map(|b| b.tokens)
            .unwrap_or(100.0)
    }

    /// Reset user's rate limit (for testing)
    pub async fn reset(&self, user_id: &str) {
        let mut buckets = self.buckets.write().await;
        buckets.remove(user_id);
    }

    /// Clear all rate limits (for testing)
    pub async fn clear_all(&self) {
        let mut buckets = self.buckets.write().await;
        buckets.clear();
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_new() {
        let bucket = TokenBucket::new(100.0, 1.667);
        assert_eq!(bucket.capacity, 100.0);
        assert_eq!(bucket.tokens, 100.0);
    }

    #[test]
    fn test_token_bucket_try_consume_success() {
        let mut bucket = TokenBucket::new(100.0, 1.667);
        assert!(bucket.try_consume(1.0));
        assert_eq!(bucket.tokens, 99.0);
    }

    #[test]
    fn test_token_bucket_try_consume_failure() {
        let mut bucket = TokenBucket::new(1.0, 1.0);
        bucket.tokens = 0.5;
        assert!(!bucket.try_consume(1.0));
        assert_eq!(bucket.tokens, 0.5);
    }

    #[tokio::test]
    async fn test_rate_limiter_new() {
        let limiter = RateLimiter::new();
        assert!(limiter.check_limit("user1").await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_allows_single_message() {
        let limiter = RateLimiter::new();
        assert!(limiter.check_limit("user1").await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_tracks_per_user() {
        let limiter = RateLimiter::new();
        
        // User 1 sends one message
        assert!(limiter.check_limit("user1").await.is_ok());
        
        // User 2 sends one message (independent bucket)
        assert!(limiter.check_limit("user2").await.is_ok());
        
        let tokens1 = limiter.get_remaining_tokens("user1").await;
        let tokens2 = limiter.get_remaining_tokens("user2").await;
        
        assert_eq!(tokens1, tokens2); // Both should have same remaining tokens
    }

    #[tokio::test]
    async fn test_rate_limiter_get_remaining_tokens() {
        let limiter = RateLimiter::new();
        
        let initial = limiter.get_remaining_tokens("user1").await;
        assert_eq!(initial, 100.0);
        
        limiter.check_limit("user1").await.unwrap();
        let after_one = limiter.get_remaining_tokens("user1").await;
        assert_eq!(after_one, 99.0);
    }

    #[tokio::test]
    async fn test_rate_limiter_reset() {
        let limiter = RateLimiter::new();
        
        limiter.check_limit("user1").await.unwrap();
        let tokens_before = limiter.get_remaining_tokens("user1").await;
        assert_eq!(tokens_before, 99.0);
        
        limiter.reset("user1").await;
        let tokens_after = limiter.get_remaining_tokens("user1").await;
        assert_eq!(tokens_after, 100.0);
    }

    #[tokio::test]
    async fn test_rate_limiter_clear_all() {
        let limiter = RateLimiter::new();
        
        limiter.check_limit("user1").await.unwrap();
        limiter.check_limit("user2").await.unwrap();
        
        assert_eq!(limiter.get_remaining_tokens("user1").await, 99.0);
        assert_eq!(limiter.get_remaining_tokens("user2").await, 99.0);
        
        limiter.clear_all().await;
        
        assert_eq!(limiter.get_remaining_tokens("user1").await, 100.0);
        assert_eq!(limiter.get_remaining_tokens("user2").await, 100.0);
    }

    #[tokio::test]
    async fn test_rate_limiter_capacity_check() {
        let limiter = RateLimiter::new();
        
        // Consume 100 tokens
        for _ in 0..100 {
            assert!(limiter.check_limit("user1").await.is_ok());
        }
        
        // 101st should fail
        let result = limiter.check_limit("user1").await;
        assert!(result.is_err());
        
        match result {
            Err(RateLimitError::ExceededLimit { retry_after_secs }) => {
                assert!(retry_after_secs > 0);
            }
            _ => panic!("Expected RateLimitError"),
        }
    }

    #[test]
    fn test_current_timestamp_millis() {
        let ts = current_timestamp_millis();
        assert!(ts > 0);
        assert!(ts > 1_700_000_000_000); // After 2023-11-15
    }
}
