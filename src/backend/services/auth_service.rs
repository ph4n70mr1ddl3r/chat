//! Authentication service for user account management and JWT token generation
//!
//! Handles user creation, password validation, hashing, and JWT token generation/verification.

use crate::models::User;
use crate::validators;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use chat_shared::protocol::TokenClaims;

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Authentication service with token revocation support
pub struct AuthService {
    jwt_secret: String,
    /// Maps token_hash -> (user_id, expiration_timestamp)
    revoked_tokens: Arc<RwLock<HashMap<String, (String, i64)>>>,
    pool: Option<SqlitePool>,
    cleanup_handle: Option<Arc<tokio::task::JoinHandle<()>>>,
}

impl Drop for AuthService {
    fn drop(&mut self) {
        if let Some(handle) = self.cleanup_handle.take() {
            if let Some(handle) = Arc::into_inner(handle) {
                handle.abort();
            }
        }
    }
}

/// JWT token expiration time in seconds (1 hour)
const TOKEN_EXPIRATION_SECONDS: i64 = 3600;

/// Default token scopes
const DEFAULT_SCOPES: [&str; 2] = ["send", "receive"];

impl AuthService {
    /// Create a new authentication service with the given secret key
    #[must_use]
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret,
            revoked_tokens: Arc::new(RwLock::new(HashMap::new())),
            pool: None,
            cleanup_handle: None,
        }
    }

    /// Create a new authentication service with database persistence for token revocation
    #[must_use]
    pub fn with_pool(jwt_secret: String, pool: SqlitePool) -> Self {
        Self {
            jwt_secret,
            revoked_tokens: Arc::new(RwLock::new(HashMap::new())),
            pool: Some(pool),
            cleanup_handle: None,
        }
    }

    /// Create a new authentication service with periodic cleanup of revoked tokens
    #[must_use]
    pub fn with_cleanup(jwt_secret: String) -> Self {
        let revoked_tokens = Arc::new(RwLock::new(HashMap::new()));
        let tokens_clone = revoked_tokens.clone();
        
        let cleanup_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                let mut tokens = tokens_clone.write().await;
                let now = chrono::Utc::now().timestamp();
                let before_count = tokens.len();
                tokens.retain(|_, (_, exp)| *exp > now);
                let removed = before_count - tokens.len();
                if removed > 0 {
                    info!(target: "auth", removed = removed, remaining = tokens.len(), "Cleaned up expired revoked tokens");
                }
            }
        });
        
        Self {
            jwt_secret,
            revoked_tokens,
            pool: None,
            cleanup_handle: Some(Arc::new(cleanup_handle)),
        }
    }

    /// Create a new authentication service with database persistence and periodic cleanup
    pub fn with_pool_and_cleanup(jwt_secret: String, pool: SqlitePool) -> Self {
        let revoked_tokens = Arc::new(RwLock::new(HashMap::new()));
        let tokens_clone = revoked_tokens.clone();
        let pool_clone = pool.clone();
        
        let cleanup_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                let now = chrono::Utc::now().timestamp();
                
                // Clean up in-memory tokens
                let mut tokens = tokens_clone.write().await;
                let before_count = tokens.len();
                tokens.retain(|_, (_, exp)| *exp > now);
                let removed = before_count - tokens.len();
                
                // Clean up database tokens
                if let Err(e) = sqlx::query("DELETE FROM revoked_tokens WHERE expires_at < ?")
                    .bind(now)
                    .execute(&pool_clone)
                    .await
                {
                    warn!(target: "auth", error = %e, "Failed to clean up expired revoked tokens from database");
                }
                
                if removed > 0 {
                    info!(target: "auth", removed = removed, remaining = tokens.len(), "Cleaned up expired revoked tokens");
                }
            }
        });
        
        Self {
            jwt_secret,
            revoked_tokens,
            pool: Some(pool),
            cleanup_handle: Some(Arc::new(cleanup_handle)),
        }
    }

    /// Load revoked tokens from database into memory on startup
    pub async fn load_revoked_tokens(&self) -> Result<(), String> {
        let Some(pool) = &self.pool else {
            return Ok(());
        };

        let now = chrono::Utc::now().timestamp();
        let rows: Vec<(String, String, i64)> = sqlx::query_as(
            "SELECT token_hash, user_id, expires_at FROM revoked_tokens WHERE expires_at > ?"
        )
        .bind(now)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to load revoked tokens: {e}"))?;

        let mut tokens = self.revoked_tokens.write().await;
        for (token_hash, user_id, expires_at) in rows {
            tokens.insert(token_hash, (user_id, expires_at));
        }

        info!(target: "auth", count = tokens.len(), "Loaded revoked tokens from database");
        Ok(())
    }

    /// Revoke a token (add to blacklist with expiration time)
    /// Stores only the hash of the token for security
    /// Persists to database if pool is configured
    pub async fn revoke_token(&self, token: &str, user_id: &str) {
        let expiration = Utc::now().timestamp() + TOKEN_EXPIRATION_SECONDS + 60;
        let token_hash = hash_token(token);
        self.revoked_tokens.write().await.insert(token_hash.clone(), (user_id.to_string(), expiration));

        // Persist to database if pool is available
        if let Some(pool) = &self.pool {
            if let Err(e) = self.revoke_token_in_db(&token_hash, user_id, expiration, pool).await {
                warn!(target: "auth", error = %e, "Failed to persist revoked token to database");
            }
        }
    }

    /// Revoke all tokens for a user (used when password is changed)
    pub async fn revoke_all_tokens_for_user(&self, user_id: &str) {
        // Remove from in-memory cache - filter by user_id
        let mut tokens = self.revoked_tokens.write().await;
        let before_count = tokens.len();
        tokens.retain(|_, (uid, _)| uid != user_id);
        let removed = before_count - tokens.len();
        drop(tokens);

        if removed > 0 {
            info!(target: "auth", user_id = %user_id, removed = removed, "Revoked tokens for user in memory");
        }

        // Delete from database if pool is available
        if let Some(pool) = &self.pool {
            if let Err(e) = sqlx::query("DELETE FROM revoked_tokens WHERE user_id = ?")
                .bind(user_id)
                .execute(pool)
                .await
            {
                warn!(target: "auth", error = %e, user_id = %user_id, "Failed to delete revoked tokens for user");
            }
        }
    }

    async fn revoke_token_in_db(&self, token_hash: &str, user_id: &str, expires_at: i64, pool: &SqlitePool) -> Result<(), String> {
        sqlx::query(
            "INSERT OR IGNORE INTO revoked_tokens (token_hash, user_id, revoked_at, expires_at) VALUES (?, ?, ?, ?)"
        )
        .bind(token_hash)
        .bind(user_id)
        .bind(chrono::Utc::now().timestamp())
        .bind(expires_at)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to insert revoked token: {e}"))?;
        Ok(())
    }

    /// Check if a token has been revoked
    /// Checks in-memory cache first, then database if pool is configured
    pub async fn is_token_revoked(&self, token: &str) -> bool {
        let token_hash = hash_token(token);
        let tokens = self.revoked_tokens.read().await;
        if let Some((_, exp)) = tokens.get(&token_hash) {
            return *exp > Utc::now().timestamp();
        }
        drop(tokens);

        // Check database if not found in memory
        if let Some(pool) = &self.pool {
            if let Ok(Some((user_id, exp))) = self.check_token_revoked_in_db(&token_hash, pool).await {
                // Cache for future lookups
                self.revoked_tokens.write().await.insert(token_hash, (user_id, exp));
                return true;
            }
        }
        false
    }

    async fn check_token_revoked_in_db(&self, token_hash: &str, pool: &SqlitePool) -> Result<Option<(String, i64)>, String> {
        let now = chrono::Utc::now().timestamp();
        let result: Option<(String, i64)> = sqlx::query_as(
            "SELECT user_id, expires_at FROM revoked_tokens WHERE token_hash = ? AND expires_at > ?"
        )
        .bind(token_hash)
        .bind(now)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to check revoked token: {e}"))?;
        Ok(result)
    }

    /// Clean up expired revoked tokens (call periodically)
    pub async fn cleanup_revoked_tokens(&self) {
        let mut tokens = self.revoked_tokens.write().await;
        let now = Utc::now().timestamp();
        let before_count = tokens.len();
        tokens.retain(|_, (_, exp)| *exp > now);
        let removed = before_count - tokens.len();
        if removed > 0 {
            info!(target: "auth", removed = removed, remaining = tokens.len(), "Cleaned up expired revoked tokens");
        }
    }

    /// Hash a password with bcrypt
    ///
    /// Returns the password_hash which includes salt internally.
    ///
    /// # Errors
    /// Returns error if password validation fails or bcrypt hashing encounters an error.
    pub fn hash_password(password: &str) -> Result<String, String> {
        // Validate password first
        validators::validate_password(password).map_err(|e| e.to_string())?;

        // Hash with bcrypt (DEFAULT_COST = 12)
        let hashed =
            hash(password, DEFAULT_COST).map_err(|e| format!("Failed to hash password: {}", e))?;

        Ok(hashed)
    }

    /// Verify a password against a hash
    ///
    /// Returns Ok(true) if password matches, Ok(false) if not.
    /// Returns Err for any error case (e.g., invalid hash format).
    /// Note: bcrypt::verify() internally uses constant-time comparison to prevent
    /// timing attacks, so no additional protection is needed here.
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
        verify(password, hash).map_err(|e| {
            tracing::debug!("Password verification error: {}", e);
            "Password verification failed".to_string()
        })
    }

    /// Create a new user with validated password
    pub async fn create_user(&self, username: String, password: String) -> Result<User, String> {
        // Validate password
        validators::validate_password(&password).map_err(|e| e.to_string())?;

        // Hash password
        let password_hash = Self::hash_password(&password)?;

        // Create user (note: actual DB save happens in the handler)
        let user = User::new(username, password_hash);
        info!(
            target: "auth",
            event = "auth.signup",
            username = %user.username,
            user_id = %user.id,
            "Validated credentials and constructed new user"
        );
        Ok(user)
    }

    /// Generate JWT token for a user
    pub fn generate_token(&self, user_id: String) -> Result<(String, u64), String> {
        let now = Utc::now().timestamp_millis() as u64;
        let expiration = now + (TOKEN_EXPIRATION_SECONDS * 1000) as u64;

        let claims = TokenClaims {
            sub: user_id,
            iss: "chat-app".to_string(),
            aud: "chat-app".to_string(),
            iat: now,
            exp: expiration,
            scopes: DEFAULT_SCOPES.iter().map(|s| s.to_string()).collect(),
        };

        let key = EncodingKey::from_secret(self.jwt_secret.as_bytes());

        encode(&Header::default(), &claims, &key)
            .map(|token| (token, TOKEN_EXPIRATION_SECONDS as u64))
            .map_err(|e| format!("Failed to generate token: {}", e))
    }

    /// Verify user login credentials with structured logging around outcomes.
    pub fn verify_login(&self, username: &str, password: &str, hash: &str) -> Result<bool, String> {
        match Self::verify_password(password, hash) {
            Ok(true) => {
                info!(
                    target: "auth",
                    event = "auth.login",
                    username = %username,
                    outcome = "success",
                    "Password verified"
                );
                Ok(true)
            }
            Ok(false) => {
                warn!(
                    target: "auth",
                    event = "auth.login",
                    username = %username,
                    outcome = "failed",
                    reason = "invalid_password"
                );
                Ok(false)
            }
            Err(e) => {
                warn!(
                    target: "auth",
                    event = "auth.login",
                    username = %username,
                    outcome = "error",
                    error = %e
                );
                Err(e)
            }
        }
    }

    /// Verify and decode a JWT token
    pub async fn verify_token(&self, token: &str) -> Result<TokenClaims, String> {
        if self.is_token_revoked(token).await {
            return Err("Token has been revoked".to_string());
        }

        let key = DecodingKey::from_secret(self.jwt_secret.as_bytes());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&["chat-app"]);
        validation.set_issuer(&["chat-app"]);
        validation.validate_nbf = true;

        match decode::<TokenClaims>(token, &key, &validation) {
            Ok(data) => Ok(data.claims),
            Err(e) => {
                let error_detail = match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Token has expired",
                    jsonwebtoken::errors::ErrorKind::InvalidSignature => "Invalid token signature",
                    jsonwebtoken::errors::ErrorKind::InvalidToken => "Malformed token",
                    jsonwebtoken::errors::ErrorKind::InvalidIssuer => "Invalid token issuer",
                    jsonwebtoken::errors::ErrorKind::InvalidAudience => "Invalid token audience",
                    _ => "Token verification failed",
                };
                warn!(target: "auth", event = "auth.token.verify_failed", error = ?e.kind(), "Token verification failed");
                Err(error_detail.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_password_valid() {
        assert!(validators::validate_password("TestPass123!").is_ok());
        assert!(validators::validate_password("AnotherPassword456@").is_ok());
    }

    #[test]
    fn test_validate_password_too_short() {
        assert!(validators::validate_password("Test1!").is_err());
    }

    #[test]
    fn test_validate_password_no_uppercase() {
        assert!(validators::validate_password("testpass123!").is_err());
    }

    #[test]
    fn test_validate_password_no_lowercase() {
        assert!(validators::validate_password("TESTPASS123!").is_err());
    }

    #[test]
    fn test_validate_password_no_digit() {
        assert!(validators::validate_password("TestPass!!").is_err());
    }

    #[test]
    fn test_validate_password_no_special_char() {
        assert!(validators::validate_password("TestPass123").is_err());
    }

    #[test]
    fn test_hash_password() {
        let hash = AuthService::hash_password("TestPass123!").unwrap();

        assert_ne!(hash, "TestPass123!");
    }

    #[test]
    fn test_verify_password_correct() {
        let hash = AuthService::hash_password("TestPass123!").unwrap();
        assert!(AuthService::verify_password("TestPass123!", &hash).unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let hash = AuthService::hash_password("TestPass123!").unwrap();
        assert!(!AuthService::verify_password("WrongPassword123!", &hash).unwrap());
    }

    #[tokio::test]
    async fn test_create_user() {
        let auth = AuthService::new(uuid::Uuid::new_v4().to_string());
        let user = auth
            .create_user("testuser".to_string(), "TestPass123!".to_string())
            .await;

        assert!(user.is_ok());
        let user = user.unwrap();
        assert_eq!(user.username, "testuser");
        assert!(!user.password_hash.is_empty());
    }

    #[test]
    fn test_generate_token() {
        let auth = AuthService::new(uuid::Uuid::new_v4().to_string());
        let result = auth.generate_token("user123".to_string());

        assert!(result.is_ok());
        let (token, exp) = result.unwrap();
        assert!(!token.is_empty());
        assert_eq!(exp, TOKEN_EXPIRATION_SECONDS as u64);
    }

    #[tokio::test]
    async fn test_verify_token_valid() {
        let auth = AuthService::new(uuid::Uuid::new_v4().to_string());
        let (token, _) = auth.generate_token("user123".to_string()).unwrap();

        let claims = auth.verify_token(&token).await;
        assert!(claims.is_ok());
        assert_eq!(claims.unwrap().sub, "user123");
    }

    #[tokio::test]
    async fn test_verify_token_invalid() {
        let auth = AuthService::new(uuid::Uuid::new_v4().to_string());
        let result = auth.verify_token("invalid.token.here").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_token_wrong_secret() {
        let auth1 = AuthService::new(uuid::Uuid::new_v4().to_string());
        let (token, _) = auth1.generate_token("user123".to_string()).unwrap();

        let auth2 = AuthService::new(uuid::Uuid::new_v4().to_string());
        let result = auth2.verify_token(&token).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_token_revocation() {
        let auth = AuthService::new(uuid::Uuid::new_v4().to_string());
        let (token, _) = auth.generate_token("user123".to_string()).unwrap();

        assert!(auth.verify_token(&token).await.is_ok());

        auth.revoke_token(&token, "user123").await;

        let result = auth.verify_token(&token).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("revoked"));
    }
}
