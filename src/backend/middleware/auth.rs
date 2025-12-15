//! Authentication middleware for protected endpoints
//!
//! Validates JWT tokens from Authorization header and extracts user ID

use crate::services::auth_service::AuthService;
use std::sync::Arc;
use warp::{
    filters::header::headers_cloned,
    http::header::{HeaderMap, AUTHORIZATION},
    reject, Filter, Rejection,
};

/// Custom rejection for authentication errors
#[derive(Debug)]
pub struct Unauthorized;
impl reject::Reject for Unauthorized {}

/// Extract and validate JWT token from Authorization header
///
/// Returns user_id from token claims if valid
pub fn with_auth(
    auth_service: Arc<AuthService>,
) -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    headers_cloned()
        .and(warp::any().map(move || auth_service.clone()))
        .and_then(|headers: HeaderMap, auth_service: Arc<AuthService>| async move {
            extract_user_id(&headers, &auth_service)
                .ok_or_else(|| reject::custom(Unauthorized))
        })
}

/// Extract user ID from Authorization header
///
/// Expected format: "Bearer <token>"
fn extract_user_id(headers: &HeaderMap, auth_service: &AuthService) -> Option<String> {
    let auth_header = headers.get(AUTHORIZATION)?;
    let auth_str = auth_header.to_str().ok()?;

    // Check if header starts with "Bearer "
    if !auth_str.starts_with("Bearer ") {
        return None;
    }

    // Extract token (skip "Bearer " prefix)
    let token = &auth_str[7..];

    // Verify token and extract user_id
    auth_service
        .verify_token(token)
        .ok()
        .map(|claims| claims.sub)
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::http::HeaderValue;

    #[test]
    fn test_extract_user_id_valid() {
        let auth_service = AuthService::new("test_secret".to_string());
        let (token, _) = auth_service.generate_token("user123".to_string()).unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );

        let result = extract_user_id(&headers, &auth_service);
        assert_eq!(result, Some("user123".to_string()));
    }

    #[test]
    fn test_extract_user_id_no_header() {
        let auth_service = AuthService::new("test_secret".to_string());
        let headers = HeaderMap::new();

        let result = extract_user_id(&headers, &auth_service);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_user_id_no_bearer_prefix() {
        let auth_service = AuthService::new("test_secret".to_string());
        let (token, _) = auth_service.generate_token("user123".to_string()).unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&token).unwrap());

        let result = extract_user_id(&headers, &auth_service);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_user_id_invalid_token() {
        let auth_service = AuthService::new("test_secret".to_string());

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str("Bearer invalid.token.here").unwrap(),
        );

        let result = extract_user_id(&headers, &auth_service);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_user_id_wrong_secret() {
        let auth1 = AuthService::new("secret1".to_string());
        let (token, _) = auth1.generate_token("user123".to_string()).unwrap();

        let auth2 = AuthService::new("secret2".to_string());

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );

        let result = extract_user_id(&headers, &auth2);
        assert_eq!(result, None);
    }
}
