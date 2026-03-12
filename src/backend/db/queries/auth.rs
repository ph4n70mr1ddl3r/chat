//! Auth log database queries

use sqlx::SqlitePool;
use uuid::Uuid;

/// Auth event types
#[derive(Debug, Clone)]
pub enum AuthEventType {
    LoginSuccess,
    LoginFailed,
    Signup,
    Logout,
}

impl AuthEventType {
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::LoginSuccess => "login_success",
            Self::LoginFailed => "login_failed",
            Self::Signup => "signup",
            Self::Logout => "logout",
        }
    }
}

/// Insert an auth log entry
pub async fn insert_auth_log(
    pool: &SqlitePool,
    ip_address: &str,
    username: Option<&str>,
    event_type: AuthEventType,
    user_agent: Option<&str>,
    details: Option<&str>,
) -> Result<(), String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query(
        "INSERT INTO auth_logs (id, ip_address, username, event_type, created_at, user_agent, details)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(ip_address)
    .bind(username)
    .bind(event_type.as_str())
    .bind(now)
    .bind(user_agent)
    .bind(details)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to insert auth log: {}", e))?;

    Ok(())
}

/// Get failed login attempts for an IP address within a time window
pub async fn get_failed_attempts(
    pool: &SqlitePool,
    ip_address: &str,
    window_seconds: i64,
) -> Result<u32, String> {
    let now = chrono::Utc::now().timestamp_millis();
    let window_start = now - (window_seconds * 1000);

    let result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM auth_logs 
         WHERE ip_address = ? AND event_type = 'login_failed' AND created_at > ?",
    )
    .bind(ip_address)
    .bind(window_start)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to get failed attempts: {}", e))?;

    Ok(result.max(0) as u32)
}
