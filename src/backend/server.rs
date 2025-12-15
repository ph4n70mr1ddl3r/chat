//! Warp HTTP server and WebSocket router
//!
//! Defines all HTTP routes and WebSocket endpoints for the chat application.
//! Routes:
//! - GET /health - server health check
//! - GET /socket - WebSocket upgrade endpoint (requires JWT authentication)
//! - POST /auth/signup - user registration
//! - POST /auth/login - user authentication
//! - GET /conversations/* - conversation management (stubs for Phase 3+)

use futures::StreamExt;
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::{info, warn};
use warp::filters::body::BodyDeserializeError;
use warp::filters::ws::ws;
use warp::filters::ws::{WebSocket, Ws};
use warp::http::StatusCode;
use warp::reject;
use warp::{Filter, Rejection, Reply};

use crate::handlers::handshake::HandshakeValidator;
use crate::services::auth_service::TokenClaims;

use crate::handlers::auth;
use crate::handlers::websocket;

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
}

impl ServerState {
    pub fn new(pool: SqlitePool, config: ServerConfig) -> Self {
        Self {
            pool,
            config,
            connection_manager: Arc::new(websocket::ConnectionManager::new()),
        }
    }
}

/// Create all routes combined into a single filter
pub fn create_routes(
    state: ServerState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let state_filter = warp::any().map(move || state.clone());

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
            ),
    );

    // Conversations routes (stubs for Phase 3+)
    let conversation_routes = warp::path("conversations").and(
        // GET /conversations (list conversations)
        warp::get()
            .and(warp::path::end())
            .and(state_filter.clone())
            .and_then(handle_list_conversations)
            .or(
                // POST /conversations/start (start new conversation)
                warp::post()
                    .and(warp::path("start"))
                    .and(warp::path::end())
                    .and(warp::body::json())
                    .and(state_filter.clone())
                    .and_then(handle_start_conversation),
            )
            .or(
                // GET /conversations/{id}/messages (get conversation messages)
                warp::get()
                    .and(warp::path::param())
                    .and(warp::path("messages"))
                    .and(warp::path::end())
                    .and(state_filter.clone())
                    .and_then(handle_get_conversation_messages),
            ),
    );

    // Combine all routes
    health_route
        .or(websocket_route)
        .or(auth_routes)
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

    // Register connection with connection manager
    let connection = websocket::ClientConnection::new(user_id.clone(), "".to_string()); // username unknown
    let connection_id = state.connection_manager.register(connection).await;
    info!(
        "Registered connection {} for user {}",
        connection_id, user_id
    );

    let mut socket = socket;

    // Wait for messages
    while let Some(result) = socket.next().await {
        match result {
            Ok(msg) => {
                info!(
                    "Received WebSocket message from user {}: {:?}",
                    user_id, msg
                );
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

/// Handle list conversations (stub)
async fn handle_list_conversations(_state: ServerState) -> Result<impl Reply, Rejection> {
    info!("List conversations request");

    // Stub implementation for Phase 3
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "conversations": [],
            "total": 0,
        })),
        warp::http::StatusCode::OK,
    ))
}

/// Handle start conversation (stub)
async fn handle_start_conversation(
    _body: serde_json::Value,
    _state: ServerState,
) -> Result<impl Reply, Rejection> {
    info!("Start conversation request");

    // Stub implementation for Phase 3
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "error": "Not implemented",
            "message": "Conversation endpoints will be implemented in Phase 3",
        })),
        warp::http::StatusCode::NOT_IMPLEMENTED,
    ))
}

/// Handle get conversation messages (stub)
async fn handle_get_conversation_messages(
    _conversation_id: String,
    _state: ServerState,
) -> Result<impl Reply, Rejection> {
    info!("Get conversation messages request");

    // Stub implementation for Phase 3
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "error": "Not implemented",
            "message": "Conversation endpoints will be implemented in Phase 3",
        })),
        warp::http::StatusCode::NOT_IMPLEMENTED,
    ))
}

/// Handle rejections (errors) and convert to JSON responses
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    warn!("Request rejected: {:?}", err);
    eprintln!("DEBUG: Rejection details: {:?}", err);

    // Convert to JSON error response
    let (code, message) = if let Some(auth_err) = err.find::<WebSocketAuthError>() {
        eprintln!("DEBUG: Found WebSocketAuthError: {:?}", auth_err);
        (auth_err.status, auth_err.message.clone())
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
