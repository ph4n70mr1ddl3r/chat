//! Unit tests for domain models

#[cfg(test)]
mod tests {
    use chat_backend::models::{User, Conversation, Message};

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
