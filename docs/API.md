# Private Chat Application - API Documentation

**Version**: 1.0.0  
**Last Updated**: 2025-12-16  
**Status**: Complete  

---

## Table of Contents

1. [Overview](#overview)
2. [Authentication](#authentication)
3. [REST API Endpoints](#rest-api-endpoints)
4. [WebSocket Protocol](#websocket-protocol)
5. [Error Handling](#error-handling)
6. [Rate Limiting](#rate-limiting)
7. [Examples](#examples)

---

## Overview

The Private Chat Application provides a real-time messaging platform with:

- **REST API**: Account management, user search, conversation history
- **WebSocket Protocol**: Real-time message delivery, presence updates, typing indicators
- **Authentication**: JWT-based stateless authentication
- **Database**: SQLite (MVP) with PostgreSQL migration path

### Base URLs

- **Development**: `http://localhost:8080`
- **Production**: `https://chat.example.com`

### Content Type

All REST API requests and responses use `application/json`.

---

## Authentication

### JWT Token Structure

Tokens are issued on successful signup or login and are valid for 1 hour.

```json
{
  "sub": "user-550e8400-e29b-41d4-a716-446655440000",
  "aud": "chat-app",
  "iat": 1702657890,
  "exp": 1702661490,
  "scopes": ["send", "receive"]
}
```

### Using Tokens

**REST API**: Include in Authorization header
```
Authorization: Bearer <JWT_TOKEN>
```

**WebSocket**: Include in query parameter
```
ws://localhost:8080/socket?token=<JWT_TOKEN>
```

---

## REST API Endpoints

### 1. Health Check

**Endpoint**: `GET /health`  
**Auth**: None  
**Description**: Check server health status

**Response (200 OK)**:
```json
{
  "status": "healthy",
  "timestamp": 1702657890000,
  "uptime_seconds": 3600
}
```

**Use Case**: Monitoring, load balancer health checks

---

### 2. Create Account (Sign Up)

**Endpoint**: `POST /auth/signup`  
**Auth**: None  
**Description**: Create a new user account

**Request Body**:
```json
{
  "username": "alice",
  "password": "SecurePass123"
}
```

**Validation Rules**:
- `username`: 1-50 characters, alphanumeric + underscore, must be unique
- `password`: Minimum 8 characters, at least 1 uppercase, 1 lowercase, 1 digit

**Response (201 Created)**:
```json
{
  "userId": "user-550e8400-e29b-41d4-a716-446655440000",
  "username": "alice",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expiresIn": 3600
}
```

**Errors**:
- `409 Conflict`: Username already exists
- `400 Bad Request`: Invalid password format

---

### 3. Login

**Endpoint**: `POST /auth/login`  
**Auth**: None  
**Description**: Authenticate user and receive JWT token

**Request Body**:
```json
{
  "username": "alice",
  "password": "SecurePass123"
}
```

**Response (200 OK)**:
```json
{
  "userId": "user-550e8400-e29b-41d4-a716-446655440000",
  "username": "alice",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expiresIn": 3600
}
```

**Errors**:
- `401 Unauthorized`: Invalid credentials
- `404 Not Found`: Account deleted
- `429 Too Many Requests`: Rate limit exceeded (5 attempts per 15 minutes)

---

### 4. Refresh Token

**Endpoint**: `POST /auth/refresh`  
**Auth**: Bearer token  
**Description**: Refresh JWT token before expiration

**Response (200 OK)**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expiresIn": 3600
}
```

**Errors**:
- `401 Unauthorized`: Token invalid or expired

---

### 5. Get Current User

**Endpoint**: `GET /user/me`  
**Auth**: Bearer token  
**Description**: Get current user profile information

**Response (200 OK)**:
```json
{
  "userId": "user-550e8400-e29b-41d4-a716-446655440000",
  "username": "alice",
  "createdAt": 1702657890000,
  "isOnline": true,
  "lastSeenAt": 1702657890000
}
```

---

### 6. Search Users

**Endpoint**: `GET /users/search`  
**Auth**: Bearer token  
**Description**: Search for users by username prefix

**Query Parameters**:
- `q`: Search query (required, minimum 1 character)
- `limit`: Max results (default 10, max 50)

**Example**: `GET /users/search?q=ali&limit=10`

**Response (200 OK)**:
```json
{
  "results": [
    {
      "userId": "user-550e8400-e29b-41d4-a716-446655440000",
      "username": "alice",
      "isOnline": true
    },
    {
      "userId": "user-660e8400-e29b-41d4-a716-446655440001",
      "username": "alice2",
      "isOnline": false
    }
  ],
  "total": 2
}
```

**Errors**:
- `400 Bad Request`: Query too short (< 1 character)

---

### 7. Get Conversations List

**Endpoint**: `GET /conversations`  
**Auth**: Bearer token  
**Description**: Get list of user's conversations

**Query Parameters**:
- `limit`: Number of conversations (default 20, max 100)
- `offset`: Pagination offset (default 0)

**Response (200 OK)**:
```json
{
  "conversations": [
    {
      "conversationId": "conv-550e8400-e29b-41d4-a716-446655440000",
      "participantId": "user-456",
      "participantUsername": "bob",
      "lastMessageAt": 1702657900000,
      "messageCount": 125,
      "lastMessage": "See you tomorrow!",
      "participantIsOnline": true
    }
  ],
  "total": 3,
  "hasMore": false
}
```

---

### 8. Get Conversation Messages

**Endpoint**: `GET /conversations/{conversationId}/messages`  
**Auth**: Bearer token  
**Description**: Get message history for a conversation

**Query Parameters**:
- `limit`: Number of messages (default 50, max 100)
- `offset`: Pagination offset (default 0)

**Response (200 OK)**:
```json
{
  "conversationId": "conv-550e8400-e29b-41d4-a716-446655440000",
  "messages": [
    {
      "id": "msg-550e8400-e29b-41d4-a716-446655440000",
      "senderId": "user-123",
      "senderUsername": "alice",
      "recipientId": "user-456",
      "content": "Hello!",
      "createdAt": 1702657890000,
      "status": "delivered"
    }
  ],
  "total": 125,
  "hasMore": true
}
```

**Errors**:
- `404 Not Found`: Conversation doesn't exist
- `403 Forbidden`: User not a participant

---

### 9. Start Conversation

**Endpoint**: `POST /conversations/start`  
**Auth**: Bearer token  
**Description**: Start a new conversation or get existing one

**Request Body**:
```json
{
  "otherUserId": "user-456"
}
```

**Response (201 Created or 200 OK)**:
```json
{
  "conversationId": "conv-550e8400-e29b-41d4-a716-446655440000",
  "participantId": "user-456",
  "participantUsername": "bob",
  "participantIsOnline": true
}
```

**Errors**:
- `400 Bad Request`: Cannot start conversation with self
- `404 Not Found`: Other user doesn't exist

---

### 10. Search Conversation Messages

**Endpoint**: `GET /conversations/{conversationId}/search`  
**Auth**: Bearer token  
**Description**: Search messages within a conversation

**Query Parameters**:
- `q`: Search keyword (required)
- `limit`: Max results (default 50)

**Response (200 OK)**:
```json
{
  "conversationId": "conv-550e8400-e29b-41d4-a716-446655440000",
  "matches": [
    {
      "id": "msg-123",
      "senderId": "user-123",
      "senderUsername": "alice",
      "content": "Hello, how are you?",
      "createdAt": 1702657890000,
      "matchContext": "...how are you..."
    }
  ],
  "total": 1
}
```

---

### 11. Change Password

**Endpoint**: `POST /user/change-password`  
**Auth**: Bearer token  
**Description**: Change user password

**Request Body**:
```json
{
  "currentPassword": "OldPass123",
  "newPassword": "NewPass456"
}
```

**Response (200 OK)**:
```json
{
  "message": "Password updated successfully"
}
```

**Errors**:
- `401 Unauthorized`: Current password incorrect
- `400 Bad Request`: New password doesn't meet requirements

---

### 12. Delete Account

**Endpoint**: `DELETE /user/me`  
**Auth**: Bearer token  
**Description**: Soft-delete user account (anonymize messages)

**Request Body**:
```json
{
  "password": "SecurePass123"
}
```

**Response (204 No Content)**

**Effects**:
- Account marked as deleted
- All sent messages anonymized (display "Deleted User")
- Cannot log in again
- No reactivation possible

**Errors**:
- `401 Unauthorized`: Password incorrect

---

### 13. Server Status

**Endpoint**: `GET /status`  
**Auth**: None  
**Description**: Get server status and metrics (admin/monitoring)

**Response (200 OK)**:
```json
{
  "status": "running",
  "version": "1.0.0",
  "startedAt": 1702657890000,
  "uptimeSeconds": 3600,
  "database": {
    "type": "sqlite",
    "path": "/path/to/chat.db",
    "status": "connected"
  },
  "connections": {
    "activeWebsocket": 42,
    "totalUsers": 256
  }
}
```

---

## WebSocket Protocol

### Connection

**Endpoint**: `ws://localhost:8080/socket?token=<JWT_TOKEN>`

**Handshake**:
```http
GET /socket?token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9... HTTP/1.1
Host: localhost:8080
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: [base64 random]
Sec-WebSocket-Version: 13
Sec-WebSocket-Protocol: chat-v1
```

**Server Response (Success)**:
```http
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: [computed]
Sec-WebSocket-Protocol: chat-v1
```

---

### Message Structure

All WebSocket messages use this base format:

```json
{
  "id": "unique-identifier",
  "type": "message|typing|presence|ack|error|heartbeat",
  "timestamp": 1702657890000,
  "data": { /* type-specific fields */ }
}
```

---

### Message Types

#### 1. Text Message (Client → Server)

Send a message to another user.

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
- `id`: UUID v4, must be unique (idempotency)
- `recipientId`: Must be a valid user
- `content`: 1-5000 characters, UTF-8 valid

---

#### 2. Text Message (Server → Recipient)

Deliver message to recipient.

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

---

#### 3. Message Acknowledgement (Server → Sender)

Confirm message receipt by server.

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
- `sent`: Message stored, awaiting delivery
- `delivered`: Recipient received message
- `failed`: Permanent delivery failure

---

#### 4. Typing Indicator (Client → Server)

Notify recipient that user is typing.

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

**Behavior**:
- Send every 1 second while typing
- Send `isTyping: false` when stopped
- Ephemeral (not persisted)
- Dropped if recipient offline

---

#### 5. Typing Indicator (Server → Recipient)

Forward typing status to recipient.

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

---

#### 6. Presence Update (Server → Clients)

Notify conversation participants of online/offline status.

**User comes online**:
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

**User goes offline**:
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

**Privacy**: Only sent to users with active conversations (not broadcast globally)

---

#### 7. Heartbeat

Server sends PING every 25 seconds, expects PONG within 5 seconds.

**Handled automatically by WebSocket protocol (RFC 6455).**

---

#### 8. Error Response (Server → Client)

Server sends error for invalid requests.

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
- `INVALID_MESSAGE_LENGTH`: Content length violation
- `RECIPIENT_NOT_FOUND`: Recipient doesn't exist
- `RECIPIENT_DELETED`: Recipient account deleted
- `INVALID_JSON`: Malformed JSON
- `RATE_LIMIT_EXCEEDED`: Quota exceeded
- `UNAUTHORIZED`: Token expired
- `SERVER_ERROR`: Internal server error

---

### Connection Closure

WebSocket closes with reason codes:

| Code | Reason | Description |
|------|--------|-------------|
| 1000 | Normal Closure | User logged out |
| 1001 | Going Away | Server shutting down |
| 1002 | Protocol Error | Invalid message format |
| 1008 | Policy Violation | Token expired or rate limit exceeded |

---

## Error Handling

### Standard Error Response Format

All REST API errors return JSON:

```json
{
  "error": "ERROR_CODE",
  "message": "Human-readable error message",
  "details": {
    "field": "Additional context (optional)"
  },
  "timestamp": 1702657890000
}
```

### HTTP Status Codes

| Code | Meaning |
|------|---------|
| 200 | Success |
| 201 | Created |
| 204 | No Content |
| 400 | Bad Request (validation error) |
| 401 | Unauthorized (auth failed) |
| 403 | Forbidden (permission denied) |
| 404 | Not Found |
| 409 | Conflict (duplicate username) |
| 429 | Too Many Requests (rate limit) |
| 500 | Internal Server Error |

---

## Rate Limiting

### REST API

**Global**: 1000 requests/minute per IP  
**Auth endpoints**: 5 attempts/minute per username

**Response (429 Too Many Requests)**:
```json
{
  "error": "RATE_LIMITED",
  "message": "Too many requests; retry after 60 seconds",
  "retryAfter": 60
}
```

**Headers**:
```
Retry-After: 60
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 342
X-RateLimit-Reset: 1702657950
```

---

### WebSocket Messages

**Per-user token bucket**:
- Capacity: 100 messages per 60 seconds
- Burst: 5 messages per second
- Refill: ~1.67 tokens/second
- Enforcement: Connection closed after 3 violations

---

## Examples

### Example 1: Complete Sign Up Flow

```bash
# 1. Sign up
curl -X POST http://localhost:8080/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "password": "SecurePass123"
  }'

# Response:
# {
#   "userId": "user-550e8400-e29b-41d4-a716-446655440000",
#   "username": "alice",
#   "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
#   "expiresIn": 3600
# }

# 2. Save token
export TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 3. Get user profile
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/user/me
```

---

### Example 2: Search Users and Start Conversation

```bash
# 1. Search for users
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/users/search?q=bob&limit=10"

# Response:
# {
#   "results": [
#     {
#       "userId": "user-456",
#       "username": "bob",
#       "isOnline": true
#     }
#   ],
#   "total": 1
# }

# 2. Start conversation
curl -X POST http://localhost:8080/conversations/start \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "otherUserId": "user-456"
  }'

# Response:
# {
#   "conversationId": "conv-789",
#   "participantId": "user-456",
#   "participantUsername": "bob",
#   "participantIsOnline": true
# }
```

---

### Example 3: Send Message via WebSocket

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:8080/socket?token=' + token);

ws.onopen = () => {
  // Send message
  ws.send(JSON.stringify({
    id: crypto.randomUUID(),
    type: 'message',
    timestamp: Date.now(),
    data: {
      recipientId: 'user-456',
      content: 'Hello, Bob!'
    }
  }));
};

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  
  if (msg.type === 'ack') {
    console.log('Message sent:', msg.data.status);
  } else if (msg.type === 'message') {
    console.log('Received message:', msg.data.content);
  }
};
```

---

### Example 4: Get Conversation History

```bash
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/conversations/conv-789/messages?limit=50&offset=0"

# Response:
# {
#   "conversationId": "conv-789",
#   "messages": [
#     {
#       "id": "msg-1",
#       "senderId": "user-123",
#       "senderUsername": "alice",
#       "recipientId": "user-456",
#       "content": "Hello, Bob!",
#       "createdAt": 1702657890000,
#       "status": "delivered"
#     }
#   ],
#   "total": 1,
#   "hasMore": false
# }
```

---

## Additional Resources

- **WebSocket Protocol Details**: `specs/001-private-chat/contracts/websocket-protocol.md`
- **Server Contract**: `specs/001-private-chat/contracts/server-contract.md`
- **Message Schema**: `specs/001-private-chat/contracts/message-envelope-schema.json`
- **Data Model**: `specs/001-private-chat/data-model.md`
- **Quick Start Guide**: `specs/001-private-chat/quickstart.md`

---

## Versioning

**Current Version**: 1.0.0

**Breaking Changes Policy**:
- Major version bump on breaking changes
- Backward compatibility maintained for 2 releases
- Clients must support up to 2 versions behind current

---

**Generated**: 2025-12-16  
**Maintained By**: Development Team
