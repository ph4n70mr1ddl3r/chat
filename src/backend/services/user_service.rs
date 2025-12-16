//! User service with cached search results
//!
//! Provides a 60s TTL in-memory cache for user search responses to reduce
//! repeated database lookups on rapid search queries.

use crate::db::queries;
use crate::models::User;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Eq, Clone)]
struct SearchKey {
    query: String,
    limit: u32,
    requester_id: String,
}

impl PartialEq for SearchKey {
    fn eq(&self, other: &Self) -> bool {
        self.query == other.query
            && self.limit == other.limit
            && self.requester_id == other.requester_id
    }
}

impl Hash for SearchKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.query.hash(state);
        self.limit.hash(state);
        self.requester_id.hash(state);
    }
}

#[derive(Clone)]
struct CacheEntry {
    expires_at: Instant,
    results: Vec<User>,
}

/// User-facing service wrapper with cached search results
#[derive(Clone)]
pub struct UserService {
    pool: SqlitePool,
    cache: Arc<RwLock<HashMap<SearchKey, CacheEntry>>>,
    ttl: Duration,
}

impl UserService {
    /// Create a new service with 60s TTL cache
    pub fn new(pool: SqlitePool) -> Self {
        Self::new_with_ttl(pool, Duration::from_secs(60))
    }

    /// Create a new service with custom TTL (useful for tests)
    pub fn new_with_ttl(pool: SqlitePool, ttl: Duration) -> Self {
        Self {
            pool,
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    /// Search users with cached results (per requester + query + limit)
    pub async fn search_users(
        &self,
        requester_id: &str,
        query: &str,
        limit: u32,
    ) -> Result<Vec<User>, String> {
        let normalized_query = query.trim().to_lowercase();
        let key = SearchKey {
            query: normalized_query.clone(),
            limit,
            requester_id: requester_id.to_string(),
        };

        // Fast path: return cached results if fresh
        let now = Instant::now();
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(&key) {
                if entry.expires_at > now {
                    return Ok(entry.results.clone());
                }
            }
        }

        // Cache miss or expired — perform query
        let users =
            queries::search_users_excluding_self(&self.pool, query, requester_id, limit).await?;

        // Insert into cache and prune expired entries opportunistically
        {
            let mut cache = self.cache.write().await;
            cache.retain(|_, entry| entry.expires_at > now);
            cache.insert(
                key,
                CacheEntry {
                    expires_at: now + self.ttl,
                    results: users.clone(),
                },
            );
        }

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::User;

    async fn setup_test_db() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let schema_sql = include_str!("../../backend/db/migrations/001_initial_schema.sql");
        for statement in schema_sql.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(statement).execute(&pool).await.unwrap();
        }

        pool
    }

    #[tokio::test]
    async fn search_results_are_cached_until_ttl_expires() {
        let pool = setup_test_db().await;
        let service = UserService::new_with_ttl(pool.clone(), Duration::from_millis(50));

        // Seed users
        let requester = User::new("alice".into(), "hash".into(), "salt".into());
        let initial_result = User::new("bob".into(), "hash2".into(), "salt2".into());
        queries::insert_user(&pool, &requester).await.unwrap();
        queries::insert_user(&pool, &initial_result).await.unwrap();

        // First search -> cache miss, returns bob
        let first = service
            .search_users(&requester.id, "b", 10)
            .await
            .unwrap();
        assert_eq!(first.len(), 1);

        // Add another user matching query
        let new_user = User::new("ben".into(), "hash3".into(), "salt3".into());
        queries::insert_user(&pool, &new_user).await.unwrap();

        // Second search before TTL expiry should still return cached single result
        let cached = service
            .search_users(&requester.id, "b", 10)
            .await
            .unwrap();
        assert_eq!(cached.len(), 1);

        // Wait for TTL to expire and confirm cache refresh includes new user
        tokio::time::sleep(Duration::from_millis(60)).await;
        let refreshed = service
            .search_users(&requester.id, "b", 10)
            .await
            .unwrap();
        assert_eq!(refreshed.len(), 2);
    }
}
