//! Token refresh endpoint
//!
//! Handles POST /auth/refresh for refreshing JWT tokens

use crate::handlers::ErrorBody;
use crate::services::{AuthService, CsrfService};
use serde::Deserialize;
use tracing::{info, warn};
use warp::{reply, Rejection, Reply};

use crate::handlers::auth::AuthResponse;

/// Token refresh request
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub token: String,
}

/// Handle POST /auth/refresh
pub async fn refresh_token_handler(
    req: RefreshRequest,
    jwt_secret: String,
    csrf_service: CsrfService,
) -> Result<impl Reply, Rejection> {
    let auth_service = AuthService::new(jwt_secret.clone());

    let claims = match auth_service.verify_token(&req.token).await {
        Ok(claims) => claims,
        Err(e) => {
            warn!("Token verification failed: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "INVALID_TOKEN".to_string(),
                    message: "Token is invalid or expired".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    let (new_token, expires_at) = match auth_service.generate_token(claims.sub.clone()) {
        Ok((token, expires_at)) => (token, expires_at),
        Err(e) => {
            warn!("Failed to generate new token: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "TOKEN_GENERATION_ERROR".to_string(),
                    message: "Failed to refresh token".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    let csrf_token = match csrf_service.generate_token(&claims.sub) {
        Ok(token) => token,
        Err(e) => {
            warn!("Failed to generate CSRF token: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "AUTH_ERROR".to_string(),
                    message: "Failed to generate security token".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    info!("Token refreshed for user: {}", claims.sub);

    Ok(reply::with_status(
        reply::json(&AuthResponse {
            user_id: claims.sub.clone(),
            username: String::new(),
            token: new_token,
            expires_in: expires_at,
            csrf_token,
        }),
        warp::http::StatusCode::OK,
    ))
}

/// Extract Bearer token from Authorization header
pub fn extract_bearer_token(auth_header: &str) -> Option<String> {
    auth_header
        .strip_prefix("Bearer ")
        .map(|token| token.to_string())
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
