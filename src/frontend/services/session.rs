//! Session storage and token management
//!
//! Handles JWT token storage, retrieval, and automatic refresh

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::fs;

/// Session data stored locally
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: String,
    pub username: String,
    pub token: String,
    pub expires_at: i64,
}

/// Session manager
pub struct SessionManager {
    session_file: PathBuf,
    current_session: Arc<Mutex<Option<SessionData>>>,
}

impl SessionManager {
    /// Create a new session manager
    ///
    /// Sessions are stored in the user's config directory:
    /// - Linux: ~/.config/chat-app/session.json
    /// - Windows: %APPDATA%/chat-app/session.json
    /// - macOS: ~/Library/Application Support/chat-app/session.json
    pub fn new() -> Self {
        let session_file = Self::get_session_file_path();

        Self {
            session_file,
            current_session: Arc::new(Mutex::new(None)),
        }
    }

    #[cfg(test)]
    fn new_with_session_file(session_file: PathBuf) -> Self {
        Self {
            session_file,
            current_session: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the session file path based on the OS
    fn get_session_file_path() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(appdata).join("chat-app").join("session.json")
        }

        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("chat-app")
                .join("session.json")
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home)
                .join(".config")
                .join("chat-app")
                .join("session.json")
        }
    }

    /// Save session with individual parameters (synchronous helper)
    pub fn save_session_sync(
        &self,
        user_id: &str,
        token: &str,
        username: &str,
        expires_at: i64,
    ) -> Result<(), String> {
        let session = SessionData {
            user_id: user_id.to_string(),
            token: token.to_string(),
            username: username.to_string(),
            expires_at,
        };

        // Ensure parent directory exists
        if let Some(parent) = self.session_file.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create session directory: {e}"))?;
        }

        // Serialize session
        let json = serde_json::to_string_pretty(&session)
            .map_err(|e| format!("Failed to serialize session: {e}"))?;

        // Write to file
        std::fs::write(&self.session_file, json)
            .map_err(|e| format!("Failed to write session file: {e}"))?;

        // Set restrictive file permissions (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) =
                std::fs::set_permissions(&self.session_file, std::fs::Permissions::from_mode(0o600))
            {
                tracing::warn!("Failed to set secure file permissions: {}. Session may have weaker permissions than expected.", e);
            }
        }

        // Update in-memory session
        *self.current_session.lock().expect("session mutex poisoned") = Some(session);

        Ok(())
    }

    /// Load session from disk
    ///
    /// Note: Currently unused in production code but maintained for future use
    /// in session restoration functionality. May be needed for auto-login features.
    #[allow(dead_code)]
    pub async fn load_session(&self) -> Result<Option<SessionData>, String> {
        // Check if file exists
        if !self.session_file.exists() {
            return Ok(None);
        }

        // Read file
        let contents = fs::read_to_string(&self.session_file)
            .await
            .map_err(|e| format!("Failed to read session file: {e}"))?;

        // Deserialize
        let session: SessionData = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse session file: {e}"))?;

        // Update in-memory session
        *self.current_session.lock().expect("session mutex poisoned") = Some(session.clone());

        Ok(Some(session))
    }

    /// Clear session (logout)
    ///
    /// Removes session from both memory and persistent storage. This is called
    /// when a user explicitly logs out or during session cleanup.
    pub async fn clear_session(&self) -> Result<(), String> {
        // Remove from memory
        *self
            .current_session
            .lock()
            .map_err(|e| format!("Failed to clear session from memory: {e}"))? = None;

        // Delete file if it exists
        if self.session_file.exists() {
            fs::remove_file(&self.session_file).await.map_err(|e| {
                format!(
                    "Failed to delete session file '{:?}': {e}",
                    self.session_file
                )
            })?;
        }

        Ok(())
    }

    /// Get current session from memory
    ///
    /// Returns the in-memory session data if available, None otherwise.
    /// This does not load from disk - use `get_session()` for that.
    pub fn get_current_session(&self) -> Option<SessionData> {
        self.current_session
            .lock()
            .map_err(|e| {
                tracing::error!("Failed to lock session mutex: {}", e);
                e
            })
            .ok()
            .and_then(|guard| guard.clone())
    }

    /// Get session (synchronous version that loads from disk if not in memory)
    pub fn get_session(&self) -> Result<Option<SessionData>, String> {
        // First check memory
        if let Some(session) = self.get_current_session() {
            return Ok(Some(session));
        }

        // If not in memory, try loading from disk synchronously
        if !self.session_file.exists() {
            return Ok(None);
        }

        let contents = std::fs::read_to_string(&self.session_file)
            .map_err(|e| format!("Failed to read session file: {e}"))?;

        let session: SessionData = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse session file: {e}"))?;

        // Update in-memory session
        *self.current_session.lock().expect("session mutex poisoned") = Some(session.clone());

        Ok(Some(session))
    }

    /// Check if user is logged in with valid (non-expired) token
    ///
    /// Returns true if a session exists and has not yet expired.
    pub fn is_logged_in(&self) -> bool {
        if let Some(session) = self.get_current_session() {
            let now = chrono::Utc::now().timestamp();
            session.expires_at > now
        } else {
            false
        }
    }
}

// Global session manager instance (lazy static)
use std::sync::OnceLock;
static SESSION_MANAGER: OnceLock<SessionManager> = OnceLock::new();

/// Get the global session manager instance
pub fn get_session_manager() -> &'static SessionManager {
    SESSION_MANAGER.get_or_init(SessionManager::new)
}

/// Helper function to get the current token
pub fn get_token() -> Option<String> {
    get_session_manager().get_current_session().map(|s| s.token)
}

/// Helper function to check if logged in
///
/// Note: Currently unused in production code but maintained for potential
/// use in session state checks throughout the application.
#[allow(dead_code)]
pub fn is_logged_in() -> bool {
    get_session_manager().is_logged_in()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_data_serialization() {
        let session = SessionData {
            user_id: "user123".to_string(),
            username: "alice".to_string(),
            token: "eyJhbGc...".to_string(),
            expires_at: 1702657890,
        };

        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("user123"));
        assert!(json.contains("alice"));

        let deserialized: SessionData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.user_id, "user123");
        assert_eq!(deserialized.username, "alice");
    }

    #[tokio::test]
    async fn test_session_manager_save_and_load() {
        let unique = {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            format!("{}-{}", std::process::id(), now)
        };
        let session_dir = std::env::temp_dir().join(format!("chat-app-test-session-{}", unique));
        let session_file = session_dir.join("session.json");
        let manager = SessionManager::new_with_session_file(session_file);

        let session = SessionData {
            user_id: "test_user".to_string(),
            username: "testuser".to_string(),
            token: "test_token".to_string(),
            expires_at: chrono::Utc::now().timestamp() + 3600,
        };

        // Save
        manager
            .save_session_sync(
                &session.user_id,
                &session.token,
                &session.username,
                session.expires_at,
            )
            .unwrap();

        // Load
        let loaded = manager.load_session().await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().user_id, "test_user");

        // Clear
        manager.clear_session().await.unwrap();
        assert!(manager.get_current_session().is_none());

        // Best-effort cleanup
        let _ = std::fs::remove_dir_all(session_dir);
    }
}
