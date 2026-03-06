//! Request context middleware
//!
//! Provides request ID tracking and security headers for better observability and security

use uuid::Uuid;
use warp::{
    filters::header::headers_cloned,
    http::HeaderMap,
    Filter, Rejection,
};

const REQUEST_ID_HEADER: &str = "X-Request-ID";

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
        }
    }

    pub fn from_headers(headers: &HeaderMap) -> Self {
        let request_id = headers
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .filter(|s| !s.is_empty() && s.len() <= 100 && s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'))
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        Self { request_id }
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn with_request_context(
) -> impl Filter<Extract = (RequestContext,), Error = Rejection> + Clone {
    headers_cloned().and_then(|headers: HeaderMap| async move {
        Ok::<_, Rejection>(RequestContext::from_headers(&headers))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::http::HeaderValue;

    #[test]
    fn test_request_context_new() {
        let ctx = RequestContext::new();
        assert!(!ctx.request_id.is_empty());
        assert!(Uuid::parse_str(&ctx.request_id).is_ok());
    }

    #[test]
    fn test_request_context_from_headers_with_id() {
        let mut headers = HeaderMap::new();
        headers.insert(REQUEST_ID_HEADER, HeaderValue::from_static("test-id-123"));
        
        let ctx = RequestContext::from_headers(&headers);
        assert_eq!(ctx.request_id, "test-id-123");
    }

    #[test]
    fn test_request_context_from_headers_empty() {
        let headers = HeaderMap::new();
        let ctx = RequestContext::from_headers(&headers);
        assert!(Uuid::parse_str(&ctx.request_id).is_ok());
    }

    #[test]
    fn test_request_context_from_headers_invalid() {
        let mut headers = HeaderMap::new();
        headers.insert(REQUEST_ID_HEADER, HeaderValue::from_static("invalid;drop table"));
        
        let ctx = RequestContext::from_headers(&headers);
        assert!(Uuid::parse_str(&ctx.request_id).is_ok());
        assert_ne!(ctx.request_id, "invalid;drop table");
    }

    #[test]
    fn test_request_context_from_headers_too_long() {
        let mut headers = HeaderMap::new();
        let long_id = "a".repeat(200);
        headers.insert(REQUEST_ID_HEADER, HeaderValue::from_str(&long_id).unwrap());
        
        let ctx = RequestContext::from_headers(&headers);
        assert!(Uuid::parse_str(&ctx.request_id).is_ok());
    }
}
