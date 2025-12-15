# WebSocket Protocol Contract

**Version**: 1.0  
**Date**: 2025-12-15  
**Status**: Approved (Spec locked)  

---

## Overview

The chat application uses a JSON-based protocol over WebSocket connections (RFC 6455). All communication uses UTF-8 text frames with the message structure defined below.

- **Endpoint**: `ws://localhost:8080/socket` (dev) or `wss://chat.example.com/socket` (prod)
- **Port**: Configurable (default 8080 for dev, 443 for prod with TLS)
- **Query Parameters**: `?token=<JWT>` for authentication
- **Protocol Version**: `Sec-WebSocket-Protocol: chat-v1` (future versioning)

---

## Authentication Handshake

### Client → Server (HTTP Upgrade Request)

```http
GET /socket?token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9... HTTP/1.1
Host: localhost:8080
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: [base64 random]
Sec-WebSocket-Version: 13
Sec-WebSocket-Protocol: chat-v1
```

### Server Response (Success)

```http
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: [computed from key]
Sec-WebSocket-Protocol: chat-v1
```

**Validation Rules**:
- JWT token must be valid (signature verified, expiration checked)
- Token subject (`sub`) must match an active user
- Server rejects with `HTTP 401 Unauthorized` if token invalid/expired
- Server rejects with `HTTP 403 Forbidden` if user deleted

### Server Response (Failure)

```http
HTTP/1.1 401 Unauthorized
Content-Type: application/json

{"error": "Invalid or expired token"}
```

---

## Message Types & Schemas

All messages use the following base structure:

```json
{
  "id": "unique-identifier",
  "type": "message|typing|presence|ack|error|heartbeat",
  "timestamp": 1702657890000,
  "data": { /* type-specific fields */ }
}
```

### 1. Text Message (Client → Server → Recipient)

#### Client sends message:

```json
{
  "id": "msg-550e8400-e29b-41d4-a716-446655440000",
  "type": "message",
  "timestamp": 1702657890000,
  "data": {
    "recipientId": "user-456",
    "content": "Hello, World!"
  }
}
```

**Validation**:
- `id`: UUID v4, must be unique (reject duplicate with 409 Conflict)
- `recipientId`: Must reference a valid user (reject with 404 if user not found)
- `content`: 1-5000 characters, UTF-8 valid (reject with 400 if invalid)
- `timestamp`: Client-provided hint; server uses server timestamp as authoritative

#### Server → Recipient (real-time delivery):

```json
{
  "id": "msg-550e8400-e29b-41d4-a716-446655440000",
  "type": "message",
  "timestamp": 1702657890123,
  "data": {
    "senderId": "user-123",
    "senderUsername": "alice",
    "recipientId": "user-456",
    "content": "Hello, World!",
    "conversationId": "conv-789",
    "status": "delivered"
  }
}
```

**Server Behavior**:
- If recipient online: send immediately via WebSocket
- If recipient offline: store in DB + queue for retry (exponential backoff)
- Always send ACK to sender (see ACK schema below)

#### Server → Sender (acknowledgement):

```json
{
  "id": "msg-550e8400-e29b-41d4-a716-446655440000",
  "type": "ack",
  "timestamp": 1702657890124,
  "data": {
    "status": "sent",
    "conversationId": "conv-789",
    "serverTimestamp": 1702657890123
  }
}
```

---

### 2. Message Acknowledgement (Server → Client)

Confirms message received and stored by server.

```json
{
  "id": "msg-550e8400-e29b-41d4-a716-446655440000",
  "type": "ack",
  "timestamp": 1702657890124,
  "data": {
    "status": "sent",
    "conversationId": "conv-789",
    "messageId": "msg-550e8400-e29b-41d4-a716-446655440000",
    "serverTimestamp": 1702657890123
  }
}
```

**Status Values**:
- `sent`: Message stored in DB; awaiting delivery to recipient
- `delivered`: Recipient received message (online delivery)
- `failed`: Permanent delivery failure (recipient deleted before delivery attempted)

**Retry Logic** (Client):
- If no ACK within 5 seconds: Resend message with same `id`
- After 3 failed attempts: Display error to user ("Message failed to send")
- Server deduplicates via `id` (idempotent insert-or-update)

---

### 3. Typing Indicator (Client → Server → Recipient)

Real-time notification that user is typing.

#### Client sends:

```json
{
  "id": "typing-550e8400-e29b-41d4-a716-446655440001",
  "type": "typing",
  "timestamp": 1702657890000,
  "data": {
    "recipientId": "user-456",
    "isTyping": true
  }
}
```

#### Server → Recipient:

```json
{
  "id": "typing-550e8400-e29b-41d4-a716-446655440001",
  "type": "typing",
  "timestamp": 1702657890000,
  "data": {
    "senderId": "user-123",
    "senderUsername": "alice",
    "isTyping": true
  }
}
```

**Behavior**:
- Send every 1 second while typing (to indicate ongoing activity)
- Send with `isTyping: false` when typing stops or after 3 seconds of inactivity
- Server does NOT persist; ephemeral broadcast only
- If recipient offline: drop (no storage, no retry)

---

### 4. Presence Status (Server → All Connected Clients)

Real-time notification of user online/offline status.

#### Server broadcasts (on connect):

```json
{
  "id": "presence-550e8400-e29b-41d4-a716-446655440002",
  "type": "presence",
  "timestamp": 1702657890000,
  "data": {
    "userId": "user-123",
    "username": "alice",
    "isOnline": true,
    "lastSeenAt": 1702657890000
  }
}
```

#### Server broadcasts (on disconnect):

```json
{
  "id": "presence-550e8400-e29b-41d4-a716-446655440003",
  "type": "presence",
  "timestamp": 1702657890000,
  "data": {
    "userId": "user-123",
    "username": "alice",
    "isOnline": false,
    "lastSeenAt": 1702657890000
  }
}
```

**Broadcast Scope**:
- Sent to all users who have an active conversation with the user
- Not sent to all users (privacy: don't leak online status to strangers)

---

### 5. Server Heartbeat (Server → Client)

RFC 6455 ping-pong keepalive frames. Handled at WebSocket protocol level (not application).

**Server behavior**:
- Sends PING every 25 seconds
- Expects PONG within 5 seconds
- Closes connection (1000 normal closure) if PONG timeout

**Client behavior**:
- Automatic PONG response (handled by WebSocket library)
- Updates `last_seen_at` on server

---

### 6. Error Response (Server → Client)

Server sends error message for invalid requests or state violations.

```json
{
  "id": "error-550e8400-e29b-41d4-a716-446655440004",
  "type": "error",
  "timestamp": 1702657890000,
  "data": {
    "code": "INVALID_MESSAGE_LENGTH",
    "message": "Message content exceeds 5000 character limit",
    "details": {
      "sentLength": 5001,
      "maxLength": 5000
    }
  }
}
```

**Error Codes**:

| Code | HTTP Equivalent | Description | Action |
|------|---|---|---|
| `INVALID_MESSAGE_LENGTH` | 400 | Content > 5000 chars or < 1 char | Display error; allow user to edit |
| `RECIPIENT_NOT_FOUND` | 404 | recipientId does not exist | Display error; prompt user to verify recipient |
| `RECIPIENT_DELETED` | 404 | Recipient account deleted | Display error; show "User no longer available" |
| `INVALID_JSON` | 400 | Message not valid JSON | Log error; close connection after 3 attempts |
| `RATE_LIMIT_EXCEEDED` | 429 | User exceeded 100 msg/min quota | Display error; show retry-after countdown |
| `UNAUTHORIZED` | 401 | Token expired or invalid | Redirect to login; clear session |
| `SERVER_ERROR` | 500 | Unexpected server error | Display generic error; suggest retry |

**Client Behavior**:
- Display user-friendly error message (above codes map to localized UI strings)
- For 429 (rate limit): Show countdown timer, disable send button until cooldown expires
- For 401 (auth): Redirect to login screen
- For 500: Log error; suggest user refresh page

---

### 7. Connection Closed (Server → Client)

WebSocket closure with reason code.

**Normal Closure** (code 1000):
```
1000 Normal Closure - "User logged out"
```

**Authentication Failure** (code 1008):
```
1008 Policy Violation - "Token expired or invalid"
```

**Rate Limiting** (code 1008):
```
1008 Policy Violation - "Rate limit exceeded; reconnect after 60s"
```

**Server Shutdown** (code 1001):
```
1001 Going Away - "Server shutting down; reconnect in 30s"
```

**Protocol Violation** (code 1002):
```
1002 Protocol Error - "Invalid message format; reconnect"
```

---

## Client → Server Command Flow

### 1. Connect & Authenticate

```
Client: HTTP upgrade with JWT token
Server: 101 Switching Protocols (or 401 Unauthorized)
Client: Receives PING every 25s → Auto-responds with PONG
```

### 2. Send Message

```
Client: {"type": "message", "id": "msg-123", "data": {...}}
         ↓
Server: Validate + Store
         ├─ If recipient online: Push message → Recipient receives
         ├─ If recipient offline: Queue for retry + update DB
         ↓
Server: Send ACK → Client receives confirmation
```

### 3. Receive Message (if recipient)

```
Server: {"type": "message", "id": "msg-123", "data": {...}}
Client: Receives + displays in UI
Client: (Optional: could send read receipt, not in MVP)
```

### 4. Typing Notification

```
Client: {"type": "typing", "id": "typing-123", "data": {"isTyping": true}}
         ↓
Server: Validate + Broadcast (if recipient online)
         ↓
Recipient: Receives typing indicator
Client: User stops typing → Send {"data": {"isTyping": false}}
```

### 5. Presence Update

```
Client: (connects to WebSocket)
         ↓
Server: Updates users.is_online = true
Server: Broadcasts presence to all users in conversations with this user
         ↓
Other clients: Receive presence update → Show "Alice is online"
```

---

## Rate Limiting & Throttling

### Per-User Token Bucket

- **Capacity**: 100 messages per 60 seconds
- **Burst**: 5 messages per second (burst allowance)
- **Refill**: 100/60 = ~1.67 tokens/second
- **Enforcement**: Server closes connection on repeated violation (after 3 failures)

**Example**:
- User sends 5 messages rapidly → OK (within burst)
- User sends 6th message within 1 second → Rejected (429)
- User sends message after waiting 1 second → OK (1 token refilled)

### Typing Indicator (No Rate Limit)

- Sent every 1 second while typing
- No quota enforcement; ephemeral only

### Presence Updates (Server Broadcast)

- Only sent on connection/disconnection
- No per-message quota

---

## Message Ordering Guarantees

### Within Single Connection

TCP guarantees frame delivery order. Messages arrive in send order.

### Across Reconnections

Server assigns `created_at` timestamp (server-authoritative). Client sorts by `created_at` to restore order after reconnection.

**Example**:
```
User A sends: "Hello" at 12:00:00
User A disconnect + reconnect at 12:00:10
User A sends: "Are you there?" at 12:00:15
User B receives both in order: ["Hello" (12:00:00), "Are you there?" (12:00:15)]
```

---

## Idempotency & Retry Semantics

### Client-Side Deduplication

Client assigns UUID v4 to each message. If send fails, resend with same `id`.

```
Send message (id=msg-123) at 12:00:00
No ACK received by 12:00:05 → Resend (id=msg-123) at 12:00:05
Server deduplicates: IF id exists THEN return existing ACK else insert
```

### Server-Side Deduplication

Unique constraint on `messages.id` (primary key) prevents duplicates.

---

## Test Scenarios

### Happy Path: Online Message Delivery

1. User A sends message to User B
2. User B online → Message delivered immediately
3. Both users receive ACK
4. ✅ Test: Message appears in B's chat within 2 seconds

### Offline Delivery & Retry

1. User A sends message to User B
2. User B offline → Message queued
3. Server retries (exponential backoff)
4. User B comes online after 5 minutes
5. Server delivers message
6. ✅ Test: Message appears after reconnection

### Rate Limiting

1. User sends 100 messages rapidly
2. Message 101 rejected with 429
3. ✅ Test: Error response received; client shows error

### Protocol Violation

1. Client sends invalid JSON
2. Server closes connection with 1002
3. ✅ Test: Client reconnects; server accepts valid message

### Typing Indicator

1. User A types in User B's conversation
2. User A online, User B online → Typing indicator sent
3. User A stops typing → isTyping=false sent
4. ✅ Test: Typing indicator appears/disappears in real-time

---

## Future Enhancements (Not MVP)

- Read receipts (`type: "read"` message when recipient opens message)
- Message reactions (emoji reactions via `type: "reaction"`)
- File sharing (binary frames or separate HTTP upload endpoint)
- Message editing (new `type: "edit"` with messageId reference)
- Conversation groups (multi-user chats)
- Message search (ephemeral; no persistence)

---

## Next Steps

✅ WebSocket protocol contract complete.  
→ Generate server contract (HTTP endpoints for setup/config)  
→ Generate client contract (Slint UI message format)
