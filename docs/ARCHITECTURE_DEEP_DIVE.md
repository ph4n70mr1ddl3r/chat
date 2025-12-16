# Chat Application - Architecture Deep Dive

**Purpose**: Comprehensive architecture documentation for PRD, research, and implementation planning  
**Generated**: 2025-12-16T08:05:00Z  
**Scope**: Backend services, frontend UI, database, and real-time communication

## Table of Contents

1. [System Architecture](#system-architecture)
2. [Backend Services](#backend-services)
3. [Frontend Architecture](#frontend-architecture)
4. [Data Flow](#data-flow)
5. [Authentication & Security](#authentication--security)
6. [Real-Time Communication](#real-time-communication)
7. [Database Design](#database-design)
8. [Key Patterns](#key-patterns)

---

## System Architecture

### High-Level Components

```
┌─────────────────────────────────────────────────────┐
│                    Chat Application                  │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌──────────────────┐        ┌──────────────────┐   │
│  │   FRONTEND       │◄─────►│   BACKEND        │   │
│  │  (Slint UI)      │        │  (Warp Server)   │   │
│  │                  │        │                  │   │
│  │ • Message View   │        │ • WebSocket Hub  │   │
│  │ • Chat Input     │        │ • Message Router │   │
│  │ • User List      │        │ • Auth Service   │   │
│  │ • Login Screen   │        │ • DB Layer       │   │
│  │                  │        │                  │   │
│  └──────────────────┘        └──────────────────┘   │
│                                      │               │
│                              ┌───────▼────────┐     │
│                              │  SQLITE / PG   │     │
│                              │  Database      │     │
│                              └────────────────┘     │
│                                                      │
└─────────────────────────────────────────────────────┘
```

### Deployment Model

- **Single-server** self-hosted deployment
- **Native desktop client** (Slint + Rust binary)
- **Stateless server** (JWT-based authentication)
- **Persistent SQLite** database
- **WebSocket** for real-time bidirectional communication

---

## Backend Services

### Service Architecture

The backend is organized into layered services:

```
┌─────────────────────────────────────┐
│  Handlers (HTTP/WebSocket)          │
│  • Auth, Conversation, Messages     │
├─────────────────────────────────────┤
│  Services (Business Logic)          │
│  • User, Message, Presence, Search  │
├─────────────────────────────────────┤
│  Models (Domain Objects)            │
│  • User, Conversation, Message      │
├─────────────────────────────────────┤
│  Database Layer (SQLx)              │
│  • Connection pool, Queries         │
├─────────────────────────────────────┤
│  Infrastructure                     │
│  • Server, Middleware, Logging      │
└─────────────────────────────────────┘
```

### Key Handler Modules

| Module | Responsibility | Key Functions |
|--------|-----------------|--------------|
| **auth.rs** (10KB) | User authentication | Login, register, token validation |
| **auth_with_rate_limit.rs** (6KB) | Rate limiting | 5 failed attempts / 15min per IP |
| **websocket.rs** (14KB) | WebSocket lifecycle | Connect, disconnect, message dispatch |
| **messages.rs** (13KB) | Message handling | Send, receive, delivery, acknowledgment |
| **conversation.rs** (14KB) | Conversation mgmt | Create, list, delete conversations |
| **dispatcher.rs** (11KB) | Message routing | Route messages between users |
| **parser.rs** (11KB) | Protocol parsing | Parse WebSocket messages |
| **heartbeat.rs** (8KB) | Connection keep-alive | Ping/pong, presence updates |
| **handshake.rs** (8KB) | WebSocket setup | Initial connection, headers |
| **user.rs** (10KB) | User operations | Profile, deletion, anonymization |
| **router.rs** (7KB) | HTTP routing | Route requests to handlers |

### Core Service Flows

#### Authentication Flow
```
Client Login Request
    ↓
RateLimit Check (5 failed/15min per IP)
    ↓
Auth Service validates credentials
    ↓
bcrypt password comparison
    ↓
JWT token generation
    ↓
Client receives token + refresh token
```

#### Message Sending Flow
```
Client sends WebSocket message
    ↓
Handler parses message envelope
    ↓
Validation (length, format, recipient)
    ↓
Message Service processes
    ↓
Database persistence
    ↓
Recipient online?
    ├─ YES → Immediate WebSocket dispatch
    └─ NO → Queue for delivery on login
    ↓
Client receives delivery confirmation
```

#### User Deletion Flow
```
User requests account deletion
    ↓
All user's conversations queried
    ↓
For each message:
    ├─ Set is_anonymized = true
    ├─ Set sender_name = "Deleted User"
    └─ Preserve message content
    ↓
User account deleted
    ↓
User sessions terminated
    ↓
Remaining users see "Deleted User" in history
```

### Service Implementations

**Authentication Service** (`auth_service.rs`)
- Password hashing with bcrypt
- JWT token generation (stateless)
- Token refresh mechanism
- Rate limiting (5 attempts / 15 min per IP)

**Message Service** (`message_service.rs`)
- Message validation (max 5000 chars)
- Persistence to database
- Offline queueing
- Delivery acknowledgment

**User Service** (`user_service.rs`)
- User registration
- Profile management
- Account deletion with anonymization
- User search

**Presence Service** (`presence_service.rs`)
- Online/offline status tracking
- Presence broadcasts to conversation partners
- Session management

**Search Service** (`search_service.rs`)
- Message search by content
- Conversation search
- User search

---

## Frontend Architecture

### UI Structure (Slint)

```
AppRoot (main.slint)
├── LoginScreen
│   ├── UsernameInput
│   ├── PasswordInput
│   └── LoginButton
│
├── ChatScreen
│   ├── UserList
│   │   ├── UserItem (repeater)
│   │   └── NewConversation
│   │
│   ├── ConversationView
│   │   ├── ConversationHeader
│   │   ├── MessageList
│   │   │   └── MessageBubble (repeater)
│   │   │       ├── SenderName
│   │   │       ├── Timestamp
│   │   │       └── MessageContent
│   │   │
│   │   └── MessageInput
│   │       ├── TextInput
│   │       └── SendButton
│   │
│   └── SettingsPanel
│       └── Logout
```

### Screen Components

| Component | File | Responsibility |
|-----------|------|-----------------|
| **ChatScreen** | `screens/chat_screen.rs` | Main chat interface controller |
| **LoginScreen** | `screens/login_screen.rs` | Authentication UI |
| **MessageBubble** | `components/message_bubble.slint` | Individual message display |
| **MessageInput** | `components/message_input.slint` | Message composition |
| **UserList** | `components/user_list.slint` | Active conversations |
| **ChatScreen.slint** | `components/chat_screen.slint` | Chat layout |

### Service Layer (Frontend)

| Service | Responsibility |
|---------|-----------------|
| **http_client.rs** | REST API communication (authentication) |
| **websocket_service.rs** | WebSocket connection and message streaming |
| **auth_service.rs** | Authentication state management |
| **message_service.rs** | Message state and caching |

### UI Update Flow

```
User Action (button click, text input)
    ↓
Event handler triggered
    ↓
Call service method (http_client, websocket_service)
    ↓
Async operation in background
    ↓
Response arrives
    ↓
Update UI properties/models
    ↓
Slint re-renders affected components
    ↓
User sees updated UI
```

### Real-Time Updates (WebSocket)

```
WebSocket message received from server
    ↓
websocket_service processes
    ↓
Extract message type (NewMessage, UserOnline, etc.)
    ↓
Update message_service state
    ↓
Trigger UI model updates
    ↓
Components re-render automatically
```

---

## Data Flow

### Message Sending (Synchronous)

```
┌─────────────┐
│   FRONTEND  │
│  (Slint UI) │
└──────┬──────┘
       │ User sends message
       │
       ▼
┌────────────────────────────────────────┐
│  websocket_service.send_message()      │
│  • Validate message length (max 5000)  │
│  • Format message envelope             │
│  • Send via WebSocket                  │
└────────┬─────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────┐
│  BACKEND - WebSocket Server                        │
│                                                    │
│  1. Handler receives WebSocket frame               │
│  2. parser.rs: Parse MessageEnvelope               │
│  3. dispatcher.rs: Route to messages handler       │
│  4. Messages handler:                              │
│     - Validate (format, length, recipient)        │
│     - Call message_service.save_message()          │
│     - Database INSERT                              │
│     - Get delivery status (online/offline)         │
│     - If online: dispatch via dispatcher           │
│     - If offline: add to queue                     │
│  5. Return delivery confirmation                   │
└────────┬──────────────────────────────────────────┘
         │ WebSocket frame
         │
         ▼
┌─────────────────────────┐
│  RECIPIENT FRONTEND     │
│  • Receive message      │
│  • Update MessageList   │
│  • Auto-scroll view     │
│  • Mark as delivered    │
└─────────────────────────┘
```

### Message Retrieval (On Login)

```
┌─────────────┐
│   FRONTEND  │
│  (Slint UI) │
└──────┬──────┘
       │ User logs in
       │
       ▼
┌────────────────────────────┐
│  auth_service.login()      │
│  • POST /auth/login        │
│  • Receive JWT token       │
│  • Store token             │
└────────┬──────────────────┘
         │
         ▼
┌────────────────────────────────────┐
│  websocket_service.connect()       │
│  • Establish WebSocket             │
│  • Authenticate with JWT           │
└────────┬──────────────────────────┘
         │
         ▼
┌──────────────────────────────────────┐
│  BACKEND - Handshake                 │
│  1. WebSocket connection accepted    │
│  2. Verify JWT token                 │
│  3. Load user's conversations        │
│  4. Load offline message queue       │
│  5. Send all pending messages        │
│  6. Set user presence = ONLINE       │
└────────┬─────────────────────────────┘
         │ WebSocket stream starts
         │
         ▼
┌─────────────────────────────────┐
│  FRONTEND receives messages     │
│  • Process each message         │
│  • Update MessageList model     │
│  • Display in conversation      │
│  • Auto-scroll to latest        │
└─────────────────────────────────┘
```

---

## Authentication & Security

### JWT Token Flow

```
Authorization Header: "Bearer eyJhbGciOiJIUzI1NiIs..."
                      │
                      ├─ Token expires after 1 hour
                      ├─ Contains user_id claim
                      ├─ Contains username claim
                      └─ Signed with server secret

Refresh Flow:
├─ Client stores: JWT + Refresh Token
├─ On JWT expiry: POST /auth/refresh + refresh token
├─ Server validates refresh token
├─ Return new JWT
└─ Client updates authorization header
```

### Rate Limiting

```
Failed Login Attempt Tracking:
├─ Key: user_ip (from request)
├─ Counter: failed attempts
├─ Window: 15 minutes
├─ Threshold: 5 attempts
└─ Action on 5th: Return 429 Too Many Requests

Example:
├─ 1:00 PM - Attempt 1 failed ✗
├─ 1:02 PM - Attempt 2 failed ✗
├─ 1:04 PM - Attempt 3 failed ✗
├─ 1:06 PM - Attempt 4 failed ✗
├─ 1:08 PM - Attempt 5 BLOCKED 🚫
├─ 1:15 PM - Counter resets
└─ 1:16 PM - Can attempt again ✓
```

### Password Security

```
Registration:
┌─────────────────────┐
│ User enters password│
│ (e.g., "Secure123")│
└──────────┬──────────┘
           │
           ▼
┌──────────────────────────────────┐
│ Validate strength:                │
│ ✓ Min 8 characters                │
│ ✓ 1+ uppercase (A-Z)              │
│ ✓ 1+ lowercase (a-z)              │
│ ✓ 1+ digit (0-9)                  │
└──────────┬───────────────────────┘
           │
           ▼
┌──────────────────────────────────┐
│ Hash with bcrypt:                 │
│ • 12 rounds (cost factor)          │
│ • Auto-salted per password        │
│ • Stored in database              │
└──────────┬───────────────────────┘
           │
           ▼
Login Verification:
├─ User enters: "Secure123"
├─ bcrypt compares against stored hash
├─ Same? → Generate JWT → Grant access
└─ Different? → Increment failed counter
```

---

## Real-Time Communication

### WebSocket Protocol

**Connection**
```
Client initiates WebSocket upgrade:
GET /ws HTTP/1.1
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: ...
Authorization: Bearer <JWT>

Server responds:
101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
```

**Message Envelope Format**
```json
{
  "id": "uuid-123",
  "type": "message" | "ack" | "ping" | "presence",
  "timestamp": "2025-12-16T08:05:00Z",
  "payload": {
    // type-specific content
  }
}
```

**Message Types**

| Type | Direction | Payload |
|------|-----------|---------|
| **message** | Bidirectional | { recipient_id, content, conversation_id } |
| **ack** | Both ways | { message_id, status: "delivered" \| "read" } |
| **ping** | Server→Client | {} (keep-alive) |
| **pong** | Client→Server | {} (response to ping) |
| **presence** | Bidirectional | { user_id, status: "online" \| "offline" } |

**Message Delivery States**
```
┌────────┐
│ Sending│
└───┬────┘
    │
    ▼
┌────────────┐
│ Sent       │  (queued on server)
└───┬────────┘
    │
    ├─ Recipient online?
    │   ├─ YES → dispatch immediately
    │   └─ NO → queue for next login
    │
    ▼
┌────────────┐
│ Delivered  │  (client received)
└───┬────────┘
    │
    ▼
┌────────────┐
│ Acknowledged│  (optional - client confirmed read)
└────────────┘
```

---

## Database Design

### Core Schema

**users**
```sql
CREATE TABLE users (
  id UUID PRIMARY KEY,
  username VARCHAR UNIQUE NOT NULL,
  password_hash VARCHAR NOT NULL,
  created_at TIMESTAMP,
  deleted_at TIMESTAMP NULL,  -- soft delete
  is_anonymized BOOLEAN,       -- anonymization flag
)
```

**conversations**
```sql
CREATE TABLE conversations (
  id UUID PRIMARY KEY,
  user1_id UUID NOT NULL,  -- first user (ordered)
  user2_id UUID NOT NULL,  -- second user
  created_at TIMESTAMP,
  UNIQUE(user1_id, user2_id)  -- only one conversation per pair
)
```

**messages**
```sql
CREATE TABLE messages (
  id UUID PRIMARY KEY,
  conversation_id UUID NOT NULL,
  sender_id UUID NOT NULL,
  content VARCHAR(5000) NOT NULL,
  sent_at TIMESTAMP,
  is_anonymized BOOLEAN,       -- show "Deleted User" if true
  deleted_user_name VARCHAR,    -- original name for reference
  FOREIGN KEY(conversation_id) → conversations,
  FOREIGN KEY(sender_id) → users
)
```

**user_sessions**
```sql
CREATE TABLE user_sessions (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  websocket_id VARCHAR,        -- unique WebSocket connection ID
  connected_at TIMESTAMP,
  last_activity TIMESTAMP,
  ip_address VARCHAR,
  FOREIGN KEY(user_id) → users
)
```

**user_presence**
```sql
CREATE TABLE user_presence (
  user_id UUID PRIMARY KEY,
  status VARCHAR ('online' | 'offline'),
  last_seen TIMESTAMP,
  FOREIGN KEY(user_id) → users
)
```

### Key Design Decisions

✓ **One-to-one conversations**: Unique constraint on (user1_id, user2_id)  
✓ **Immutable messages**: No update/delete; only anonymization  
✓ **Soft user deletion**: Keep user record, anonymize messages  
✓ **Message anonymization**: is_anonymized flag + deleted_user_name  
✓ **No message quota**: Unlimited message storage per conversation  
✓ **Session tracking**: Track active WebSocket connections  
✓ **Presence state**: Separate table for efficient presence updates  

---

## Key Patterns

### Error Handling

**Unified Error Type**
```rust
pub enum ChatError {
    NotFound,
    Unauthorized,
    BadRequest(String),
    ConflictingState(String),
    InternalServer,
}

// Every handler returns Result<Response, ChatError>
// Middleware converts ChatError → HTTP status
```

**Error Propagation**
```rust
message_service.send_message(...)? 
  // ? operator propagates error up
  // Handler catches via error middleware
  // Returns appropriate HTTP status
```

### Async Patterns

**Tokio-based concurrency**
```rust
// All handlers are async
async fn handle_message(
    ws: WebSocket,
    db: DatabasePool,
) -> Result<(), ChatError> {
    // Can spawn concurrent tasks
    tokio::spawn(async { ... });
    // Can await database queries
    let user = db.get_user(id).await?;
}
```

### State Management

**Shared Database Pool**
```rust
// Created once at startup
let db = create_pool(connection_string);

// Passed to all handlers
// ConnectionPool is thread-safe
// Async queries via sqlx
let user = db.query("SELECT * FROM users ...").fetch_one().await?;
```

**WebSocket Connection Hub**
```rust
// Track active connections in memory
// Key: user_id
// Value: broadcast channel sender
let connections: DashMap<UserId, Sender>;

// When message arrives for user_id:
if let Some(sender) = connections.get(&user_id) {
    sender.broadcast(message);
}
```

---

## Performance Considerations

### Bottlenecks & Solutions

| Bottleneck | Solution |
|-----------|----------|
| Database queries | Connection pooling (SQLx) |
| Message parsing | Streaming JSON parser |
| WebSocket messages | Broadcast channels (Tokio) |
| Memory (many connections) | Efficient session tracking |
| Disk (message growth) | Archival/retention policy (future) |

### Scalability Path

**MVP** (current)
- SQLite single-server
- Handles ~100 concurrent users
- ~1000 messages/conversation average

**Production** (PostgreSQL)
- Replace SQLite with PostgreSQL
- Connection pooling across replicas
- Load balancing (nginx)
- Message archival service

**High Scale** (distributed)
- Message queue (Redis pub/sub or similar)
- Microservices (separate presence, search services)
- Database sharding by conversation_id
- Caching layer for presence/user data

---

## Testing Strategy

### Test Coverage

**Unit Tests**: 
- Message validation
- Model behavior
- Error handling

**Integration Tests**:
- Multi-user conversations
- Message delivery
- Presence tracking
- User deletion with anonymization
- Login/logout flows
- WebSocket handshakes

**Contract Tests**:
- Message envelope schema validation
- Server API contract compliance

**Performance Tests**:
- Load testing (100+ concurrent users)
- Message throughput (up to 100 msgs/sec)
- Presence update latency

### CI/CD

```
Git Push
  ↓
GitHub Actions
  ├─ cargo test (all tests)
  ├─ cargo clippy (linting)
  ├─ cargo fmt --check (formatting)
  └─ Cross-compilation (Windows binary)
     ↓
Build Artifacts
  ├─ Backend binary
  └─ Frontend binary
```

---

*This documentation was generated by the exhaustive project scan. Last updated: 2025-12-16T08:05:00Z*
