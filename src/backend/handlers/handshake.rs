//! WebSocket handshake validation and authentication
//!
//! Validates JWT tokens from query parameters or Sec-WebSocket-Protocol header and manages the WebSocket upgrade process.
//! Ensures only authenticated users can establish WebSocket connections.
//!
//! # Token Sources (in order of preference)
//! 1. Sec-WebSocket-Protocol header - recommended, avoids token exposure in logs
//! 2. URL query parameter - supported for backwards compatibility

use crate::services::AuthService;
use chat_shared::protocol::TokenClaims;
use warp::http::StatusCode;

/// Extract JWT token from WebSocket upgrade request query string
pub fn extract_token_from_query(query: &str) -> Result<String, String> {
    for param in query.split('&') {
        if let Some(value) = param.strip_prefix("token=") {
            if value.is_empty() {
                return Err("Token parameter is empty".to_string());
            }
            let decoded = percent_decode(value);
            return Ok(decoded);
        }
    }

    Err("Token parameter not found in query string".to_string())
}

/// Extract JWT token from Sec-WebSocket-Protocol header
///
/// The token is expected as a subprotocol in the format: "jwt.<token>"
/// This is the recommended approach as it avoids logging tokens in access logs.
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

/// Basic percent-decoding for URL-encoded tokens
fn percent_decode(s: &str) -> String {
    let mut result = String::new();
    let mut bytes = s.bytes().peekable();

    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let mut hex_buf = [0u8; 2];
            let mut hex_idx = 0;

            while hex_idx < 2 {
                if let Some(&next_byte) = bytes.peek() {
                    let next_char = char::from(next_byte);
                    if next_char.is_ascii_hexdigit() {
                        hex_buf[hex_idx] = next_byte;
                        hex_idx += 1;
                        bytes.next();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if hex_idx == 2 {
                if let Ok(utf8_str) = std::str::from_utf8(&hex_buf) {
                    if let Ok(decoded) = u8::from_str_radix(utf8_str, 16) {
                        result.push(decoded as char);
                        continue;
                    }
                }
            }

            result.push('%');
            for &byte in hex_buf.iter().take(hex_idx) {
                result.push(byte as char);
            }
        } else if byte == b'+' {
            result.push(' ');
        } else {
            result.push(byte as char);
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
    ///
    /// Checks for token in Sec-WebSocket-Protocol header first (recommended),
    /// falls back to URL query parameter for backwards compatibility.
    pub async fn validate_upgrade(
        &self,
        query: &str,
        protocol_header: Option<&str>,
    ) -> Result<TokenClaims, (StatusCode, String)> {
        let token = protocol_header
            .and_then(extract_token_from_protocol_header)
            .or_else(|| extract_token_from_query(query).ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Token required in Sec-WebSocket-Protocol header or query parameter".to_string(),
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

    #[tokio::test]
    async fn test_handshake_validator_valid_token() {
        let validator = HandshakeValidator::new("test_secret".to_string());
        let (token, _) = validator
            .auth_service
            .generate_token("user123".to_string())
            .unwrap();

        let protocol_header = format!("jwt.{}", token);
        let result = validator.validate_upgrade("", Some(&protocol_header)).await;

        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.aud, "chat-app");
    }

    #[tokio::test]
    async fn test_handshake_validator_missing_token() {
        let validator = HandshakeValidator::new("test_secret".to_string());
        let result = validator.validate_upgrade("foo=bar", None).await;

        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(msg.contains("Token required"));
    }

    #[tokio::test]
    async fn test_handshake_validator_invalid_token() {
        let validator = HandshakeValidator::new("test_secret".to_string());
        let protocol_header = "jwt.invalid.token.here";
        let result = validator.validate_upgrade("", Some(protocol_header)).await;

        assert!(result.is_err());
        let (status, msg) = result.unwrap_err();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(msg.contains("Invalid") || msg.contains("malformed"));
    }

    #[tokio::test]
    async fn test_handshake_validator_wrong_secret() {
        let validator1 = HandshakeValidator::new("secret1".to_string());
        let (token, _) = validator1
            .auth_service
            .generate_token("user123".to_string())
            .unwrap();

        let validator2 = HandshakeValidator::new("secret2".to_string());
        let protocol_header = format!("jwt.{}", token);
        let result = validator2.validate_upgrade("", Some(&protocol_header)).await;

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

    #[tokio::test]
    async fn test_handshake_validator_token_from_protocol_header() {
        let validator = HandshakeValidator::new("test_secret".to_string());
        let (token, _) = validator
            .auth_service
            .generate_token("user123".to_string())
            .unwrap();

        let protocol_header = format!("jwt.{}", token);
        let result = validator.validate_upgrade("", Some(&protocol_header)).await;

        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, "user123");
    }

    #[tokio::test]
    async fn test_handshake_validator_only_protocol_header_supported() {
        let validator = HandshakeValidator::new("test_secret".to_string());
        let (token, _) = validator
            .auth_service
            .generate_token("user123".to_string())
            .unwrap();

        let protocol_header = format!("jwt.{}", token);
        let result = validator.validate_upgrade("", Some(&protocol_header)).await;

        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, "user123");
    }
}
