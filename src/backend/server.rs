//! Warp HTTP server and WebSocket router
//!
//! Defines all HTTP routes and WebSocket endpoints for the chat application.
//! Routes:
//! - GET /health - server health check
//! - GET /socket - WebSocket upgrade endpoint (requires JWT authentication)
//! - POST /auth/signup - user registration
//! - POST /auth/login - user authentication
//! - GET /conversations/* - conversation management (stubs for Phase 3+)

use anyhow::Error;
use futures::{SinkExt, StreamExt};
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::{info, warn};
use tokio::sync::mpsc;
use warp::filters::body::BodyDeserializeError;
use warp::filters::ws::ws;
use warp::filters::ws::{WebSocket, Ws};
use warp::http::StatusCode;
use warp::reject;
use warp::{Filter, Rejection, Reply};

use crate::handlers::dispatcher::{DispatchResult, MessageDispatcher};
use crate::handlers::handshake::HandshakeValidator;
use crate::handlers::messages::MessageHandler;
use crate::services::auth_service::TokenClaims;
use crate::services::{MessageQueueService, PresenceService};

use crate::handlers::{auth, conversation, user, websocket};
use crate::middleware::auth as auth_middleware;

/// Server configuration
#[derive(Clone)]
pub struct ServerConfig {
    pub jwt_secret: String,
    pub max_message_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string()),
            max_message_size: 10 * 1024, // 10 KB
        }
    }
}

/// Server state shared across routes
#[derive(Clone)]
pub struct ServerState {
    pub pool: SqlitePool,
    pub config: ServerConfig,
    pub connection_manager: Arc<websocket::ConnectionManager>,
    pub presence_service: PresenceService,
    pub message_queue: MessageQueueService,
}

impl ServerState {
    pub fn new(pool: SqlitePool, config: ServerConfig) -> Self {
        let connection_manager = Arc::new(websocket::ConnectionManager::new());
        let pool_for_services = pool.clone();
        Self {
            pool,
            config,
            presence_service: PresenceService::new(
                pool_for_services.clone(),
                connection_manager.clone(),
            ),
            message_queue: MessageQueueService::new(pool_for_services, connection_manager.clone()),
            connection_manager,
        }
    }
}

/// Create all routes combined into a single filter
pub fn create_routes(
    state: ServerState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let state_clone_for_filter = state.clone();
    let state_filter = warp::any().map(move || state_clone_for_filter.clone());

    let auth_service = Arc::new(crate::services::auth_service::AuthService::new(
        state.config.jwt_secret.clone(),
    ));
    let with_auth = auth_middleware::with_auth(auth_service.clone());

    // Health endpoint
    let health_route = warp::path!("health").and(warp::get()).map(|| {
        info!("Health check requested");
        warp::reply::json(&serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }))
    });

    // WebSocket endpoint with JWT authentication
    let websocket_route = warp::path!("socket")
        .and(warp::ws())
        .and(warp::query::raw())
        .and(state_filter.clone())
        .and_then(handle_websocket_upgrade);

    // Auth routes
    let auth_routes = warp::path("auth").and(
        // POST /auth/signup
        warp::post()
            .and(warp::path("signup"))
            .and(warp::body::json())
            .and(state_filter.clone())
            .and_then(handle_signup)
            .or(
                // POST /auth/login
                warp::post()
                    .and(warp::path("login"))
                    .and(warp::body::json())
                    .and(state_filter.clone())
                    .and_then(handle_login),
            )
            .or(
                // POST /auth/logout
                warp::post()
                    .and(warp::path("logout"))
                    .and(warp::path::end())
                    .and(with_auth.clone())
                    .and(state_filter.clone())
                    .and_then(handle_logout),
            ),
    );

    // User routes
    let user_routes = warp::path("user").and(
        // GET /user/me
        warp::get()
            .and(warp::path("me"))
            .and(warp::path::end())
            .and(with_auth.clone())
            .and(state_filter.clone())
            .and_then(handle_get_current_user)
            .or(
                // DELETE /user/me
                warp::delete()
                    .and(warp::path("me"))
                    .and(warp::path::end())
                    .and(with_auth.clone())
                    .and(warp::body::json())
                    .and(state_filter.clone())
                    .and_then(handle_delete_account)
            )
            .or(
                // POST /user/change-password
                warp::post()
                    .and(warp::path("change-password"))
                    .and(warp::path::end())
                    .and(with_auth.clone())
                    .and(warp::body::json())
                    .and(state_filter.clone())
                    .and_then(handle_change_password)
            ),
    );
    
    // User Search route (GET /users/search)
    // Note: This was separate in handlers/user.rs, likely mapped to /users/search
    let users_routes = warp::path("users").and(
        warp::path("search")
        .and(warp::get())
        .and(with_auth.clone())
        .and(warp::query::<user::SearchQuery>())
        .and(state_filter.clone())
        .and_then(|user_id, query, state: ServerState| async move {
            user::search_users(user_id, query, state.pool).await
        })
    );

    // Conversations routes (stubs for Phase 3+)
    let conversation_routes = warp::path("conversations").and(
        // GET /conversations (list conversations)
        warp::get()
            .and(warp::path::end())
            .and(with_auth.clone())
            .and(warp::query::<conversation::ConversationsQuery>())
            .and(state_filter.clone())
            .and_then(|user_id, query, state: ServerState| async move {
                conversation::get_conversations(user_id, query, state.pool).await
            })
            .or(
                // POST /conversations/start (start new conversation)
                warp::post()
                    .and(warp::path("start"))
                    .and(warp::path::end())
                    .and(with_auth.clone())
                    .and(warp::body::json())
                    .and(state_filter.clone())
                    .and_then(|user_id, body, state: ServerState| async move {
                        conversation::start_conversation(user_id, body, state.pool).await
                    }),
            )
            .or(
                // GET /conversations/{id}/messages (get conversation messages)
                warp::get()
                    .and(warp::path::param())
                    .and(warp::path("messages"))
                    .and(warp::path::end())
                    .and(with_auth.clone())
                    .and(warp::query::<conversation::MessagesQuery>())
                    .and(state_filter.clone())
                    .and_then(
                        |conversation_id: String, user_id, query, state: ServerState| async move {
                            conversation::get_conversation_messages(
                                user_id,
                                conversation_id,
                                query,
                                state.pool,
                            )
                            .await
                        },
                    ),
            )
            .or(
                // GET /conversations/{id}/search?q=keyword
                warp::get()
                    .and(warp::path::param())
                    .and(warp::path("search"))
                    .and(warp::path::end())
                    .and(with_auth.clone())
                    .and(warp::query::<conversation::SearchMessagesQuery>())
                    .and(state_filter.clone())
                    .and_then(
                        |conversation_id: String, user_id, query, state: ServerState| async move {
                            conversation::search_messages(
                                user_id,
                                conversation_id,
                                query,
                                state.pool,
                            )
                            .await
                        },
                    ),
            ),
    );

    // Combine all routes
    health_route
        .or(websocket_route)
        .or(auth_routes)
        .or(user_routes)
        .or(users_routes)
        .or(conversation_routes)
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allow_headers(vec!["Content-Type", "Authorization"]),
        )
        .with(warp::log("chat_server"))
        .recover(handle_rejection)
}

/// Handle WebSocket upgrade with JWT authentication
async fn handle_websocket_upgrade(
    ws: Ws,
    query: String,
    state: ServerState,
) -> Result<impl Reply, Rejection> {
    info!("WebSocket connection request, query: {}", query);
    eprintln!("Calling validator with query: '{}'", query);

    // Validate JWT token using handshake validator
    let validator = HandshakeValidator::new(state.config.jwt_secret.clone());
    match validator.validate_upgrade(&query) {
        Ok(claims) => {
            info!(
                "WebSocket authentication successful for user: {}",
                claims.sub
            );
            Ok(ws.on_upgrade(move |socket| handle_websocket_connection(socket, state, claims)))
        }
        Err((status, message)) => {
            warn!("WebSocket authentication failed: {} - {}", status, message);
            eprintln!("DEBUG: creating custom rejection with status {}", status);
            // Reject the WebSocket upgrade with appropriate HTTP status
            Err(warp::reject::custom(WebSocketAuthError { status, message }))
        }
    }
}

/// Custom rejection type for WebSocket authentication errors
#[derive(Debug)]
struct WebSocketAuthError {
    status: StatusCode,
    message: String,
}

impl warp::reject::Reject for WebSocketAuthError {}

/// Handle WebSocket connection after upgrade
async fn handle_websocket_connection(socket: WebSocket, state: ServerState, claims: TokenClaims) {
    let user_id = claims.sub.clone();
    info!("WebSocket connection established for user: {}", user_id);

    // Lookup username from database
    let username = match crate::db::queries::find_user_by_id(&state.pool, &user_id).await {
        Ok(Some(user)) => user.username,
        _ => "unknown".to_string(),
    };

    // Register connection with connection manager
    let connection = websocket::ClientConnection::new(user_id.clone(), username);

    // Channel used by other parts of the system to push frames to this socket
    let (tx, mut rx) = mpsc::unbounded_channel::<warp::ws::Message>();
    let connection_id = state
        .connection_manager
        .register(connection.clone(), tx.clone())
        .await;
    info!(
        "Registered connection {} for user {}",
        connection_id, user_id
    );

    if let Err(e) = state.presence_service.mark_online(&user_id).await {
        warn!("Failed to mark presence online: {}", e);
    }

    // Create message handler
    let message_handler = MessageHandler::new(
        state.pool.clone(),
        state.connection_manager.clone(),
        state.message_queue.clone(),
    );

    let (ws_tx, mut ws_rx) = socket.split();
    let ws_tx = Arc::new(tokio::sync::Mutex::new(ws_tx));

    // Forward messages from channel to websocket sink
    let ws_tx_forward = ws_tx.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let mut sender = ws_tx_forward.lock().await;
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Process incoming messages
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                info!(
                    "Received WebSocket message from user {}: {:?}",
                    user_id, msg
                );

                // Parse and dispatch message
                let dispatch_result = MessageDispatcher::parse_message(&msg);

                match dispatch_result {
                    DispatchResult::RequiresAck { envelope, .. } => {
                        // Handle text message
                        match message_handler.handle_message(&envelope, &connection).await {
                            Ok(responses) => {
                                // Send all responses
                                for response in responses {
                                    let mut sender = ws_tx.lock().await;
                                    if let Err(e) = sender.send(response).await {
                                        warn!("Failed to send response: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Message handling error: {}", e);
                                let error_response = websocket::ErrorResponse::server_error(&e);
                                let mut sender = ws_tx.lock().await;
                                if let Err(e) = sender.send(error_response).await {
                                    warn!("Failed to send error response: {}", e);
                                }
                            }
                        }
                    }
                    DispatchResult::Success { msg_type, .. } => {
                        // Heartbeat, typing, etc. - just log
                        info!("Handled {} message from {}", msg_type, user_id);
                    }
                    DispatchResult::Error { error_msg } => {
                        // Send error response
                        let mut sender = ws_tx.lock().await;
                        if let Err(e) = sender.send(error_msg).await {
                            warn!("Failed to send error response: {}", e);
                        }
                    }
                    DispatchResult::Close { code, reason } => {
                        info!("Client requested close: {} - {}", code, reason);
                        break;
                    }
                }
            }
            Err(e) => {
                warn!("WebSocket error for user {}: {}", user_id, e);
                break;
            }
        }
    }

    // Unregister connection
    state
        .connection_manager
        .unregister(&user_id, &connection_id)
        .await;

    if let Err(e) = state.presence_service.mark_offline(&user_id).await {
        warn!("Failed to mark presence offline: {}", e);
    }
    info!("WebSocket connection closed for user: {}", user_id);
}

/// Handle signup request
async fn handle_signup(
    req: auth::SignupRequest,
    state: ServerState,
) -> Result<impl Reply, Rejection> {
    info!("Signup request for username: {}", req.username);

    // Delegate to auth handler
    auth::signup_handler(req, state.pool, state.config.jwt_secret).await
}

/// Handle login request
async fn handle_login(
    req: auth::LoginRequest,
    state: ServerState,
) -> Result<impl Reply, Rejection> {
    info!("Login request for username: {}", req.username);

    // Delegate to auth handler
    auth::login_handler(req, state.pool, state.config.jwt_secret).await
}

/// Handle logout request
async fn handle_logout(
    user_id: String,
    state: ServerState,
) -> Result<impl Reply, Rejection> {
    info!("Logout request for user: {}", user_id);
    auth::logout_handler(user_id, state.connection_manager, state.pool).await
}

/// Handle GET /user/me
async fn handle_get_current_user(
    user_id: String,
    state: ServerState,
) -> Result<impl Reply, Rejection> {
    user::get_current_user(user_id, state.pool).await
}

/// Handle DELETE /user/me
async fn handle_delete_account(
    user_id: String,
    req: user::DeleteAccountRequest,
    state: ServerState,
) -> Result<impl Reply, Rejection> {
    user::delete_account(user_id, req, state.pool).await
}

/// Handle POST /user/change-password
async fn handle_change_password(
    user_id: String,
    req: user::ChangePasswordRequest,
    state: ServerState,
) -> Result<impl Reply, Rejection> {
    user::change_password(user_id, req, state.pool).await
}

/// Handle rejections (errors) and convert to JSON responses
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    warn!("Request rejected: {:?}", err);
    eprintln!("DEBUG: Rejection details: {:?}", err);

    // Convert to JSON error response
    let (code, message) = if let Some(auth_err) = err.find::<WebSocketAuthError>() {
        eprintln!("DEBUG: Found WebSocketAuthError: {:?}", auth_err);
        (auth_err.status, auth_err.message.clone())
    } else if err.find::<auth_middleware::Unauthorized>().is_some() {
        (
            warp::http::StatusCode::UNAUTHORIZED,
            "Unauthorized".to_string(),
        )
    } else if err.is_not_found() {
        (warp::http::StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        (
            warp::http::StatusCode::BAD_REQUEST,
            "Invalid request body".to_string(),
        )
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        (
            warp::http::StatusCode::METHOD_NOT_ALLOWED,
            "Method Not Allowed".to_string(),
        )
    } else {
        (
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "error": code.as_u16(),
            "message": message,
        })),
        code,
    ))
}

/// Start the HTTP server
pub async fn start_server(
    port: u16,
    pool: SqlitePool,
    config: Option<ServerConfig>,
) -> anyhow::Result<()> {
    let config = config.unwrap_or_default();
    let state = ServerState::new(pool, config);

    // Start background workers (offline delivery)
    state
        .message_queue
        .load_pending_messages()
        .await
        .map_err(Error::msg)?;
    state.message_queue.start().await;

    let routes = create_routes(state);

    info!("Starting HTTP server on port {}", port);

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use warp::http::StatusCode;
    use warp::test::request;

    #[tokio::test]
    async fn test_health_endpoint() {
        let pool = init_test_pool().await;
        let state = ServerState::new(pool, ServerConfig::default());
        let routes = create_routes(state);

        let resp = request().method("GET").path("/health").reply(&routes).await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert!(String::from_utf8_lossy(resp.body()).contains("healthy"));
    }

    #[tokio::test]
    async fn test_websocket_upgrade_without_token() {
        let pool = init_test_pool().await;
        let state = ServerState::new(pool, ServerConfig::default());
        let routes = create_routes(state);

        let resp = request()
            .method("GET")
            .path("/socket?")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .reply(&routes)
            .await;

        eprintln!(
            "Response status: {}, body: {:?}",
            resp.status(),
            String::from_utf8_lossy(resp.body())
        );
        // Should reject WebSocket upgrade due to missing token
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_websocket_upgrade_with_invalid_token() {
        let pool = init_test_pool().await;
        let state = ServerState::new(pool, ServerConfig::default());
        let routes = create_routes(state);

        let resp = request()
            .method("GET")
            .path("/socket?token=invalid")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .reply(&routes)
            .await;

        // Should reject WebSocket upgrade due to invalid token
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_not_found() {
        let pool = init_test_pool().await;
        let state = ServerState::new(pool, ServerConfig::default());
        let routes = create_routes(state);

        let resp = request()
            .method("GET")
            .path("/nonexistent")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    async fn init_test_pool() -> SqlitePool {
        sqlx::SqlitePool::connect(":memory:")
            .await
            .expect("Failed to create test pool")
    }
}
