//! User profile endpoints
//!
//! Handles GET /user/me and other user-related endpoints

use crate::db::queries;
use crate::handlers::auth::ErrorResponse;
use crate::services::AuthService;
use serde::Serialize;
use sqlx::SqlitePool;
use tracing::warn;
use warp::{reply, Rejection, Reply};

/// User profile response
#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub user_id: String,
    pub username: String,
    pub created_at: i64,
    pub is_online: bool,
    pub last_seen_at: Option<i64>,
}

/// Handle GET /user/me
pub async fn get_current_user(
    token: String,
    pool: SqlitePool,
    jwt_secret: String,
) -> Result<impl Reply, Rejection> {
    let auth_service = AuthService::new(jwt_secret);

    // Verify token
    let claims = match auth_service.verify_token(&token) {
        Ok(claims) => claims,
        Err(e) => {
            warn!("Token verification failed: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "INVALID_TOKEN".to_string(),
                    message: "Token is invalid or expired".to_string(),
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    // Fetch user from database
    let user = match queries::find_user_by_id(&pool, &claims.sub).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("User not found for token: {}", claims.sub);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "USER_NOT_FOUND".to_string(),
                    message: "User account not found".to_string(),
                }),
                warp::http::StatusCode::NOT_FOUND,
            ));
        }
        Err(e) => {
            warn!("Database error: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "DATABASE_ERROR".to_string(),
                    message: "Failed to fetch user profile".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Check if deleted
    if user.is_deleted() {
        return Ok(reply::with_status(
            reply::json(&ErrorResponse {
                error: "ACCOUNT_DELETED".to_string(),
                message: "Account has been deleted".to_string(),
            }),
            warp::http::StatusCode::NOT_FOUND,
        ));
    }

    Ok(reply::with_status(
        reply::json(&UserProfileResponse {
            user_id: user.id,
            username: user.username,
            created_at: user.created_at,
            is_online: user.is_online,
            last_seen_at: user.last_seen_at,
        }),
        warp::http::StatusCode::OK,
    ))
}
