// Integration tests for message search within a conversation
//
// Covers happy path, empty result set, and behavior when participants are offline.
// Requirement: T097 - Message Search

use chat_backend::db::queries;
use chat_backend::models::{Conversation, Message, User};
use crate::fixtures::setup_test_db;

fn sorted_conversation(user1_id: &str, user2_id: &str) -> Conversation {
    if user1_id < user2_id {
        Conversation::new(user1_id.to_string(), user2_id.to_string())
    } else {
        Conversation::new(user2_id.to_string(), user1_id.to_string())
    }
}

/// Test ID: T097-001
/// Given: Multiple messages in a conversation with different content
/// When: Searching for a specific keyword
/// Then: Only messages containing that keyword should be returned
#[tokio::test]
async fn search_returns_matching_messages() {
    let pool = setup_test_db().await;

    let alice = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    let bob = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());

    queries::insert_user(&pool, &alice).await.unwrap();
    queries::insert_user(&pool, &bob).await.unwrap();

    let conversation = sorted_conversation(&alice.id, &bob.id);
    queries::insert_conversation(&pool, &conversation)
        .await
        .unwrap();

    let hello = Message::new(
        conversation.id.clone(),
        alice.id.clone(),
        bob.id.clone(),
        "Hello Bob, this is a search test".to_string(),
    );
    let unrelated = Message::new(
        conversation.id.clone(),
        alice.id.clone(),
        bob.id.clone(),
        "Random content with no keyword".to_string(),
    );

    queries::insert_message(&pool, &hello).await.unwrap();
    queries::insert_message(&pool, &unrelated).await.unwrap();

    let results = queries::search_messages_in_conversation(&pool, &conversation.id, "search", 10)
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].content, hello.content);
}

/// Test ID: T097-002
/// Given: Messages in a conversation
/// When: Searching for text that doesn't appear in any message
/// Then: Search should return an empty result set
#[tokio::test]
    let pool = setup_test_db().await;

    let alice = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    let bob = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());

    queries::insert_user(&pool, &alice).await.unwrap();
    queries::insert_user(&pool, &bob).await.unwrap();

    let conversation = sorted_conversation(&alice.id, &bob.id);
    queries::insert_conversation(&pool, &conversation)
        .await
        .unwrap();

    let message = Message::new(
        conversation.id.clone(),
        alice.id.clone(),
        bob.id.clone(),
        "This message will not match".to_string(),
    );
    queries::insert_message(&pool, &message).await.unwrap();

    let results = queries::search_messages_in_conversation(&pool, &conversation.id, "nomatch", 10)
        .await
        .unwrap();

    assert!(results.is_empty());
}

/// Test ID: T097-003
/// Given: A conversation where both participants are offline
/// When: Performing a message search in that conversation
/// Then: Search should still return matching messages regardless of participant online status
#[tokio::test]
    let pool = setup_test_db().await;

    let mut alice = User::new("alice".to_string(), "hash1".to_string(), "salt1".to_string());
    let mut bob = User::new("bob".to_string(), "hash2".to_string(), "salt2".to_string());

    // Mark both users online then offline to simulate status changes
    alice.is_online = true;
    bob.is_online = true;

    queries::insert_user(&pool, &alice).await.unwrap();
    queries::insert_user(&pool, &bob).await.unwrap();

    // Flip to offline after insert
    queries::update_online_status(&pool, &alice.id, false)
        .await
        .unwrap();
    queries::update_online_status(&pool, &bob.id, false)
        .await
        .unwrap();

    let conversation = sorted_conversation(&alice.id, &bob.id);
    queries::insert_conversation(&pool, &conversation)
        .await
        .unwrap();

    let message = Message::new(
        conversation.id.clone(),
        alice.id.clone(),
        bob.id.clone(),
        "Searchable even when offline".to_string(),
    );
    queries::insert_message(&pool, &message).await.unwrap();

    let results = queries::search_messages_in_conversation(&pool, &conversation.id, "offline", 10)
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].content, message.content);
}
