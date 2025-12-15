//! Authentication service for user account management and JWT token generation
//!
//! Handles user creation, password validation, hashing, and JWT token generation/verification.

use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::models::User;

/// JWT token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,      // Subject (user ID)
    pub aud: String,      // Audience
    pub iat: i64,         // Issued at
    pub exp: i64,         // Expiration time
    #[serde(default)]
    pub scopes: Vec<String>,
}

/// Password validation error types
#[derive(Debug, Clone)]
pub enum PasswordError {
    TooShort,
    MissingUppercase,
    MissingLowercase,
    MissingDigit,
}

impl std::fmt::Display for PasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordError::TooShort => write!(f, "Password must be at least 8 characters"),
            PasswordError::MissingUppercase => write!(f, "Password must contain at least one uppercase letter"),
            PasswordError::MissingLowercase => write!(f, "Password must contain at least one lowercase letter"),
            PasswordError::MissingDigit => write!(f, "Password must contain at least one digit"),
        }
    }
}

/// Authentication service
pub struct AuthService {
    jwt_secret: String,
}

impl AuthService {
    /// Create a new authentication service with the given secret key
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }

    /// Validate password strength according to spec:
    /// - Minimum 8 characters
    /// - At least 1 uppercase letter
    /// - At least 1 lowercase letter
    /// - At least 1 digit
    pub fn validate_password(password: &str) -> Result<(), PasswordError> {
        if password.len() < 8 {
            return Err(PasswordError::TooShort);
        }
        
        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(PasswordError::MissingUppercase);
        }
        
        if !password.chars().any(|c| c.is_lowercase()) {
            return Err(PasswordError::MissingLowercase);
        }
        
        if !password.chars().any(|c| c.is_numeric()) {
            return Err(PasswordError::MissingDigit);
        }
        
        Ok(())
    }

    /// Hash a password with bcrypt + salt
    ///
    /// Returns (password_hash, password_salt) tuple
    /// Note: Bcrypt handles salt internally, so we return the hash as both fields for compatibility
    pub fn hash_password(password: &str) -> Result<(String, String), String> {
        // Validate password first
        Self::validate_password(password)
            .map_err(|e| e.to_string())?;
        
        // Hash with bcrypt (DEFAULT_COST = 12)
        let hashed = hash(password, DEFAULT_COST)
            .map_err(|e| format!("Failed to hash password: {}", e))?;
        
        // Bcrypt includes the salt in the hash, so we use the hash for both
        Ok((hashed.clone(), hashed))
    }

    /// Verify a password against a hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
        verify(password, hash)
            .map_err(|e| format!("Failed to verify password: {}", e))
    }

    /// Create a new user with validated password
    pub async fn create_user(&self, username: String, password: String) -> Result<User, String> {
        // Validate password
        Self::validate_password(&password)
            .map_err(|e| e.to_string())?;
        
        // Hash password
        let (password_hash, password_salt) = Self::hash_password(&password)?;
        
        // Create user (note: actual DB save happens in the handler)
        Ok(User::new(username, password_hash, password_salt))
    }

    /// Generate JWT token for a user
    pub fn generate_token(&self, user_id: String) -> Result<(String, i64), String> {
        let now = Utc::now().timestamp();
        let expiration = now + 3600; // 1 hour expiration
        
        let claims = TokenClaims {
            sub: user_id,
            aud: "chat-app".to_string(),
            iat: now,
            exp: expiration,
            scopes: vec!["send".to_string(), "receive".to_string()],
        };
        
        let key = EncodingKey::from_secret(self.jwt_secret.as_bytes());
        
        encode(&Header::default(), &claims, &key)
            .map(|token| (token, expiration))
            .map_err(|e| format!("Failed to generate token: {}", e))
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
        assert!(AuthService::validate_password("TestPass123").is_ok());
        assert!(AuthService::validate_password("AnotherPassword456").is_ok());
    }

    #[test]
    fn test_validate_password_too_short() {
        assert!(AuthService::validate_password("Test1").is_err());
    }

    #[test]
    fn test_validate_password_no_uppercase() {
        assert!(AuthService::validate_password("testpass123").is_err());
    }

    #[test]
    fn test_validate_password_no_lowercase() {
        assert!(AuthService::validate_password("TESTPASS123").is_err());
    }

    #[test]
    fn test_validate_password_no_digit() {
        assert!(AuthService::validate_password("TestPass").is_err());
    }

    #[test]
    fn test_hash_password() {
        let (hash, salt) = AuthService::hash_password("TestPass123").unwrap();
        
        // Hash should not be the password itself
        assert_ne!(hash, "TestPass123");
        
        // Hash and salt should be the same (bcrypt includes salt in hash)
        assert_eq!(hash, salt);
    }

    #[test]
    fn test_verify_password_correct() {
        let (hash, _) = AuthService::hash_password("TestPass123").unwrap();
        assert!(AuthService::verify_password("TestPass123", &hash).unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let (hash, _) = AuthService::hash_password("TestPass123").unwrap();
        assert!(!AuthService::verify_password("WrongPassword123", &hash).unwrap());
    }

    #[tokio::test]
    async fn test_create_user() {
        let auth = AuthService::new("test_secret".to_string());
        let user = auth.create_user("testuser".to_string(), "TestPass123".to_string()).await;
        
        assert!(user.is_ok());
        let user = user.unwrap();
        assert_eq!(user.username, "testuser");
        assert!(!user.password_hash.is_empty());
    }

    #[test]
    fn test_generate_token() {
        let auth = AuthService::new("test_secret".to_string());
        let result = auth.generate_token("user123".to_string());
        
        assert!(result.is_ok());
        let (token, exp) = result.unwrap();
        assert!(!token.is_empty());
        assert!(exp > Utc::now().timestamp());
    }

    #[test]
    fn test_verify_token_valid() {
        let auth = AuthService::new("test_secret".to_string());
        let (token, _) = auth.generate_token("user123".to_string()).unwrap();
        
        let claims = auth.verify_token(&token);
        assert!(claims.is_ok());
        assert_eq!(claims.unwrap().sub, "user123");
    }

    #[test]
    fn test_verify_token_invalid() {
        let auth = AuthService::new("test_secret".to_string());
        let result = auth.verify_token("invalid.token.here");
        
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_token_wrong_secret() {
        let auth1 = AuthService::new("secret1".to_string());
        let (token, _) = auth1.generate_token("user123".to_string()).unwrap();
        
        let auth2 = AuthService::new("secret2".to_string());
        let result = auth2.verify_token(&token);
        
        assert!(result.is_err());
    }
}
