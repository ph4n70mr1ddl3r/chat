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

    /// Save session to disk
    pub async fn save_session(&self, session: SessionData) -> Result<(), String> {
        // Ensure parent directory exists
        if let Some(parent) = self.session_file.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create session directory: {}", e))?;
        }

        // Serialize session
        let json = serde_json::to_string_pretty(&session)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;

        // Write to file
        fs::write(&self.session_file, json)
            .await
            .map_err(|e| format!("Failed to write session file: {}", e))?;

        // Update in-memory session
        *self.current_session.lock().unwrap() = Some(session);

        Ok(())
    }
    
    /// Save session with individual parameters (synchronous helper)
    pub fn save_session_sync(&self, user_id: &str, token: &str, username: &str, expires_at: i64) -> Result<(), String> {
        let session = SessionData {
            user_id: user_id.to_string(),
            token: token.to_string(),
            username: username.to_string(),
            expires_at,
        };
        
        // Ensure parent directory exists
        if let Some(parent) = self.session_file.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create session directory: {}", e))?;
        }

        // Serialize session
        let json = serde_json::to_string_pretty(&session)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;

        // Write to file
        std::fs::write(&self.session_file, json)
            .map_err(|e| format!("Failed to write session file: {}", e))?;

        // Update in-memory session
        *self.current_session.lock().unwrap() = Some(session);

        Ok(())
    }

    /// Load session from disk
    pub async fn load_session(&self) -> Result<Option<SessionData>, String> {
        // Check if file exists
        if !self.session_file.exists() {
            return Ok(None);
        }

        // Read file
        let contents = fs::read_to_string(&self.session_file)
            .await
            .map_err(|e| format!("Failed to read session file: {}", e))?;

        // Deserialize
        let session: SessionData = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse session file: {}", e))?;

        // Update in-memory session
        *self.current_session.lock().unwrap() = Some(session.clone());

        Ok(Some(session))
    }

    /// Clear session (logout)
    pub async fn clear_session(&self) -> Result<(), String> {
        // Remove from memory
        *self.current_session.lock().unwrap() = None;

        // Delete file if it exists
        if self.session_file.exists() {
            fs::remove_file(&self.session_file)
                .await
                .map_err(|e| format!("Failed to delete session file: {}", e))?;
        }

        Ok(())
    }

    /// Get current session from memory
    pub fn get_current_session(&self) -> Option<SessionData> {
        self.current_session.lock().unwrap().clone()
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
            .map_err(|e| format!("Failed to read session file: {}", e))?;
        
        let session: SessionData = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse session file: {}", e))?;
        
        // Update in-memory session
        *self.current_session.lock().unwrap() = Some(session.clone());
        
        Ok(Some(session))
    }

    /// Check if token is expired or will expire soon (within 5 minutes)
    pub fn should_refresh_token(&self) -> bool {
        if let Some(session) = self.get_current_session() {
            let now = chrono::Utc::now().timestamp();
            let expires_in = session.expires_at - now;
            
            // Refresh if expires in less than 5 minutes
            expires_in < 300
        } else {
            false
        }
    }

    /// Check if user is logged in with valid token
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
    SESSION_MANAGER.get_or_init(|| SessionManager::new())
}

/// Helper function to get the current token
pub fn get_token() -> Option<String> {
    get_session_manager()
        .get_current_session()
        .map(|s| s.token)
}

/// Helper function to check if logged in
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
        let manager = SessionManager::new();
        
        let session = SessionData {
            user_id: "test_user".to_string(),
            username: "testuser".to_string(),
            token: "test_token".to_string(),
            expires_at: chrono::Utc::now().timestamp() + 3600,
        };

        // Save
        manager.save_session(session.clone()).await.unwrap();
        
        // Load
        let loaded = manager.load_session().await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().user_id, "test_user");
        
        // Clear
        manager.clear_session().await.unwrap();
        assert!(manager.get_current_session().is_none());
    }
}
