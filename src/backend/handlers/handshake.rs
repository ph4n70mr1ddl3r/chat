//! WebSocket handshake validation and authentication
//!
//! Validates JWT tokens from Sec-WebSocket-Protocol header and manages the WebSocket upgrade process.
//! Ensures only authenticated users can establish WebSocket connections.
//!
//! # Security Note
//! Tokens MUST be provided via the Sec-WebSocket-Protocol header (format: "jwt.<token>").
//! Query parameter tokens are NOT supported to prevent token leakage via:
//! - Server access logs
//! - Browser history
//! - Referer headers
//! - Proxy logs

use crate::services::AuthService;
use chat_shared::protocol::TokenClaims;
use std::sync::Arc;
use warp::http::StatusCode;

/// Extract JWT token from Sec-WebSocket-Protocol header
///
/// The token is expected as a subprotocol in the format: "jwt.<token>"
#[must_use]
pub fn extract_token_from_protocol_header(header_value: &str) -> Option<String> {
    for protocol in header_value.split(',') {
        let protocol = protocol.trim();
        if let Some(token) = protocol.strip_prefix("jwt.") {
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
    }
    None
}

/// WebSocket handshake handler
pub struct HandshakeValidator {
    auth_service: Arc<AuthService>,
}

impl HandshakeValidator {
    #[must_use]
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }

    /// Validate WebSocket upgrade request and extract user claims
    ///
    /// Token must be provided in Sec-WebSocket-Protocol header as "jwt.<token>"
    ///
    /// # Errors
    ///
    /// Returns an error tuple with status code and message if:
    /// - Token is missing from the protocol header
    /// - Token is invalid, expired, or revoked
    /// - Token is missing required claims (sub, aud)
    pub async fn validate_upgrade(
        &self,
        protocol_header: Option<&str>,
    ) -> Result<TokenClaims, (StatusCode, String)> {
        let token = protocol_header
            .and_then(extract_token_from_protocol_header)
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Token required in Sec-WebSocket-Protocol header as 'jwt.<token>'".to_string(),
            ))?;

        let claims = self.auth_service.verify_token(&token).await.map_err(|e| {
            if e.contains("expired") || e.contains("Expiration") || e.contains("revoked") {
                (StatusCode::UNAUTHORIZED, e)
            } else {
                (
                    StatusCode::UNAUTHORIZED,
                    "Invalid or malformed token".to_string(),
                )
            }
        })?;

        if claims.sub.is_empty() {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Token missing subject claim".to_string(),
            ));
        }

        if claims.aud != "chat-app" {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Token audience mismatch".to_string(),
            ));
        }

        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_validator() -> HandshakeValidator {
        let auth_service = Arc::new(AuthService::new("test_secret".to_string()));
        HandshakeValidator::new(auth_service)
    }

    #[test]
    fn test_handshake_validator_new() {
        let validator = create_test_validator();
        assert!(!validator
            .auth_service
            .generate_token("user123".to_string())
            .unwrap()
            .0
            .is_empty());
    }

    #[tokio::test]
    async fn test_handshake_validator_valid_token() {
        let validator = create_test_validator();
        let (token, _) = validator
            .auth_service
            .generate_token("user123".to_string())
            .unwrap();

        let protocol_header = format!("jwt.{}", token);
        let result = validator.validate_upgrade(Some(&protocol_header)).await;

        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.aud, "chat-app");
    }

    #[tokio::test]
    async fn test_handshake_validator_missing_token() {
        let validator = create_test_validator();
        let result = validator.validate_upgrade(None).await;

        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(msg.contains("Token required"));
    }

    #[tokio::test]
    async fn test_handshake_validator_invalid_token() {
        let validator = create_test_validator();
        let protocol_header = "jwt.invalid.token.here";
        let result = validator.validate_upgrade(Some(protocol_header)).await;

        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(msg.contains("Invalid") || msg.contains("malformed"));
    }

    #[tokio::test]
    async fn test_handshake_validator_wrong_secret() {
        let auth_service1 = Arc::new(AuthService::new("secret1".to_string()));
        let validator1 = HandshakeValidator::new(auth_service1);
        let (token, _) = validator1
            .auth_service
            .generate_token("user123".to_string())
            .unwrap();

        let auth_service2 = Arc::new(AuthService::new("secret2".to_string()));
        let validator2 = HandshakeValidator::new(auth_service2);
        let protocol_header = format!("jwt.{}", token);
        let result = validator2.validate_upgrade(Some(&protocol_header)).await;

        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_extract_token_from_protocol_header_valid() {
        let header = "jwt.eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let result = extract_token_from_protocol_header(header);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
    }

    #[test]
    fn test_extract_token_from_protocol_header_multiple_protocols() {
        let header = "chat, jwt.mytoken, other";
        let result = extract_token_from_protocol_header(header);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "mytoken");
    }

    #[test]
    fn test_extract_token_from_protocol_header_not_present() {
        let header = "chat, other";
        let result = extract_token_from_protocol_header(header);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_token_from_protocol_header_empty_jwt() {
        let header = "jwt.";
        let result = extract_token_from_protocol_header(header);
        assert!(result.is_none());
    }
}
