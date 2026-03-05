//! CSRF protection service
//!
//! Provides stateless CSRF token generation and validation using JWT.
//! This is a security measure to prevent cross-site request forgery attacks.

use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

const CSRF_TOKEN_VALIDITY_SECS: i64 = 3600;

#[derive(Debug, Clone, PartialEq)]
pub enum CsrfValidationError {
    Expired,
    UserMismatch,
    InvalidToken,
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

impl CsrfService {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

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
            .map_err(|e| format!("CSRF token encoding failed: {}", e))
    }

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
                if data.claims.sub != user_id {
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
}
