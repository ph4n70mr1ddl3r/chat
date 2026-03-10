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
    #[serde(skip_serializing_if = "Option::is_none")]
    warning: Option<&'static str>,
}

#[derive(Serialize)]
struct StatusResponse {
    status: &'static str,
    version: &'static str,
    timestamp: i64,
    uptime_seconds: u64,
    database: DatabaseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    warning: Option<&'static str>,
}

#[derive(Serialize)]
struct DatabaseStatus {
    status: &'static str,
    engine: &'static str,
}

/// GET /health - lightweight readiness check
pub async fn health(state: ServerState) -> Result<impl Reply, Rejection> {
    let uptime = state.start_time.elapsed().as_secs();
    let (status, warning) = if state.config.is_ephemeral_secret {
        ("degraded", Some("Running with ephemeral JWT secret - not suitable for production"))
    } else {
        ("healthy", None)
    };
    
    let response = HealthResponse {
        status,
        timestamp: chrono::Utc::now().timestamp_millis(),
        uptime_seconds: uptime,
        warning,
    };

    info!(
        target: "server",
        event = "server.health",
        uptime_seconds = uptime,
        is_ephemeral_secret = state.config.is_ephemeral_secret,
        "Health check served"
    );

    Ok(reply::json(&response))
}

/// GET /status - basic server diagnostics with database connectivity
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
    
    let warning = if state.config.is_ephemeral_secret {
        Some("Running with ephemeral JWT secret - not suitable for production")
    } else {
        None
    };

    let response = StatusResponse {
        status: "running",
        version: env!("CARGO_PKG_VERSION"),
        timestamp,
        uptime_seconds: uptime,
        database: DatabaseStatus {
            status: "connected",
            engine: "sqlite",
        },
        warning,
    };

    info!(
        target: "server",
        event = "server.status",
        uptime_seconds = uptime,
        is_ephemeral_secret = state.config.is_ephemeral_secret,
        "Status check served"
    );

    Ok(reply::json(&response))
}
