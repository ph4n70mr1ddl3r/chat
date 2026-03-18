//! Utility functions shared across the backend

use std::net::IpAddr;
use std::sync::OnceLock;

/// Default trusted proxy CIDR ranges for extracting client IPs from X-Forwarded-For headers.
const DEFAULT_TRUSTED_PROXIES: &[&str] = &[
    "10.0.0.0/8",
    "172.16.0.0/12",
    "192.168.0.0/16",
    "127.0.0.0/8",
    "::1/128",
];

/// Cached list of trusted proxy networks loaded from environment.
static TRUSTED_PROXIES: OnceLock<Vec<ipnet::IpNet>> = OnceLock::new();

/// Load trusted proxies from environment variable or use defaults.
///
/// The `TRUSTED_PROXIES` environment variable should contain a comma-separated
/// list of CIDR ranges. Example: `TRUSTED_PROXIES=10.0.0.0/8,192.168.0.0/16`
///
/// For cloud deployments with load balancers that have public IPs, configure
/// this variable with your load balancer's IP range.
fn get_trusted_proxies() -> &'static Vec<ipnet::IpNet> {
    TRUSTED_PROXIES.get_or_init(|| {
        let mut proxies = Vec::new();

        if let Ok(env_proxies) = std::env::var("TRUSTED_PROXIES") {
            for cidr in env_proxies.split(',') {
                let cidr = cidr.trim();
                if cidr.is_empty() {
                    continue;
                }
                match cidr.parse::<ipnet::IpNet>() {
                    Ok(net) => {
                        tracing::info!("Added trusted proxy range: {}", net);
                        proxies.push(net);
                    }
                    Err(e) => {
                        tracing::warn!("Invalid CIDR in TRUSTED_PROXIES '{}': {}", cidr, e);
                    }
                }
            }
        }

        if proxies.is_empty() {
            for cidr in DEFAULT_TRUSTED_PROXIES {
                if let Ok(net) = cidr.parse::<ipnet::IpNet>() {
                    proxies.push(net);
                }
            }
            tracing::debug!("Using default trusted proxy ranges (RFC1918 + localhost)");
        }

        proxies
    })
}

/// Check if an IP address belongs to a trusted proxy range.
///
/// Trusted proxies are internal network ranges (RFC 1918) and localhost by default,
/// or custom ranges from the `TRUSTED_PROXIES` environment variable.
/// Used to determine whether to trust X-Forwarded-For headers.
#[must_use]
pub fn is_trusted_proxy(remote_ip: &IpAddr) -> bool {
    for network in get_trusted_proxies() {
        if network.contains(remote_ip) {
            return true;
        }
    }
    false
}

/// Sanitize a string for safe logging.
///
/// Limits length to 50 characters and only allows alphanumeric,
/// underscore, and hyphen characters to prevent log injection.
#[must_use]
pub fn sanitize_for_log(s: &str) -> String {
    s.chars()
        .take(50)
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

/// Escape SQL LIKE wildcards to prevent wildcard injection.
///
/// Escapes backslash, percent, and underscore characters which have
/// special meaning in SQL LIKE patterns. Also removes null bytes
/// which could cause issues with some database drivers.
#[must_use]
pub fn escape_like_pattern(s: &str) -> String {
    // Remove null bytes which could cause issues with some database drivers
    let s: String = s.chars().filter(|c| *c != '\0').collect();
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
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

    #[test]
    fn test_escape_like_pattern_no_special() {
        assert_eq!(escape_like_pattern("hello"), "hello");
    }

    #[test]
    fn test_escape_like_pattern_percent() {
        assert_eq!(escape_like_pattern("100%"), "100\\%");
    }

    #[test]
    fn test_escape_like_pattern_underscore() {
        assert_eq!(escape_like_pattern("user_name"), "user\\_name");
    }

    #[test]
    fn test_escape_like_pattern_backslash() {
        assert_eq!(escape_like_pattern("path\\to\\file"), "path\\\\to\\\\file");
    }

    #[test]
    fn test_escape_like_pattern_all_special() {
        assert_eq!(escape_like_pattern("%_\\test"), "\\%\\_\\\\test");
    }
}
