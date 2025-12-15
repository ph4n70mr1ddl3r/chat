# Real-Time Chat Application Backend in Rust: Comprehensive Architecture Guide

**Last Updated**: December 15, 2025  
**Target Rust Version**: 1.75+

---

## Executive Summary

This guide provides a production-ready architecture for building scalable, reliable real-time chat applications in Rust. Each section covers decision-making frameworks, implementation patterns, and Rust-specific best practices based on current ecosystem standards.

---

## 1. WebSocket Server Architecture

### Decision: Tokio + Tokio-Tungstenite with Actor Pattern

**Recommended Stack**:
- **Runtime**: Tokio (multi-threaded, work-stealing runtime)
- **WebSocket Library**: Tokio-Tungstenite (tokio-tungstenite 0.28+)
- **Architecture Pattern**: Actor pattern using tokio channels for internal communication
- **Connection Management**: Dedicated task per client with shared state via Arc<DashMap>

### Rationale

1. **Tokio is Industry Standard**: Over 70% of async Rust projects use Tokio. It provides:
   - Multi-threaded work-stealing runtime for CPU-bound and I/O-bound workloads
   - Composable async primitives (channels, locks, timers)
   - Built-in fairness and no surprising latency spikes
   - Extensive ecosystem integration

2. **Tokio-Tungstenite**: Lightweight WebSocket implementation with clean Sink/Stream traits
   - `WebSocketStream` implements both `Sink` and `Stream` for bidirectional communication
   - No custom framing needed; protocol handling delegated to tungstenite
   - Support for both plain and TLS connections

3. **Actor Pattern**: Provides clean isolation and clear message flow
   - Each client is an independent actor (task)
   - Central message broker handles client registry and broadcast
   - Natural backpressure handling via channel capacities

### Implementation Considerations

#### Key Libraries
```toml
[dependencies]
tokio = { version = "1.48", features = ["full"] }
tokio-tungstenite = "0.28"
futures = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
```

#### Core Architecture Pattern

```rust
// 1. Connection handler spawned per client
async fn handle_client(
    socket: WebSocketStream<TcpStream>,
    client_id: ClientId,
    message_tx: mpsc::Sender<Message>,
) {
    // Split socket into sink and stream
    let (mut write, mut read) = socket.split();
    
    // Listen for client messages
    while let Some(msg) = read.next().await {
        // Process and forward to broadcast channel
    }
    // Cleanup on disconnect
}

// 2. Central message broker
async fn message_broker(
    mut rx: mpsc::Receiver<Message>,
    clients: Arc<DashMap<ClientId, mpsc::Sender<Message>>>,
) {
    while let Some(msg) = rx.recv().await {
        // Route message to subscribers
        // Handle offline queuing
    }
}

// 3. Accept new connections
let listener = TcpListener::bind("0.0.0.0:8080").await?;
loop {
    let (stream, _) = listener.accept().await?;
    let client_tx = message_tx.clone();
    let clients = clients.clone();
    
    tokio::spawn(async move {
        match tokio_tungstenite::accept_async(stream).await {
            Ok(socket) => handle_client(socket, client_id, client_tx).await,
            Err(e) => eprintln!("WebSocket error: {}", e),
        }
    });
}
```

#### Connection Limits and Backpressure

```rust
// Configure per-client message buffer capacity
const CLIENT_BUFFER_CAPACITY: usize = 128;

// Handle backpressure - don't accept messages faster than client can consume
match message_tx.try_send(msg) {
    Ok(_) => {},
    Err(mpsc::error::TrySendError::Full(_)) => {
        // Client too slow, apply rate limiting or disconnect
        eprintln!("Client {} buffer full", client_id);
    }
    Err(mpsc::error::TrySendError::Closed(_)) => {
        // Client disconnected, remove from registry
    }
}
```

#### Graceful Shutdown

```rust
// Use a broadcast channel for shutdown signal
let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);

// In main loop
tokio::select! {
    _ = shutdown_rx.recv() => {
        eprintln!("Shutdown signal received");
        break;
    }
    result = listener.accept() => {
        // Handle new connection
    }
}
```

### Rust-Specific Best Practices

1. **Use `split()` on WebSocket**: Separate read/write allows concurrent operations
2. **Avoid `Mutex` for client registry**: Use `DashMap` (concurrent hash map) instead
3. **Leverage `tokio::select!`**: For handling multiple async operations (accepts, messages, shutdown)
4. **Pin futures carefully**: Use `tokio::pin!` when storing futures in select blocks
5. **Never block the runtime**: Use `spawn_blocking()` for any CPU-intensive work

### Potential Gotchas

- **WebSocket frame fragmentation**: Handle `Message::Fragment` variants
- **Connection limits**: Configure OS-level limits (`ulimit -n`); Tokio doesn't batch-accept
- **Memory leaks**: Ensure clients are removed from registry on disconnect
- **Panic handling**: Use `.catch_unwind()` or error propagation to prevent task panics

---

## 2. Message Delivery & Offline Handling

### Decision: Hybrid In-Memory + Persistent Database Queueing

**Recommended Approach**:
- **In-Memory**: Fast, bounded queue (5-30 messages per user)
- **Persistent Storage**: PostgreSQL with SQLx for historical queuing and recovery
- **Message Queue Alternative**: RabbitMQ for distributed deployments (optional)
- **Retry Strategy**: Exponential backoff with jitter (1s, 2s, 4s, 8s, 16s, 32s, 64s)

### Rationale

1. **Hybrid Approach Balances Concerns**:
   - In-memory is fast for active clients
   - Database persists across restarts
   - Prevents unbounded memory growth
   - Survives service failures

2. **Why NOT pure in-memory?**
   - Service restart loses all queued messages
   - Unbounded growth on long-term disconnects

3. **Why NOT pure database polling?**
   - Latency for connected clients (query every 30-60s)
   - Database overhead for every message

4. **Why RabbitMQ is optional**:
   - Adds operational complexity
   - Good for multi-instance deployments with high throughput
   - Overkill for small-medium applications
   - PostgreSQL can handle reasonable message volumes

### Implementation Pattern

#### Database Schema

```sql
-- Message queue table
CREATE TABLE message_queue (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL,
    sender_id UUID NOT NULL,
    conversation_id UUID NOT NULL,
    message_body TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, delivered, failed
    attempt_count INT DEFAULT 0,
    next_retry_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    delivered_at TIMESTAMP WITH TIME ZONE,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_user_status (user_id, status),
    INDEX idx_retry_time (next_retry_at)
);

-- Track message delivery per recipient
CREATE TABLE message_delivery (
    id BIGSERIAL PRIMARY KEY,
    message_id BIGINT NOT NULL,
    recipient_id UUID NOT NULL,
    delivered_at TIMESTAMP WITH TIME ZONE,
    
    UNIQUE(message_id, recipient_id),
    FOREIGN KEY (message_id) REFERENCES message_queue(id) ON DELETE CASCADE
);
```

#### In-Memory Queue Structure

```rust
use std::collections::VecDeque;
use dashmap::DashMap;

pub struct MessageQueue {
    // In-memory: user_id -> bounded queue of messages
    in_memory: DashMap<UserId, VecDeque<QueuedMessage>>,
    // Database connection pool
    db_pool: PgPool,
    // Configuration
    in_memory_limit: usize, // 30 messages per user
    max_retry_attempts: u32,
}

#[derive(Clone)]
pub struct QueuedMessage {
    pub id: MessageId,
    pub sender_id: UserId,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub attempt_count: u32,
}

impl MessageQueue {
    pub async fn enqueue(
        &self,
        user_id: UserId,
        message: QueuedMessage,
    ) -> Result<()> {
        // 1. Try in-memory queue first
        if let Some(mut queue) = self.in_memory.get_mut(&user_id) {
            if queue.len() < self.in_memory_limit {
                queue.push_back(message.clone());
                return Ok(());
            }
        } else {
            let mut queue = VecDeque::new();
            queue.push_back(message.clone());
            self.in_memory.insert(user_id, queue);
            return Ok(());
        }
        
        // 2. Fall back to database
        sqlx::query!(
            "INSERT INTO message_queue (user_id, sender_id, message_body, status)
             VALUES ($1, $2, $3, 'pending')",
            user_id,
            message.sender_id,
            message.body
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_pending_messages(&self, user_id: UserId) -> Result<Vec<QueuedMessage>> {
        let mut messages = Vec::new();
        
        // Get from in-memory queue
        if let Some(queue) = self.in_memory.get(&user_id) {
            messages.extend(queue.iter().cloned());
        }
        
        // Get from database (recent failures or after restart)
        let db_messages = sqlx::query_as::<_, QueuedMessage>(
            "SELECT * FROM message_queue 
             WHERE user_id = $1 AND status = 'pending'
             ORDER BY created_at ASC LIMIT 100"
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await?;
        
        messages.extend(db_messages);
        Ok(messages)
    }
    
    pub async fn mark_delivered(&self, message_id: MessageId, user_id: UserId) -> Result<()> {
        // Remove from in-memory
        if let Some(mut queue) = self.in_memory.get_mut(&user_id) {
            queue.retain(|m| m.id != message_id);
        }
        
        // Mark in database
        sqlx::query!(
            "UPDATE message_queue SET status = 'delivered', delivered_at = NOW()
             WHERE id = $1",
            message_id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
}
```

#### Exponential Backoff Retry Logic

```rust
use std::time::Duration;

pub struct RetryScheduler {
    base_delay: Duration,
    max_delay: Duration,
    max_attempts: u32,
}

impl RetryScheduler {
    pub fn calculate_next_retry(&self, attempt_count: u32) -> Duration {
        // Exponential backoff: 1s * 2^attempt, capped at max_delay
        let delay_secs = self.base_delay.as_secs() as u32 * 2_u32.pow(attempt_count);
        let delay = Duration::from_secs(delay_secs as u64);
        
        // Cap at max_delay
        if delay > self.max_delay {
            self.max_delay
        } else {
            delay
        }
    }
    
    pub fn should_retry(&self, attempt_count: u32) -> bool {
        attempt_count < self.max_attempts
    }
}

// Usage: spawn retry task
async fn retry_failed_messages(queue: Arc<MessageQueue>, scheduler: Arc<RetryScheduler>) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    
    loop {
        interval.tick().await;
        
        // Find messages due for retry
        let failed_messages = sqlx::query_as::<_, QueuedMessage>(
            "SELECT * FROM message_queue 
             WHERE status = 'failed' 
             AND next_retry_at <= NOW()
             AND attempt_count < $1
             LIMIT 100",
        )
        .bind(scheduler.max_attempts)
        .fetch_all(&queue.db_pool)
        .await
        .unwrap_or_default();
        
        for msg in failed_messages {
            // Attempt redelivery
            let should_retry = scheduler.should_retry(msg.attempt_count + 1);
            
            if should_retry {
                let next_retry = scheduler.calculate_next_retry(msg.attempt_count + 1);
                sqlx::query!(
                    "UPDATE message_queue 
                     SET attempt_count = attempt_count + 1, 
                         next_retry_at = NOW() + $1::interval,
                         status = 'pending'
                     WHERE id = $2",
                    format!("{} seconds", next_retry.as_secs()),
                    msg.id
                )
                .execute(&queue.db_pool)
                .await
                .ok();
            } else {
                // Max retries exceeded, mark as failed
                sqlx::query!(
                    "UPDATE message_queue SET status = 'failed' WHERE id = $1",
                    msg.id
                )
                .execute(&queue.db_pool)
                .await
                .ok();
            }
        }
    }
}
```

### Rust-Specific Best Practices

1. **Use `Arc<DashMap>` for thread-safe registry**: Better than `Mutex<HashMap>` for high concurrency
2. **Bounded channels prevent runaway memory**: Set explicit capacities on `mpsc::channel()`
3. **Use `tokio::time::interval()` for retry loops**: Not `sleep()` in a loop
4. **Separate read/write concerns**: Use different connection pools if needed
5. **Leverage `sqlx::query_as!` macro**: Compile-time query validation

### Potential Gotchas

- **In-memory queue persistence on crash**: Add periodic flush to database
- **Database connection pool exhaustion**: Monitor pool size, set appropriate limits
- **Stale messages in database**: Implement cleanup job for messages older than 30 days
- **Delivery acknowledgment race condition**: Ensure idempotent mark_delivered operations

### RabbitMQ Alternative (for distributed systems)

If scaling across multiple server instances:

```rust
// Use lapin for RabbitMQ client
use lapin::{Connection, ConnectionProperties};

async fn setup_rabbit_queue() -> Result<Channel> {
    let conn = Connection::connect(
        "amqp://guest:guest@localhost:5672/%2F",
        ConnectionProperties::default(),
    ).await?;
    
    let channel = conn.create_channel().await?;
    
    // Declare durable queue with dead-letter exchange for failed messages
    channel.queue_declare(
        "chat_messages",
        QueueDeclareOptions {
            durable: true,
            ..Default::default()
        },
        FieldTable::default(),
    ).await?;
    
    Ok(channel)
}
```

---

## 3. Authentication & Session Management

### Decision: JWT with Refresh Token Rotation + Redis Session Store

**Recommended Stack**:
- **Token Type**: JWT (RS256 or ES256, NOT HS256 for public/private key separation)
- **Token Lifetime**: 15-30 minutes (access token), 7 days (refresh token)
- **Session Store**: Redis for blacklist and active sessions
- **Token Validation**: Cache verification keys with TTL

### Rationale

1. **JWT advantages**:
   - Stateless verification (no database lookup on every request)
   - Self-contained claims (user_id, permissions)
   - Scalable across multiple servers

2. **Refresh token rotation**:
   - Limits damage from stolen access tokens
   - Forces periodic re-authentication
   - Detects compromised tokens faster

3. **Redis session store**:
   - Fast revocation checking
   - Track active sessions per user
   - Enforce single-device sessions if needed

### Implementation Pattern

#### JWT Claims Structure

```rust
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    // Standard claims
    pub sub: String,        // subject (user_id)
    pub iat: i64,          // issued at
    pub exp: i64,          // expiration
    pub iss: String,       // issuer
    
    // Custom claims
    pub user_id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub session_id: uuid::Uuid, // rotation ID for refresh
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub session_id: uuid::Uuid,
    pub refresh_count: u32, // Detect abuse
}
```

#### Token Service with Rotation

```rust
use std::sync::Arc;

pub struct TokenService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    redis_client: redis::Client,
    access_token_ttl: Duration,
    refresh_token_ttl: Duration,
}

impl TokenService {
    pub fn new(private_key: &[u8], public_key: &[u8]) -> Result<Self> {
        Ok(TokenService {
            // For RS256: load RSA private key
            encoding_key: EncodingKey::from_rsa_pem(private_key)?,
            decoding_key: DecodingKey::from_rsa_pem(public_key)?,
            redis_client: redis::Client::open("redis://localhost")?,
            access_token_ttl: Duration::from_secs(15 * 60),
            refresh_token_ttl: Duration::from_secs(7 * 24 * 60 * 60),
        })
    }
    
    pub async fn issue_tokens(&self, user: &User) -> Result<TokenPair> {
        let session_id = uuid::Uuid::new_v4();
        let now = chrono::Utc::now();
        
        // Access token
        let access_claims = JwtClaims {
            sub: user.id.to_string(),
            iat: now.timestamp(),
            exp: (now + self.access_token_ttl).timestamp(),
            iss: "chat-app".to_string(),
            user_id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            session_id,
        };
        
        let access_token = encode(
            &Header::new(Algorithm::RS256),
            &access_claims,
            &self.encoding_key,
        )?;
        
        // Refresh token
        let refresh_claims = RefreshTokenClaims {
            sub: user.id.to_string(),
            iat: now.timestamp(),
            exp: (now + self.refresh_token_ttl).timestamp(),
            session_id,
            refresh_count: 0,
        };
        
        let refresh_token = encode(
            &Header::new(Algorithm::RS256),
            &refresh_claims,
            &self.encoding_key,
        )?;
        
        // Store session in Redis
        let mut conn = self.redis_client.get_async_connection().await?;
        let session_key = format!("session:{}:{}", user.id, session_id);
        redis::cmd("SET")
            .arg(&session_key)
            .arg("active")
            .arg("EX")
            .arg(self.refresh_token_ttl.as_secs())
            .query_async(&mut conn)
            .await?;
        
        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: self.access_token_ttl.as_secs(),
        })
    }
    
    pub async fn refresh_access_token(
        &self,
        refresh_token: &str,
        db: &PgPool,
    ) -> Result<String> {
        // Validate refresh token
        let token_data = decode::<RefreshTokenClaims>(
            refresh_token,
            &self.decoding_key,
            &Validation::new(Algorithm::RS256),
        )?;
        
        let claims = token_data.claims;
        let user_id = uuid::Uuid::parse_str(&claims.sub)?;
        
        // Check session exists in Redis
        let mut conn = self.redis_client.get_async_connection().await?;
        let session_key = format!("session:{}:{}", user_id, claims.session_id);
        let session_exists: bool = redis::cmd("EXISTS")
            .arg(&session_key)
            .query_async(&mut conn)
            .await?;
        
        if !session_exists {
            return Err(anyhow::anyhow!("Session revoked"));
        }
        
        // Get user and issue new access token
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(db)
        .await?
        .ok_or(anyhow::anyhow!("User not found"))?;
        
        let tokens = self.issue_tokens(&user).await?;
        Ok(tokens.access_token)
    }
    
    pub async fn revoke_session(
        &self,
        user_id: uuid::Uuid,
        session_id: uuid::Uuid,
    ) -> Result<()> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let session_key = format!("session:{}:{}", user_id, session_id);
        redis::cmd("DEL")
            .arg(&session_key)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
}
```

#### WebSocket Connection Authentication

```rust
async fn handle_websocket_connection(
    socket: WebSocketStream<TcpStream>,
    token_service: Arc<TokenService>,
) -> Result<()> {
    let (mut write, mut read) = socket.split();
    
    // Wait for authentication message
    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(auth_msg)) = msg {
            if let Ok(auth_payload) = serde_json::from_str::<AuthMessage>(&auth_msg) {
                // Verify token
                let token_data = decode::<JwtClaims>(
                    &auth_payload.access_token,
                    &token_service.decoding_key,
                    &Validation::new(Algorithm::RS256),
                ).ok();
                
                if let Some(token_data) = token_data {
                    let claims = token_data.claims;
                    // Successfully authenticated, proceed with chat protocol
                    return handle_authenticated_client(
                        write,
                        read,
                        claims.user_id,
                    ).await;
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("Authentication failed"))
}

#[derive(Serialize, Deserialize)]
pub struct AuthMessage {
    pub access_token: String,
}
```

#### Refresh Token Rotation Safety

```rust
// Detect token abuse: track refresh count
pub async fn check_token_abuse(
    claims: &RefreshTokenClaims,
    redis: &redis::Client,
) -> Result<bool> {
    let mut conn = redis.get_async_connection().await?;
    let abuse_key = format!("token_abuse:{}:{}", claims.sub, claims.session_id);
    
    // Get current refresh count from Redis
    let current_count: u32 = redis::cmd("INCR")
        .arg(&abuse_key)
        .query_async(&mut conn)
        .await
        .unwrap_or(1);
    
    // Set expiry if first increment
    if current_count == 1 {
        redis::cmd("EXPIRE")
            .arg(&abuse_key)
            .arg(3600) // 1 hour window
            .query_async(&mut conn)
            .await
            .ok();
    }
    
    // More than 10 refreshes in 1 hour = suspicious
    Ok(current_count > 10)
}
```

### Rust-Specific Best Practices

1. **Use `jsonwebtoken::encode/decode`**: Built-in validation and strong types
2. **Separate encoding/decoding keys**: Use RS256 (asymmetric) not HS256
3. **Store secrets in environment variables**: Use `dotenv` or `config` crate
4. **Use `Arc<TokenService>` to share**: Avoid recreating keys per request
5. **Implement `Debug` carefully**: Don't expose keys in debug output

```rust
impl std::fmt::Debug for TokenService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenService")
            .field("redis_client", &"<redis>")
            // Don't print keys!
            .finish()
    }
}
```

### Potential Gotchas

- **Clock skew**: Account for server time differences (use 30s leeway in `Validation`)
- **Key rotation**: Plan for algorithm updates and key rotation procedures
- **Token size**: JWT in WebSocket URL is limited; use header or message body instead
- **HTTPS requirement**: Always use TLS for token transmission
- **Redis dependency**: Single point of failure for session revocation; consider replication

---

## 4. Database Integration in Rust

### Decision: SQLx with Connection Pooling and Migrations

**Recommended Stack**:
- **Library**: SQLx 0.8+ with `runtime-tokio` feature
- **Database**: PostgreSQL (mature async driver, best SQLx support)
- **Connection Pool**: SQLx::Pool with default settings or tuned for workload
- **Migrations**: SQLx CLI for versioned SQL migrations
- **Query Validation**: Compile-time checked queries with `sqlx::query!` macro

### Rationale

1. **SQLx advantages**:
   - Compile-time query validation (no runtime surprises)
   - Pure Rust, no system dependencies
   - Ergonomic async/await API
   - Connection pooling built-in

2. **PostgreSQL choice**:
   - Superior async driver support
   - JSONB for chat metadata
   - Arrays for user tags/groups
   - Excellent for strong consistency requirements

### Implementation Pattern

#### Database Pool Initialization

```rust
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        // Maximum connections in pool
        .max_connections(32)
        // Minimum connections to keep open
        .min_connections(5)
        // Max lifetime of a connection
        .max_lifetime(Duration::from_secs(30 * 60))
        // Idle timeout before connection is closed
        .idle_timeout(Duration::from_secs(10 * 60))
        // Connection test interval (health check)
        .test_on_checkout(true)
        .connect(database_url)
        .await?;
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    
    Ok(pool)
}
```

#### Structured Migrations

```sql
-- migrations/20250115_001_init_schema.sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    type VARCHAR(20) NOT NULL, -- 'direct', 'group'
    name VARCHAR(255),
    description TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE conversation_members (
    id BIGSERIAL PRIMARY KEY,
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    UNIQUE(conversation_id, user_id),
    INDEX idx_user_conversations (user_id, conversation_id)
);

CREATE TABLE messages (
    id BIGSERIAL PRIMARY KEY,
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    sender_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    body TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    is_edited BOOLEAN DEFAULT FALSE,
    edited_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    INDEX idx_conversation_created (conversation_id, created_at DESC)
);

-- migrations/20250115_002_message_queue.sql
CREATE TABLE message_queue (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    sender_id UUID NOT NULL REFERENCES users(id),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    message_body TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    attempt_count INT DEFAULT 0,
    next_retry_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    delivered_at TIMESTAMP WITH TIME ZONE,
    
    INDEX idx_user_status (user_id, status),
    INDEX idx_retry_time (next_retry_at)
);
```

#### Type-Safe Query Patterns

```rust
use sqlx::{PgPool, Row, FromRow};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Message {
    pub id: i64,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub body: String,
    pub metadata: sqlx::types::JsonValue,
    pub is_edited: bool,
    pub created_at: DateTime<Utc>,
}

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub async fn find_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        // Compile-time validated query
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, created_at, updated_at 
             FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, created_at, updated_at 
             FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    pub async fn create(&self, username: &str, email: &str, password_hash: &str) 
        -> Result<User> 
    {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email, password_hash)
             VALUES ($1, $2, $3)
             RETURNING id, username, email, password_hash, created_at, updated_at"
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }
}

pub struct MessageRepository {
    pool: PgPool,
}

impl MessageRepository {
    pub async fn save(&self, msg: &ChatMessage) -> Result<i64> {
        let row = sqlx::query(
            "INSERT INTO messages (conversation_id, sender_id, body, metadata)
             VALUES ($1, $2, $3, $4)
             RETURNING id"
        )
        .bind(msg.conversation_id)
        .bind(msg.sender_id)
        .bind(&msg.body)
        .bind(serde_json::to_value(&msg.metadata)?)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(row.get::<i64, _>(0))
    }
    
    pub async fn get_conversation_history(
        &self,
        conversation_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>> {
        let messages = sqlx::query_as::<_, Message>(
            "SELECT id, conversation_id, sender_id, body, metadata, is_edited, created_at
             FROM messages
             WHERE conversation_id = $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3"
        )
        .bind(conversation_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(messages)
    }
}
```

#### Transaction Management

```rust
pub async fn add_user_to_conversation(
    pool: &PgPool,
    user_id: Uuid,
    conversation_id: Uuid,
) -> Result<()> {
    let mut tx = pool.begin().await?;
    
    // Verify conversation exists
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM conversations WHERE id = $1)"
    )
    .bind(conversation_id)
    .fetch_one(&mut *tx)
    .await?;
    
    if !exists {
        return Err(anyhow::anyhow!("Conversation not found"));
    }
    
    // Add member
    sqlx::query(
        "INSERT INTO conversation_members (conversation_id, user_id)
         VALUES ($1, $2)
         ON CONFLICT DO NOTHING"
    )
    .bind(conversation_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    
    // Update conversation timestamp
    sqlx::query("UPDATE conversations SET updated_at = NOW() WHERE id = $1")
        .bind(conversation_id)
        .execute(&mut *tx)
        .await?;
    
    tx.commit().await?;
    Ok(())
}
```

#### Connection Pool Tuning

```rust
// For different workload types:

// High-concurrency (many connections, short queries):
PgPoolOptions::new()
    .max_connections(50)
    .min_connections(10)
    .acquire_timeout(Duration::from_secs(5))
    .idle_timeout(Duration::from_secs(5 * 60))
    .test_on_checkout(true)

// Low-concurrency (few connections, long operations):
PgPoolOptions::new()
    .max_connections(10)
    .min_connections(2)
    .acquire_timeout(Duration::from_secs(10))
    .max_lifetime(Duration::from_secs(60 * 60))
    .idle_timeout(Duration::from_secs(30 * 60))
```

### Rust-Specific Best Practices

1. **Use `FromRow` derive for automatic mapping**:
   ```rust
   #[derive(FromRow)]
   pub struct User { /* fields */ }
   ```

2. **Leverage `sqlx::query_as!` macro for compile-time checking**:
   ```rust
   sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
   ```

3. **Use `Optional` for nullable results**:
   ```rust
   .fetch_optional(&pool).await? // Returns Option<T>
   ```

4. **Transactions ensure consistency**:
   ```rust
   let mut tx = pool.begin().await?;
   // ... multiple operations
   tx.commit().await?;
   ```

5. **Connection pooling prevents resource exhaustion**: Never create individual connections in handlers

### Potential Gotchas

- **Compile-time macro overhead**: `query!` macros require database at compile time (use `SQLX_OFFLINE` for CI)
- **Connection leak**: Always use `?` or error handling, don't drop connections in weird places
- **Pool exhaustion**: Monitor pool usage, increase max_connections if needed
- **Statement caching**: SQLx caches statements automatically; monitor memory on long-running processes
- **Transaction deadlocks**: Keep transactions short, order writes consistently

---

## 5. Error Handling & Idempotency

### Decision: Custom Error Types with `thiserror` + Idempotency Keys

**Recommended Approach**:
- **Error Framework**: `thiserror` for custom error types + `anyhow` for context
- **Idempotency**: Message ID + delivery tracking prevents duplicate processing
- **Retry Strategy**: Idempotency keys stored with 24-hour TTL
- **Error Propagation**: Result types with contextual information

### Rationale

1. **Custom errors over `anyhow` alone**:
   - Type-safe error handling
   - Different handlers for different error cases
   - Better error recovery logic

2. **Idempotency keys**:
   - Client provides UUID with each message
   - Server stores {key: message_id} mapping
   - Resubmit with same key returns cached result
   - Prevents double-messages on reconnect

### Implementation Pattern

#### Error Type Definition

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChatError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("User not found: {0}")]
    UserNotFound(uuid::Uuid),
    
    #[error("Conversation not found: {0}")]
    ConversationNotFound(uuid::Uuid),
    
    #[error("Not authorized to perform this action")]
    Unauthorized,
    
    #[error("Message delivery failed after {0} attempts")]
    DeliveryFailed(u32),
    
    #[error("Rate limited: {retry_after_secs} seconds until retry")]
    RateLimited { retry_after_secs: u64 },
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Internal server error")]
    Internal,
}

impl ChatError {
    pub fn to_websocket_message(&self) -> serde_json::Value {
        match self {
            ChatError::AuthenticationFailed(msg) => serde_json::json!({
                "type": "error",
                "code": "AUTH_FAILED",
                "message": msg
            }),
            ChatError::RateLimited { retry_after_secs } => serde_json::json!({
                "type": "error",
                "code": "RATE_LIMITED",
                "retry_after_secs": retry_after_secs
            }),
            _ => serde_json::json!({
                "type": "error",
                "code": "INTERNAL_ERROR",
                "message": "An error occurred"
            }),
        }
    }
}

pub type Result<T> = std::result::Result<T, ChatError>;
```

#### Idempotency Key Pattern

```rust
use uuid::Uuid;
use redis::AsyncCommands;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,                    // Generated server-side
    pub idempotency_key: Uuid,       // Provided by client
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub body: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct IdempotencyStore {
    redis: redis::Client,
    ttl: usize, // seconds
}

impl IdempotencyStore {
    pub async fn get_or_create(
        &self,
        idempotency_key: Uuid,
        creator: impl Fn() -> futures::future::BoxFuture<'static, Result<Uuid>>,
    ) -> Result<Uuid> {
        let mut conn = self.redis
            .get_async_connection()
            .await
            .map_err(|_| ChatError::Internal)?;
        
        let cache_key = format!("idempotency:{}", idempotency_key);
        
        // Check if already processed
        if let Ok(Some(message_id)) = conn.get::<_, String>(&cache_key).await {
            return Ok(Uuid::parse_str(&message_id).unwrap_or_default());
        }
        
        // Generate new message ID
        let message_id = creator().await?;
        
        // Store for TTL
        let result: Result<(), _> = conn.set_ex(
            &cache_key,
            message_id.to_string(),
            self.ttl,
        ).await.map_err(|_| ChatError::Internal);
        
        result?;
        Ok(message_id)
    }
}

// Usage:
async fn handle_send_message(
    msg: IncomingMessage,
    sender_id: Uuid,
    idempotency_store: &IdempotencyStore,
    db: &PgPool,
) -> Result<ChatMessage> {
    // Get or create message with idempotency
    let message_id = idempotency_store
        .get_or_create(msg.idempotency_key, || {
            Box::pin(async {
                Ok(Uuid::new_v4())
            })
        })
        .await?;
    
    // Insert message (idempotent due to unique constraint)
    let created_msg = sqlx::query_as::<_, ChatMessage>(
        "INSERT INTO messages (id, conversation_id, sender_id, body, created_at)
         VALUES ($1, $2, $3, $4, NOW())
         ON CONFLICT (id) DO UPDATE SET id = EXCLUDED.id
         RETURNING *"
    )
    .bind(message_id)
    .bind(msg.conversation_id)
    .bind(sender_id)
    .bind(&msg.body)
    .fetch_one(db)
    .await?;
    
    Ok(created_msg)
}
```

#### Error Recovery Patterns

```rust
pub async fn send_message_with_retry(
    msg: ChatMessage,
    delivery_queue: &MessageQueue,
    max_attempts: u32,
) -> Result<()> {
    let mut attempt = 0;
    
    loop {
        match delivery_queue.enqueue(msg.clone()).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                attempt += 1;
                
                match &e {
                    // Retryable errors
                    ChatError::DatabaseError(_) if attempt < max_attempts => {
                        tokio::time::sleep(
                            std::time::Duration::from_secs(2_u64.pow(attempt))
                        ).await;
                        continue;
                    }
                    // Rate limit: respect retry-after
                    ChatError::RateLimited { retry_after_secs } => {
                        tokio::time::sleep(
                            std::time::Duration::from_secs(*retry_after_secs)
                        ).await;
                        continue;
                    }
                    // Non-retryable errors
                    _ => return Err(e),
                }
            }
        }
    }
}
```

#### WebSocket Error Response Pattern

```rust
use futures::{Sink, SinkExt};

async fn send_error_response(
    sink: &mut (impl Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    error: ChatError,
    request_id: Option<Uuid>,
) -> Result<()> {
    let payload = serde_json::json!({
        "type": "error",
        "request_id": request_id,
        "error": error.to_websocket_message()
    });
    
    let msg = Message::Text(payload.to_string());
    sink.send(msg).await.map_err(|_| ChatError::Internal)?;
    
    Ok(())
}

// Usage in message handler:
match process_message(incoming_msg, user_id, db).await {
    Ok(response) => {
        write.send(Message::Text(response)).await.ok();
    }
    Err(e) => {
        send_error_response(&mut write, e, incoming_msg.request_id).await.ok();
    }
}
```

#### Logging Error Context

```rust
use tracing::{error, warn, debug};

async fn handle_client_error(e: ChatError, user_id: Uuid) {
    match e {
        ChatError::AuthenticationFailed(ref msg) => {
            warn!(user_id = %user_id, msg = msg, "Authentication failed");
        }
        ChatError::Unauthorized => {
            warn!(user_id = %user_id, "Unauthorized access attempt");
        }
        ChatError::DeliveryFailed(attempts) => {
            error!(user_id = %user_id, attempts, "Message delivery failed");
        }
        ChatError::DatabaseError(ref db_err) => {
            error!(user_id = %user_id, error = ?db_err, "Database error");
        }
        ChatError::Internal => {
            error!("Internal server error");
        }
        _ => {
            debug!(user_id = %user_id, error = ?e, "Handled error");
        }
    }
}
```

### Rust-Specific Best Practices

1. **Use `From` impl for error conversion**:
   ```rust
   impl From<sqlx::Error> for ChatError {
       fn from(e: sqlx::Error) -> Self {
           ChatError::DatabaseError(e)
       }
   }
   ```

2. **Don't panic in async handlers**: Use proper error propagation
3. **Preserve error context with `#[source]`**:
   ```rust
   #[error("Failed to process: {msg}")]
   pub enum Error {
       ProcessFailed { 
           msg: String,
           #[source] source: Box<dyn std::error::Error>
       }
   }
   ```

4. **Test error paths explicitly**:
   ```rust
   #[tokio::test]
   async fn test_handles_delivery_failure() {
       let result = send_message(invalid_recipient).await;
       assert!(matches!(result, Err(ChatError::UserNotFound(_))));
   }
   ```

### Potential Gotchas

- **Idempotency key cleanup**: Set TTL in Redis to prevent indefinite storage
- **Message ID collisions**: Use UUID v4 (randomness) or Snowflake IDs for ordering
- **Error information leakage**: Don't expose internal details in client-facing errors
- **Race conditions on idempotency check**: Implement properly with database constraints

---

## 6. Deployment & Operational Considerations

### Production Checklist

```rust
// Environment-based configuration
pub struct Config {
    pub bind_addr: String,           // 0.0.0.0:8080
    pub database_url: String,        // postgresql://...
    pub redis_url: String,           // redis://...
    pub jwt_secret: String,          // Env var
    pub environment: Environment,    // dev/staging/prod
    pub log_level: tracing::Level,   // debug/info/warn/error
    pub max_connections_per_client: usize,
    pub message_queue_ttl: Duration,
    pub token_refresh_ttl: Duration,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            bind_addr: std::env::var("BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            database_url: std::env::var("DATABASE_URL")?,
            redis_url: std::env::var("REDIS_URL")?,
            jwt_secret: std::env::var("JWT_SECRET")?,
            environment: std::env::var("ENVIRONMENT")
                .ok()
                .and_then(|e| Environment::from_str(&e).ok())
                .unwrap_or(Environment::Development),
            log_level: std::env::var("LOG_LEVEL")
                .ok()
                .and_then(|l| l.parse().ok())
                .unwrap_or(tracing::Level::INFO),
            max_connections_per_client: std::env::var("MAX_CONNECTIONS_PER_CLIENT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            message_queue_ttl: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            token_refresh_ttl: Duration::from_secs(24 * 60 * 60),    // 24 hours
        })
    }
}
```

### Monitoring & Metrics

```rust
use prometheus::{Counter, Gauge, Histogram, Registry};

pub struct Metrics {
    pub active_connections: Gauge,
    pub messages_sent: Counter,
    pub messages_queued: Counter,
    pub delivery_latency: Histogram,
    pub authentication_failures: Counter,
}

impl Metrics {
    pub fn register(registry: &Registry) -> Result<Self> {
        Ok(Metrics {
            active_connections: Gauge::new("chat_active_connections", "Active WebSocket connections")?,
            messages_sent: Counter::new("chat_messages_sent_total", "Total messages sent")?,
            messages_queued: Counter::new("chat_messages_queued_total", "Total messages queued")?,
            delivery_latency: Histogram::new("chat_delivery_latency_seconds", "Message delivery latency")?,
            authentication_failures: Counter::new("chat_auth_failures_total", "Authentication failures")?,
        })
    }
}
```

---

## Summary Table: Key Decision Framework

| Requirement | Recommendation | Why | Libraries |
|---|---|---|---|
| **WebSocket Server** | Tokio + Tokio-Tungstenite | Industry standard, proven at scale | tokio@1.48, tokio-tungstenite@0.28 |
| **Message Queuing** | PostgreSQL + Redis hybrid | Balance speed (memory) with durability | sqlx@0.8, redis@0.25 |
| **Authentication** | JWT (RS256) + Refresh Rotation | Stateless, secure token rotation | jsonwebtoken@10.2 |
| **Session Management** | Redis + revocation blacklist | Fast session checks, easy revocation | redis@0.25 |
| **Database** | SQLx + Connection pooling | Compile-time queries, async-native | sqlx@0.8 |
| **Migrations** | SQLx CLI | Version control, reproducible deploys | sqlx-cli |
| **Error Handling** | Custom types + `thiserror` | Type-safe, contextual errors | thiserror@2.0 |
| **Idempotency** | Message IDs + Redis cache | Prevents duplicates on reconnect | uuid@1.19, redis@0.25 |

---

## Getting Started: Minimal Example

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use futures::{StreamExt, SinkExt};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    let (tx, _) = broadcast::channel(100);
    
    println!("Chat server listening on 0.0.0.0:8080");
    
    loop {
        let (stream, addr) = listener.accept().await?;
        let tx = tx.clone();
        
        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(ws) => {
                    println!("New connection from {}", addr);
                    let (mut write, mut read) = ws.split();
                    let mut rx = tx.subscribe();
                    
                    loop {
                        tokio::select! {
                            msg = read.next() => {
                                if let Some(Ok(msg)) = msg {
                                    let _ = tx.send(msg.clone());
                                    let _ = write.send(msg).await;
                                } else {
                                    break;
                                }
                            }
                            msg = rx.recv() => {
                                if let Ok(msg) = msg {
                                    if write.send(msg).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    println!("Connection from {} closed", addr);
                }
                Err(e) => eprintln!("WebSocket error: {}", e),
            }
        });
    }
}
```

---

## References & Additional Resources

- **Tokio Documentation**: https://tokio.rs
- **SQLx Documentation**: https://github.com/launchbadge/sqlx
- **JWT Best Practices**: https://tools.ietf.org/html/rfc7519
- **OWASP WebSocket Security**: https://owasp.org/www-community/attacks/websocket_hijacking
- **Rust Async Book**: https://rust-lang.github.io/async-book/

---

**End of Guide**

This document provides production-ready guidance. Adapt recommendations based on your specific scale, consistency requirements, and operational constraints. Start with the minimal example and incrementally add features guided by the patterns shown.
