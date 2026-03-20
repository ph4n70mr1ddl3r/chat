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
use tokio::sync::{broadcast, RwLock};
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
    /// Maps `token_hash` -> (`user_id`, `expiration_timestamp`)
    revoked_tokens: Arc<RwLock<HashMap<String, (String, i64)>>>,
    /// Maps `user_id` -> timestamp (seconds since epoch) after which tokens are valid
    /// Used to invalidate all tokens for a user (e.g., on password change)
    tokens_valid_after: Arc<RwLock<HashMap<String, i64>>>,
    pool: Option<SqlitePool>,
    cleanup_handle: Option<Arc<tokio::task::JoinHandle<()>>>,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl Drop for AuthService {
    fn drop(&mut self) {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }
        if let Some(handle) = self.cleanup_handle.take() {
            handle.abort();
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
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            jwt_secret,
            revoked_tokens: Arc::new(RwLock::new(HashMap::new())),
            tokens_valid_after: Arc::new(RwLock::new(HashMap::new())),
            pool: None,
            cleanup_handle: None,
            shutdown_tx: Some(shutdown_tx),
        }
    }

    /// Create a new authentication service with database persistence for token revocation
    #[must_use]
    pub fn with_pool(jwt_secret: String, pool: SqlitePool) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            jwt_secret,
            revoked_tokens: Arc::new(RwLock::new(HashMap::new())),
            tokens_valid_after: Arc::new(RwLock::new(HashMap::new())),
            pool: Some(pool),
            cleanup_handle: None,
            shutdown_tx: Some(shutdown_tx),
        }
    }

    /// Create a new authentication service with periodic cleanup of revoked tokens
    #[must_use]
    pub fn with_cleanup(jwt_secret: String) -> Self {
        let revoked_tokens = Arc::new(RwLock::new(HashMap::new()));
        let tokens_valid_after = Arc::new(RwLock::new(HashMap::new()));
        let tokens_clone = revoked_tokens.clone();
        let tokens_valid_after_clone = tokens_valid_after.clone();
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        
        let cleanup_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!(target: "auth", "Cleanup task shutting down");
                        break;
                    }
                    _ = interval.tick() => {
                        let now = chrono::Utc::now().timestamp();
                        
                        // Clean up expired revoked tokens
                        let mut tokens = tokens_clone.write().await;
                        let before_count = tokens.len();
                        tokens.retain(|_, (_, exp)| *exp > now);
                        let removed = before_count - tokens.len();
                        if removed > 0 {
                            info!(target: "auth", removed = removed, remaining = tokens.len(), "Cleaned up expired revoked tokens");
                        }
                        drop(tokens);
                        
                        // Clean up stale tokens_valid_after entries
                        // Entries older than TOKEN_EXPIRATION_SECONDS are no longer needed
                        // since all tokens they would invalidate have already expired
                        let mut valid_after = tokens_valid_after_clone.write().await;
                        let before_count = valid_after.len();
                        let cutoff = now - TOKEN_EXPIRATION_SECONDS;
                        valid_after.retain(|_, ts| *ts > cutoff);
                        let removed = before_count - valid_after.len();
                        if removed > 0 {
                            info!(target: "auth", removed = removed, remaining = valid_after.len(), "Cleaned up stale token invalidation timestamps");
                        }
                    }
                }
            }
        });
        
        Self {
            jwt_secret,
            revoked_tokens,
            tokens_valid_after,
            pool: None,
            cleanup_handle: Some(Arc::new(cleanup_handle)),
            shutdown_tx: Some(shutdown_tx),
        }
    }

    /// Create a new authentication service with database persistence and periodic cleanup
    #[must_use]
    pub fn with_pool_and_cleanup(jwt_secret: String, pool: SqlitePool) -> Self {
        let revoked_tokens = Arc::new(RwLock::new(HashMap::new()));
        let tokens_valid_after = Arc::new(RwLock::new(HashMap::new()));
        let tokens_clone = revoked_tokens.clone();
        let tokens_valid_after_clone = tokens_valid_after.clone();
        let pool_clone = pool.clone();
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        
        let cleanup_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!(target: "auth", "Cleanup task shutting down");
                        break;
                    }
                    _ = interval.tick() => {
                        let now = chrono::Utc::now().timestamp();
                        
                        // Clean up in-memory revoked tokens
                        let mut tokens = tokens_clone.write().await;
                        let before_count = tokens.len();
                        tokens.retain(|_, (_, exp)| *exp > now);
                        let removed = before_count - tokens.len();
                        drop(tokens);
                        
                        // Clean up database tokens
                        if let Err(e) = sqlx::query("DELETE FROM revoked_tokens WHERE expires_at < ?")
                            .bind(now)
                            .execute(&pool_clone)
                            .await
                        {
                            warn!(target: "auth", error = %e, "Failed to clean up expired revoked tokens from database");
                        }
                        
                        if removed > 0 {
                            info!(target: "auth", removed = removed, "Cleaned up expired revoked tokens");
                        }
                        
                        // Clean up stale tokens_valid_after entries
                        let mut valid_after = tokens_valid_after_clone.write().await;
                        let before_count = valid_after.len();
                        let cutoff = now - TOKEN_EXPIRATION_SECONDS;
                        valid_after.retain(|_, ts| *ts > cutoff);
                        let removed = before_count - valid_after.len();
                        if removed > 0 {
                            info!(target: "auth", removed = removed, remaining = valid_after.len(), "Cleaned up stale token invalidation timestamps");
                        }
                    }
                }
            }
        });
        
        Self {
            jwt_secret,
            revoked_tokens,
            tokens_valid_after,
            pool: Some(pool),
            cleanup_handle: Some(Arc::new(cleanup_handle)),
            shutdown_tx: Some(shutdown_tx),
        }
    }

    /// Load revoked tokens from database into memory on startup
    ///
    /// # Errors
    /// Returns an error string if the database query fails.
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

    /// Revoke a token by its JTI (add to blacklist with expiration time)
    /// Persists to database if pool is configured
    pub async fn revoke_token_by_jti(&self, jti: &str, user_id: &str) {
        let expiration = Utc::now().timestamp() + TOKEN_EXPIRATION_SECONDS + 60;
        self.revoked_tokens.write().await.insert(jti.to_string(), (user_id.to_string(), expiration));

        // Persist to database if pool is available
        if let Some(pool) = &self.pool {
            if let Err(e) = self.revoke_token_in_db(jti, user_id, expiration, pool).await {
                warn!(target: "auth", error = %e, "Failed to persist revoked token to database");
            }
        }
    }

    /// Revoke a token (add to blacklist with expiration time)
    /// For new tokens with JTI, revokes by JTI for efficiency.
    /// For legacy tokens without JTI, revokes by token hash.
    /// Persists to database if pool is configured
    pub async fn revoke_token(&self, token: &str, user_id: &str) {
        let expiration = Utc::now().timestamp() + TOKEN_EXPIRATION_SECONDS + 60;
        
        // Try to extract JTI and revoke by it for new tokens
        // SAFETY: This is called from logout handlers after verify_token() has validated the token.
        // We're extracting the JTI for efficient revocation lookup.
        #[allow(deprecated)]
        if let Ok(claims) = self.decode_token_without_verification(token) {
            if !claims.jti.is_empty() {
                self.revoked_tokens.write().await.insert(claims.jti.clone(), (user_id.to_string(), expiration));
                
                if let Some(pool) = &self.pool {
                    if let Err(e) = self.revoke_token_in_db(&claims.jti, user_id, expiration, pool).await {
                        warn!(target: "auth", error = %e, "Failed to persist revoked token to database");
                    }
                }
                return;
            }
        }
        
        // Fallback: revoke by token hash for legacy tokens
        let token_hash = hash_token(token);
        self.revoked_tokens.write().await.insert(token_hash.clone(), (user_id.to_string(), expiration));

        if let Some(pool) = &self.pool {
            if let Err(e) = self.revoke_token_in_db(&token_hash, user_id, expiration, pool).await {
                warn!(target: "auth", error = %e, "Failed to persist revoked token to database");
            }
        }
    }

    /// Invalidate all tokens for a user (used when password is changed).
    ///
    /// This sets a "tokens valid after" timestamp for the user. Any token
    /// issued before this timestamp will be rejected during verification.
    /// We add 1 second to the current time to ensure tokens generated in the
    /// same second as revocation are also invalidated.
    pub async fn revoke_all_tokens_for_user(&self, user_id: &str) {
        let now_secs = Utc::now().timestamp() + 1;
        
        self.tokens_valid_after.write().await.insert(user_id.to_string(), now_secs);
        
        info!(target: "auth", user_id = %user_id, "Invalidated all tokens for user");
    }
    
    /// Check if a token was issued before the user's "tokens valid after" timestamp
    async fn is_token_issued_before_invalidation(&self, user_id: &str, token_iat: u64) -> bool {
        let tokens_valid_after = self.tokens_valid_after.read().await;
        if let Some(&valid_after_secs) = tokens_valid_after.get(user_id) {
            return token_iat.cast_signed() < valid_after_secs;
        }
        false
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

    /// Check if a token has been revoked by its JTI
    /// Checks in-memory cache first, then database if pool is configured
    pub async fn is_jti_revoked(&self, jti: &str) -> bool {
        let tokens = self.revoked_tokens.read().await;
        if let Some((_, exp)) = tokens.get(jti) {
            return *exp > Utc::now().timestamp();
        }
        drop(tokens);

        // Check database if not found in memory
        if let Some(pool) = &self.pool {
            if let Ok(Some((user_id, exp))) = self.check_token_revoked_in_db(jti, pool).await {
                // Cache for future lookups
                self.revoked_tokens.write().await.insert(jti.to_string(), (user_id, exp));
                return true;
            }
        }
        false
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
    /// Returns the `password_hash` which includes salt internally.
    ///
    /// # Errors
    /// Returns error if password validation fails or bcrypt hashing encounters an error.
    pub fn hash_password(password: &str) -> Result<String, String> {
        // Validate password first
        validators::validate_password(password).map_err(|e| e.clone())?;

        // Hash with bcrypt (DEFAULT_COST = 12)
        let hashed =
            hash(password, DEFAULT_COST).map_err(|e| format!("Failed to hash password: {e}"))?;

        Ok(hashed)
    }

    /// Verify a password against a hash
    ///
    /// Returns Ok(true) if password matches, Ok(false) if not.
    /// Returns Err for any error case (e.g., invalid hash format).
    /// Note: `bcrypt::verify()` internally uses constant-time comparison to prevent
    /// timing attacks, so no additional protection is needed here.
    ///
    /// # Errors
    /// Returns an error string if password verification fails due to invalid hash format.
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
        verify(password, hash).map_err(|e| {
            tracing::debug!("Password verification error: {}", e);
            "Password verification failed".to_string()
        })
    }

    /// Create a new user with validated password
    ///
    /// # Errors
    /// Returns an error string if password validation fails.
    pub fn create_user(&self, username: String, password: &str) -> Result<User, String> {
        validators::validate_password(password).map_err(|e| e.clone())?;

        let password_hash = Self::hash_password(password)?;

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
    ///
    /// # Errors
    /// Returns an error string if token encoding fails.
    pub fn generate_token(&self, user_id: String) -> Result<(String, u64), String> {
        let now = Utc::now().timestamp().max(0).cast_unsigned();
        let expiration = now + TOKEN_EXPIRATION_SECONDS as u64;
        let jti = uuid::Uuid::new_v4().to_string();

        let claims = TokenClaims {
            sub: user_id,
            iss: "chat-app".to_string(),
            aud: "chat-app".to_string(),
            iat: now,
            nbf: now,
            exp: expiration,
            jti,
            scopes: DEFAULT_SCOPES.iter().map(std::string::ToString::to_string).collect(),
        };

        let key = EncodingKey::from_secret(self.jwt_secret.as_bytes());

        encode(&Header::default(), &claims, &key)
            .map(|token| (token, TOKEN_EXPIRATION_SECONDS as u64))
            .map_err(|e| format!("Failed to generate token: {e}"))
    }

    /// Verify user login credentials with structured logging around outcomes.
    ///
    /// # Errors
    /// Returns an error string if password verification fails.
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

    /// Decode a token without verification (for extracting claims like JTI)
    ///
    /// # Safety
    ///
    /// This method MUST NOT be called on untrusted tokens. It should only be
    /// used when the token has already been verified through `verify_token()`.
    /// The primary use case is extracting the JTI for token revocation during logout.
    ///
    /// Calling this on an untrusted token could allow an attacker to inject
    /// arbitrary claims into your system.
    ///
    /// # Deprecation
    ///
    /// This method is deprecated for security reasons. Consider using verified token
    /// claims from `verify_token()` instead. If you must use this, ensure the token
    /// has been pre-verified.
    #[deprecated(
        since = "0.2.0",
        note = "This method bypasses signature verification. Use verified claims from verify_token() when possible."
    )]
    /// Decode token without verification
    ///
    /// # Errors
    /// Returns an error string if token decoding fails.
    pub fn decode_token_without_verification(&self, token: &str) -> Result<TokenClaims, String> {
        #[cfg(debug_assertions)]
        tracing::warn!(
            target: "auth",
            "decode_token_without_verification called - ensure token was pre-verified"
        );

        let mut validation = Validation::new(Algorithm::HS256);
        // SAFETY: This method is deprecated and only called from `revoke_token()`
        // after the token has been verified via `verify_token()`. The token is
        // being decoded solely to extract the JTI for revocation purposes.
        // The caller (logout handler) has already validated the token signature.
        #[allow(deprecated)]
        validation.insecure_disable_signature_validation();
        validation.set_audience(&["chat-app"]);
        validation.set_issuer(&["chat-app"]);

        match decode::<TokenClaims>(token, &DecodingKey::from_secret(&[]), &validation) {
            Ok(data) => Ok(data.claims),
            Err(e) => Err(format!("Failed to decode token: {e}")),
        }
    }

    /// Verify and decode a JWT token
    ///
    /// # Errors
    /// Returns an error string if token verification fails.
    pub async fn verify_token(&self, token: &str) -> Result<TokenClaims, String> {
        let key = DecodingKey::from_secret(self.jwt_secret.as_bytes());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&["chat-app"]);
        validation.set_issuer(&["chat-app"]);
        validation.validate_nbf = true;

        let claims = match decode::<TokenClaims>(token, &key, &validation) {
            Ok(data) => data.claims,
            Err(e) => {
                // Use generic error message for most cases to prevent information leakage.
                // Only expose "expired" status which is not security-sensitive.
                let error_detail = match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Token has expired",
                    _ => "Invalid token",
                };
                warn!(target: "auth", event = "auth.token.verify_failed", error = ?e.kind(), "Token verification failed");
                return Err(error_detail.to_string());
            }
        };

        // Check if JTI has been revoked (primary check for new tokens)
        if !claims.jti.is_empty() && self.is_jti_revoked(&claims.jti).await {
            return Err("Token has been revoked".to_string());
        }

        // Fallback: check by token hash for legacy tokens without JTI
        if claims.jti.is_empty() && self.is_token_revoked(token).await {
            return Err("Token has been revoked".to_string());
        }

        // Check if token was issued before user's invalidation timestamp
        if self.is_token_issued_before_invalidation(&claims.sub, claims.iat).await {
            warn!(target: "auth", user_id = %claims.sub, "Token rejected due to user-wide invalidation");
            return Err("Token has been invalidated".to_string());
        }

        Ok(claims)
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

    #[test]
    fn test_create_user() {
        let auth = AuthService::new(uuid::Uuid::new_v4().to_string());
        let user = auth
            .create_user("testuser".to_string(), "TestPass123!");

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

    #[tokio::test]
    async fn test_revoke_all_tokens_for_user() {
        let auth = AuthService::new(uuid::Uuid::new_v4().to_string());
        
        // Generate multiple tokens for the same user
        let (token1, _) = auth.generate_token("user123".to_string()).unwrap();
        let (token2, _) = auth.generate_token("user123".to_string()).unwrap();
        
        // Both should be valid initially
        assert!(auth.verify_token(&token1).await.is_ok());
        assert!(auth.verify_token(&token2).await.is_ok());
        
        // Invalidate all tokens for user
        auth.revoke_all_tokens_for_user("user123").await;
        
        // Both should now be invalid
        let result1 = auth.verify_token(&token1).await;
        let result2 = auth.verify_token(&token2).await;
        assert!(result1.is_err());
        assert!(result1.unwrap_err().contains("invalidated"));
        assert!(result2.is_err());
        assert!(result2.unwrap_err().contains("invalidated"));
    }
    
    #[tokio::test]
    async fn test_revoke_all_tokens_does_not_affect_other_users() {
        let auth = AuthService::new(uuid::Uuid::new_v4().to_string());
        
        let (token_user1, _) = auth.generate_token("user1".to_string()).unwrap();
        let (token_user2, _) = auth.generate_token("user2".to_string()).unwrap();
        
        // Revoke all tokens for user1
        auth.revoke_all_tokens_for_user("user1").await;
        
        // user1's token should be invalid
        assert!(auth.verify_token(&token_user1).await.is_err());
        
        // user2's token should still be valid
        assert!(auth.verify_token(&token_user2).await.is_ok());
    }
    
    #[tokio::test]
    async fn test_new_token_after_revoke_all_is_valid() {
        let auth = AuthService::new(uuid::Uuid::new_v4().to_string());
        
        // Generate and then invalidate
        let (old_token, _) = auth.generate_token("user123".to_string()).unwrap();
        auth.revoke_all_tokens_for_user("user123").await;
        assert!(auth.verify_token(&old_token).await.is_err());
        
        // Wait for at least 2 seconds to ensure new token has a later timestamp
        // The revocation sets valid_after = now + 1sec, so we need to generate
        // the new token at least 2 seconds after the old token for it to be valid
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Generate new token after invalidation
        let (new_token, _) = auth.generate_token("user123".to_string()).unwrap();
        assert!(auth.verify_token(&new_token).await.is_ok());
    }
}
