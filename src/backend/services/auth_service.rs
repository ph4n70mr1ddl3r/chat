//! Authentication service for user account management and JWT token generation
//!
//! Handles user creation, password validation, hashing, and JWT token generation/verification.

use crate::models::User;
use crate::validators;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use sha2::{Digest, Sha256};
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
    revoked_tokens: Arc<RwLock<HashMap<String, i64>>>,
    _cleanup_handle: Option<Arc<tokio::task::JoinHandle<()>>>,
}

/// JWT token expiration time in seconds (1 hour)
const TOKEN_EXPIRATION_SECONDS: i64 = 3600;

/// Default token scopes
const DEFAULT_SCOPES: [&str; 2] = ["send", "receive"];

impl AuthService {
    /// Create a new authentication service with the given secret key
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret,
            revoked_tokens: Arc::new(RwLock::new(HashMap::new())),
            _cleanup_handle: None,
        }
    }

    /// Create a new authentication service with periodic cleanup of revoked tokens
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
                tokens.retain(|_, exp| *exp > now);
                let removed = before_count - tokens.len();
                if removed > 0 {
                    info!(target: "auth", removed = removed, remaining = tokens.len(), "Cleaned up expired revoked tokens");
                }
            }
        });
        
        Self {
            jwt_secret,
            revoked_tokens,
            _cleanup_handle: Some(Arc::new(cleanup_handle)),
        }
    }

    /// Revoke a token (add to blacklist with expiration time)
    /// Stores only the hash of the token for security
    pub async fn revoke_token(&self, token: &str) {
        let expiration = Utc::now().timestamp() + TOKEN_EXPIRATION_SECONDS + 60;
        let token_hash = hash_token(token);
        self.revoked_tokens.write().await.insert(token_hash, expiration);
    }

    /// Check if a token has been revoked
    pub async fn is_token_revoked(&self, token: &str) -> bool {
        let token_hash = hash_token(token);
        let tokens = self.revoked_tokens.read().await;
        if let Some(&exp) = tokens.get(&token_hash) {
            return exp > Utc::now().timestamp();
        }
        false
    }

    /// Clean up expired revoked tokens (call periodically)
    pub async fn cleanup_revoked_tokens(&self) {
        let mut tokens = self.revoked_tokens.write().await;
        let now = Utc::now().timestamp();
        let before_count = tokens.len();
        tokens.retain(|_, exp| *exp > now);
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
    /// Returns Err for any error case to avoid timing attacks that could
    /// distinguish between invalid hash format and password mismatch.
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
        verify(password, hash).map_err(|_| "Password verification failed".to_string())
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

        decode::<TokenClaims>(token, &key, &validation)
            .map(|data| data.claims)
            .map_err(|e| format!("Failed to verify token: {}", e))
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

        auth.revoke_token(&token).await;

        let result = auth.verify_token(&token).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("revoked"));
    }
}
