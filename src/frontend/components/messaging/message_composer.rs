// ============================================================================
// MessageComposer Rust Handler
// ============================================================================
// Handles integration between MessageComposer Slint component and backend
// services for keyboard handling, debounced typing notifications, and
// message sending via WebSocket.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Configuration for debouncing typing notifications
pub const TYPING_DEBOUNCE_MS: u64 = 300; // Wait 300ms after last keystroke before emitting false

pub const INACTIVITY_TIMEOUT_MS: u64 = 1000; // Emit false after 1 second of inactivity

/// Manages typing state with debouncing
pub struct TypingManager {
    /// When the last typing event was detected
    last_type_time: Arc<Mutex<Option<Instant>>>,
    /// Whether we're currently in a typing state
    is_typing: Arc<Mutex<bool>>,
    /// ID of current debounce timer (for cancellation)
    debounce_timer: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
}

impl TypingManager {
    /// Create a new typing manager
    pub fn new() -> Self {
        TypingManager {
            last_type_time: Arc::new(Mutex::new(None)),
            is_typing: Arc::new(Mutex::new(false)),
            debounce_timer: Arc::new(Mutex::new(None)),
        }
    }

    /// Call when user starts typing
    pub fn on_typing(&self) {
        let now = Instant::now();
        let mut last_time = self.last_type_time.lock().unwrap();
        *last_time = Some(now);

        let mut currently_typing = self.is_typing.lock().unwrap();
        if !*currently_typing {
            *currently_typing = true;
            // In a real implementation, emit TypingIndicator command here
        }
    }

    /// Call when inactivity is detected (e.g., on message send or after timer)
    pub fn on_inactivity(&self) {
        let mut currently_typing = self.is_typing.lock().unwrap();
        if *currently_typing {
            *currently_typing = false;
            // In a real implementation, emit TypingIndicator command here
        }
    }

    /// Check if enough time has passed to emit typing(false)
    pub fn should_stop_typing(&self) -> bool {
        if let Ok(last_time) = self.last_type_time.lock() {
            if let Some(last) = *last_time {
                let elapsed = Instant::now().duration_since(last);
                return elapsed > Duration::from_millis(TYPING_DEBOUNCE_MS);
            }
        }
        false
    }
}

impl Default for TypingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typing_manager_creation() {
        let manager = TypingManager::new();
        let is_typing = manager.is_typing.lock().unwrap();
        assert!(!*is_typing, "Should start in non-typing state");
    }

    #[test]
    fn test_typing_on_start() {
        let manager = TypingManager::new();
        manager.on_typing();
        let is_typing = manager.is_typing.lock().unwrap();
        assert!(*is_typing, "Should be in typing state after on_typing");
    }

    #[test]
    fn test_typing_on_inactivity() {
        let manager = TypingManager::new();
        manager.on_typing();
        manager.on_inactivity();
        let is_typing = manager.is_typing.lock().unwrap();
        assert!(!*is_typing, "Should exit typing state on inactivity");
    }

    #[test]
    fn test_should_stop_typing() {
        let manager = TypingManager::new();
        manager.on_typing();
        
        // Should not immediately be ready to stop
        assert!(!manager.should_stop_typing(), "Should not stop immediately");
        
        // Wait for debounce time
        std::thread::sleep(Duration::from_millis(TYPING_DEBOUNCE_MS + 100));
        assert!(manager.should_stop_typing(), "Should be ready to stop after debounce");
    }
}
