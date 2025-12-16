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
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{info, warn};
use warp::cors::Cors;
use warp::filters::ws::{WebSocket, Ws};
use warp::http::header::{AUTHORIZATION, CONTENT_TYPE};
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

use crate::handlers::dispatcher::{DispatchResult, MessageDispatcher};
use crate::handlers::handshake::HandshakeValidator;
use crate::handlers::messages::MessageHandler;
use crate::services::auth_service::TokenClaims;
use crate::services::{MessageQueueService, PresenceService};

use crate::handlers::{self, auth, conversation, server as server_handlers, user, websocket};
use crate::middleware::{auth as auth_middleware, rate_limit};

/// Server configuration
#[derive(Clone)]
pub struct ServerConfig {
    pub jwt_secret: String,
    pub max_message_size: usize,
    pub allowed_origins: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        let origins = std::env::var("CORS_ALLOWED_ORIGINS")
            .ok()
            .map(|value| {
                value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            })
            .filter(|list| !list.is_empty())
            .unwrap_or_else(|| vec!["*".to_string()]);

        Self {
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string()),
            max_message_size: 10 * 1024, // 10 KB
            allowed_origins: origins,
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
    pub user_service: Arc<crate::services::UserService>,
    pub global_rate_limiter: Arc<rate_limit::RateLimiter>,
    pub auth_rate_limiter: Arc<rate_limit::RateLimiter>,
    pub start_time: Instant,
}

impl ServerState {
    pub fn new(pool: SqlitePool, config: ServerConfig) -> Self {
        let connection_manager = Arc::new(websocket::ConnectionManager::new());
        let pool_for_services = pool.clone();
        let global_rate_limiter = Arc::new(rate_limit::RateLimiter::global());
        let auth_rate_limiter = Arc::new(rate_limit::RateLimiter::auth());
        let user_service = Arc::new(crate::services::UserService::new(pool.clone()));
        Self {
            pool,
            config,
            presence_service: PresenceService::new(
                pool_for_services.clone(),
                connection_manager.clone(),
            ),
            message_queue: MessageQueueService::new(pool_for_services, connection_manager.clone()),
            connection_manager,
            user_service,
            global_rate_limiter,
            auth_rate_limiter,
            start_time: Instant::now(),
        }
    }
}

/// Create all routes combined into a single filter
pub fn create_routes(
    state: ServerState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let state_clone_for_filter = state.clone();
    let state_filter = warp::any().map(move || state_clone_for_filter.clone());
    let cors = build_cors(&state.config);
    let rate_limit_filter = rate_limit::rate_limit_filter(state.global_rate_limiter.clone());

    let auth_service = Arc::new(crate::services::auth_service::AuthService::new(
        state.config.jwt_secret.clone(),
    ));
    let with_auth = auth_middleware::with_auth(auth_service.clone());

    // Health endpoint
    let health_route = warp::path!("health")
        .and(warp::get())
        .and(rate_limit_filter.clone())
        .and(state_filter.clone())
        .and_then(server_handlers::health);

    // Status endpoint
    let status_route = warp::path!("status")
        .and(warp::get())
        .and(rate_limit_filter.clone())
        .and(state_filter.clone())
        .and_then(server_handlers::status);

    // WebSocket endpoint with JWT authentication
    let websocket_route = warp::path!("socket")
        .and(warp::ws())
        .and(rate_limit_filter.clone())
        .and(warp::query::raw())
        .and(state_filter.clone())
        .and_then(handle_websocket_upgrade);

    // Auth routes
    let auth_routes = warp::path("auth").and(
        // POST /auth/signup
        warp::post()
            .and(warp::path("signup"))
            .and(rate_limit_filter.clone())
            .and(warp::body::json())
            .and(warp::addr::remote())
            .and(state_filter.clone())
            .and_then(handle_signup)
            .or(
                // POST /auth/login
                warp::post()
                    .and(warp::path("login"))
                    .and(rate_limit_filter.clone())
                    .and(warp::body::json())
                    .and(warp::addr::remote())
                    .and(state_filter.clone())
                    .and_then(handle_login),
            )
            .or(
                // POST /auth/logout
                warp::post()
                    .and(warp::path("logout"))
                    .and(warp::path::end())
                    .and(with_auth.clone())
                    .and(rate_limit_filter.clone())
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
            .and(rate_limit_filter.clone())
            .and(state_filter.clone())
            .and_then(handle_get_current_user)
            .or(
                // DELETE /user/me
                warp::delete()
                    .and(warp::path("me"))
                    .and(warp::path::end())
                    .and(with_auth.clone())
                    .and(rate_limit_filter.clone())
                    .and(warp::body::json())
                    .and(state_filter.clone())
                    .and_then(handle_delete_account),
            )
            .or(
                // POST /user/change-password
                warp::post()
                    .and(warp::path("change-password"))
                    .and(warp::path::end())
                    .and(with_auth.clone())
                    .and(rate_limit_filter.clone())
                    .and(warp::body::json())
                    .and(state_filter.clone())
                    .and_then(handle_change_password),
            ),
    );

    // User Search route (GET /users/search)
    // Note: This was separate in handlers/user.rs, likely mapped to /users/search
    let users_routes = warp::path("users").and(
        warp::path("search")
            .and(warp::get())
            .and(with_auth.clone())
            .and(rate_limit_filter.clone())
            .and(warp::query::<user::SearchQuery>())
            .and(state_filter.clone())
            .and_then(|user_id, query, state: ServerState| async move {
                user::search_users(user_id, query, state.user_service.clone()).await
            }),
    );

    // Conversations routes (stubs for Phase 3+)
    let conversation_routes = warp::path("conversations").and(
        // GET /conversations (list conversations)
        warp::get()
            .and(warp::path::end())
            .and(with_auth.clone())
            .and(rate_limit_filter.clone())
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
                    .and(rate_limit_filter.clone())
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
                    .and(rate_limit_filter.clone())
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
                    .and(rate_limit_filter.clone())
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
        .or(status_route)
        .or(auth_routes)
        .or(user_routes)
        .or(users_routes)
        .or(conversation_routes)
        .with(cors)
        .with(warp::reply::with::default_header(
            "Strict-Transport-Security",
            "max-age=63072000; includeSubDomains; preload",
        ))
        .with(warp::reply::with::default_header("X-Frame-Options", "DENY"))
        .with(warp::reply::with::default_header(
            "X-Content-Type-Options",
            "nosniff",
        ))
        .with(warp::reply::with::default_header(
            "Referrer-Policy",
            "no-referrer",
        ))
        .with(warp::reply::with::default_header(
            "Permissions-Policy",
            "geolocation=(), microphone=()",
        ))
        .with(warp::reply::with::default_header(
            "X-XSS-Protection",
            "1; mode=block",
        ))
        .with(warp::log("chat_server"))
        .recover(handle_rejection)
}

/// Build CORS policy based on server configuration
fn build_cors(config: &ServerConfig) -> Cors {
    let mut cors = warp::cors()
        .allow_headers(vec![CONTENT_TYPE, AUTHORIZATION])
        .allow_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
        .max_age(86_400);

    let allow_any = config.allowed_origins.iter().any(|o| o == "*");

    if allow_any {
        cors = cors.allow_any_origin();
    } else {
        for origin in &config.allowed_origins {
            cors = cors.allow_origin(origin.as_str());
        }
    }

    cors.build()
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

                if let Err(error_response) = enforce_frame_size(&msg, state.config.max_message_size)
                {
                    warn!(
                        "Closing connection for user {} due to oversized frame",
                        user_id
                    );
                    let mut sender = ws_tx.lock().await;
                    let _ = sender.send(error_response).await;
                    break;
                }

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
    remote_addr: Option<SocketAddr>,
    state: ServerState,
) -> Result<impl Reply, Rejection> {
    info!("Signup request for username: {}", req.username);
    let ip = client_ip(remote_addr);

    state
        .auth_rate_limiter
        .check_and_record(&ip)
        .await
        .map_err(warp::reject::custom)?;

    // Delegate to auth handler
    auth::signup_handler(req, state.pool, state.config.jwt_secret).await
}

/// Handle login request
async fn handle_login(
    req: auth::LoginRequest,
    remote_addr: Option<SocketAddr>,
    state: ServerState,
) -> Result<impl Reply, Rejection> {
    info!("Login request for username: {}", req.username);
    let ip = client_ip(remote_addr);

    if state.auth_rate_limiter.is_rate_limited(&ip).await {
        return Err(warp::reject::custom(rate_limit::RateLimitExceeded {
            retry_after_secs: state.auth_rate_limiter.retry_after_seconds(&ip).await,
        }));
    }

    // Delegate to auth handler
    match auth::login_handler(req, state.pool.clone(), state.config.jwt_secret.clone()).await {
        Ok(response) => {
            state.auth_rate_limiter.reset(&ip).await;
            Ok(response)
        }
        Err(err) => {
            state.auth_rate_limiter.record_attempt(&ip).await;
            Err(err)
        }
    }
}

/// Handle logout request
async fn handle_logout(user_id: String, state: ServerState) -> Result<impl Reply, Rejection> {
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

fn client_ip(remote_addr: Option<SocketAddr>) -> String {
    remote_addr
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn enforce_frame_size(
    msg: &warp::ws::Message,
    max_message_size: usize,
) -> Result<(), warp::ws::Message> {
    let payload_len = msg.as_bytes().len();
    if payload_len > max_message_size {
        Err(websocket::ErrorResponse::invalid_message_length(
            payload_len,
            max_message_size,
        ))
    } else {
        Ok(())
    }
}

/// Handle rejections (errors) and convert to JSON responses
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    warn!("Request rejected: {:?}", err);
    eprintln!("DEBUG: Rejection details: {:?}", err);

    if let Some(api_err) = err.find::<handlers::ApiError>() {
        let body = handlers::ErrorBody {
            code: api_err.code.to_string(),
            message: api_err.message.clone(),
            details: api_err.details.clone(),
        };

        return Ok(warp::reply::with_status(
            warp::reply::json(&body),
            api_err.status,
        ));
    }

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
    } else if let Some(rate_err) = err.find::<rate_limit::RateLimitExceeded>() {
        let retry_after = rate_err.retry_after_secs;
        let mut body = serde_json::json!({
            "error": "RATE_LIMITED",
            "message": "Too many requests; retry later"
        });

        if retry_after > 0 {
            body["retryAfter"] = serde_json::json!(retry_after);
        }

        return Ok(warp::reply::with_status(
            warp::reply::json(&body),
            warp::http::StatusCode::TOO_MANY_REQUESTS,
        ));
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
    use std::sync::Arc;
    use warp::http::header::CONTENT_TYPE;
    use warp::http::header::{
        ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
        REFERRER_POLICY, STRICT_TRANSPORT_SECURITY, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
        X_XSS_PROTECTION,
    };
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

    #[tokio::test]
    async fn test_status_endpoint() {
        let pool = init_test_pool().await;
        let state = ServerState::new(pool, ServerConfig::default());
        let routes = create_routes(state);

        let resp = request().method("GET").path("/status").reply(&routes).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8_lossy(resp.body());
        assert!(body.contains("\"status\":\"running\""));
    }

    #[tokio::test]
    async fn test_global_rate_limit_blocks_requests() {
        let pool = init_test_pool().await;
        let mut state = ServerState::new(pool, ServerConfig::default());
        state.global_rate_limiter = Arc::new(rate_limit::RateLimiter::new(1, 60));
        let routes = create_routes(state);

        let first = request().method("GET").path("/health").reply(&routes).await;
        assert_eq!(first.status(), StatusCode::OK);

        let second = request().method("GET").path("/health").reply(&routes).await;
        assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_auth_rate_limit_blocks_after_failures() {
        let pool = init_test_pool().await;
        let mut state = ServerState::new(pool, ServerConfig::default());
        state.global_rate_limiter = Arc::new(rate_limit::RateLimiter::new(10, 60));
        state.auth_rate_limiter = Arc::new(rate_limit::RateLimiter::new(2, 60));
        let routes = create_routes(state);

        let login_req = auth::LoginRequest {
            username: "ghost".to_string(),
            password: "wrong".to_string(),
        };

        let first = request()
            .method("POST")
            .path("/auth/login")
            .header(CONTENT_TYPE, "application/json")
            .json(&login_req)
            .reply(&routes)
            .await;
        assert_eq!(first.status(), StatusCode::UNAUTHORIZED);

        let second = request()
            .method("POST")
            .path("/auth/login")
            .header(CONTENT_TYPE, "application/json")
            .json(&login_req)
            .reply(&routes)
            .await;
        assert_eq!(second.status(), StatusCode::UNAUTHORIZED);

        let third = request()
            .method("POST")
            .path("/auth/login")
            .header(CONTENT_TYPE, "application/json")
            .json(&login_req)
            .reply(&routes)
            .await;
        assert_eq!(third.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_security_headers_present() {
        let pool = init_test_pool().await;
        let state = ServerState::new(pool, ServerConfig::default());
        let routes = create_routes(state);

        let resp = request().method("GET").path("/health").reply(&routes).await;
        let headers = resp.headers();

        assert_eq!(
            headers
                .get(STRICT_TRANSPORT_SECURITY)
                .and_then(|h| h.to_str().ok()),
            Some("max-age=63072000; includeSubDomains; preload")
        );
        assert_eq!(
            headers.get(X_FRAME_OPTIONS).and_then(|h| h.to_str().ok()),
            Some("DENY")
        );
        assert_eq!(
            headers
                .get(X_CONTENT_TYPE_OPTIONS)
                .and_then(|h| h.to_str().ok()),
            Some("nosniff")
        );
        assert_eq!(
            headers.get(X_XSS_PROTECTION).and_then(|h| h.to_str().ok()),
            Some("1; mode=block")
        );
        assert_eq!(
            headers.get(REFERRER_POLICY).and_then(|h| h.to_str().ok()),
            Some("no-referrer")
        );
    }

    #[tokio::test]
    async fn test_cors_headers_present_on_options() {
        let pool = init_test_pool().await;
        let state = ServerState::new(pool, ServerConfig::default());
        let routes = create_routes(state);

        let resp = request()
            .method("OPTIONS")
            .path("/health")
            .header("Origin", "https://example.com")
            .header("Access-Control-Request-Method", "GET")
            .reply(&routes)
            .await;

        let headers = resp.headers();
        assert_eq!(
            headers
                .get(ACCESS_CONTROL_ALLOW_ORIGIN)
                .and_then(|h| h.to_str().ok()),
            Some("*")
        );
        let methods = headers
            .get(ACCESS_CONTROL_ALLOW_METHODS)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("")
            .to_ascii_uppercase();
        assert!(methods.contains("GET"));
        assert!(methods.contains("POST"));
        assert!(methods.contains("DELETE"));
        assert!(methods.contains("OPTIONS"));

        let allowed_headers = headers
            .get(ACCESS_CONTROL_ALLOW_HEADERS)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("")
            .to_ascii_lowercase();
        assert!(allowed_headers.contains("content-type"));
        assert!(allowed_headers.contains("authorization"));
    }

    #[test]
    fn test_enforce_frame_size_rejects_large_frames() {
        let msg = warp::ws::Message::text("123456");
        let result = enforce_frame_size(&msg, 4);
        assert!(result.is_err());
    }

    async fn init_test_pool() -> SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create test pool");

        let schema_sql = include_str!("db/migrations/001_initial_schema.sql");
        for statement in schema_sql.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(statement)
                .execute(&pool)
                .await
                .expect("Failed to run schema statement");
        }

        pool
    }
}
