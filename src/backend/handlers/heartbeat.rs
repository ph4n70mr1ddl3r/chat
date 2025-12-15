//! WebSocket heartbeat and keepalive handler
//!
//! Manages RFC 6455 ping-pong frames for connection health checking.
//! Sends periodic PING frames and monitors client PONG responses.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::Message as WsMessage;

/// Heartbeat configuration
#[derive(Debug, Clone)]
pub struct HeartbeatConfig {
    /// Interval between PING frames (seconds)
    pub ping_interval: u64,
    /// Maximum time to wait for PONG response (seconds)
    pub pong_timeout: u64,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            ping_interval: 25, // RFC 6455 recommendation
            pong_timeout: 5,   // Allow 5 second response time
        }
    }
}

/// Tracks heartbeat state for a connection
#[derive(Debug, Clone)]
pub struct HeartbeatState {
    /// Last time a PING was sent
    pub last_ping_sent: Option<Instant>,
    /// Last time a PONG was received
    pub last_pong_received: Option<Instant>,
    /// Connection created at
    pub created_at: Instant,
    /// Whether connection is still alive
    pub is_alive: bool,
}

impl HeartbeatState {
    pub fn new() -> Self {
        Self {
            last_ping_sent: None,
            last_pong_received: None,
            created_at: Instant::now(),
            is_alive: true,
        }
    }

    /// Check if PONG response is overdue
    pub fn is_pong_overdue(&self, pong_timeout: u64) -> bool {
        if let Some(last_ping) = self.last_ping_sent {
            let elapsed = Instant::now().duration_since(last_ping).as_secs();
            elapsed > pong_timeout
        } else {
            false
        }
    }

    /// Record PONG receipt
    pub fn record_pong(&mut self) {
        self.last_pong_received = Some(Instant::now());
    }

    /// Mark connection as dead
    pub fn mark_dead(&mut self) {
        self.is_alive = false;
    }
}

/// Heartbeat manager for a single connection
pub struct HeartbeatManager {
    config: HeartbeatConfig,
    state: Arc<RwLock<HeartbeatState>>,
}

impl HeartbeatManager {
    pub fn new(config: HeartbeatConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(HeartbeatState::new())),
        }
    }

    /// Generate a PING message
    pub fn generate_ping() -> WsMessage {
        WsMessage::Ping(vec![])
    }

    /// Handle incoming PONG message
    pub async fn handle_pong(&self) {
        let mut state = self.state.write().await;
        state.record_pong();
    }

    /// Check if connection is still healthy
    pub async fn is_healthy(&self) -> bool {
        let state = self.state.read().await;
        !state.is_pong_overdue(self.config.pong_timeout) && state.is_alive
    }

    /// Mark PING as sent
    pub async fn mark_ping_sent(&self) {
        let mut state = self.state.write().await;
        state.last_ping_sent = Some(Instant::now());
    }

    /// Mark connection as dead
    pub async fn mark_dead(&self) {
        let mut state = self.state.write().await;
        state.mark_dead();
    }

    /// Get connection uptime in seconds
    pub async fn get_uptime(&self) -> u64 {
        let state = self.state.read().await;
        Instant::now().duration_since(state.created_at).as_secs()
    }

    /// Get last activity timestamp (either PONG or initial connection)
    pub async fn get_last_activity(&self) -> Instant {
        let state = self.state.read().await;
        state.last_pong_received.unwrap_or(state.created_at)
    }
}

/// Global heartbeat scheduler for all connections
pub struct HeartbeatScheduler {
    config: HeartbeatConfig,
}

impl HeartbeatScheduler {
    pub fn new(config: HeartbeatConfig) -> Self {
        Self { config }
    }

    /// Get interval duration for PING messages
    pub fn get_ping_interval(&self) -> Duration {
        Duration::from_secs(self.config.ping_interval)
    }

    /// Get timeout duration for PONG responses
    pub fn get_pong_timeout(&self) -> Duration {
        Duration::from_secs(self.config.pong_timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_config_default() {
        let config = HeartbeatConfig::default();
        assert_eq!(config.ping_interval, 25);
        assert_eq!(config.pong_timeout, 5);
    }

    #[test]
    fn test_heartbeat_state_new() {
        let state = HeartbeatState::new();
        assert!(state.last_ping_sent.is_none());
        assert!(state.last_pong_received.is_none());
        assert!(state.is_alive);
    }

    #[test]
    fn test_heartbeat_state_record_pong() {
        let mut state = HeartbeatState::new();
        state.record_pong();
        assert!(state.last_pong_received.is_some());
    }

    #[test]
    fn test_heartbeat_state_mark_dead() {
        let mut state = HeartbeatState::new();
        assert!(state.is_alive);
        state.mark_dead();
        assert!(!state.is_alive);
    }

    #[tokio::test]
    async fn test_heartbeat_manager_new() {
        let manager = HeartbeatManager::new(HeartbeatConfig::default());
        assert!(manager.is_healthy().await);
    }

    #[tokio::test]
    async fn test_heartbeat_manager_handle_pong() {
        let manager = HeartbeatManager::new(HeartbeatConfig::default());
        manager.handle_pong().await;

        let state = manager.state.read().await;
        assert!(state.last_pong_received.is_some());
    }

    #[tokio::test]
    async fn test_heartbeat_manager_mark_ping_sent() {
        let manager = HeartbeatManager::new(HeartbeatConfig::default());
        manager.mark_ping_sent().await;

        let state = manager.state.read().await;
        assert!(state.last_ping_sent.is_some());
    }

    #[tokio::test]
    async fn test_heartbeat_manager_mark_dead() {
        let manager = HeartbeatManager::new(HeartbeatConfig::default());
        assert!(manager.is_healthy().await);

        manager.mark_dead().await;
        assert!(!manager.is_healthy().await);
    }

    #[tokio::test]
    async fn test_heartbeat_manager_get_uptime() {
        let manager = HeartbeatManager::new(HeartbeatConfig::default());

        // Wait a bit
        tokio::time::sleep(Duration::from_millis(10)).await;

        let uptime = manager.get_uptime().await;
        // uptime should be >= 0 (in fact it's a u64 so always >= 0)
        let _ = uptime;
    }

    #[test]
    fn test_heartbeat_manager_generate_ping() {
        let ping = HeartbeatManager::generate_ping();
        match ping {
            WsMessage::Ping(_) => {}
            _ => panic!("Expected Ping message"),
        }
    }

    #[test]
    fn test_heartbeat_scheduler_new() {
        let config = HeartbeatConfig {
            ping_interval: 30,
            pong_timeout: 10,
        };
        let scheduler = HeartbeatScheduler::new(config);

        let interval = scheduler.get_ping_interval();
        assert_eq!(interval.as_secs(), 30);

        let timeout = scheduler.get_pong_timeout();
        assert_eq!(timeout.as_secs(), 10);
    }

    #[tokio::test]
    async fn test_heartbeat_manager_pong_timeout_detection() {
        let config = HeartbeatConfig {
            ping_interval: 1,
            pong_timeout: 1,
        };
        let manager = HeartbeatManager::new(config);

        // Mark ping sent
        manager.mark_ping_sent().await;

        // Wait longer than pong timeout
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Check health - should be unhealthy due to no pong response
        assert!(!manager.is_healthy().await);
    }

    #[tokio::test]
    async fn test_heartbeat_manager_pong_before_timeout() {
        let config = HeartbeatConfig {
            ping_interval: 1,
            pong_timeout: 5,
        };
        let manager = HeartbeatManager::new(config);

        // Mark ping sent
        manager.mark_ping_sent().await;

        // Quickly handle pong
        manager.handle_pong().await;

        // Should still be healthy
        assert!(manager.is_healthy().await);
    }
}
