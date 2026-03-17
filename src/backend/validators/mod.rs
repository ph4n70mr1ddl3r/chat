//! Input validation module for chat application
//!
//! Provides reusable validators for usernames, passwords, emails, and other inputs

use uuid::Uuid;

/// Minimum username length
const MIN_USERNAME_LENGTH: usize = 1;

/// Maximum username length
const MAX_USERNAME_LENGTH: usize = 50;

/// Minimum password length
const MIN_PASSWORD_LENGTH: usize = 8;

/// Maximum password length
const MAX_PASSWORD_LENGTH: usize = 128;

/// Maximum email local part length
const MAX_EMAIL_LOCAL_LENGTH: usize = 64;

/// Maximum email domain length
const MAX_EMAIL_DOMAIN_LENGTH: usize = 255;

/// Validate UUID format
///
/// Returns Ok(()) if the string is a valid UUID, Err with a message otherwise.
pub fn validate_uuid(id: &str) -> Result<(), String> {
    Uuid::parse_str(id)
        .map(|_| ())
        .map_err(|e| format!("Invalid UUID: {}", e))
}

/// Validate username
///
/// Rules:
/// - 1-50 characters
/// - Must start with alphanumeric or underscore
/// - Can contain alphanumeric, underscore, hyphen, and dot
/// - Case-sensitive
pub fn validate_username(username: &str) -> Result<(), String> {
    if username.len() < MIN_USERNAME_LENGTH || username.len() > MAX_USERNAME_LENGTH {
        return Err(format!(
            "Username must be between {} and {} characters",
            MIN_USERNAME_LENGTH, MAX_USERNAME_LENGTH
        ));
    }

    let first_char = username.chars().next();
    let Some(first_char) = first_char else {
        return Err("Username cannot be empty".to_string());
    };
    if !first_char.is_alphanumeric() && first_char != '_' {
        return Err("Username must start with a letter or underscore".to_string());
    }

    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(
            "Username can only contain alphanumeric characters, underscore, hyphen, and dot"
                .to_string(),
        );
    }

    Ok(())
}

/// Validate password strength (spec requirement)
///
/// Rules:
/// - 8-128 characters
/// - At least 1 uppercase letter
/// - At least 1 lowercase letter
/// - At least 1 digit
/// - At least 1 special character (non-alphanumeric)
pub fn validate_password(password: &str) -> Result<(), String> {
    let len = password.len();

    if len < MIN_PASSWORD_LENGTH {
        return Err(format!(
            "Password must be at least {} characters",
            MIN_PASSWORD_LENGTH
        ));
    }

    if len > MAX_PASSWORD_LENGTH {
        return Err(format!(
            "Password must be at most {} characters",
            MAX_PASSWORD_LENGTH
        ));
    }

    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;
    let mut has_special = false;

    for c in password.chars() {
        if c.is_control() {
            return Err("Password cannot contain control characters".to_string());
        }
        if c.is_uppercase() {
            has_upper = true;
        } else if c.is_lowercase() {
            has_lower = true;
        } else if c.is_numeric() {
            has_digit = true;
        } else if !c.is_alphanumeric() {
            has_special = true;
        }
    }

    if !has_upper {
        return Err("Password must contain at least one uppercase letter".to_string());
    }

    if !has_lower {
        return Err("Password must contain at least one lowercase letter".to_string());
    }

    if !has_digit {
        return Err("Password must contain at least one digit".to_string());
    }

    if !has_special {
        return Err("Password must contain at least one special character".to_string());
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

    let local_part = parts[0];
    let domain = parts[1];

    if local_part.len() > MAX_EMAIL_LOCAL_LENGTH {
        return Err(format!(
            "Email local part is too long (max {} characters)",
            MAX_EMAIL_LOCAL_LENGTH
        ));
    }

    if domain.len() > MAX_EMAIL_DOMAIN_LENGTH {
        return Err(format!(
            "Email domain is too long (max {} characters)",
            MAX_EMAIL_DOMAIN_LENGTH
        ));
    }

    if !domain.contains('.') {
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
        assert!(validate_username("user-name").is_ok());
        assert!(validate_username("user.name").is_ok());
        assert!(validate_username("first.last").is_ok());
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
        assert!(validate_username("user@name").is_err());
        assert!(validate_username("user name").is_err());
        assert!(validate_username("-startwithdash").is_err());
    }

    #[test]
    fn test_validate_password_valid() {
        assert!(validate_password("TestPass123!").is_ok());
        assert!(validate_password("AnotherPassword456@").is_ok());
    }

    #[test]
    fn test_validate_password_too_short() {
        assert!(validate_password("Test1").is_err());
    }

    #[test]
    fn test_validate_password_too_long() {
        let long_password = "TestPass123!".repeat(20);
        assert!(validate_password(&long_password).is_err());
    }

    #[test]
    fn test_validate_password_no_uppercase() {
        assert!(validate_password("testpass123!").is_err());
    }

    #[test]
    fn test_validate_password_no_lowercase() {
        assert!(validate_password("TESTPASS123!").is_err());
    }

    #[test]
    fn test_validate_password_no_digit() {
        assert!(validate_password("TestPass!!").is_err());
    }

    #[test]
    fn test_validate_password_no_special_char() {
        assert!(validate_password("TestPass123").is_err());
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

    #[test]
    fn test_validate_uuid_valid() {
        assert!(validate_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(validate_uuid("00000000-0000-0000-0000-000000000000").is_ok());
    }

    #[test]
    fn test_validate_uuid_invalid() {
        assert!(validate_uuid("not-a-uuid").is_err());
        assert!(validate_uuid("").is_err());
        assert!(validate_uuid("550e8400-e29b-41d4-a716").is_err());
    }
}
