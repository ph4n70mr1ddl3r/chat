//! CSRF protection service
//!
//! Provides stateless CSRF token generation and validation using JWT.
//! This is a security measure to prevent cross-site request forgery attacks.

use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

const CSRF_TOKEN_VALIDITY_SECS: i64 = 3600;

/// Derive a CSRF-specific secret from the JWT secret.
/// This follows the principle of key separation - each cryptographic purpose
/// should use a distinct key to prevent cross-protocol attacks.
#[must_use]
pub fn derive_csrf_secret(jwt_secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"csrf-key-derivation-v1:");
    hasher.update(jwt_secret.as_bytes());
    format!("csrf-{:x}", hasher.finalize())
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum CsrfValidationError {
    Expired,
    UserMismatch,
    InvalidToken,
    TokenTooOld,
}

#[derive(Debug, Clone)]
pub struct CsrfService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

#[derive(Debug, Serialize, Deserialize)]
struct CsrfClaims {
    sub: String,
    iat: i64,
    exp: i64,
    nonce: String,
}

impl CsrfClaims {
    fn validate_nonce(&self) -> bool {
        !self.nonce.is_empty() && uuid::Uuid::parse_str(&self.nonce).is_ok()
    }
}

impl CsrfService {
    /// Create a new CSRF service with a direct secret
    #[must_use]
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    /// Create a new CSRF service from a JWT secret.
    /// Derives a separate CSRF secret to follow key separation best practices.
    #[must_use]
    pub fn from_jwt_secret(jwt_secret: &str) -> Self {
        Self::new(&derive_csrf_secret(jwt_secret))
    }

    /// Generate a CSRF token for a user
    ///
    /// # Errors
    /// Returns an error string if token encoding fails.
    pub fn generate_token(&self, user_id: &str) -> Result<String, String> {
        let now = Utc::now().timestamp();
        let nonce = uuid::Uuid::new_v4().to_string();

        let claims = CsrfClaims {
            sub: user_id.to_string(),
            iat: now,
            exp: now + CSRF_TOKEN_VALIDITY_SECS,
            nonce,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| format!("CSRF token encoding failed: {e}"))
    }

    /// Validate a CSRF token
    ///
    /// # Errors
    /// Returns a `CsrfValidationError` if token validation fails.
    pub fn validate_token(&self, token: &str, user_id: &str) -> Result<(), CsrfValidationError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_required_spec_claims(&["sub", "iat", "exp"]);

        match decode::<CsrfClaims>(token, &self.decoding_key, &validation) {
            Ok(data) => {
                let now = Utc::now().timestamp();
                if data.claims.exp < now {
                    tracing::warn!(
                        user_id_prefix = &user_id[..8.min(user_id.len())],
                        "CSRF token expired"
                    );
                    return Err(CsrfValidationError::Expired);
                }
                if !data.claims.validate_nonce() {
                    tracing::warn!("CSRF token has invalid or missing nonce");
                    return Err(CsrfValidationError::InvalidToken);
                }
                if !bool::from(data.claims.sub.as_bytes().ct_eq(user_id.as_bytes())) {
                    tracing::warn!(
                        expected_prefix = &user_id[..8.min(user_id.len())],
                        "CSRF token user mismatch"
                    );
                    return Err(CsrfValidationError::UserMismatch);
                }
                Ok(())
            }
            Err(e) => {
                tracing::warn!("CSRF token validation failed: {}", e);
                Err(CsrfValidationError::InvalidToken)
            }
        }
    }

    /// Validate a CSRF token with freshness requirement for sensitive operations.
    /// Requires the token to have been issued within the specified number of seconds.
    ///
    /// # Errors
    /// Returns a `CsrfValidationError` if token validation fails or token is too old.
    pub fn validate_token_fresh(
        &self,
        token: &str,
        user_id: &str,
        max_age_secs: i64,
    ) -> Result<(), CsrfValidationError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_required_spec_claims(&["sub", "iat", "exp"]);

        match decode::<CsrfClaims>(token, &self.decoding_key, &validation) {
            Ok(data) => {
                let now = Utc::now().timestamp();
                if data.claims.exp < now {
                    tracing::warn!(
                        user_id_prefix = &user_id[..8.min(user_id.len())],
                        "CSRF token expired"
                    );
                    return Err(CsrfValidationError::Expired);
                }
                let token_age = now - data.claims.iat;
                if token_age > max_age_secs {
                    tracing::warn!(
                        user_id_prefix = &user_id[..8.min(user_id.len())],
                        token_age_secs = token_age,
                        max_age_secs = max_age_secs,
                        "CSRF token too old for sensitive operation"
                    );
                    return Err(CsrfValidationError::TokenTooOld);
                }
                if !data.claims.validate_nonce() {
                    tracing::warn!("CSRF token has invalid or missing nonce");
                    return Err(CsrfValidationError::InvalidToken);
                }
                if !bool::from(data.claims.sub.as_bytes().ct_eq(user_id.as_bytes())) {
                    tracing::warn!(
                        expected_prefix = &user_id[..8.min(user_id.len())],
                        "CSRF token user mismatch"
                    );
                    return Err(CsrfValidationError::UserMismatch);
                }
                Ok(())
            }
            Err(e) => {
                tracing::warn!("CSRF token validation failed: {}", e);
                Err(CsrfValidationError::InvalidToken)
            }
        }
    }

    /// Convenience method for backward compatibility
    #[must_use]
    pub fn is_valid(&self, token: &str, user_id: &str) -> bool {
        self.validate_token(token, user_id).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csrf_token_generation() {
        let service = CsrfService::new("test-secret");
        let token = service.generate_token("user123").unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_csrf_token_validation() {
        let service = CsrfService::new("test-secret");
        let token = service.generate_token("user123").unwrap();
        assert!(service.is_valid(&token, "user123"));
        assert!(!service.is_valid(&token, "wrong_user"));
    }

    #[test]
    fn test_csrf_token_wrong_secret() {
        let service1 = CsrfService::new("secret1");
        let service2 = CsrfService::new("secret2");
        let token = service1.generate_token("user123").unwrap();
        assert!(!service2.is_valid(&token, "user123"));
    }

    #[test]
    fn test_csrf_validation_error_types() {
        let service = CsrfService::new("test-secret");
        let token = service.generate_token("user123").unwrap();

        // Valid token should return Ok
        assert_eq!(service.validate_token(&token, "user123"), Ok(()));

        // Wrong user should return UserMismatch
        assert_eq!(
            service.validate_token(&token, "wrong_user"),
            Err(CsrfValidationError::UserMismatch)
        );
    }

    #[test]
    fn test_derive_csrf_secret() {
        let jwt_secret = "my-jwt-secret";
        let csrf_secret = derive_csrf_secret(jwt_secret);

        // Should produce a different secret
        assert_ne!(jwt_secret, csrf_secret);

        // Should be deterministic
        assert_eq!(csrf_secret, derive_csrf_secret(jwt_secret));

        // Different JWT secrets should produce different CSRF secrets
        let csrf_secret2 = derive_csrf_secret("different-jwt-secret");
        assert_ne!(csrf_secret, csrf_secret2);
    }

    #[test]
    fn test_from_jwt_secret_key_separation() {
        let jwt_secret = "my-jwt-secret";
        let service = CsrfService::from_jwt_secret(jwt_secret);

        // Should work with the derived secret
        let token = service.generate_token("user123").unwrap();
        assert!(service.is_valid(&token, "user123"));

        // A service created with the raw JWT secret should NOT validate tokens
        // from the derived-secret service (proves key separation)
        let service_with_jwt_secret = CsrfService::new(jwt_secret);
        assert!(!service_with_jwt_secret.is_valid(&token, "user123"));
    }
}
