//! Rate limiting middleware
//!
//! Implements token-bucket rate limiting for authentication endpoints

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use warp::{self, addr::remote, reject, Filter, Rejection};

/// Maximum number of entries in the rate limiter to prevent memory exhaustion
const MAX_RATE_LIMIT_ENTRIES: usize = 100_000;

/// Rate limit entry tracking attempts and reset time
///
/// Stores the number of failed attempts within the current time window
/// for a given IP address or identifier.
#[derive(Debug, Clone)]
struct RateLimitEntry {
    attempts: u32,
    window_start: Instant,
}

/// Rate limiter for authentication endpoints
///
/// Implements token-bucket rate limiting to prevent brute force attacks
/// and API abuse. Uses in-memory storage with periodic cleanup.
#[derive(Clone)]
pub struct RateLimiter {
    /// Map of IP addresses to rate limit entries
    entries: Arc<Mutex<HashMap<String, RateLimitEntry>>>,
    /// Maximum attempts allowed per window
    max_attempts: u32,
    /// Time window for rate limiting (in seconds)
    window_duration: Duration,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(5, 900)
    }
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

    /// Convenience constructor for global requests (1000 req/min)
    pub fn global() -> Self {
        Self::new(1000, 60)
    }

    /// Check if an IP address is rate limited
    ///
    /// Returns true if the IP has exceeded the rate limit
    pub async fn is_rate_limited(&self, ip: &str) -> bool {
        let mut entries = self.entries.lock().expect("rate limiter mutex poisoned");

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
        let mut entries = self.entries.lock().expect("rate limiter mutex poisoned");

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
            // First attempt - check capacity to prevent memory exhaustion
            if entries.len() >= MAX_RATE_LIMIT_ENTRIES {
                // Remove expired entries to make room
                entries.retain(|_, entry| entry.window_start.elapsed() <= self.window_duration);
            }

            // Only add if we're under capacity after cleanup
            if entries.len() < MAX_RATE_LIMIT_ENTRIES {
                entries.insert(
                    ip.to_owned(),
                    RateLimitEntry {
                        attempts: 1,
                        window_start: now,
                    },
                );
            }
        }
    }

    /// Get remaining attempts for an IP address
    pub async fn get_remaining_attempts(&self, ip: &str) -> u32 {
        let entries = self.entries.lock().expect("rate limiter mutex poisoned");

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
        let mut entries = self.entries.lock().expect("rate limiter mutex poisoned");
        entries.remove(ip);
    }

    /// Determine how long until the window resets for a given key
    pub async fn retry_after_seconds(&self, ip: &str) -> u64 {
        let entries = self.entries.lock().expect("rate limiter mutex poisoned");

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
    ///
    /// This operation is atomic within a single lock acquisition to prevent
    /// race conditions where multiple concurrent requests could bypass the limit
    pub async fn check_and_record(&self, ip: &str) -> Result<(), RateLimitExceeded> {
        let mut entries = self.entries.lock().expect("rate limiter mutex poisoned");

        // Check current state atomically
        let now = Instant::now();
        if let Some(entry) = entries.get_mut(ip) {
            let elapsed = entry.window_start.elapsed();

            // If window has expired, reset and allow
            if elapsed > self.window_duration {
                entry.attempts = 1;
                entry.window_start = now;
                return Ok(());
            }

            // Check if limit exceeded
            if entry.attempts >= self.max_attempts {
                return Err(RateLimitExceeded {
                    retry_after_secs: (self.window_duration - elapsed).as_secs().max(1),
                });
            }

            // Increment attempts
            entry.attempts += 1;
            return Ok(());
        }

        // First attempt - check capacity
        if entries.len() >= MAX_RATE_LIMIT_ENTRIES {
            entries.retain(|_, entry| entry.window_start.elapsed() <= self.window_duration);
        }

        if entries.len() < MAX_RATE_LIMIT_ENTRIES {
            entries.insert(
                ip.to_owned(),
                RateLimitEntry {
                    attempts: 1,
                    window_start: now,
                },
            );
        }

        Ok(())
    }

    /// Clean up expired entries (should be called periodically)
    pub async fn cleanup_expired(&self) {
        let mut entries = self.entries.lock().expect("rate limiter mutex poisoned");

        entries.retain(|_, entry| entry.window_start.elapsed() <= self.window_duration);
    }

    /// Start a background task that periodically cleans up expired entries
    ///
    /// This spawns a Tokio task that runs indefinitely. The task should not be
    /// cancelled unless the rate limiter is being dropped.
    #[must_use]
    pub fn start_periodic_cleanup(&self) -> tokio::task::JoinHandle<()> {
        let limiter = self.clone();
        let interval = self.window_duration;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                interval.tick().await;
                limiter.cleanup_expired().await;
            }
        })
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
        .and(warp::header::optional::<String>("X-Forwarded-For"))
        .and(warp::any().map(move || limiter.clone()))
        .and_then(
            |remote_ip: Option<std::net::SocketAddr>,
             forwarded_header: Option<String>,
             limiter: Arc<RateLimiter>| async move {
                let ip = if let Some(header) = forwarded_header {
                    if let Ok(parsed_ip) = header
                        .split(',')
                        .next()
                        .unwrap_or(&header)
                        .trim()
                        .parse::<IpAddr>()
                    {
                        let is_trusted = match parsed_ip {
                            IpAddr::V4(ipv4) => ipv4.is_loopback(),
                            IpAddr::V6(ipv6) => ipv6.is_loopback(),
                        };
                        if is_trusted {
                            remote_ip
                                .map(|a| a.ip().to_string())
                                .unwrap_or_else(|| "unknown".to_string())
                        } else {
                            parsed_ip.to_string()
                        }
                    } else {
                        remote_ip
                            .map(|a| a.ip().to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    }
                } else {
                    remote_ip
                        .map(|a| a.ip().to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                };
                limiter.check_and_record(&ip).await.map_err(reject::custom)
            },
        )
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
