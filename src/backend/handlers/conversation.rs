//! Conversation management endpoints
//!
//! Handles conversation creation, retrieval, and participant management

use crate::db::queries;
use crate::handlers::auth::ErrorResponse;
use crate::services::{ConversationService, MessageService};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::warn;
use warp::{reply, Rejection, Reply};

/// Start conversation request
#[derive(Debug, Deserialize)]
pub struct StartConversationRequest {
    pub other_user_id: String,
}

/// Conversation response
#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    pub conversation_id: String,
    pub participant_id: String,
    pub participant_username: String,
    pub participant_is_online: bool,
    pub created_at: i64,
    pub last_message_at: Option<i64>,
    pub message_count: i32,
}

/// Conversations list query parameters
#[derive(Debug, Deserialize)]
pub struct ConversationsQuery {
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

fn default_limit() -> u32 {
    20
}

/// Messages query parameters
#[derive(Debug, Deserialize)]
pub struct MessagesQuery {
    #[serde(default = "default_messages_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

fn default_messages_limit() -> u32 {
    50
}

/// Message response
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub sender_id: String,
    pub sender_username: String,
    pub recipient_id: String,
    pub content: String,
    pub created_at: i64,
    pub delivered_at: Option<i64>,
    pub status: String,
}

/// Handle POST /conversations/start
///
/// Creates or retrieves existing conversation between current user and other user
pub async fn start_conversation(
    user_id: String,
    request: StartConversationRequest,
    pool: SqlitePool,
) -> Result<impl Reply, Rejection> {
    // Validate other_user_id exists
    let other_user = match queries::find_user_by_id(&pool, &request.other_user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "USER_NOT_FOUND".to_string(),
                    message: "The specified user does not exist".to_string(),
                }),
                warp::http::StatusCode::NOT_FOUND,
            ));
        }
        Err(e) => {
            warn!("Failed to find user: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "DATABASE_ERROR".to_string(),
                    message: "Failed to verify user exists".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Prevent self-conversation
    if user_id == request.other_user_id {
        return Ok(reply::with_status(
            reply::json(&ErrorResponse {
                error: "INVALID_REQUEST".to_string(),
                message: "Cannot create conversation with yourself".to_string(),
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Check if other user is deleted
    if other_user.is_deleted() {
        return Ok(reply::with_status(
            reply::json(&ErrorResponse {
                error: "USER_DELETED".to_string(),
                message: "Cannot start conversation with deleted user".to_string(),
            }),
            warp::http::StatusCode::GONE,
        ));
    }

    // Create or get conversation
    let service = ConversationService::new(pool.clone());
    let (conversation, was_created) = match service
        .create_or_get_conversation(user_id.clone(), request.other_user_id.clone())
        .await
    {
        Ok(result) => result,
        Err(e) => {
            warn!("Failed to create conversation: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "DATABASE_ERROR".to_string(),
                    message: "Failed to create conversation".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Determine participant info (the other user)
    let participant_id = if conversation.user1_id == user_id {
        conversation.user2_id.clone()
    } else {
        conversation.user1_id.clone()
    };

    let status_code = if was_created {
        warp::http::StatusCode::CREATED
    } else {
        warp::http::StatusCode::OK
    };

    Ok(reply::with_status(
        reply::json(&ConversationResponse {
            conversation_id: conversation.id,
            participant_id: participant_id.clone(),
            participant_username: other_user.username,
            participant_is_online: other_user.is_online,
            created_at: conversation.created_at,
            last_message_at: conversation.last_message_at,
            message_count: conversation.message_count,
        }),
        status_code,
    ))
}

/// Handle GET /conversations?limit=20&offset=0
///
/// Returns list of conversations for the current user
pub async fn get_conversations(
    user_id: String,
    query: ConversationsQuery,
    pool: SqlitePool,
) -> Result<impl Reply, Rejection> {
    // Cap limit at 50
    let limit = query.limit.min(50);

    // Get conversations
    let service = ConversationService::new(pool.clone());
    let conversations = match service
        .get_user_conversations(&user_id, limit, query.offset)
        .await
    {
        Ok(convs) => convs,
        Err(e) => {
            warn!("Failed to get conversations: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "DATABASE_ERROR".to_string(),
                    message: "Failed to retrieve conversations".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Enrich with participant info
    let mut responses = Vec::new();
    for conv in conversations {
        // Determine participant (the other user)
        let participant_id = if conv.user1_id == user_id {
            &conv.user2_id
        } else {
            &conv.user1_id
        };

        // Fetch participant info
        let participant = match queries::find_user_by_id(&pool, participant_id).await {
            Ok(Some(user)) => user,
            Ok(None) => continue, // Skip if participant not found
            Err(e) => {
                warn!("Failed to fetch participant info: {}", e);
                continue;
            }
        };

        responses.push(ConversationResponse {
            conversation_id: conv.id,
            participant_id: participant.id,
            participant_username: participant.username,
            participant_is_online: participant.is_online,
            created_at: conv.created_at,
            last_message_at: conv.last_message_at,
            message_count: conv.message_count,
        });
    }

    Ok(reply::with_status(
        reply::json(&responses),
        warp::http::StatusCode::OK,
    ))
}

/// Handle GET /conversations/{id}/messages?limit=50&offset=0
///
/// Returns paginated messages for a conversation
pub async fn get_conversation_messages(
    user_id: String,
    conversation_id: String,
    query: MessagesQuery,
    pool: SqlitePool,
) -> Result<impl Reply, Rejection> {
    // Cap limit at 100
    let limit = query.limit.min(100);

    // Verify conversation exists and user is participant
    let conversation = match queries::get_conversation_by_id(&pool, &conversation_id).await {
        Ok(Some(conv)) => conv,
        Ok(None) => {
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "CONVERSATION_NOT_FOUND".to_string(),
                    message: "The specified conversation does not exist".to_string(),
                }),
                warp::http::StatusCode::NOT_FOUND,
            ));
        }
        Err(e) => {
            warn!("Failed to get conversation: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "DATABASE_ERROR".to_string(),
                    message: "Failed to retrieve conversation".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Verify user is participant
    if conversation.user1_id != user_id && conversation.user2_id != user_id {
        return Ok(reply::with_status(
            reply::json(&ErrorResponse {
                error: "FORBIDDEN".to_string(),
                message: "You are not a participant in this conversation".to_string(),
            }),
            warp::http::StatusCode::FORBIDDEN,
        ));
    }

    // Get messages
    let service = MessageService::new(pool.clone());
    let messages = match service
        .get_conversation_messages(&conversation_id, &user_id, limit, query.offset)
        .await
    {
        Ok(msgs) => msgs,
        Err(e) => {
            warn!("Failed to get messages: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "DATABASE_ERROR".to_string(),
                    message: "Failed to retrieve messages".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Enrich with sender username
    let mut responses = Vec::new();
    for msg in messages {
        // Fetch sender info
        let sender = match queries::find_user_by_id(&pool, &msg.sender_id).await {
            Ok(Some(user)) => user,
            Ok(None) => continue, // Skip if sender not found
            Err(e) => {
                warn!("Failed to fetch sender info: {}", e);
                continue;
            }
        };

        responses.push(MessageResponse {
            id: msg.id,
            sender_id: msg.sender_id,
            sender_username: sender.username,
            recipient_id: msg.recipient_id,
            content: msg.content,
            created_at: msg.created_at,
            delivered_at: msg.delivered_at,
            status: msg.status,
        });
    }

    Ok(reply::with_status(
        reply::json(&responses),
        warp::http::StatusCode::OK,
    ))
}
