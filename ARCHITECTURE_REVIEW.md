# Architecture Code Review

**Date**: March 12, 2026
**Reviewer**: OpenCode
**Scope**: Architecture analysis and recommendations

---

## Executive Summary

**Overall Architecture**: ✅ **GOOD**

The codebase demonstrates a well-organized layered architecture with clear separation of concerns. The backend follows a clean service-oriented pattern, and the frontend uses a reactive component model with Slint.

 However, several architectural improvements could enhance maintainability, scalability, and testability.

---

## Current Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Shared Layer                         │
│  (Protocol, Errors, DTOs - 297 lines)                    │
└─────────────────────────────────────────────────────────────┘
                         │
          ┌────────────┴────────────┐
          ▼            ▼            ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│    Frontend      │  │    Backend        │  │    Database       │
│  (Slint UI)      │  │  (Warp + Tokio)   │  │   (SQLite)        │
│  4,332 lines     │  │  8,703 lines    │  │   828 lines      │
└─────────────────┘  └─────────────────┘  └─────────────────┘
```

### Layer Responsibilities

| Layer | Lines | Purpose | Coupling |
|-------|-------|---------|---------|
| `shared` | 534 | Protocol definitions, error types, DTOs | Low |
| `frontend` | 4,332 | UI components, WebSocket client, HTTP client | Medium |
| `backend` | 8,703 | HTTP handlers, WebSocket handlers, services, middleware | Low |
| `db/queries` | 828 | SQL queries, data access | High |

---

## Backend Architecture Analysis

### Current Structure

```
backend/
├── lib.rs              (53) - Public API, tracing
├── main.rs             (72) - Entry point, CLI args
├── models/             (191) - Domain models
├── validators/        (276) - Input validation
├── utils/             (70) - Utilities
├── db/
│   ├── mod.rs          (70) - DB initialization
│   └── queries/        (828) - SQL queries
├── middleware/
│   ├── auth.rs         (124) - JWT auth middleware
│   ├── rate_limit.rs   (381) - Rate limiting
│   └── request_context.rs (72) - Request context
├── handlers/
│   ├── mod.rs          (136) - Error types
│   ├── auth.rs         (372) - Auth endpoints
│   ├── websocket.rs    (584) - WebSocket handler
│   ├── messages.rs     (552) - Message handler
│   ├── conversation.rs (534) - Conversation endpoints
│   ├── user.rs         (468) - User endpoints
│   ├── dispatcher.rs   (375) - Message routing
│   └── ... (other handlers)
├── services/
│   ├── auth_service.rs     (358) - Auth business logic
│   ├── message_service.rs  (515) - Message business logic
│   ├── user_service.rs     (199) - User search with cache
│   ├── message_queue.rs    (408) - Offline delivery queue
│   ├── presence.rs         (106) - Presence tracking
│   ├── csrf.rs             (145) - CSRF protection
│   └── ... (other services)
└── server.rs            (1,238) - HTTP routes, WebSocket upgrade
```

### Strengths

1. ✅ **Clear Layer Separation**
   - Handlers → Services → Database
   - Each layer has a single responsibility

2. ✅ **Consistent Error Handling**
   - `ApiError` for HTTP responses
   - `ChatError` in shared layer
   - `Result<T, String>` for services

3. ✅ **Good Use of Async/Await**
   - Tokio runtime throughout
   - Proper async boundaries

4. ✅ **Security in Depth**
   - JWT authentication with token revocation
   - CSRF protection for state-changing operations
   - Rate limiting with configurable windows

### Areas for Improvement

#### 1. 🟡 Service Layer Coupling (Medium Priority)

**Issue**: Services are tightly coupled to `SqlitePool` and each other.

**Location**: 
- `src/backend/handlers/messages.rs:72-77`
- `src/backend/handlers/auth.rs:67-74`

```rust
pub struct MessageHandler {
    pool: SqlitePool,
    message_service: MessageService,
    connection_manager: Arc<ConnectionManager>,
    message_queue: MessageQueueService,
    conversation_service: ConversationService,
}
```

**Problems**:
- Handlers create services internally
- Difficult to mock services in tests
- Changes to service signatures require updating multiple handlers
- No dependency injection pattern

**Recommendation**: Introduce a Service Registry pattern

```rust
// New: src/backend/services/registry.rs
pub struct ServiceRegistry {
    pool: SqlitePool,
    auth: Arc<AuthService>,
    messages: MessageService,
    conversations: ConversationService,
    users: UserService,
    presence: PresenceService,
    queue: MessageQueueService,
}

impl ServiceRegistry {
    pub fn new(pool: SqlitePool, connection_manager: Arc<ConnectionManager>) -> Self {
        Self {
            pool: pool.clone(),
            auth: Arc::new(AuthService::new(config.jwt_secret)),
            messages: Arc::new(MessageService::new(pool.clone())),
            // ... etc
        }
    }
}

// Updated handler
pub struct MessageHandler {
    services: Arc<ServiceRegistry>,
}
```

**Benefits**:
- Single place to configure all services
- Easy to mock in tests
- Clear dependency graph
- Simplified handler construction

---

#### 2. 🟡 Large Handler Files (Medium Priority)

**Issue**: Several handler files exceed 500 lines, violating Single Responsibility Principle.

**Locations**:
- `server.rs`: 1,238 lines
- `handlers/messages.rs`: 552 lines
- `handlers/websocket.rs`: 584 lines
- `handlers/conversation.rs`: 534 lines
- `handlers/user.rs`: 468 lines

**Problems**:
- Hard to navigate and understand
- Multiple responsibilities per file
- Difficult to test individual features
- Code review burden

**Recommendation**: Split into smaller, focused modules

```rust
// Current: handlers/messages.rs (552 lines)
// Proposed split:
handlers/
├── messages/
│   ├── mod.rs           (exports, types)
│   ├── send.rs          (send message endpoint)
│   ├── delivery.rs      (delivery status updates)
│   ├── search.rs        (message search)
│   └── validation.rs    (message validation helpers)
```

**Benefits**:
- Each file < 200 lines
- Clear single responsibility
- Easier to find code
- Better testability

---

#### 3. 🟡 Database Query Organization (Medium Priority)

**Issue**: `db/queries/mod.rs` is 828 lines with all SQL queries in one file.

**Problems**:
- Difficult to find specific queries
- No clear organization by domain
- Hard to maintain query consistency
- Large file impacts compile times

**Recommendation**: Organize queries by domain

```rust
// Current: db/queries/mod.rs (828 lines)
// Proposed:
db/
├── mod.rs              (re-exports)
├── users.rs           (user queries - ~150 lines)
├── messages.rs        (message queries - ~200 lines)
├── conversations.rs   (conversation queries - ~150 lines)
├── auth.rs            (auth log queries - ~100 lines)
└── transactions.rs    (transaction helpers - ~50 lines)
```

---

#### 4. 🟡 Error Type Fragmentation (Low Priority)

**Issue**: Three different error types across the codebase.

**Locations**:
- `shared/errors/mod.rs`: `ChatError` enum
- `handlers/mod.rs`: `ApiError` struct
- `lib.rs`: `DbResult<T>`, `ServiceResult<T>`, `HandlerResult<T>` type aliases

**Problems**:
- Inconsistent error handling patterns
- Confusion about which error type to use where
- Difficult to convert between error types

**Recommendation**: Consolidate to a single error hierarchy

```rust
// Proposed: shared/errors/mod.rs
#[derive(Debug, Error)]
pub enum AppError {
    // Domain errors
    #[error("Authentication failed: {0}")]
    Auth(String),
    
    #[error("Validation failed: {0}")]
    Validation(String),
    
    // ... other variants
    
    // Infrastructure errors
    #[error("Database error")]
    Database { #[source] source: DatabaseError },
    
    #[error("Internal error")]
    Internal { message: String },
}

// Type aliases in lib.rs
pub type Result<T> = std::result::Result<T, AppError>;
```

**Benefits**:
- Single error type for entire application
- Clear error hierarchy
- Easy to convert errors
- Better error context propagation

---

#### 5. 🟡 Configuration Management (Low Priority)

**Issue**: Configuration scattered across multiple files with no central config.

**Locations**:
- `server.rs:52-63` - `ServerConfig` struct
- `main.rs:11-27` - CLI args
- Environment variables in various places

**Problems**:
- No single source of truth for configuration
- Difficult to validate configuration at startup
- Hard to understand all available options

**Recommendation**: Create a centralized configuration module

```rust
// New: src/backend/config/mod.rs
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub max_message_size: usize,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
    pub min_connections: u32,
    pub max_connections: u32,
}

// ... etc

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load and validate all config
    }
}
```

**Benefits**:
- Single source of truth
- Validation at startup
- Easy to add new config options
- Better documentation

---

## Frontend Architecture Analysis

### Current Structure

```
frontend/
├── main.rs             (72) - Entry point
├── lib.rs              (53) - Exports
├── design_tokens.rs   (280) - UI constants
├── ui.rs               (72) - UI definitions
├── screens/
│   ├── chat_screen.rs          (1,144) - Main chat UI
│   ├── login_screen.rs         (124) - Login UI
│   ├── signup_screen.rs        (176) - Signup UI
│   ├── settings_screen.rs      (178) - Settings UI
│   └── user_search_screen.rs   (280) - User search UI
├── components/
│   ├── messaging/
│   │   └── message_composer.rs  (72) - Message input
│   └── presence/
│       └── conversation_header.rs (72) - Presence display
├── services/
│   ├── session.rs              (335) - Session management
│   ├── http_client.rs          (172) - HTTP client
│   └── websocket_client.rs      (617) - WebSocket client
└── handlers/
    ├── connection_handlers.rs  (53) - Connection callbacks
    └── delivery_handlers.rs    (53) - Message delivery
```

### Strengths

1. ✅ **Component-Based Architecture**
   - Screens are separated from components
   - Reusable UI components
   - Clear separation of concerns

2. ✅ **Service Layer Pattern**
   - HTTP client for REST API
   - WebSocket client for real-time
   - Session management service

3. ✅ **Handler Pattern**
   - Event handlers separated from UI logic
   - Clear callback structure

### Areas for Improvement

#### 6. 🟡 Screen File Size (Medium Priority)

**Issue**: `chat_screen.rs` is 1,144 lines, too large.

**Problems**:
- Hard to navigate
- Multiple responsibilities
- Difficult to test

**Recommendation**: Split into smaller components

```rust
// Current: screens/chat_screen.rs (1,144 lines)
// Proposed:
screens/
└── chat/
    ├── mod.rs              (exports, state)
    ├── message_list.rs     (message rendering)
    ├── input_bar.rs        (message input)
    ├── sidebar.rs          (conversation list)
    └── header.rs           (conversation header)
```

---

#### 7. 🟡 State Management (Low Priority)

**Issue**: Application state managed through Slint properties, no clear state management pattern.

**Problems**:
- State changes trigger UI updates implicitly
- No central state store
- Difficult to track state changes
- No state persistence strategy

**Recommendation**: Consider a simple state management pattern

```rust
// New: src/frontend/state/mod.rs
pub struct AppState {
    pub current_user: Option<User>,
    pub conversations: Vec<Conversation>,
    pub active_conversation: Option<String>,
    pub connection_status: ConnectionStatus,
}

impl AppState {
    pub fn reduce(&mut self, action: Action) {
        match action {
            Action::UserLoggedIn(user) => {
                self.current_user = Some(user);
            }
            Action::ConversationSelected(id) => {
                self.active_conversation = Some(id);
            }
            // ... etc
        }
    }
}
```

**Note**: This is optional - Slint's property-based approach works well for this application size.

---

## Cross-Cutting Concerns

### 8. 🟡 Testing Architecture (Medium Priority)

**Issue**: No integration tests, only unit tests.

**Current State**:
- 158 unit tests in backend
- Tests use in-memory SQLite
- No end-to-end tests
- No frontend tests (fontconfig dependency issue)

**Recommendation**: Add integration test suite

```rust
// New: tests/integration/
tests/
└── integration/
    ├── mod.rs
    ├── auth_flow.rs          (signup -> login -> logout)
    ├── messaging_flow.rs     (send -> receive -> ack)
    ├── presence_flow.rs      (online -> offline -> broadcast)
    └── rate_limiting.rs      (exhaust limit -> wait -> retry)
```

**Benefits**:
- Test real workflows
- Catch integration issues
- Validate error handling
- Document expected behavior

---

### 9. 🟡 Observability (Low Priority)

**Issue**: Good logging but no metrics or tracing.

**Current State**:
- Structured JSON logging with tracing
- Good log coverage
- No metrics collection
- No distributed tracing

**Recommendation**: Add optional metrics/tracing support

```rust
// New: src/backend/observability/mod.rs
#[cfg(feature = "metrics")]
pub mod metrics {
    use metrics::{counter, gauge, histogram};
    
    pub fn record_message_sent() {
        counter!("messages_sent").increment(1);
    }
    
    pub fn record_connection_count(count: usize) {
        gauge!("active_connections").set(count as f64);
    }
}

#[cfg(feature = "tracing")]
pub mod tracing_setup {
    // OpenTelemetry integration
}
```

**Note**: Only needed for production deployments with monitoring requirements.

---

## Scalability Considerations

### 10. 🟢 Connection Management (Good)

**Current**: In-memory connection tracking with limits.

```rust
const MAX_TOTAL_CONNECTIONS: usize = 10_000;
const MAX_CONNECTIONS_PER_USER: usize = 10;
```

**Good**: 
- Connection limits prevent resource exhaustion
- Per-user limits prevent abuse
- Clean connection lifecycle

### 11. 🟡 Message Queue (Could Improve)

**Current**: In-memory queue with exponential backoff.

**Limitations**:
- Queue lost on server restart
- No persistence
- Memory-bound

**Future Consideration**: For production, consider persistent queue

```rust
// Future: Use SQLite as queue backend
pub struct PersistentMessageQueue {
    pool: SqlitePool,
}

impl PersistentMessageQueue {
    pub async fn enqueue(&self, msg: QueuedMessage) -> Result<(), Error> {
        sqlx::query!(
            "INSERT INTO message_queue (id, recipient_id, message_id, retry_at) VALUES (?, ?, ?, ?)",
            msg.id, msg.recipient_id, msg.message_id, msg.retry_at
        ).execute(&self.pool).await?;
        Ok(())
    }
}
```

**Note**: Current in-memory queue is fine for MVP and single-server deployments.

---

## Recommended Action Plan

### Phase 1: High Impact, Low Effort (1-2 days)

1. **Split large handler files** (`messages.rs`, `websocket.rs`, `conversation.rs`, `user.rs`)
   - Each file > 400 lines
   - Split into subdirectories with focused modules
   - **Impact**: Better maintainability, easier navigation
   - **Effort**: Medium (refactoring, no logic changes)

2. **Split `db/queries/mod.rs`** (828 lines)
   - Organize by domain (users, messages, conversations, auth)
   - **Impact**: Easier to find queries, better organization
   - **Effort**: Low (file splitting, update imports)

### Phase 2: Medium Impact, Medium Effort (3-5 days)

3. **Introduce Service Registry pattern**
   - Create `ServiceRegistry` struct
   - Update handlers to use registry
   - **Impact**: Better testability, clearer dependencies
   - **Effort**: Medium (new pattern, update handlers)

4. **Add integration tests**
   - Create integration test framework
   - Test key workflows (auth, messaging, presence)
   - **Impact**: Catch integration issues, document behavior
   - **Effort**: Medium (new test files, setup)

5. **Split `chat_screen.rs`** (1,144 lines)
   - Create `screens/chat/` subdirectory
   - Split into focused components
   - **Impact**: Easier to maintain UI code
   - **Effort**: Medium (refactoring Slint components)

### Phase 3: Lower Priority (Optional, 1-2 days each)

6. **Consolidate error types**
   - Single `AppError` hierarchy
   - Remove duplicate error types
   - **Impact**: Consistency, easier error handling
   - **Effort**: Low (refactoring)

7. **Centralize configuration**
   - Create `config/` module
   - Load from environment with validation
   - **Impact**: Better config management
   - **Effort**: Low (new module, migrate existing config)

8. **Add observability** (if needed)
   - Optional metrics feature
   - Optional tracing feature
   - **Impact**: Production monitoring
   - **Effort**: Low (feature flags, optional integration)

---

## Metrics

| Aspect | Current | Target | Priority |
|--------|---------|--------|----------|
| **Layer Separation** | ✅ Good | ✅ Good | Maintain |
| **Service Coupling** | 🟡 Tight | ✅ Looser | Medium |
| **File Size** | 🟡 Large files | ✅ <300 lines | Medium |
| **Error Handling** | 🟡 Fragmented | ✅ Unified | Low |
| **Configuration** | 🟡 Scattered | ✅ Centralized | Low |
| **Testing** | 🟡 Unit only | ✅ Unit + Integration | Medium |
| **Documentation** | ✅ Good | ✅ Good | Maintain |
| **Scalability** | ✅ MVP-ready | ✅ Production-ready | Low |

---

## Conclusion

The architecture is **fundamentally sound** with good separation of concerns and clear patterns. The main improvements focus on **refactoring for maintainability** (splitting large files) rather than architectural changes. This is a sign of good initial architecture that has grown organically.

### Immediate Actions (Do Now)

1. ✅ No critical issues - continue with current architecture
2. 📋 Plan Phase 1 refactoring when convenient
3. 📋 Add integration tests for new features

### Future Considerations

1. **Multi-server deployment**: Consider service registry for service discovery
2. **Message persistence**: Consider persistent queue for production
3. **Monitoring**: Add observability features when deploying to production
4. **Database migration**: Plan PostgreSQL migration path for scaling

---

**Architecture Grade**: **B+ (Good, with clear path to A)
**Maintainability Grade**: **B** (Good, would improve with file splitting)
**Scalability Grade**: **B** (MVP-ready, production considerations documented)
