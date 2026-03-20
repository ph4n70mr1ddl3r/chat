//! Rate limiting middleware
//!
//! Implements token-bucket rate limiting for authentication endpoints

use crate::utils::is_trusted_proxy;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, broadcast};
use warp::{self, addr::remote, reject, Filter, Rejection};

const DEFAULT_MAX_ENTRIES: usize = 100_000;

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
    entries: Arc<Mutex<HashMap<String, RateLimitEntry>>>,
    max_attempts: u32,
    window_duration: Duration,
    max_entries: usize,
    shutdown_tx: broadcast::Sender<()>,
}

impl Drop for RateLimiter {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(5, 900)
    }
}

impl RateLimiter {
    #[must_use]
    pub fn new(max_attempts: u32, window_secs: u64) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            max_attempts,
            window_duration: Duration::from_secs(window_secs),
            max_entries: DEFAULT_MAX_ENTRIES,
            shutdown_tx,
        }
    }

    #[must_use]
    pub fn with_max_entries(max_attempts: u32, window_secs: u64, max_entries: usize) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            max_attempts,
            window_duration: Duration::from_secs(window_secs),
            max_entries,
            shutdown_tx,
        }
    }

    #[must_use]
    pub fn global() -> Self {
        Self::new(1000, 60)
    }

    /// Signal the cleanup task to shut down gracefully
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }

    /// Get remaining attempts for an IP address
    #[must_use]
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
    #[must_use]
    pub async fn retry_after_seconds(&self, ip: &str) -> u64 {
        let entries = self.entries.lock().await;

        if let Some(entry) = entries.get(ip) {
            let elapsed = entry.window_start.elapsed();
            if elapsed >= self.window_duration {
                0
            } else {
                self.window_duration.checked_sub(elapsed).unwrap_or_default().as_secs().max(1)
            }
        } else {
            0
        }
    }

    /// Check if limited and record usage if allowed
    ///
    /// This operation is atomic within a single lock acquisition to prevent
    /// race conditions where multiple concurrent requests could bypass the limit
    ///
    /// # Errors
    /// Returns `RateLimitExceeded` if the IP has exceeded the rate limit.
    pub async fn check_and_record(&self, ip: &str) -> Result<(), RateLimitExceeded> {
        let mut entries = self.entries.lock().await;

        let now = Instant::now();
        if let Some(entry) = entries.get_mut(ip) {
            let elapsed = entry.window_start.elapsed();

            if elapsed > self.window_duration {
                entry.attempts = 1;
                entry.window_start = now;
                return Ok(());
            }

            if entry.attempts >= self.max_attempts {
                let remaining = self.window_duration.saturating_sub(elapsed);
                return Err(RateLimitExceeded {
                    retry_after_secs: remaining.as_secs().max(1),
                });
            }

            entry.attempts += 1;
            return Ok(());
        }

        if entries.len() >= self.max_entries {
            let now = Instant::now();
            let window_duration = self.window_duration;
            
            entries.retain(|_, entry| now.duration_since(entry.window_start) <= window_duration);

            if entries.len() >= self.max_entries {
                let eviction_count = (entries.len() / 4).max(250).min(entries.len());
                let keys_to_remove: Vec<String> = {
                    let mut entries_vec: Vec<_> = entries.iter().collect();
                    entries_vec.sort_by_key(|(_, entry)| entry.window_start);
                    entries_vec.into_iter()
                        .take(eviction_count)
                        .map(|(ip, _)| ip.clone())
                        .collect()
                };
                for ip in keys_to_remove {
                    entries.remove(&ip);
                }
                tracing::warn!(
                    "Rate limiter evicted {} entries due to memory pressure ({} entries before, {} after)",
                    eviction_count,
                    entries.len() + eviction_count,
                    entries.len()
                );
            }
        }

        entries.insert(
            ip.to_owned(),
            RateLimitEntry {
                attempts: 1,
                window_start: now,
            },
        );

        Ok(())
    }

    /// Clean up expired entries (should be called periodically)
    pub async fn cleanup_expired(&self) {
        let mut entries = self.entries.lock().await;

        entries.retain(|_, entry| entry.window_start.elapsed() <= self.window_duration);
    }

    /// Start a background task that periodically cleans up expired entries
    ///
    /// The task will run until `shutdown()` is called on the rate limiter,
    /// or the rate limiter is dropped.
    ///
    /// # Returns
    /// A task handle for graceful shutdown.
    #[must_use]
    pub fn start_periodic_cleanup(&self) -> tokio::task::JoinHandle<()> {
        let limiter = self.clone();
        let interval_duration = self.window_duration;
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval_duration);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        tracing::debug!("Rate limiter cleanup task shutting down");
                        break;
                    }
                    _ = interval.tick() => {
                        limiter.cleanup_expired().await;
                    }
                }
            }
        })
    }
}

/// Rejection used to signal rate limiting to the caller
#[derive(Debug, Clone)]
pub struct RateLimitExceeded {
    pub retry_after_secs: u64,
}

impl std::fmt::Display for RateLimitExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Rate limit exceeded. Retry after {} seconds.",
            self.retry_after_secs
        )
    }
}

impl std::error::Error for RateLimitExceeded {}

impl reject::Reject for RateLimitExceeded {}

impl RateLimiter {
    fn extract_client_ip(x_forwarded_for: &str, remote_ip: Option<std::net::SocketAddr>) -> String {
        let ips: Vec<&str> = x_forwarded_for
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();

        if ips.is_empty() {
            return remote_ip
                .map_or_else(|| "unknown".to_string(), |a| a.ip().to_string());
        }

        let should_trust_header = remote_ip
            .is_some_and(|addr| is_trusted_proxy(&addr.ip()));

        if !should_trust_header {
            tracing::warn!(
                "Ignoring X-Forwarded-For header from untrusted source: {:?}",
                remote_ip
            );
            return remote_ip
                .map_or_else(|| "unknown".to_string(), |a| a.ip().to_string());
        }

        for ip_str in ips.iter().rev() {
            if let Ok(ip_addr) = ip_str.parse::<std::net::IpAddr>() {
                if !is_trusted_proxy(&ip_addr) {
                    return ip_addr.to_string();
                }
            }
        }

        remote_ip
            .map_or_else(|| "unknown".to_string(), |a| a.ip().to_string())
    }
}

/// Warp filter to enforce rate limits based on remote IP address
#[must_use]
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
                    RateLimiter::extract_client_ip(&header, remote_ip)
                } else {
                    remote_ip
                        .map_or_else(|| "unknown".to_string(), |a| a.ip().to_string())
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

        assert!(limiter.check_and_record("192.168.1.1").await.is_ok());
        assert_eq!(limiter.get_remaining_attempts("192.168.1.1").await, 4);
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_after_max_attempts() {
        let limiter = RateLimiter::new(3, 60);
        let ip = "192.168.1.2";

        assert!(limiter.check_and_record(ip).await.is_ok());
        assert!(limiter.check_and_record(ip).await.is_ok());
        assert!(limiter.check_and_record(ip).await.is_ok());
        assert!(limiter.check_and_record(ip).await.is_err());

        assert_eq!(limiter.get_remaining_attempts(ip).await, 0);
    }

    #[tokio::test]
    async fn test_rate_limiter_reset() {
        let limiter = RateLimiter::new(3, 60);
        let ip = "192.168.1.3";

        assert!(limiter.check_and_record(ip).await.is_ok());
        assert!(limiter.check_and_record(ip).await.is_ok());
        assert!(limiter.check_and_record(ip).await.is_ok());
        assert!(limiter.check_and_record(ip).await.is_err());

        limiter.reset(ip).await;

        assert!(limiter.check_and_record(ip).await.is_ok());
        assert_eq!(limiter.get_remaining_attempts(ip).await, 2);
    }

    #[tokio::test]
    async fn test_rate_limiter_window_expiry() {
        let limiter = RateLimiter::new(3, 1);
        let ip = "192.168.1.4";

        assert!(limiter.check_and_record(ip).await.is_ok());
        assert!(limiter.check_and_record(ip).await.is_ok());

        assert_eq!(limiter.get_remaining_attempts(ip).await, 1);

        tokio::time::sleep(Duration::from_secs(2)).await;

        assert!(limiter.check_and_record(ip).await.is_ok());
        assert_eq!(limiter.get_remaining_attempts(ip).await, 2);
    }

    #[tokio::test]
    async fn test_check_and_record_atomic() {
        let limiter = RateLimiter::new(2, 60);

        assert!(limiter.check_and_record("192.168.1.5").await.is_ok());
        assert!(limiter.check_and_record("192.168.1.5").await.is_ok());
        assert!(limiter.check_and_record("192.168.1.5").await.is_err());
    }

    #[tokio::test]
    async fn test_check_and_record_reset_on_window_expiry() {
        let limiter = RateLimiter::new(2, 1);

        assert!(limiter.check_and_record("192.168.1.6").await.is_ok());
        assert!(limiter.check_and_record("192.168.1.6").await.is_ok());
        assert!(limiter.check_and_record("192.168.1.6").await.is_err());

        tokio::time::sleep(Duration::from_secs(2)).await;

        assert!(limiter.check_and_record("192.168.1.6").await.is_ok());
    }
}
