//! Database transaction helpers

use futures::future::BoxFuture;
use sqlx::{Sqlite, SqlitePool, Transaction};

/// Execute a database transaction with automatic commit/rollback.
///
/// # Errors
///
/// Returns an error string if the transaction fails to begin, commit, or rollback,
/// or if the provided closure returns an error.
pub async fn execute_transaction<F, T>(
    pool: &SqlitePool,
    f: F,
) -> Result<T, String>
where
    F: for<'a> FnOnce(&'a mut Transaction<Sqlite>) -> BoxFuture<'a, Result<T, String>>,
{
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("Failed to begin transaction: {e}"))?;

    let result = f(&mut tx).await;

    match result {
        Ok(value) => {
            tx.commit()
                .await
                .map_err(|e| format!("Failed to commit transaction: {e}"))?;
            Ok(value)
        }
        Err(e) => {
            tx.rollback()
                .await
                .map_err(|err| {
                    format!(
                        "Failed to rollback transaction: {err}. Original error: {e}"
                    )
                })?;
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::User;
    use crate::test_utils;

    #[tokio::test]
    async fn test_execute_transaction_commit() -> Result<(), Box<dyn std::error::Error>> {
        let pool = test_utils::setup_test_db().await;

        let user = User::new("alice".to_string(), "hash123".to_string());
        let username = user.username.clone();

        let result: String = execute_transaction(&pool, |tx| {
            Box::pin(async move {
                sqlx::query(
                    "INSERT INTO users (id, username, password_hash, created_at, updated_at, is_online, deleted_at, last_seen_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(&user.id)
                .bind(&user.username)
                .bind(&user.password_hash)
                .bind(user.created_at)
                .bind(user.updated_at)
                .bind(user.is_online)
                .bind(user.deleted_at)
                .bind(user.last_seen_at)
                .execute(&mut **tx)
                .await
                .map_err(|e| format!("Insert failed: {e}"))?;

                Ok(username)
            })
        })
        .await?;

        assert_eq!(result, "alice");

        let found = super::super::users::find_user_by_username(&pool, "alice").await?;
        assert!(found.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_transaction_rollback() -> Result<(), Box<dyn std::error::Error>> {
        let pool = test_utils::setup_test_db().await;

        let user = User::new("alice".to_string(), "hash123".to_string());

        let result: Result<String, String> = execute_transaction(&pool, |tx| {
            Box::pin(async move {
                sqlx::query(
                    "INSERT INTO users (id, username, password_hash, created_at, updated_at, is_online, deleted_at, last_seen_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(&user.id)
                .bind(&user.username)
                .bind(&user.password_hash)
                .bind(user.created_at)
                .bind(user.updated_at)
                .bind(user.is_online)
                .bind(user.deleted_at)
                .bind(user.last_seen_at)
                .execute(&mut **tx)
                .await
                .map_err(|e| format!("Insert failed: {e}"))?;

                Err("Intentional rollback".to_string())
            })
        })
        .await;

        assert!(result.is_err());

        let found = super::super::users::find_user_by_username(&pool, "alice").await?;
        assert!(found.is_none());

        Ok(())
    }
}
