//! Server-level HTTP handlers (health and status endpoints)

use crate::handlers::{rejection, ApiError};
use crate::server::ServerState;
use serde::Serialize;
use tracing::{info, warn};
use warp::{reply, Rejection, Reply};

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    timestamp: i64,
    uptime_seconds: u64,
}

#[derive(Serialize)]
struct StatusResponse {
    status: &'static str,
    version: &'static str,
    timestamp: i64,
    uptime_seconds: u64,
    database: DatabaseStatus,
    metrics: StatusMetrics,
}

#[derive(Serialize)]
struct DatabaseStatus {
    status: &'static str,
    engine: &'static str,
}

#[derive(Serialize)]
struct StatusMetrics {
    total_users: i64,
    total_messages: i64,
    online_connections: usize,
}

/// GET /health - lightweight readiness check
pub async fn health(state: ServerState) -> Result<impl Reply, Rejection> {
    let uptime = state.start_time.elapsed().as_secs();
    let response = HealthResponse {
        status: "healthy",
        timestamp: chrono::Utc::now().timestamp_millis(),
        uptime_seconds: uptime,
    };

    info!(
        target: "server",
        event = "server.health",
        uptime_seconds = uptime,
        "Health check served"
    );

    Ok(reply::json(&response))
}

/// GET /status - richer diagnostics with database connectivity and simple metrics
pub async fn status(state: ServerState) -> Result<impl Reply, Rejection> {
    let uptime = state.start_time.elapsed().as_secs();
    let timestamp = chrono::Utc::now().timestamp_millis();

    // Basic connectivity check
    if let Err(e) = sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(&state.pool)
        .await
    {
        warn!(
            target: "server",
            event = "server.status",
            outcome = "database_unavailable",
            error = %e
        );
        return Err(rejection(ApiError::internal("Database unreachable")));
    }

    let total_users = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            warn!(target: "server", event = "server.status", error = %e, "Failed to count users");
            rejection(ApiError::internal("Failed to read user metrics"))
        })?;

    let total_messages = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM messages")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            warn!(
                target: "server",
                event = "server.status",
                error = %e,
                "Failed to count messages"
            );
            rejection(ApiError::internal("Failed to read message metrics"))
        })?;

    let online_connections = state.connection_manager.get_online_users().await.len();

    let response = StatusResponse {
        status: "running",
        version: env!("CARGO_PKG_VERSION"),
        timestamp,
        uptime_seconds: uptime,
        database: DatabaseStatus {
            status: "connected",
            engine: "sqlite",
        },
        metrics: StatusMetrics {
            total_users,
            total_messages,
            online_connections,
        },
    };

    info!(
        target: "server",
        event = "server.status",
        total_users,
        total_messages,
        online_connections,
        "Status check served"
    );

    Ok(reply::json(&response))
}
