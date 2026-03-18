//! Database queries organized by domain
//!
//! This module provides database operations for the chat application.
//! Queries are organized into submodules by domain:
//! - `auth`: Authentication logging and rate limiting queries
//! - `users`: User CRUD operations
//! - `messages`: Message CRUD operations  
//! - `conversations`: Conversation CRUD operations
//! - `transactions`: Transaction helpers

pub mod auth;
pub mod conversations;
pub mod messages;
pub mod transactions;
pub mod users;

pub use auth::{get_failed_attempts, insert_auth_log, AuthEventType};
pub use conversations::{
    get_conversation_by_id, get_conversation_by_users,
    get_user_conversations, insert_conversation, update_conversation_stats,
};
pub use messages::{
    find_message_by_id, get_all_pending_messages,
    get_messages_by_conversation, get_pending_messages, insert_message,
    insert_message_or_ignore, mark_message_delivered, search_messages_in_conversation,
    update_message_status, VALID_STATUSES,
};
pub use users::{
    find_user_by_id, find_user_by_username, find_users_by_ids, insert_user,
    search_users_by_prefix, search_users_excluding_self, update_last_seen,
    update_online_status, update_password, soft_delete_user,
};
