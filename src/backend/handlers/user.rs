//! User profile endpoints
//!
//! Handles GET /user/me and other user-related endpoints

use crate::db::queries;
use crate::handlers::auth::ErrorResponse;
use crate::services::{AuthService, UserService};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
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

/// User search result item
#[derive(Debug, Serialize)]
pub struct UserSearchResult {
    pub user_id: String,
    pub username: String,
    pub is_online: bool,
}

/// User search query parameters
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    10
}

/// Delete account request payload
#[derive(Debug, Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
}

/// Change password request payload
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Handle GET /user/me
pub async fn get_current_user(user_id: String, pool: SqlitePool) -> Result<impl Reply, Rejection> {
    // Fetch user from database
    let user = match queries::find_user_by_id(&pool, &user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("User not found for id: {}", user_id);
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

/// Handle GET /users/search?q=<query>&limit=<limit>
///
/// Searches for users by username prefix (case-insensitive)
/// Excludes current user and deleted users
/// Returns up to `limit` results (max 50, default 10)
pub async fn search_users(
    user_id: String,
    query: SearchQuery,
    user_service: Arc<UserService>,
) -> Result<impl Reply, Rejection> {
    // Validate query length (minimum 1 character)
    if query.q.is_empty() {
        return Ok(reply::with_status(
            reply::json(&ErrorResponse {
                error: "INVALID_QUERY".to_string(),
                message: "Search query must be at least 1 character".to_string(),
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Cap limit at 50
    let limit = query.limit.min(50);

    // Search users (excluding self) with cached results
    let users = match user_service.search_users(&user_id, &query.q, limit).await {
        Ok(users) => users,
        Err(e) => {
            warn!("Failed to search users: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "DATABASE_ERROR".to_string(),
                    message: "Failed to search users".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Map to search results (exclude sensitive data)
    let results: Vec<UserSearchResult> = users
        .into_iter()
        .map(|u| UserSearchResult {
            user_id: u.id,
            username: u.username,
            is_online: u.is_online,
        })
        .collect();

    Ok(reply::with_status(
        reply::json(&results),
        warp::http::StatusCode::OK,
    ))
}

/// Handle DELETE /user/me
pub async fn delete_account(
    user_id: String,
    request: DeleteAccountRequest,
    pool: SqlitePool,
) -> Result<impl Reply, Rejection> {
    // 1. Fetch user to get password hash
    let user = match queries::find_user_by_id(&pool, &user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
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
                    message: "Failed to retrieve user".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // 2. Verify password
    match AuthService::verify_password(&request.password, &user.password_hash) {
        Ok(true) => {
            // Password correct
        }
        Ok(false) => {
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "INVALID_PASSWORD".to_string(),
                    message: "Incorrect password".to_string(),
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
        Err(e) => {
            warn!("Password verification error: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "AUTH_ERROR".to_string(),
                    message: "Authentication failed".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }

    // 3. Perform soft delete
    if let Err(e) = queries::soft_delete_user(&pool, &user_id).await {
        warn!("Failed to delete user: {}", e);
        return Ok(reply::with_status(
            reply::json(&ErrorResponse {
                error: "DATABASE_ERROR".to_string(),
                message: "Failed to delete account".to_string(),
            }),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    // 4. Return success (No Content)
    Ok(reply::with_status(
        reply::json(&serde_json::json!({})), // warp reply needs body even for 204? Usually empty.
        warp::http::StatusCode::NO_CONTENT,
    ))
}

/// Handle POST /user/change-password
pub async fn change_password(
    user_id: String,
    request: ChangePasswordRequest,
    pool: SqlitePool,
) -> Result<impl Reply, Rejection> {
    // 1. Fetch user to verify current password
    let user = match queries::find_user_by_id(&pool, &user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
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
                    message: "Failed to retrieve user".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // 2. Verify current password
    match AuthService::verify_password(&request.current_password, &user.password_hash) {
        Ok(true) => {}
        Ok(false) => {
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "INVALID_PASSWORD".to_string(),
                    message: "Incorrect current password".to_string(),
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
        Err(e) => {
            warn!("Password verification error: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "AUTH_ERROR".to_string(),
                    message: "Authentication failed".to_string(),
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }

    // 3. Validate and hash new password
    // Using AuthService directly requires jwt_secret, but hash_password is static?
    // AuthService::hash_password is static method.
    let (new_hash, new_salt) = match AuthService::hash_password(&request.new_password) {
        Ok(pair) => pair,
        Err(e) => {
            return Ok(reply::with_status(
                reply::json(&ErrorResponse {
                    error: "VALIDATION_ERROR".to_string(),
                    message: e,
                }),
                warp::http::StatusCode::BAD_REQUEST,
            ));
        }
    };

    // 4. Update password in database
    if let Err(e) = queries::update_password(&pool, &user_id, &new_hash, &new_salt).await {
        warn!("Failed to update password: {}", e);
        return Ok(reply::with_status(
            reply::json(&ErrorResponse {
                error: "DATABASE_ERROR".to_string(),
                message: "Failed to update password".to_string(),
            }),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    Ok(reply::with_status(
        reply::json(&serde_json::json!({ "message": "Password changed successfully" })),
        warp::http::StatusCode::OK,
    ))
}
