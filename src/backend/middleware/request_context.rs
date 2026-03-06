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
    pub client_ip: Option<String>,
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            client_ip: None,
        }
    }

    pub fn from_headers(headers: &HeaderMap) -> Self {
        let request_id = headers
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .filter(|s| !s.is_empty() && s.len() <= 100 && s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'))
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let client_ip = Self::extract_client_ip(headers);

        Self { request_id, client_ip }
    }

    /// Extract client IP address from request headers
    ///
    /// Checks headers in order of preference:
    /// 1. X-Forwarded-For (first IP in chain)
    /// 2. X-Real-IP
    /// 3. None (will fall back to connection info)
    fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
        // Try X-Forwarded-For first (standard proxy header)
        if let Some(forwarded_for) = headers.get("X-Forwarded-For") {
            if let Ok(forwarded_str) = forwarded_for.to_str() {
                // X-Forwarded-For can contain multiple IPs: client, proxy1, proxy2
                // We want the first one (the original client)
                if let Some(client_ip) = forwarded_str.split(',').next() {
                    let trimmed = client_ip.trim();
                    if !trimmed.is_empty() && Self::is_valid_ip(trimmed) {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }

        // Try X-Real-IP (used by some proxies)
        if let Some(real_ip) = headers.get("X-Real-IP") {
            if let Ok(ip_str) = real_ip.to_str() {
                let trimmed = ip_str.trim();
                if !trimmed.is_empty() && Self::is_valid_ip(trimmed) {
                    return Some(trimmed.to_string());
                }
            }
        }

        None
    }

    /// Basic validation for IP address format
    fn is_valid_ip(ip: &str) -> bool {
        // Check for IPv4 or IPv6 format
        // This is a basic check - more comprehensive validation could be added
        ip.parse::<std::net::IpAddr>().is_ok()
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
        assert!(ctx.client_ip.is_none());
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

    #[test]
    fn test_extract_client_ip_from_x_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Forwarded-For", HeaderValue::from_static("192.168.1.1, 10.0.0.1"));
        
        let ctx = RequestContext::from_headers(&headers);
        assert_eq!(ctx.client_ip, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_extract_client_ip_from_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Real-IP", HeaderValue::from_static("192.168.1.2"));
        
        let ctx = RequestContext::from_headers(&headers);
        assert_eq!(ctx.client_ip, Some("192.168.1.2".to_string()));
    }

    #[test]
    fn test_extract_client_ip_none() {
        let headers = HeaderMap::new();
        let ctx = RequestContext::from_headers(&headers);
        assert_eq!(ctx.client_ip, None);
    }

    #[test]
    fn test_extract_client_ip_invalid() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Forwarded-For", HeaderValue::from_static("not-a-valid-ip"));
        
        let ctx = RequestContext::from_headers(&headers);
        assert_eq!(ctx.client_ip, None);
    }
}
