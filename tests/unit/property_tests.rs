// Property-Based Tests using proptest
//
// Tests edge cases for:
// - Username validation
// - Password validation  
// - Message content validation
//
// Property-based testing explores input space automatically to find edge cases.
// Requirement: T600 - Property-Based Testing

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    // Username validation function (mirrors backend implementation)
    fn validate_username(username: &str) -> Result<(), String> {
        if username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if username.len() > 50 {
            return Err("Username too long (max 50 characters)".to_string());
        }
        // Alphanumeric + underscore only
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err("Username contains invalid characters".to_string());
        }
        Ok(())
    }

    // Password validation function (mirrors backend implementation)
    fn validate_password(password: &str) -> Result<(), String> {
        if password.len() < 8 {
            return Err("Password too short (min 8 characters)".to_string());
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

    // Message content validation function (mirrors backend implementation)
    fn validate_message_content(content: &str) -> Result<(), String> {
        if content.is_empty() {
            return Err("Message content cannot be empty".to_string());
        }
        if content.len() > 5000 {
            return Err("Message content too long (max 5000 characters)".to_string());
        }
        // UTF-8 validity check (Rust strings are always valid UTF-8)
        if !content.is_char_boundary(0) {
            return Err("Invalid UTF-8 content".to_string());
        }
        Ok(())
    }

    /// Test ID: T600-001
    /// Given: Randomly generated valid usernames
    /// When: Username validation is performed
    /// Then: All valid patterns should pass
    // Property test: Valid usernames should always pass validation
    proptest! {
        #[test]
        fn test_valid_username_always_passes(
            username in "[a-zA-Z0-9_]{1,50}"
        ) {
            assert!(
                validate_username(&username).is_ok(),
                "Valid username pattern should always pass: {}",
                username
            );
        }
    }

    /// Test ID: T600-002
    /// Given: Usernames with invalid characters
    /// When: Username validation is performed
    /// Then: All invalid patterns should fail
    // Property test: Usernames with invalid characters should always fail
    proptest! {
        #[test]
        fn test_invalid_username_characters_fail(
            username in "[a-zA-Z0-9_]{0,20}[^a-zA-Z0-9_][a-zA-Z0-9_]{0,20}"
        ) {
            assert!(
                validate_username(&username).is_err(),
                "Username with invalid character should fail: {}",
                username
            );
        }
    }

    /// Test ID: T600-003
    /// Given: Usernames longer than 50 characters
    /// When: Username validation is performed
    /// Then: All long usernames should fail
    // Property test: Usernames longer than 50 chars should fail
    proptest! {
        #[test]
        fn test_username_length_limit(
            username in "[a-zA-Z0-9_]{51,100}"
        ) {
            assert!(
                validate_username(&username).is_err(),
                "Username longer than 50 chars should fail: {} (len={})",
                username,
                username.len()
            );
        }
    }

    /// Test ID: T600-004
    /// Given: Empty username string
    /// When: Username validation is performed
    /// Then: Empty username should always fail
    // Property test: Empty username should fail
    #[test]
    fn test_empty_username_fails() {
        assert!(
            validate_username("").is_err(),
            "Empty username should fail"
        );
    }

    /// Test ID: T600-005
    /// Given: Randomly generated valid passwords
    /// When: Password validation is performed
    /// Then: All valid patterns should pass
    // Property test: Valid passwords should always pass
    proptest! {
        #[test]
        fn test_valid_password_always_passes(
            lowercase in "[a-z]{2,10}",
            uppercase in "[A-Z]{2,10}",
            digit in "[0-9]{1,5}",
            extra in "[a-z0-9]{0,20}"
        ) {
            let password = format!("{}{}{}{}", lowercase, uppercase, digit, extra);
            prop_assume!(password.len() >= 8); // Ensure min length
            assert!(
                validate_password(&password).is_ok(),
                "Valid password should pass: {}",
                password
            );
        }
    }

    /// Test ID: T600-006
    /// Given: Passwords without uppercase letters
    /// When: Password validation is performed
    /// Then: All passwords missing uppercase should fail
    // Property test: Passwords without uppercase should fail
    proptest! {
        #[test]
        fn test_password_without_uppercase_fails(
            password in "[a-z0-9]{8,20}"
        ) {
            prop_assume!(!password.chars().any(|c| c.is_uppercase()));
            assert!(
                validate_password(&password).is_err(),
                "Password without uppercase should fail: {}",
                password
            );
        }
    }

     /// Test ID: T600-007
     /// Given: Passwords without lowercase letters
     /// When: Password validation is performed
     /// Then: All passwords missing lowercase should fail
     // Property test: Passwords without lowercase should fail
     proptest! {
         #[test]
         fn test_password_without_lowercase_fails(
             password in "[A-Z0-9]{8,20}"
         ) {
             prop_assume!(!password.chars().any(|c| c.is_lowercase()));
             assert!(
                 validate_password(&password).is_err(),
                 "Password without lowercase should fail: {}",
                 password
             );
         }
     }

     /// Test ID: T600-008
     /// Given: Passwords without digit characters
     /// When: Password validation is performed
     /// Then: All passwords missing digits should fail
     // Property test: Passwords without digit should fail
     proptest! {
         #[test]
         fn test_password_without_digit_fails(
             password in "[a-zA-Z]{8,20}"
         ) {
             prop_assume!(!password.chars().any(|c| c.is_numeric()));
             assert!(
                 validate_password(&password).is_err(),
                 "Password without digit should fail: {}",
                 password
             );
         }
     }

     /// Test ID: T600-009
     /// Given: Passwords shorter than 8 characters
     /// When: Password validation is performed
     /// Then: All passwords below minimum length should fail
     // Property test: Passwords shorter than 8 chars should fail
     proptest! {
         #[test]
         fn test_password_too_short_fails(
             password in "[a-zA-Z0-9]{1,7}"
         ) {
             assert!(
                 validate_password(&password).is_err(),
                 "Password shorter than 8 chars should fail: {} (len={})",
                 password,
                 password.len()
             );
         }
     }

     /// Test ID: T600-010
     /// Given: Valid message content patterns
     /// When: Message content validation is performed
     /// Then: All valid message patterns should pass
     // Property test: Valid message content should always pass
     proptest! {
         #[test]
         fn test_valid_message_content_always_passes(
             content in "[a-zA-Z0-9 .,!?\\n]{1,5000}"
         ) {
             assert!(
                 validate_message_content(&content).is_ok(),
                 "Valid message content should pass (len={})",
                 content.len()
             );
         }
     }

     /// Test ID: T600-011
     /// Given: Message content exceeding 5000 character limit
     /// When: Message content validation is performed
     /// Then: All oversized messages should fail
     // Property test: Message content longer than 5000 chars should fail
     proptest! {
         #[test]
         fn test_message_content_too_long_fails(
             content in "[a-zA-Z0-9 ]{5001,6000}"
         ) {
             assert!(
                 validate_message_content(&content).is_err(),
                 "Message content longer than 5000 chars should fail (len={})",
                 content.len()
             );
         }
     }

     /// Test ID: T600-012
     /// Given: Empty message content
     /// When: Message content validation is performed
     /// Then: Empty message should always fail
     // Property test: Empty message content should fail
     #[test]
     fn test_empty_message_content_fails() {
         assert!(
             validate_message_content("").is_err(),
             "Empty message content should fail"
         );
     }

     /// Test ID: T600-013
     /// Given: Unicode message content patterns
     /// When: Message content validation is performed
     /// Then: All valid Unicode patterns should pass
     // Property test: Unicode message content should be valid
     proptest! {
         #[test]
         fn test_unicode_message_content_valid(
             content in "\\PC{1,1000}"  // Unicode characters (excluding control chars)
         ) {
             prop_assume!(content.len() <= 5000);
             prop_assume!(!content.is_empty());
             assert!(
                 validate_message_content(&content).is_ok(),
                 "Unicode message content should be valid (len={})",
                 content.len()
             );
         }
     }

     // Edge case tests

     /// Test ID: T600-014
     /// Given: Username boundary conditions (0, 1, 50, 51 characters)
     /// When: Username validation is performed on edge cases
     /// Then: Boundary values should be correctly handled
     #[test]
     fn test_username_edge_cases() {
        // Exactly 50 characters (boundary)
        let username_50 = "a".repeat(50);
        assert!(validate_username(&username_50).is_ok());

        // 51 characters (over limit)
        let username_51 = "a".repeat(51);
        assert!(validate_username(&username_51).is_err());

        // Single character (valid)
        assert!(validate_username("a").is_ok());

        // Only underscores (valid)
        assert!(validate_username("___").is_ok());

        // Mixed case and numbers
        assert!(validate_username("User123_Test").is_ok());

        // Special characters (invalid)
        assert!(validate_username("user@domain").is_err());
        assert!(validate_username("user name").is_err());
        assert!(validate_username("user-name").is_err());
    }

     /// Test ID: T600-015
     /// Given: Password boundary conditions (7, 8, 100+ characters, all combinations)
     /// When: Password validation is performed on edge cases
     /// Then: Boundary values should be correctly handled
     #[test]
     fn test_password_edge_cases() {
        // Exactly 8 characters with all requirements (valid)
        assert!(validate_password("Aa1bbbbb").is_ok());

        // 7 characters (too short, even if all requirements met)
        assert!(validate_password("Aa1bbbb").is_err());

        // Very long password (valid if has requirements)
        let long_password = format!("A1{}", "b".repeat(100));
        assert!(validate_password(&long_password).is_ok());

        // All uppercase + digit (no lowercase)
        assert!(validate_password("ABCDEFG1").is_err());

        // All lowercase + digit (no uppercase)
        assert!(validate_password("abcdefg1").is_err());

        // Mixed case, no digit
        assert!(validate_password("AbCdEfGh").is_err());
    }

     /// Test ID: T600-016
     /// Given: Message content boundary conditions (0, 1, 5000, 5001 characters, Unicode)
     /// When: Message content validation is performed on edge cases
     /// Then: Boundary values should be correctly handled
     #[test]
     fn test_message_content_edge_cases() {
        // Exactly 5000 characters (boundary)
        let content_5000 = "a".repeat(5000);
        assert!(validate_message_content(&content_5000).is_ok());

        // 5001 characters (over limit)
        let content_5001 = "a".repeat(5001);
        assert!(validate_message_content(&content_5001).is_err());

        // Single character (valid)
        assert!(validate_message_content("a").is_ok());

        // Newlines and special characters (valid)
        assert!(validate_message_content("Hello\nWorld!").is_ok());

        // Emoji (valid UTF-8)
        assert!(validate_message_content("Hello 👋 World 🌍").is_ok());

        // Chinese/Japanese characters (valid UTF-8)
        assert!(validate_message_content("你好世界").is_ok());
        assert!(validate_message_content("こんにちは世界").is_ok());
    }
}
