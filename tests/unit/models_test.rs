//! Unit tests for domain models
//! Requirement: T502 - Domain Models

#[cfg(test)]
mod tests {
    use chat_backend::models::{User, Conversation, Message};

    /// Test ID: T502-001
    /// Given: User model creation parameters
    /// When: User::new() is called
    /// Then: User should be created with default values (not online, not deleted, active)
    #[test]
    fn test_user_creation() {
        let user = User::new(
            "alice".to_string(),
            "hash123".to_string(),
            "salt456".to_string(),
        );
        assert_eq!(user.username, "alice");
        assert!(!user.is_online);
        assert!(!user.is_deleted());
        assert!(user.is_active());
    }

    /// Test ID: T502-002
    /// Given: Conversation creation with two user IDs
    /// When: Conversation validation is performed
    /// Then: Valid conversations pass, self-conversations and wrong-order conversations fail
    #[test]
    fn test_conversation_validation() {
        let conv = Conversation::new(
            "user_a".to_string(),
            "user_b".to_string(),
        );
        assert!(conv.validate().is_ok());

        // Test self-conversation prevention
        let self_conv = Conversation::new(
            "user_a".to_string(),
            "user_a".to_string(),
        );
        assert!(self_conv.validate().is_err());

        // Test ordering constraint
        let wrong_order = Conversation::new(
            "user_b".to_string(),
            "user_a".to_string(),
        );
        assert!(wrong_order.validate().is_err());
    }

    /// Test ID: T502-003
    /// Given: Message model creation parameters
    /// When: Message::new() is called
    /// Then: Message should be created with provided content and metadata
    #[test]
    fn test_message_creation() {
        let msg = Message::new(
            "conv123".to_string(),
            "sender123".to_string(),
            "recipient456".to_string(),
            "Hello!".to_string(),
        );
        assert_eq!(msg.content, "Hello!");
        assert_eq!(msg.status, "pending");
        assert!(msg.is_pending());
        assert!(!msg.is_anonymized);
    }

    /// Test ID: T502-004
    /// Given: Message content with various validation scenarios
    /// When: Message validation is performed
    /// Then: Valid messages pass, empty messages fail
    #[test]
    fn test_message_validation() {
        let valid_msg = Message::new(
            "conv123".to_string(),
            "sender123".to_string(),
            "recipient456".to_string(),
            "Hello!".to_string(),
        );
        assert!(valid_msg.validate().is_ok());

        // Test empty content
        let empty_msg = Message::new(
            "conv123".to_string(),
            "sender123".to_string(),
            "recipient456".to_string(),
            "".to_string(),
        );
        assert!(empty_msg.validate().is_err());

        // Test overly long content
        let long_content = "x".repeat(5001);
        let long_msg = Message::new(
            "conv123".to_string(),
            "sender123".to_string(),
            "recipient456".to_string(),
            long_content,
        );
        assert!(long_msg.validate().is_err());

        // Test self-message
        let self_msg = Message::new(
            "conv123".to_string(),
            "sender123".to_string(),
            "sender123".to_string(),
            "Hello!".to_string(),
        );
        assert!(self_msg.validate().is_err());
    }

    /// Test ID: T502-005
    /// Given: Message with different status values
    /// When: Message status check methods are called
    /// Then: Correct status flags should be returned for each status
    #[test]
    fn test_message_status_checks() {
        let mut msg = Message::new(
            "conv123".to_string(),
            "sender123".to_string(),
            "recipient456".to_string(),
            "Hello!".to_string(),
        );
        assert!(msg.is_pending());
        assert!(!msg.is_delivered());
        assert!(!msg.is_failed());

        msg.status = "delivered".to_string();
        assert!(!msg.is_pending());
        assert!(msg.is_delivered());
        assert!(!msg.is_failed());

        msg.status = "failed".to_string();
        assert!(!msg.is_pending());
        assert!(!msg.is_delivered());
        assert!(msg.is_failed());
    }

    /// Test ID: T502-006
    /// Given: User model with deleted_at timestamp
    /// When: User deletion state is checked
    /// Then: User should be marked as deleted and inactive
    #[test]
    fn test_user_deletion_state() {
        let mut user = User::new(
            "alice".to_string(),
            "hash123".to_string(),
            "salt456".to_string(),
        );
        assert!(user.is_active());
        
        user.deleted_at = Some(1000);
        assert!(!user.is_active());
        assert!(user.is_deleted());
    }
}
