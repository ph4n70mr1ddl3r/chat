//! Conversation service for managing one-to-one chats
//!
//! Handles conversation creation, retrieval, and enforcement of one-to-one constraints

use crate::db::queries;
use crate::models::Conversation;
use sqlx::SqlitePool;

/// Conversation service
pub struct ConversationService {
    pool: SqlitePool,
}

impl ConversationService {
    /// Create a new conversation service
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create or get existing conversation between two users
    ///
    /// Enforces one-to-one constraint (prevents duplicate conversations)
    /// Prevents self-chat (user1_id != user2_id)
    /// Returns (conversation, was_created: bool)
    pub async fn create_or_get_conversation(
        &self,
        user1_id: String,
        user2_id: String,
    ) -> Result<(Conversation, bool), String> {
        // Prevent self-chat
        if user1_id == user2_id {
            return Err("Cannot create conversation with self".to_string());
        }

        // Ensure user1_id < user2_id (lexical ordering for uniqueness)
        let (u1, u2) = if user1_id < user2_id {
            (user1_id, user2_id)
        } else {
            (user2_id, user1_id)
        };

        // Check if conversation already exists
        if let Some(conversation) =
            queries::get_conversation_by_users(&self.pool, &u1, &u2).await?
        {
            return Ok((conversation, false));
        }

        // Create new conversation
        let conversation = Conversation::new(u1, u2);
        conversation.validate()?;

        let created = queries::insert_conversation(&self.pool, &conversation).await?;
        Ok((created, true))
    }

    /// Get all conversations for a user
    ///
    /// Returns conversations sorted by last_message_at (most recent first)
    /// Supports pagination via limit and offset
    pub async fn get_user_conversations(
        &self,
        user_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Conversation>, String> {
        queries::get_user_conversations(&self.pool, user_id, limit, offset).await
    }

    /// Get conversation by ID
    ///
    /// Validates that the user is a participant
    pub async fn get_conversation_by_id(
        &self,
        conversation_id: &str,
        user_id: &str,
    ) -> Result<Option<Conversation>, String> {
        let conversation = queries::get_conversation_by_id(&self.pool, conversation_id).await?;

        // Verify user is participant
        if let Some(ref conv) = conversation {
            if conv.user1_id != user_id && conv.user2_id != user_id {
                return Err("User is not a participant in this conversation".to_string());
            }
        }

        Ok(conversation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::User;

    async fn setup_test_db() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        // Run migrations
        let schema_sql =
            include_str!("../../backend/db/migrations/001_initial_schema.sql");
        for statement in schema_sql.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(statement).execute(&pool).await.unwrap();
        }

        pool
    }

    #[tokio::test]
    async fn test_create_conversation() {
        let pool = setup_test_db().await;
        let service = ConversationService::new(pool.clone());

        // Create test users
        let user1 = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
        let user2 = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());

        queries::insert_user(&pool, &user1).await.unwrap();
        queries::insert_user(&pool, &user2).await.unwrap();

        // Create conversation
        let (conv, created) = service
            .create_or_get_conversation(user1.id.clone(), user2.id.clone())
            .await
            .unwrap();

        assert!(created);
        assert!(conv.user1_id < conv.user2_id); // Verify ordering
    }

    #[tokio::test]
    async fn test_prevent_duplicate_conversation() {
        let pool = setup_test_db().await;
        let service = ConversationService::new(pool.clone());

        let user1 = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
        let user2 = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());

        queries::insert_user(&pool, &user1).await.unwrap();
        queries::insert_user(&pool, &user2).await.unwrap();

        // Create first conversation
        let (conv1, created1) = service
            .create_or_get_conversation(user1.id.clone(), user2.id.clone())
            .await
            .unwrap();

        assert!(created1);

        // Try to create again (should return existing)
        let (conv2, created2) = service
            .create_or_get_conversation(user1.id.clone(), user2.id.clone())
            .await
            .unwrap();

        assert!(!created2);
        assert_eq!(conv1.id, conv2.id);
    }

    #[tokio::test]
    async fn test_prevent_self_chat() {
        let pool = setup_test_db().await;
        let service = ConversationService::new(pool.clone());

        let user1 = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
        queries::insert_user(&pool, &user1).await.unwrap();

        // Try to create conversation with self
        let result = service
            .create_or_get_conversation(user1.id.clone(), user1.id.clone())
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot create conversation with self"));
    }

    #[tokio::test]
    async fn test_conversation_ordering_independence() {
        let pool = setup_test_db().await;
        let service = ConversationService::new(pool.clone());

        let user1 = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
        let user2 = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());

        queries::insert_user(&pool, &user1).await.unwrap();
        queries::insert_user(&pool, &user2).await.unwrap();

        // Create conversation (user1, user2)
        let (conv1, _) = service
            .create_or_get_conversation(user1.id.clone(), user2.id.clone())
            .await
            .unwrap();

        // Create conversation (user2, user1) - should return same conversation
        let (conv2, created2) = service
            .create_or_get_conversation(user2.id.clone(), user1.id.clone())
            .await
            .unwrap();

        assert!(!created2); // Should not create new
        assert_eq!(conv1.id, conv2.id); // Should return same conversation
    }
}
