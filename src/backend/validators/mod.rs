//! Input validation module for chat application
//!
//! Provides reusable validators for usernames, passwords, emails, and other inputs

/// Validate username
/// 
/// Rules:
/// - 1-50 characters
/// - Alphanumeric + underscore only
/// - Case-sensitive
pub fn validate_username(username: &str) -> Result<(), String> {
    if username.is_empty() || username.len() > 50 {
        return Err("Username must be between 1 and 50 characters".to_string());
    }
    
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Username can only contain alphanumeric characters and underscores".to_string());
    }
    
    Ok(())
}

/// Validate password strength (spec requirement)
///
/// Rules:
/// - Minimum 8 characters
/// - At least 1 uppercase letter
/// - At least 1 lowercase letter
/// - At least 1 digit
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters".to_string());
    }
    
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Password must contain at least one uppercase letter".to_string());
    }
    
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err("Password must contain at least one lowercase letter".to_string());
    }
    
    if !password.chars().any(|c| c.is_numeric()) {
        return Err("Password must contain at least one digit".to_string());
    }
    
    Ok(())
}

/// Validate email address (optional, for future use)
pub fn validate_email(email: &str) -> Result<(), String> {
    if email.is_empty() {
        return Err("Email cannot be empty".to_string());
    }
    
    if !email.contains('@') {
        return Err("Email must contain @ symbol".to_string());
    }
    
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err("Email format is invalid".to_string());
    }
    
    if !parts[1].contains('.') {
        return Err("Email domain must contain a dot".to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username_valid() {
        assert!(validate_username("alice").is_ok());
        assert!(validate_username("bob_123").is_ok());
        assert!(validate_username("_underscore_user_").is_ok());
    }

    #[test]
    fn test_validate_username_empty() {
        assert!(validate_username("").is_err());
    }

    #[test]
    fn test_validate_username_too_long() {
        let long_username = "a".repeat(51);
        assert!(validate_username(&long_username).is_err());
    }

    #[test]
    fn test_validate_username_invalid_chars() {
        assert!(validate_username("user-name").is_err());
        assert!(validate_username("user@name").is_err());
        assert!(validate_username("user name").is_err());
    }

    #[test]
    fn test_validate_password_valid() {
        assert!(validate_password("TestPass123").is_ok());
        assert!(validate_password("AnotherPassword456").is_ok());
    }

    #[test]
    fn test_validate_password_too_short() {
        assert!(validate_password("Test1").is_err());
    }

    #[test]
    fn test_validate_password_no_uppercase() {
        assert!(validate_password("testpass123").is_err());
    }

    #[test]
    fn test_validate_password_no_lowercase() {
        assert!(validate_password("TESTPASS123").is_err());
    }

    #[test]
    fn test_validate_password_no_digit() {
        assert!(validate_password("TestPass").is_err());
    }

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("test.user@domain.co.uk").is_ok());
    }

    #[test]
    fn test_validate_email_missing_at() {
        assert!(validate_email("userexample.com").is_err());
    }

    #[test]
    fn test_validate_email_invalid_format() {
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("user@").is_err());
    }

    #[test]
    fn test_validate_email_missing_domain_dot() {
        assert!(validate_email("user@example").is_err());
    }
}
