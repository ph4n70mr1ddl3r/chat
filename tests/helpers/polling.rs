//! Deterministic Wait Patterns for Tests
//!
//! Replace hardcoded sleeps with deterministic polling and timeout patterns.
//! This prevents test flakiness and ensures tests complete as fast as possible.
//!
//! **Why not sleep()?**
//! - sleep(600ms) might be too short on slow CI systems, causing timeouts
//! - sleep(600ms) might be too long on fast machines, wasting test time
//! - sleep() is non-deterministic: always waits full duration even if condition met early
//!
//! **Use instead**: Poll until condition met OR timeout.

use std::future::Future;
use tokio::time::{sleep, timeout, Duration, Instant};

/// Poll a condition until it becomes true or timeout is reached.
///
/// # Example: Wait until message is queued
///
/// ```ignore
/// // Instead of: sleep(Duration::from_millis(600)).await;
/// poll_until(Duration::from_secs(2), || async {
///     let msg = db::find_message_by_id(&pool, &message_id).await.ok().flatten();
///     msg.map(|m| m.status == "queued").unwrap_or(false)
/// })
/// .await
/// .expect("message never queued");
/// ```
pub async fn poll_until<F, Fut>(max_duration: Duration, mut condition: F) -> Result<(), tokio::time::error::Elapsed>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = bool>,
{
    let start = Instant::now();
    
    loop {
        // Check condition immediately (before first sleep)
        if condition().await {
            return Ok(());
        }
        
        // If we've exceeded max duration, timeout
        if start.elapsed() >= max_duration {
            return Err(tokio::time::error::Elapsed::new());
        }
        
        // Sleep for a short interval before next check
        // 50ms is a good balance: checks ~20 times per second
        sleep(Duration::from_millis(50)).await;
    }
}

/// Wait for a channel to receive a message OR timeout.
///
/// # Example: Wait for message delivery event
///
/// ```ignore
/// let delivered = wait_for_channel_message(&mut rx, Duration::from_secs(2))
///     .await
///     .expect("no delivery event received");
/// ```
pub async fn wait_for_channel_message<T>(
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<T>,
    max_duration: Duration,
) -> Result<T, tokio::time::error::Elapsed> {
    timeout(max_duration, rx.recv())
        .await
        .map(|opt| opt.expect("channel closed"))
}

/// Wait for a predicate to become true with detailed diagnostics.
///
/// Logs polling attempts (useful for debugging flaky tests).
///
/// # Example: Wait for user status with diagnostics
///
/// ```ignore
/// poll_with_diagnostics(
///     Duration::from_secs(2),
///     "user_online",
///     || async {
///         let user = presence_service.get_user_status(&user_id).await;
///         user.status == "online"
///     }
/// )
/// .await
/// .expect("user never came online");
/// ```
pub async fn poll_with_diagnostics<F, Fut>(
    max_duration: Duration,
    diagnostic_name: &str,
    mut condition: F,
) -> Result<(), tokio::time::error::Elapsed>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = bool>,
{
    let start = Instant::now();
    let mut attempts = 0;
    
    loop {
        attempts += 1;
        
        if condition().await {
            eprintln!(
                "✓ [{}] Success after {} attempts ({:.0}ms)",
                diagnostic_name,
                attempts,
                start.elapsed().as_millis()
            );
            return Ok(());
        }
        
        if start.elapsed() >= max_duration {
            eprintln!(
                "✗ [{}] TIMEOUT after {} attempts ({:.0}ms)",
                diagnostic_name,
                attempts,
                start.elapsed().as_millis()
            );
            return Err(tokio::time::error::Elapsed::new());
        }
        
        sleep(Duration::from_millis(50)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn poll_until_succeeds_when_condition_true() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = poll_until(Duration::from_secs(1), || {
            let c = counter_clone.clone();
            async move {
                let count = c.fetch_add(1, Ordering::SeqCst);
                count >= 3 // Succeed after 3 attempts
            }
        })
        .await;

        assert!(result.is_ok());
        assert!(counter.load(Ordering::SeqCst) >= 3);
    }

    #[tokio::test]
    async fn poll_until_times_out_when_condition_never_true() {
        let result = poll_until(Duration::from_millis(100), || async { false }).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn poll_until_returns_immediately_when_condition_true_first_try() {
        let start = Instant::now();

        let result = poll_until(Duration::from_secs(1), || async { true }).await;

        assert!(result.is_ok());
        // Should return almost immediately (within 50ms)
        assert!(start.elapsed() < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn wait_for_channel_message_receives_successfully() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let _ = tx.send("message");
        });

        let result = wait_for_channel_message(&mut rx, Duration::from_secs(1))
            .await
            .expect("should receive message");

        assert_eq!(result, "message");
    }

    #[tokio::test]
    async fn wait_for_channel_message_times_out() {
        let (_tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<&str>();

        let result = wait_for_channel_message(&mut rx, Duration::from_millis(100)).await;

        assert!(result.is_err());
    }
}
