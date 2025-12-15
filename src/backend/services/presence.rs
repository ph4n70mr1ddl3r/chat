//! Presence service
//!
//! Tracks online/offline state and broadcasts presence updates to conversation participants.

use crate::db::queries;
use crate::handlers::websocket::ConnectionManager;
use chat_shared::protocol::{MessageEnvelope, PresenceData};
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::warn;
use warp::ws::Message as WsMessage;

#[derive(Clone)]
pub struct PresenceService {
    pool: SqlitePool,
    connection_manager: Arc<ConnectionManager>,
}

impl PresenceService {
    pub fn new(pool: SqlitePool, connection_manager: Arc<ConnectionManager>) -> Self {
        Self {
            pool,
            connection_manager,
        }
    }

    /// Mark user online and broadcast to participants.
    pub async fn mark_online(&self, user_id: &str) -> Result<(), String> {
        queries::update_online_status(&self.pool, user_id, true).await?;
        self.broadcast_presence(user_id, true).await
    }

    /// Mark user offline and broadcast to participants.
    pub async fn mark_offline(&self, user_id: &str) -> Result<(), String> {
        queries::update_online_status(&self.pool, user_id, false).await?;
        self.broadcast_presence(user_id, false).await
    }

    /// Broadcast presence update to all users that share a conversation with this user.
    async fn broadcast_presence(&self, user_id: &str, is_online: bool) -> Result<(), String> {
        let user = match queries::find_user_by_id(&self.pool, user_id).await? {
            Some(user) => user,
            None => return Ok(()), // User removed; nothing to do
        };

        let conversations =
            queries::get_user_conversations(&self.pool, user_id, 200, 0).await?;

        // Collect participant ids (the other user in each conversation)
        let mut recipients = Vec::new();
        for conv in conversations {
            let partner = if conv.user1_id == user_id {
                conv.user2_id
            } else {
                conv.user1_id
            };
            recipients.push(partner);
        }

        if recipients.is_empty() {
            return Ok(());
        }

        let envelope = MessageEnvelope {
            id: uuid::Uuid::new_v4().to_string(),
            msg_type: "presence".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            data: json!(PresenceData {
                user_id: user.id.clone(),
                username: user.username.clone(),
                is_online,
                last_seen_at: user
                    .last_seen_at
                    .unwrap_or_else(|| chrono::Utc::now().timestamp_millis())
                    as u64,
            }),
        };

        let message = WsMessage::text(
            serde_json::to_string(&envelope)
                .map_err(|e| format!("Failed to serialize presence: {}", e))?,
        );

        self.connection_manager
            .broadcast_to_users(recipients, message)
            .await;

        Ok(())
    }
}
