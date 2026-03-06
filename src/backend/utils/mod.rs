//! Utility functions shared across the backend

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
