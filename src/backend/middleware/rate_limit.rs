//! Rate limiting middleware
//!
//! Implements token-bucket rate limiting for authentication endpoints

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use warp::{self, addr::remote, reject, Filter, Rejection};

/// Rate limit entry tracking attempts and reset time
#[derive(Debug, Clone)]
struct RateLimitEntry {
    attempts: u32,
    window_start: Instant,
}

/// Rate limiter for authentication endpoints
#[derive(Clone)]
pub struct RateLimiter {
    /// Map of IP addresses to rate limit entries
    entries: Arc<Mutex<HashMap<String, RateLimitEntry>>>,
    /// Maximum attempts allowed per window
    max_attempts: u32,
    /// Time window for rate limiting (in seconds)
    window_duration: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `max_attempts` - Maximum failed attempts allowed (default: 5)
    /// * `window_secs` - Time window in seconds (default: 900 = 15 minutes)
    pub fn new(max_attempts: u32, window_secs: u64) -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            max_attempts,
            window_duration: Duration::from_secs(window_secs),
        }
    }

    /// Convenience constructor for auth endpoints (5 attempts / 15 minutes)
    pub fn auth() -> Self {
        Self::new(5, 900)
    }

    /// Convenience constructor for global requests (1000 req/min)
    pub fn global() -> Self {
        Self::new(1000, 60)
    }

    /// Create default rate limiter (5 attempts per 15 minutes)
    pub fn default_auth() -> Self {
        Self::new(5, 900)
    }

    /// Check if an IP address is rate limited
    ///
    /// Returns true if the IP has exceeded the rate limit
    pub async fn is_rate_limited(&self, ip: &str) -> bool {
        let mut entries = self.entries.lock().await;

        if let Some(entry) = entries.get(ip) {
            let elapsed = entry.window_start.elapsed();

            // If window has expired, reset the entry
            if elapsed > self.window_duration {
                entries.remove(ip);
                return false;
            }

            // Check if attempts exceeded
            entry.attempts >= self.max_attempts
        } else {
            false
        }
    }

    /// Record a failed attempt for an IP address
    pub async fn record_attempt(&self, ip: &str) {
        let mut entries = self.entries.lock().await;

        let now = Instant::now();

        if let Some(entry) = entries.get_mut(ip) {
            let elapsed = entry.window_start.elapsed();

            // If window has expired, reset
            if elapsed > self.window_duration {
                entry.attempts = 1;
                entry.window_start = now;
            } else {
                entry.attempts += 1;
            }
        } else {
            // First attempt
            entries.insert(
                ip.to_string(),
                RateLimitEntry {
                    attempts: 1,
                    window_start: now,
                },
            );
        }
    }

    /// Get remaining attempts for an IP address
    pub async fn get_remaining_attempts(&self, ip: &str) -> u32 {
        let entries = self.entries.lock().await;

        if let Some(entry) = entries.get(ip) {
            let elapsed = entry.window_start.elapsed();

            if elapsed > self.window_duration {
                self.max_attempts
            } else {
                self.max_attempts.saturating_sub(entry.attempts)
            }
        } else {
            self.max_attempts
        }
    }

    /// Reset rate limit for an IP address (e.g., after successful login)
    pub async fn reset(&self, ip: &str) {
        let mut entries = self.entries.lock().await;
        entries.remove(ip);
    }

    /// Determine how long until the window resets for a given key
    pub async fn retry_after_seconds(&self, ip: &str) -> u64 {
        let entries = self.entries.lock().await;

        if let Some(entry) = entries.get(ip) {
            let elapsed = entry.window_start.elapsed();
            if elapsed >= self.window_duration {
                0
            } else {
                (self.window_duration - elapsed).as_secs().max(1)
            }
        } else {
            0
        }
    }

    /// Check if limited and record usage if allowed
    pub async fn check_and_record(&self, ip: &str) -> Result<(), RateLimitExceeded> {
        if self.is_rate_limited(ip).await {
            return Err(RateLimitExceeded {
                retry_after_secs: self.retry_after_seconds(ip).await,
            });
        }

        self.record_attempt(ip).await;
        Ok(())
    }

    /// Clean up expired entries (should be called periodically)
    pub async fn cleanup_expired(&self) {
        let mut entries = self.entries.lock().await;

        entries.retain(|_, entry| entry.window_start.elapsed() <= self.window_duration);
    }
}

/// Rejection used to signal rate limiting to the caller
#[derive(Debug, Clone)]
pub struct RateLimitExceeded {
    pub retry_after_secs: u64,
}

impl reject::Reject for RateLimitExceeded {}

/// Warp filter to enforce rate limits based on remote IP address
pub fn rate_limit_filter(
    limiter: Arc<RateLimiter>,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    remote()
        .map(|addr: Option<std::net::SocketAddr>| {
            addr.map(|a| a.ip().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        })
        .and(warp::any().map(move || limiter.clone()))
        .and_then(|ip: String, limiter: Arc<RateLimiter>| async move {
            limiter
                .check_and_record(&ip)
                .await
                .map_err(|e| reject::custom(e))
        })
        .untuple_one()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_initial_attempts() {
        let limiter = RateLimiter::new(5, 60);

        assert!(!limiter.is_rate_limited("192.168.1.1").await);
        assert_eq!(limiter.get_remaining_attempts("192.168.1.1").await, 5);
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_after_max_attempts() {
        let limiter = RateLimiter::new(3, 60);
        let ip = "192.168.1.2";

        // Record 3 failed attempts
        for _ in 0..3 {
            limiter.record_attempt(ip).await;
        }

        // Should now be rate limited
        assert!(limiter.is_rate_limited(ip).await);
        assert_eq!(limiter.get_remaining_attempts(ip).await, 0);
    }

    #[tokio::test]
    async fn test_rate_limiter_reset() {
        let limiter = RateLimiter::new(3, 60);
        let ip = "192.168.1.3";

        // Record attempts and get rate limited
        for _ in 0..3 {
            limiter.record_attempt(ip).await;
        }
        assert!(limiter.is_rate_limited(ip).await);

        // Reset
        limiter.reset(ip).await;

        // Should no longer be rate limited
        assert!(!limiter.is_rate_limited(ip).await);
        assert_eq!(limiter.get_remaining_attempts(ip).await, 3);
    }

    #[tokio::test]
    async fn test_rate_limiter_window_expiry() {
        let limiter = RateLimiter::new(3, 1); // 1 second window
        let ip = "192.168.1.4";

        // Record attempts
        limiter.record_attempt(ip).await;
        limiter.record_attempt(ip).await;

        assert_eq!(limiter.get_remaining_attempts(ip).await, 1);

        // Wait for window to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should be reset
        assert!(!limiter.is_rate_limited(ip).await);
        assert_eq!(limiter.get_remaining_attempts(ip).await, 3);
    }
}
