//! User profile endpoints
//!
//! Handles GET /user/me and other user-related endpoints

use crate::db::queries;
use crate::handlers::ErrorBody;
use crate::services::{AuthService, CsrfService, UserService};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::warn;
use warp::{reply, Rejection, Reply};
use crate::validators;

const MAX_PASSWORD_LENGTH: usize = 128;
const MAX_SEARCH_QUERY_LENGTH: usize = 100;

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
                reply::json(&ErrorBody {
                    code: "USER_NOT_FOUND".to_string(),
                    message: "User account not found".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::NOT_FOUND,
            ));
        }
        Err(e) => {
            warn!("Database error: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "DATABASE_ERROR".to_string(),
                    message: "Failed to fetch user profile".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Check if deleted
    if user.is_deleted() {
        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "ACCOUNT_DELETED".to_string(),
                message: "Account has been deleted".to_string(),
                details: None,
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
    if query.q.is_empty() {
        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "INVALID_QUERY".to_string(),
                message: "Search query must be at least 1 character".to_string(),
                details: None,
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }
    
    if query.q.len() > MAX_SEARCH_QUERY_LENGTH {
        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "INVALID_QUERY".to_string(),
                message: "Search query exceeds maximum length".to_string(),
                details: None,
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Validate query doesn't contain suspicious patterns (XSS prevention)
    let suspicious_patterns = [
        "<script", "</script", "javascript:", "onerror", "onload", "onclick",
        "onmouseover", "onfocus", "onblur", "<img", "<svg", "<iframe",
        "data:", "vbscript:", "expression(", "<body", "<html", "<link",
        "<style", "<object", "<embed", "<form", "<input", "<meta",
        "&#x3c", "&#x3e", "&#60", "&#62", "%3c", "%3e", "&lt;", "&gt;",
    ];
    let query_lower = query.q.to_lowercase();
    for pattern in suspicious_patterns {
        if query_lower.contains(pattern) {
            warn!("Suspicious search query pattern detected: {}", pattern);
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "INVALID_QUERY".to_string(),
                    message: "Search query contains invalid characters".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::BAD_REQUEST,
            ));
        }
    }

    // Cap limit at 50
    let limit = query.limit.min(50);

    // Search users (excluding self) with cached results
    let users = match user_service.search_users(&user_id, &query.q, limit).await {
        Ok(users) => users,
        Err(e) => {
            warn!("Failed to search users: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "DATABASE_ERROR".to_string(),
                    message: "Failed to search users".to_string(),
                    details: None,
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
    csrf_token: Option<String>,
    csrf_service: CsrfService,
    pool: SqlitePool,
) -> Result<impl Reply, Rejection> {
    if request.password.is_empty() || request.password.len() > MAX_PASSWORD_LENGTH {
        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "VALIDATION_ERROR".to_string(),
                message: if request.password.is_empty() {
                    "Password is required"
                } else {
                    "Password exceeds maximum length"
                }.to_string(),
                details: None,
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    let csrf_token = match csrf_token {
        Some(token) => token,
        None => {
            warn!("Missing CSRF token for delete account request");
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "FORBIDDEN".to_string(),
                    message: "CSRF token required".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::FORBIDDEN,
            ));
        }
    };

    if let Err(e) = csrf_service.validate_token(&csrf_token, &user_id) {
        warn!("CSRF validation failed for delete account request: {:?}", e);
        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "FORBIDDEN".to_string(),
                message: "Invalid or expired CSRF token".to_string(),
                details: None,
            }),
            warp::http::StatusCode::FORBIDDEN,
        ));
    }

    let user = match queries::find_user_by_id(&pool, &user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "USER_NOT_FOUND".to_string(),
                    message: "User account not found".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::NOT_FOUND,
            ));
        }
        Err(e) => {
            warn!("Database error: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "DATABASE_ERROR".to_string(),
                    message: "Failed to retrieve user".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    match AuthService::verify_password(&request.password, &user.password_hash) {
        Ok(true) => {}
        Ok(false) => {
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "INVALID_PASSWORD".to_string(),
                    message: "Incorrect password".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
        Err(e) => {
            warn!("Password verification error: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "AUTH_ERROR".to_string(),
                    message: "Authentication failed".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }

    if let Err(e) = queries::soft_delete_user(&pool, &user_id).await {
        warn!("Failed to delete user: {}", e);
        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "DATABASE_ERROR".to_string(),
                message: "Failed to delete account".to_string(),
                details: None,
            }),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    Ok(reply::with_status(
        reply::json(&serde_json::json!({ "message": "Account deleted successfully" })),
        warp::http::StatusCode::OK,
    ))
}

/// Handle POST /user/change-password
pub async fn change_password(
    user_id: String,
    csrf_token: Option<String>,
    req: ChangePasswordRequest,
    csrf_service: CsrfService,
    pool: SqlitePool,
    auth_service: Arc<AuthService>,
) -> Result<impl Reply, Rejection> {
    let csrf_token = match csrf_token {
        Some(token) => token,
        None => {
            warn!("Missing CSRF token for change password request");
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "FORBIDDEN".to_string(),
                    message: "CSRF token required".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::FORBIDDEN,
            ));
        }
    };

    if let Err(e) = csrf_service.validate_token(&csrf_token, &user_id) {
        warn!("CSRF validation failed for change password request: {:?}", e);
        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "FORBIDDEN".to_string(),
                message: "Invalid or expired CSRF token".to_string(),
                details: None,
            }),
            warp::http::StatusCode::FORBIDDEN,
        ));
    }

    if req.current_password.is_empty() || req.current_password.len() > MAX_PASSWORD_LENGTH {
        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "VALIDATION_ERROR".to_string(),
                message: if req.current_password.is_empty() {
                    "Current password is required"
                } else {
                    "Current password exceeds maximum length"
                }.to_string(),
                details: None,
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    if let Err(e) = validators::validate_password(&req.new_password) {
        warn!("New password validation failed: {}", e);
        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "VALIDATION_ERROR".to_string(),
                message: format!("New password does not meet requirements: {e}"),
                details: None,
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    let user = match queries::find_user_by_id(&pool, &user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "USER_NOT_FOUND".to_string(),
                    message: "User account not found".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::NOT_FOUND,
            ));
        }
        Err(e) => {
            warn!("Database error: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "DATABASE_ERROR".to_string(),
                    message: "Failed to retrieve user".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    match AuthService::verify_password(&req.current_password, &user.password_hash) {
        Ok(true) => {}
        Ok(false) => {
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "INVALID_PASSWORD".to_string(),
                    message: "Incorrect current password".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
        Err(e) => {
            warn!("Password verification error: {}", e);
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "AUTH_ERROR".to_string(),
                    message: "Authentication failed".to_string(),
                    details: None,
                }),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }

    let new_hash = match AuthService::hash_password(&req.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            return Ok(reply::with_status(
                reply::json(&ErrorBody {
                    code: "VALIDATION_ERROR".to_string(),
                    message: e,
                    details: None,
                }),
                warp::http::StatusCode::BAD_REQUEST,
            ));
        }
    };

    if let Err(e) = queries::update_password(&pool, &user_id, &new_hash).await {
        warn!("Failed to update password: {}", e);
        return Ok(reply::with_status(
            reply::json(&ErrorBody {
                code: "DATABASE_ERROR".to_string(),
                message: "Failed to update password".to_string(),
                details: None,
            }),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    // Invalidate all existing tokens for this user (security best practice)
    auth_service.revoke_all_tokens_for_user(&user_id).await;

    Ok(reply::with_status(
        reply::json(&serde_json::json!({ "message": "Password changed successfully" })),
        warp::http::StatusCode::OK,
    ))
}
