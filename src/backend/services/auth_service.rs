//! Authentication service for user account management and JWT token generation
//!
//! Handles user creation, password validation, hashing, and JWT token generation/verification.

use crate::models::User;
use crate::validators;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use tracing::{info, warn};

use chat_shared::protocol::TokenClaims;

/// Authentication service
pub struct AuthService {
    jwt_secret: String,
}

/// JWT token expiration time in seconds (1 hour)
const TOKEN_EXPIRATION_SECONDS: i64 = 3600;

/// Default token scopes
const DEFAULT_SCOPES: [&str; 2] = ["send", "receive"];

impl AuthService {
    /// Create a new authentication service with the given secret key
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
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
            .map(|token| (token, expiration))
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
    pub fn verify_token(&self, token: &str) -> Result<TokenClaims, String> {
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
        assert!(exp > Utc::now().timestamp_millis() as u64);
    }

    #[test]
    fn test_verify_token_valid() {
        let auth = AuthService::new(uuid::Uuid::new_v4().to_string());
        let (token, _) = auth.generate_token("user123".to_string()).unwrap();

        let claims = auth.verify_token(&token);
        assert!(claims.is_ok());
        assert_eq!(claims.unwrap().sub, "user123");
    }

    #[test]
    fn test_verify_token_invalid() {
        let auth = AuthService::new(uuid::Uuid::new_v4().to_string());
        let result = auth.verify_token("invalid.token.here");

        assert!(result.is_err());
    }

    #[test]
    fn test_verify_token_wrong_secret() {
        let auth1 = AuthService::new(uuid::Uuid::new_v4().to_string());
        let (token, _) = auth1.generate_token("user123".to_string()).unwrap();

        let auth2 = AuthService::new(uuid::Uuid::new_v4().to_string());
        let result = auth2.verify_token(&token);

        assert!(result.is_err());
    }
}
