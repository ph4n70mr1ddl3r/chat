//! Data Factory Functions for Tests
//!
//! Provides reusable factory functions for creating test data instead of hardcoding values.
//! Each factory accepts overrides to allow test-specific customization while providing sensible defaults.
//!
//! **Benefits**:
//! - Parallel safety: UUIDs prevent collisions between parallel tests
//! - Schema evolution: Default factories adapt as schema changes
//! - Clear test intent: Overrides show what matters for each test
//! - Reusability: Share factories across all test files

use chat_backend::models::{Conversation, Message, User};
use uuid::Uuid;

/// Create a test user with sensible defaults and override support.
///
/// # Example: Create default user
///
/// ```ignore
/// let alice = create_test_user(Default::default());
/// ```
///
/// # Example: Create user with specific role
///
/// ```ignore
/// let admin = create_test_user(UserFactoryOverrides {
///     username: Some("admin_user".to_string()),
///     ..Default::default()
/// });
/// ```
#[derive(Debug, Clone, Default)]
pub struct UserFactoryOverrides {
    pub username: Option<String>,
    pub password_hash: Option<String>,
    pub salt: Option<String>,
}

pub fn create_test_user(overrides: UserFactoryOverrides) -> User {
    User::new(
        overrides.username.unwrap_or_else(|| format!("user_{}", Uuid::new_v4())),
        overrides.password_hash.unwrap_or_else(|| format!("hash_{}", Uuid::new_v4())),
        overrides.salt.unwrap_or_else(|| format!("salt_{}", Uuid::new_v4())),
    )
}

/// Create a test conversation between two users.
///
/// # Example: Create default conversation
///
/// ```ignore
/// let user1 = create_test_user(Default::default());
/// let user2 = create_test_user(Default::default());
/// let conversation = create_test_conversation(&user1, &user2);
/// ```
pub fn create_test_conversation(user1: &User, user2: &User) -> Conversation {
    let mut ids = vec![user1.id.clone(), user2.id.clone()];
    ids.sort();
    Conversation::new(ids[0].clone(), ids[1].clone())
}

/// Create a test message with sensible defaults and override support.
///
/// # Example: Create message from sender to recipient
///
/// ```ignore
/// let sender = create_test_user(Default::default());
/// let recipient = create_test_user(Default::default());
/// let message = create_test_message(
///     &sender.id,
///     &recipient.id,
///     MessageFactoryOverrides {
///         content: Some("Hello!".to_string()),
///         ..Default::default()
///     }
/// );
/// ```
#[derive(Debug, Clone, Default)]
pub struct MessageFactoryOverrides {
    pub content: Option<String>,
    pub status: Option<String>,
}

pub fn create_test_message(
    sender_id: &str,
    recipient_id: &str,
    overrides: MessageFactoryOverrides,
) -> Message {
    Message {
        id: Uuid::new_v4().to_string(),
        conversation_id: "test_conversation".to_string(), // Will be set by test
        sender_id: sender_id.to_string(),
        recipient_id: recipient_id.to_string(),
        content: overrides.content.unwrap_or_else(|| "Test message".to_string()),
        status: overrides.status.unwrap_or_else(|| "sent".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_test_user_generates_unique_usernames() {
        let user1 = create_test_user(Default::default());
        let user2 = create_test_user(Default::default());

        assert_ne!(user1.username, user2.username);
        assert_ne!(user1.id, user2.id);
    }

    #[test]
    fn create_test_user_allows_overrides() {
        let overrides = UserFactoryOverrides {
            username: Some("alice".to_string()),
            ..Default::default()
        };
        let user = create_test_user(overrides);

        assert_eq!(user.username, "alice");
    }

    #[test]
    fn create_test_conversation_sorts_ids() {
        let user1 = create_test_user(UserFactoryOverrides {
            username: Some("user_a".to_string()),
            ..Default::default()
        });
        let user2 = create_test_user(UserFactoryOverrides {
            username: Some("user_b".to_string()),
            ..Default::default()
        });

        let conversation = create_test_conversation(&user1, &user2);

        // IDs should be sorted
        let mut ids = vec![user1.id.clone(), user2.id.clone()];
        ids.sort();
        assert_eq!(conversation.id, format!("{}-{}", ids[0], ids[1]));
    }

    #[test]
    fn create_test_message_has_defaults() {
        let sender_id = "sender_123";
        let recipient_id = "recipient_456";
        let message = create_test_message(sender_id, recipient_id, Default::default());

        assert_eq!(message.sender_id, sender_id);
        assert_eq!(message.recipient_id, recipient_id);
        assert_eq!(message.content, "Test message");
        assert_eq!(message.status, "sent");
    }
}
