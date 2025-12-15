# Server REST API Contract

**Version**: 1.0  
**Date**: 2025-12-15  
**Status**: Approved  

---

## Overview

The server exposes REST HTTP endpoints for account management and health checks. Real-time messaging happens over WebSocket (see websocket-protocol.md).

- **Base URL**: `http://localhost:8080` (dev) or `https://chat.example.com` (prod)
- **Content-Type**: `application/json`
- **Error Format**: Standard JSON error responses (see below)

---

## Authentication

### JWT Token Format

Token issued on successful signup/login. Valid for 1 hour.

```json
{
  "sub": "user-123",
  "aud": "chat-app",
  "iat": 1702657890,
  "exp": 1702661490,
  "scopes": ["send", "receive"]
}
```

Token included in WebSocket handshake query parameter: `?token=<JWT>`

---

## Endpoints

### 1. Health Check

**Method**: `GET`  
**Path**: `/health`  
**Auth**: None  

**Response (200 OK)**:
```json
{
  "status": "healthy",
  "timestamp": 1702657890000,
  "uptime_seconds": 3600
}
```

**Use Case**: Monitoring, load balancer health checks.

---

### 2. Create Account (Sign Up)

**Method**: `POST`  
**Path**: `/auth/signup`  
**Auth**: None  
**Body**:
```json
{
  "username": "alice",
  "password": "SecurePass123"
}
```

**Validation**:
- `username`: 1-50 alphanumeric + underscore; must be unique
- `password`: Min 8 chars; at least 1 uppercase, 1 lowercase, 1 digit

**Response (201 Created)**:
```json
{
  "userId": "user-550e8400-e29b-41d4-a716-446655440000",
  "username": "alice",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expiresIn": 3600
}
```

**Error (409 Conflict)** - Username already exists:
```json
{
  "error": "USERNAME_TAKEN",
  "message": "Username 'alice' is already taken"
}
```

**Error (400 Bad Request)** - Invalid password:
```json
{
  "error": "INVALID_PASSWORD",
  "message": "Password must be at least 8 characters with uppercase, lowercase, and digit"
}
```

---

### 3. Login

**Method**: `POST`  
**Path**: `/auth/login`  
**Auth**: None  
**Body**:
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

**Error (401 Unauthorized)** - Invalid credentials:
```json
{
  "error": "INVALID_CREDENTIALS",
  "message": "Username or password is incorrect"
}
```

**Error (404 Not Found)** - Account deleted:
```json
{
  "error": "ACCOUNT_DELETED",
  "message": "This account has been deleted and cannot be reactivated"
}
```

---

### 4. Get Current User

**Method**: `GET`  
**Path**: `/user/me`  
**Auth**: Bearer token (HTTP Authorization header)  
**Headers**:
```
Authorization: Bearer <JWT>
```

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

**Error (401 Unauthorized)** - Token invalid/expired:
```json
{
  "error": "UNAUTHORIZED",
  "message": "Token invalid or expired"
}
```

---

### 5. Search Users

**Method**: `GET`  
**Path**: `/users/search?q=ali`  
**Auth**: Bearer token  
**Query Params**:
- `q`: Search query (username prefix match; case-insensitive)
- `limit`: Max results (default 10, max 50)

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

**Error (400 Bad Request)** - Query too short:
```json
{
  "error": "INVALID_QUERY",
  "message": "Search query must be at least 1 character"
}
```

---

### 6. Get Conversation (Message History)

**Method**: `GET`  
**Path**: `/conversations/{conversationId}/messages?limit=50&offset=0`  
**Auth**: Bearer token  
**Query Params**:
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
    },
    {
      "id": "msg-550e8400-e29b-41d4-a716-446655440001",
      "senderId": "user-456",
      "senderUsername": "bob",
      "recipientId": "user-123",
      "content": "Hi Alice!",
      "createdAt": 1702657895000,
      "status": "delivered"
    }
  ],
  "total": 125,
  "hasMore": true
}
```

**Error (404 Not Found)** - Conversation doesn't exist:
```json
{
  "error": "CONVERSATION_NOT_FOUND",
  "message": "Conversation does not exist"
}
```

**Error (403 Forbidden)** - User not participant:
```json
{
  "error": "UNAUTHORIZED",
  "message": "You are not a participant in this conversation"
}
```

---

### 7. Get Conversation List

**Method**: `GET`  
**Path**: `/conversations?limit=20&offset=0`  
**Auth**: Bearer token  
**Query Params**:
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

### 8. Delete Account

**Method**: `DELETE`  
**Path**: `/user/me`  
**Auth**: Bearer token  
**Body**:
```json
{
  "password": "SecurePass123"
}
```

**Response (204 No Content)**:
- Account marked as deleted (soft delete)
- All sent messages anonymized (sender → "Deleted User")
- Cannot log in again
- No reactivation possible

**Error (401 Unauthorized)** - Invalid password:
```json
{
  "error": "INVALID_PASSWORD",
  "message": "Password incorrect"
}
```

---

### 9. Refresh Token

**Method**: `POST`  
**Path**: `/auth/refresh`  
**Auth**: Bearer token  

**Response (200 OK)**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expiresIn": 3600
}
```

**Error (401 Unauthorized)** - Token invalid:
```json
{
  "error": "UNAUTHORIZED",
  "message": "Token invalid or expired"
}
```

---

### 10. Server Status & Configuration (CLI Only)

**Method**: `GET`  
**Path**: `/status`  
**Auth**: Optional (no-op if provided)  

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

**Use Case**: Diagnostics, monitoring dashboards.

---

## Standard Error Response Format

All errors return JSON with consistent structure:

```json
{
  "error": "ERROR_CODE",
  "message": "Human-readable error message",
  "details": {
    "field": "description of field error (if applicable)"
  },
  "timestamp": 1702657890000
}
```

**HTTP Status Codes**:

| Code | Meaning |
|------|---------|
| 200 | Success |
| 201 | Created (signup) |
| 204 | No Content (deletion) |
| 400 | Bad Request (validation error) |
| 401 | Unauthorized (auth failed) |
| 403 | Forbidden (permission denied) |
| 404 | Not Found |
| 409 | Conflict (duplicate username) |
| 429 | Too Many Requests (rate limit) |
| 500 | Internal Server Error |

---

## Rate Limiting (HTTP Level)

**Global limit**: 1000 requests per minute per IP  
**Auth endpoints**: 5 attempts per minute per username (prevent brute force)

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

## CORS Headers

Responses include CORS headers for browser clients:

```
Access-Control-Allow-Origin: * (dev) or specific domain (prod)
Access-Control-Allow-Methods: GET, POST, DELETE, OPTIONS
Access-Control-Allow-Headers: Authorization, Content-Type
Access-Control-Max-Age: 86400
```

---

## Request/Response Examples

### Example 1: Sign Up Flow

**Request**:
```bash
curl -X POST http://localhost:8080/auth/signup \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "password": "SecurePass123"}'
```

**Response**:
```json
{
  "userId": "user-550e8400-e29b-41d4-a716-446655440000",
  "username": "alice",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyLTEyMyIsImF1ZCI6ImNoYXQtYXBwIiwiaWF0IjoxNzAyNjU3ODkwLCJleHAiOjE3MDI2NjE0OTB9.6N-7w_WvPqCp8QkZqZ2ZzZzZzZzZzZzZzZzZzZzZzZz",
  "expiresIn": 3600
}
```

### Example 2: Get Conversation History

**Request**:
```bash
curl -X GET "http://localhost:8080/conversations/conv-123/messages?limit=10" \
  -H "Authorization: Bearer $TOKEN"
```

**Response**:
```json
{
  "conversationId": "conv-123",
  "messages": [
    {
      "id": "msg-1",
      "senderId": "user-alice",
      "senderUsername": "alice",
      "recipientId": "user-bob",
      "content": "Hey Bob!",
      "createdAt": 1702657890000,
      "status": "delivered"
    }
  ],
  "total": 1,
  "hasMore": false
}
```

---

## Test Scenarios

### Happy Path: Sign Up & Login

1. POST /auth/signup → 201 Created (token issued)
2. Use token for WebSocket connection → 101 Switching Protocols
3. ✅ Test: User can send/receive messages

### Duplicate Username

1. POST /auth/signup with username "alice"
2. POST /auth/signup with username "alice" again
3. ✅ Test: 409 Conflict error returned

### Invalid Password Format

1. POST /auth/signup with password "short"
2. ✅ Test: 400 Bad Request; message specifies requirements

### Token Expiration

1. Issue token with 1-second expiration (for testing)
2. Wait 2 seconds
3. Use token for WebSocket connection
4. ✅ Test: 401 Unauthorized returned

### Rate Limiting

1. Send 6 signup requests with same username within 60 seconds
2. ✅ Test: 5th request succeeds, 6th returns 429 Too Many Requests

---

## Versioning Strategy

**Current Version**: 1.0  
**Breaking Changes Policy**: 
- Major version bump on breaking changes (e.g., endpoint removal, response format change)
- Backward compatibility maintained for 2 releases
- Client must support up to 2 versions behind current

---

## Next Steps

✅ Server REST API contract complete.  
→ Generate message schema (JSON serialization format)  
→ Generate client UI contract (Slint bindings)
