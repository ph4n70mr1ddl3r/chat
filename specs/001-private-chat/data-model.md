# Data Model: Private Chat Application

**Date**: 2025-12-15  
**Last Updated**: 2025-12-15  

---

## Overview

The data model consists of three core entities: **User**, **Conversation**, and **Message**. Relationships enforce one-to-one conversation constraints, message ordering, and user lifecycle (deletion with anonymization).

---

## Entity: User

**Purpose**: Represents a person using the chat application.

### Fields

| Field | Type | Constraints | Notes |
|-------|------|-----------|-------|
| `id` | UUID v4 (primary key) | NOT NULL, UNIQUE | Auto-generated on account creation |
| `username` | STRING | NOT NULL, UNIQUE, 1-50 chars | Case-sensitive; allows alphanumeric + underscore |
| `password_hash` | STRING | NOT NULL | Bcrypt hash (min 8 chars: 1 upper, 1 lower, 1 digit) |
| `password_salt` | STRING | NOT NULL | Generated with hash (Bcrypt handles internally) |
| `created_at` | TIMESTAMP | NOT NULL, DEFAULT NOW() | UTC timezone |
| `updated_at` | TIMESTAMP | NOT NULL, DEFAULT NOW() | Updated on password change |
| `deleted_at` | TIMESTAMP | NULL | Soft delete marker; NULL = active account |
| `is_online` | BOOLEAN | NOT NULL, DEFAULT FALSE | Transient; reset on server startup |
| `last_seen_at` | TIMESTAMP | NULL | Last WebSocket heartbeat timestamp |

### Validation Rules

- **username**: Alphanumeric + underscore; 1-50 characters; must be unique
- **password**: Minimum 8 characters; at least 1 uppercase, 1 lowercase, 1 digit (from spec)
- **deleted_at**: If NOT NULL, account is marked for deletion; messages anonymized; cannot log in
- **is_online**: Set to TRUE on WebSocket connection; FALSE on disconnect or heartbeat timeout (>30s without pong)

### Relationships

- 1:many with **Conversation** (user can have multiple conversations)
- 1:many with **Message** (user can send multiple messages)

### State Transitions

```
Active User
├─ WebSocket connects → is_online = TRUE, last_seen_at = NOW()
├─ Heartbeat received (PONG) → last_seen_at = NOW()
├─ No PONG for 30s → is_online = FALSE
├─ Logout → is_online = FALSE
└─ Delete account → deleted_at = NOW(); all messages anonymized

Deleted User
└─ Cannot log in; messages remain with sender_id = "deleted-user"
```

### SQLite Schema

```sql
CREATE TABLE users (
  id TEXT PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  password_salt TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  deleted_at TIMESTAMP,
  is_online BOOLEAN NOT NULL DEFAULT FALSE,
  last_seen_at TIMESTAMP,
  CHECK (length(username) >= 1 AND length(username) <= 50),
  CHECK (length(password_hash) > 0)
);

CREATE INDEX idx_users_username ON users(username) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_deleted_at ON users(deleted_at);
```

---

## Entity: Conversation

**Purpose**: Represents a private one-to-one chat between exactly two users.

### Fields

| Field | Type | Constraints | Notes |
|-------|------|-----------|-------|
| `id` | UUID v4 (primary key) | NOT NULL, UNIQUE | Auto-generated on conversation creation |
| `user1_id` | UUID (foreign key) | NOT NULL | References users.id (sorted < user2_id) |
| `user2_id` | UUID (foreign key) | NOT NULL | References users.id (sorted > user1_id) |
| `created_at` | TIMESTAMP | NOT NULL, DEFAULT NOW() | UTC timezone |
| `updated_at` | TIMESTAMP | NOT NULL, DEFAULT NOW() | Updated on new message |
| `last_message_at` | TIMESTAMP | NULL | Timestamp of most recent message |
| `message_count` | INTEGER | NOT NULL, DEFAULT 0 | Denormalized for UI (avoid COUNT query) |

### Validation Rules

- **one-to-one constraint**: `user1_id < user2_id` (lexical ordering prevents duplicates)
- **self-chat prevention**: `user1_id != user2_id` (enforced at app level + DB constraint)
- **unique pair**: UNIQUE constraint on (user1_id, user2_id) prevents multiple conversations between same users
- **soft delete aware**: If either user deleted, conversation remains but marked implicitly (user.deleted_at not NULL)

### Relationships

- many:1 with **User** (user1 and user2)
- 1:many with **Message** (conversation contains many messages)

### SQLite Schema

```sql
CREATE TABLE conversations (
  id TEXT PRIMARY KEY,
  user1_id TEXT NOT NULL,
  user2_id TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  last_message_at TIMESTAMP,
  message_count INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (user1_id) REFERENCES users(id),
  FOREIGN KEY (user2_id) REFERENCES users(id),
  UNIQUE (user1_id, user2_id),
  CHECK (user1_id < user2_id),
  CHECK (user1_id != user2_id)
);

CREATE INDEX idx_conversations_user1_id ON conversations(user1_id);
CREATE INDEX idx_conversations_user2_id ON conversations(user2_id);
CREATE INDEX idx_conversations_updated_at ON conversations(updated_at DESC);
```

---

## Entity: Message

**Purpose**: Represents a single message sent in a conversation.

### Fields

| Field | Type | Constraints | Notes |
|-------|------|-----------|-------|
| `id` | UUID v4 (primary key) | NOT NULL, UNIQUE | Client-generated UUID v4; enforces idempotency |
| `conversation_id` | UUID (foreign key) | NOT NULL | References conversations.id |
| `sender_id` | UUID (foreign key) | NOT NULL | References users.id; set to "Deleted User" if sender deleted |
| `recipient_id` | UUID (foreign key) | NOT NULL | References users.id (for efficient queries) |
| `content` | TEXT | NOT NULL, 1-5000 chars | Message body; plaintext (no rich formatting) |
| `created_at` | TIMESTAMP | NOT NULL, DEFAULT NOW() | Server-assigned timestamp (authoritative) |
| `delivered_at` | TIMESTAMP | NULL | When message first delivered to recipient (online or via retry) |
| `status` | ENUM | NOT NULL, DEFAULT 'pending' | pending / sent / delivered / failed |

### Validation Rules

- **idempotency**: UUID v4 generated by client using `uuid::Uuid::new_v4()`; server enforces UNIQUE constraint on id field; duplicate sends with same message id are silently rejected (ACK returned, no re-insert)
- **idempotency implementation**: 
  - Client generates UUID v4 before sending message over WebSocket
  - Server receives message with id, validates format
  - If id already exists in messages table, return ACK with existing message data (no re-insert)
  - If id is new, insert into messages table with (id, conversation_id, sender_id, recipient_id, content, created_at, ...)
  - At-least-once delivery guaranteed: client can safely retry failed sends; duplicate detection prevents message duplication
- **content length**: 1-5000 characters (from spec SC-006)
- **sender authorization**: Only the sender can update their own message (immutable after creation)
- **recipient authorization**: Only the recipient can see messages in a conversation (checked at query time)
- **created_at immutable**: Server timestamp is authoritative; prevents clock skew attacks
- **status transitions**: pending → sent → delivered (or failed if offline too long)

### Note on Status Lifecycle

- **pending**: Message stored; awaiting delivery to recipient (recipient offline or not yet connected)
- **sent**: Message received by recipient's client
- **delivered**: Message ACK'd by recipient's client; `delivered_at` timestamp set
- **failed**: Message delivery failed permanently (e.g., recipient account deleted)

Read receipts (tracking when user reads a message) are NOT implemented in MVP.

### Relationships

- many:1 with **Conversation**
- many:1 with **User** (sender)
- many:1 with **User** (recipient)

### Anonymization (Account Deletion)

When a user deletes their account:
- All messages from that user remain in conversation history
- `sender_id` is NOT changed (references deleted user)
- App layer displays "Deleted User" instead of resolving user name
- This preserves message thread coherence while respecting privacy

### SQLite Schema

```sql
CREATE TABLE messages (
  id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  sender_id TEXT NOT NULL,
  recipient_id TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  delivered_at TIMESTAMP,
  status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'sent', 'delivered', 'failed')),
  FOREIGN KEY (conversation_id) REFERENCES conversations(id),
  FOREIGN KEY (sender_id) REFERENCES users(id),
  FOREIGN KEY (recipient_id) REFERENCES users(id),
  CHECK (length(content) >= 1 AND length(content) <= 5000)
);

CREATE INDEX idx_messages_conversation_id ON messages(conversation_id, created_at DESC);
CREATE INDEX idx_messages_sender_id ON messages(sender_id, created_at DESC);
CREATE INDEX idx_messages_status ON messages(status) WHERE status IN ('pending', 'failed');
CREATE INDEX idx_messages_delivered_at ON messages(delivered_at) WHERE delivered_at IS NULL;
```

---

## State Machine: Message Lifecycle

```
Client creates message (UUID assigned)
         ↓
   Send via WebSocket
         ↓
   Server receives → Validate + Store (status='pending')
         ↓
   Is recipient online?
   ├─ YES → Push via WebSocket → Status='delivered', delivered_at=NOW()
   ├─ NO → Schedule retry (exponential backoff, infinite until online/deleted)
         ↓
   Recipient receives message
         ↓
   Status='sent' (visible in recipient's client)
         ↓
   Recipient comes online → Status='delivered', delivered_at=NOW()
```

### Offline Delivery Logic

- **Indefinite retry**: Message never expires while recipient account exists
- **Exponential backoff**: 0.5-60s (from research.md)
- **Delivery guarantee**: At-least-once (client deduplicates via UUID)
- **Account deletion edge case**: If sender deleted while message pending, deliver normally (sender_id preserved); if recipient deleted, stop retrying and mark failed

---

## Database Initialization & Migration

### V1 Schema (MVP)

All three tables created atomically on first server startup (from setup phase).

```rust
// Pseudo-code
sqlx::migrate!("migrations/")
    .run(&pool)
    .await?;
```

### No Migrations for MVP

Schema is fixed; changes (e.g., add fields for typing indicators) require manual ALTER TABLE statements documented in release notes.

---

## Performance Considerations

### Indexes

1. **users**
   - `idx_users_username`: O(log n) username lookups (critical for login)
   - `idx_users_deleted_at`: Soft delete filtering

2. **conversations**
   - `idx_conversations_user1_id`: Find all conversations for user1
   - `idx_conversations_user2_id`: Find all conversations for user2
   - `idx_conversations_updated_at`: Recent conversations (chat list)

3. **messages**
   - `idx_messages_conversation_id + created_at DESC`: Load message history (most common query)
   - `idx_messages_status WHERE status IN ('pending', 'failed')`: Retry queue scan
   - `idx_messages_delivered_at WHERE delivered_at IS NULL`: Undelivered messages (future optimization)

### Denormalization

- **conversations.message_count**: Avoid expensive COUNT queries in UI
- **conversations.last_message_at**: UI displays "last message 5 min ago" without querying messages
- **messages.recipient_id**: Denormalized for efficient authorization checks (avoid conversation join)

### Query Performance Targets

| Operation | Query | Target Latency |
|-----------|-------|--------|
| Load conversation history (100 messages) | SELECT * FROM messages WHERE conversation_id=? ORDER BY created_at DESC LIMIT 100 | <100ms |
| Get undelivered messages (retry queue) | SELECT * FROM messages WHERE status='pending' OR status='failed' | <50ms |
| Login (username lookup) | SELECT * FROM users WHERE username=? | <10ms |
| Get online status | SELECT is_online FROM users WHERE id=? | <5ms |

---

## Concurrency & Transaction Safety

### Transaction Isolation

All writes use serializable isolation (SQLite default with WAL mode):
- Account creation: Insert user + initial presence record
- Message send: Insert message + update conversation.updated_at + update message_count
- Account deletion: Soft delete (mark deleted_at)

### Race Conditions Handled

1. **Duplicate message**: UUID constraint prevents re-insertion
2. **Self-conversation**: CHECK constraint prevents user1_id == user2_id
3. **Concurrent message sends**: Append-only; no conflict resolution needed (TCP orders within connection)
4. **Concurrent user deletion**: Soft delete safe; messages remain; sender_id foreign key still valid

---

## Compliance & Privacy

### Data Retention

- **Messages**: Retained indefinitely (spec: "unlimited retention indefinitely per conversation")
- **User accounts**: Soft-deleted (deleted_at marker) but data never purged (simplifies GDPR "right to be forgotten" via anonymization)
- **Online status**: Transient (lost on server restart); not persisted to disk

### GDPR / Privacy Considerations

- **Right to be forgotten**: Implement anonymization (messages remain, sender name → "Deleted User")
- **Data export**: Future feature (not in MVP); would export full message history as JSON
- **Encryption at rest**: SQLite encryption deferred (use full-disk encryption or encrypt-on-insert)

---

## Diagram: Entity Relationships

```
┌──────────────────────────────────────────────────┐
│                    Users                         │
├──────────────────────────────────────────────────┤
│ id (PK)                                          │
│ username (UNIQUE)                                │
│ password_hash + password_salt                    │
│ created_at, updated_at, deleted_at (soft delete) │
│ is_online, last_seen_at                          │
└──────────────────────────────────────────────────┘
         ↑                                ↑
         │1                              │1
         │                               │
    user1_id                        user2_id
         │                               │
         │many                          │many
┌────────────────────────────────────────────────┐
│              Conversations                     │
├────────────────────────────────────────────────┤
│ id (PK)                                        │
│ user1_id, user2_id (UNIQUE pair constraint)   │
│ created_at, updated_at, last_message_at       │
│ message_count (denormalized)                  │
└────────────────────────────────────────────────┘
              ↑
              │1
              │
        conversation_id
              │
              │many
┌──────────────────────────────────────────────┐
│               Messages                       │
├──────────────────────────────────────────────┤
│ id (PK, client-assigned UUID)                │
│ conversation_id (FK)                         │
│ sender_id (FK), recipient_id (FK)            │
│ content (1-5000 chars)                       │
│ created_at (server timestamp)                │
│ delivered_at, read_at, status               │
└──────────────────────────────────────────────┘
         ↑                        ↑
         │                        │
    sender_id              recipient_id
         │                        │
         └────────┬───────────────┘
                  │many
                  │
              Users (FK)
```

---

## Next Steps

✅ Data model complete.  
→ Phase 1: Generate API contracts (WebSocket protocol)
