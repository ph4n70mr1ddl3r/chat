//! Utility functions shared across the backend

use std::net::IpAddr;

/// Default trusted proxy CIDR ranges for extracting client IPs from X-Forwarded-For headers.
const DEFAULT_TRUSTED_PROXIES: &[&str] = &[
    "10.0.0.0/8",
    "172.16.0.0/12",
    "192.168.0.0/16",
    "127.0.0.0/8",
    "::1/128",
];

/// Check if an IP address belongs to a trusted proxy range.
///
/// Trusted proxies are internal network ranges (RFC 1918) and localhost.
/// Used to determine whether to trust X-Forwarded-For headers.
pub fn is_trusted_proxy(remote_ip: &IpAddr) -> bool {
    for cidr in DEFAULT_TRUSTED_PROXIES {
        if cidr.contains('/') {
            if let Ok(network) = cidr.parse::<ipnet::IpNet>() {
                if network.contains(remote_ip) {
                    return true;
                }
            }
        }
    }
    false
}

/// Sanitize a string for safe logging.
///
/// Limits length to 50 characters and only allows alphanumeric,
/// underscore, and hyphen characters to prevent log injection.
pub fn sanitize_for_log(s: &str) -> String {
    s.chars()
        .take(50)
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_for_log_normal() {
        assert_eq!(sanitize_for_log("alice"), "alice");
        assert_eq!(sanitize_for_log("user_123"), "user_123");
        assert_eq!(sanitize_for_log("test-user"), "test-user");
    }

    #[test]
    fn test_sanitize_for_log_removes_special_chars() {
        assert_eq!(sanitize_for_log("user<script>"), "userscript");
        assert_eq!(sanitize_for_log("user\nlog"), "userlog");
        assert_eq!(sanitize_for_log("user\x00null"), "usernull");
    }

    #[test]
    fn test_sanitize_for_log_truncates() {
        let long = "a".repeat(100);
        assert_eq!(sanitize_for_log(&long), "a".repeat(50));
    }

    #[test]
    fn test_sanitize_for_log_empty() {
        assert_eq!(sanitize_for_log(""), "");
    }
}
