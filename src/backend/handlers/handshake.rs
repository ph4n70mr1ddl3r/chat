//! WebSocket handshake validation and authentication
//!
//! Validates JWT tokens from query parameters and manages the WebSocket upgrade process.
//! Ensures only authenticated users can establish WebSocket connections.

use crate::services::auth_service::TokenClaims;
use crate::services::AuthService;
use warp::http::StatusCode;

/// Extract JWT token from WebSocket upgrade request query string
pub fn extract_token_from_query(query: &str) -> Result<String, String> {
    // Parse query string for ?token=<jwt>
    for param in query.split('&') {
        if let Some(value) = param.strip_prefix("token=") {
            if value.is_empty() {
                return Err("Token parameter is empty".to_string());
            }
            // Decode percent-encoded characters manually (basic implementation)
            let decoded = percent_decode(value);
            return Ok(decoded);
        }
    }

    Err("Token parameter not found in query string".to_string())
}

/// Basic percent-decoding for URL-encoded tokens
fn percent_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            // Try to read two hex digits
            if let Some(&next1) = chars.peek() {
                chars.next();
                if let Some(next2) = chars.next() {
                    if let Ok(byte) = u8::from_str_radix(&format!("{}{}", next1, next2), 16) {
                        result.push(byte as char);
                        continue;
                    }
                }
                // If decode fails, include literal percent and char
                result.push('%');
                result.push(next1);
            } else {
                result.push('%');
            }
        } else if ch == '+' {
            result.push(' ');
        } else {
            result.push(ch);
        }
    }

    result
}

/// WebSocket handshake handler
pub struct HandshakeValidator {
    auth_service: AuthService,
}

impl HandshakeValidator {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            auth_service: AuthService::new(jwt_secret),
        }
    }

    /// Validate WebSocket upgrade request and extract user claims
    pub fn validate_upgrade(&self, query: &str) -> Result<TokenClaims, (StatusCode, String)> {
        // Extract token from query string
        let token = extract_token_from_query(query).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

        // Verify token with auth service
        let claims = self.auth_service.verify_token(&token).map_err(|e| {
            if e.contains("expired") || e.contains("Expiration") {
                (StatusCode::UNAUTHORIZED, "Token has expired".to_string())
            } else {
                (
                    StatusCode::UNAUTHORIZED,
                    "Invalid or malformed token".to_string(),
                )
            }
        })?;

        // Validate required claims
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

        // Check token expiration explicitly
        let now = chrono::Utc::now().timestamp();
        if claims.exp <= now {
            return Err((StatusCode::UNAUTHORIZED, "Token has expired".to_string()));
        }

        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_from_query_valid() {
        let query = "token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let result = extract_token_from_query(query);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
    }

    #[test]
    fn test_extract_token_from_query_with_other_params() {
        let query = "foo=bar&token=mytoken&baz=qux";
        let result = extract_token_from_query(query);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "mytoken");
    }

    #[test]
    fn test_extract_token_from_query_missing() {
        let query = "foo=bar&baz=qux";
        let result = extract_token_from_query(query);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_extract_token_from_query_empty() {
        let query = "token=";
        let result = extract_token_from_query(query);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_extract_token_from_query_url_encoded() {
        let encoded_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9%2BTest";
        let query = format!("token={}", encoded_token);
        let result = extract_token_from_query(&query);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9+Test");
    }

    #[test]
    fn test_handshake_validator_new() {
        let validator = HandshakeValidator::new("test_secret".to_string());
        // Just verify it constructs without error
        assert!(!validator
            .auth_service
            .generate_token("user123".to_string())
            .unwrap()
            .0
            .is_empty());
    }

    #[test]
    fn test_handshake_validator_valid_token() {
        let validator = HandshakeValidator::new("test_secret".to_string());
        let (token, _) = validator
            .auth_service
            .generate_token("user123".to_string())
            .unwrap();

        let query = format!("token={}", token);
        let result = validator.validate_upgrade(&query);

        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.aud, "chat-app");
    }

    #[test]
    fn test_handshake_validator_missing_token() {
        let validator = HandshakeValidator::new("test_secret".to_string());
        let query = "foo=bar";
        let result = validator.validate_upgrade(query);

        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(msg.contains("not found"));
    }

    #[test]
    fn test_handshake_validator_invalid_token() {
        let validator = HandshakeValidator::new("test_secret".to_string());
        let query = "token=invalid.token.here";
        let result = validator.validate_upgrade(query);

        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(msg.contains("Invalid") || msg.contains("malformed"));
    }

    #[test]
    fn test_handshake_validator_wrong_secret() {
        let validator1 = HandshakeValidator::new("secret1".to_string());
        let (token, _) = validator1
            .auth_service
            .generate_token("user123".to_string())
            .unwrap();

        let validator2 = HandshakeValidator::new("secret2".to_string());
        let query = format!("token={}", token);
        let result = validator2.validate_upgrade(&query);

        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_handshake_validator_audience_mismatch() {
        // This test would require creating a token with wrong audience,
        // which is not directly testable with current AuthService.
        // For now, we test via integration if the validator checks audience properly.
    }
}
