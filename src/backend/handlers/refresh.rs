//! Token refresh endpoint
//!
//! Handles POST /auth/refresh for refreshing JWT tokens

use crate::handlers::auth::{AuthResponse, ErrorResponse};
use crate::services::AuthService;
use serde::Deserialize;
use tracing::{info, warn};
use warp::{reply, Rejection, Reply};

/// Token refresh request
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub token: String,
}

/// Handle POST /auth/refresh
pub async fn refresh_token_handler(
    req: RefreshRequest,
    jwt_secret: String,
) -> Result<impl Reply, Rejection> {
    let auth_service = AuthService::new(jwt_secret.clone());

    // Verify the existing token
    let claims = match auth_service.verify_token(&req.token) {
        Ok(claims) => claims,
        Err(e) => {
            warn!("Token verification failed: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "INVALID_TOKEN".to_string(),
                    message: "Token is invalid or expired".to_string(),
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    // Generate a new token with the same user ID
    let (new_token, expires_at) = match auth_service.generate_token(claims.sub.clone()) {
        Ok((token, expires_at)) => (token, expires_at),
        Err(e) => {
            warn!("Failed to generate new token: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "TOKEN_GENERATION_ERROR".to_string(),
                    message: "Failed to refresh token".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    info!("Token refreshed for user: {}", claims.sub);

    Ok(reply::with_status(
        reply::json(&AuthResponse {
            user_id: claims.sub.clone(),
            username: "".to_string(), // Username not needed for refresh
            token: new_token,
            expires_in: expires_at as u64,
        }),
        warp::http::StatusCode::OK,
    ))
}

/// Extract Bearer token from Authorization header
pub fn extract_bearer_token(auth_header: &str) -> Option<String> {
    if auth_header.starts_with("Bearer ") {
        Some(auth_header[7..].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_token() {
        let header = "Bearer eyJhbGc...";
        let token = extract_bearer_token(header);
        assert_eq!(token, Some("eyJhbGc...".to_string()));
    }

    #[test]
    fn test_extract_bearer_token_invalid() {
        let header = "InvalidFormat";
        let token = extract_bearer_token(header);
        assert_eq!(token, None);
    }
}
