//! User and conversation creation utilities for tests.
//!
//! Provides factory functions to create test users and conversations with sensible defaults.

use chat_backend::db;
use chat_backend::models::{Conversation, User};
use sqlx::SqlitePool;

/// Creates two test users (alice and bob) and a conversation between them.
///
/// Creates a pair of users with standard test credentials and inserts them into the database,
/// then creates a conversation between them. The users are always created in sorted ID order
/// to ensure consistent conversation ID generation.
///
/// # Errors
///
/// Returns an error if database operations fail (user or conversation insertion).
///
/// # Example
///
/// ```ignore
/// let (sender, recipient, conversation) = create_users_and_conversation(&pool).await;
/// assert_eq!(sender.username, "alice");
/// assert_eq!(recipient.username, "bob");
/// ```
pub async fn create_users_and_conversation(pool: &SqlitePool) -> (User, User, Conversation) {
    let sender = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    let recipient = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());
    db::queries::insert_user(pool, &sender).await.unwrap();
    db::queries::insert_user(pool, &recipient).await.unwrap();

    let mut ids = vec![sender.id.clone(), recipient.id.clone()];
    ids.sort();
    let conversation = Conversation::new(ids[0].clone(), ids[1].clone());
    db::queries::insert_conversation(pool, &conversation)
        .await
        .unwrap();

    (sender, recipient, conversation)
}
