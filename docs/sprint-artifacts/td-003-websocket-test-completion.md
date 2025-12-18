# Story TD-003: Complete WebSocket Handshake Test Placeholders

**Status:** 🆕 Backlog  
**Priority:** P3 (Low - Test Completeness)  
**Epic:** Technical Debt - Test Quality  
**Owner:** TBD  
**Created:** 2025-12-17

---

## 📋 Story

**As a** test engineer  
**I want** to complete the placeholder WebSocket handshake tests  
**So that** WebSocket authentication is fully validated with real connection tests

---

## 🎯 Acceptance Criteria

### AC1: Implement test_websocket_handshake_without_token (T060-002) ✓
- **Current:** Placeholder with `println!()` statement
- **Implement:** Spawn test server, attempt connection without token, verify rejection
- **Expected:** HTTP 400 Bad Request or WebSocket upgrade rejection
- **Test:** Connection attempt fails as expected

### AC2: Implement test_websocket_handshake_with_invalid_token (T060-003) ✓
- **Current:** Placeholder with `println!()` statement
- **Implement:** Spawn test server, attempt connection with malformed token
- **Expected:** HTTP 401 Unauthorized
- **Test:** Token validation rejects malformed tokens

### AC3: Implement test_websocket_handshake_with_expired_token (T060-004) ✓
- **Current:** Placeholder with `println!()` statement
- **Implement:** Spawn test server, generate expired token, attempt connection
- **Expected:** HTTP 401 Unauthorized with expiration error
- **Test:** Expired tokens are rejected

### AC 4: Implement test_websocket_handshake_with_wrong_secret (T060-005) ✓
- **Current:** Placeholder with `println!()` statement  
**Implement:** Spawn test server with correct secret, connect with token signed by wrong secret
- **Expected:** HTTP 401 Unauthorized with signature validation error
- **Test:** Tokens signed with wrong secret are rejected

---

## 🔨 Tasks & Subtasks

### Task 1: Create Test Server Harness
- [ ] Implement `spawn_test_server()` helper function
- [ ] Start server on random available port
- [ ] Return server handle and port number
- [ ] Ensure clean shutdown after test

### Task 2: Implement T060-002 (No Token)
- [ ] Remove `println!()` placeholder
- [ ] Spawn test server
- [ ] Attempt WebSocket connection without token parameter
- [ ] Assert connection is rejected (400/401)
- [ ] Test: Verify rejection behavior

### Task 3: Implement T060-003 (Invalid Token)
- [ ] Remove `println!()` placeholder
- [ ] Spawn test server
- [ ] Generate malformed token string
- [ ] Attempt WebSocket connection
- [ ] Assert 401 Unauthorized
- [ ] Test: Verify token validation

### Task 4: Implement T060-004 (Expired Token)
- [ ] Remove `println!()` placeholder
- [ ] Spawn test server
- [ ] Use existing `generate_ex pired_token()` function
- [ ] Attempt WebSocket connection
- [ ] Assert 401 Unauthorized with expiration message
- [ ] Test: Verify expiration check

### Task 5: Implement T060-005 (Wrong Secret)
- [ ] Remove `println!()` placeholder
- [ ] Spawn test server with secret "correct-secret"
- [ ] Generate token with "wrong-secret"
- [ ] Attempt WebSocket connection
- [ ] Assert 401 Unauthorized
- [ ] Test: Verify signature validation

### Task 6: Verify All WebSocket Tests Pass
- [ ] Run `cargo test websocket_handshake`
- [ ] All 8 tests passing (T060-001 through T060-008)
- [ ] No placeholders remaining

---

## 📊 Definition of Done Checklist

- [ ] **AC1:** T060-002 implemented with real connection test
- [ ] **AC2:** T060-003 implemented with invalid token test
- [ ] **AC3:** T060-004 implemented with expired token test
- [ ] **AC4:** T060-005 implemented with wrong secret test
- [ ] **Test Harness:** `spawn_test_server()` helper created
- [ ] **All Tests Pass:** 8/8 WebSocket handshake tests passing
- [ ] **No Placeholders:** All `println!()` placeholders removed
- [ ] **Code Review:** Implementation reviewed and approved

---

## 📈 Estimation

**Size:** M (3-4 hours)  
**Complexity:** Medium (requires test server infrastructure)  
**Risk:** Medium (WebSocket + server spawning complexity)  
**Time Breakdown:**
- Test server harness: 1.5 hours
- Implement 4 test cases: 1.5 hours
- Testing and debugging: 1 hour
- **Total: 4 hours**

---

## 📁 File References

### Files to Modify
- `tests/integration/websocket_handshake_test.rs` (lines 184-253)

### Files to Create
- `tests/helpers/server_harness.rs` (test server spawning helper)

### Reference Files
- `src/backend/server.rs` - Server implementation to spawn
- `tests/integration/websocket_handshake_test.rs` - Existing test structure
- `test-quality-review.md` - Original issue identification

---

## 💡 Implementation Notes

### Test Server Harness Pattern
```rust
// tests/helpers/server_harness.rs
pub async fn spawn_test_server() -> (ServerHandle, u16) {
    // 1. Create in-memory test database
    let pool = setup_test_db().await;
    
    // 2. Create server with test configuration
    let routes = create_routes(pool);
    
    // 3. Bind to random available port
    let port = get_available_port();
    
    // 4. Spawn server in background task
    let server = tokio::spawn(warp::serve(routes).run(([127, 0, 0, 1], port)));
    
    (ServerHandle { task: server }, port)
}
```

### WebSocket Connection Test Pattern
```rust
#[tokio::test]
async fn test_websocket_handshake_without_token() {
    let (server, port) = spawn_test_server().await;
    
    let ws_url = format!("ws://127.0.0.1:{}/socket", port);
    let result = tokio_tungstenite::connect_async(ws_url).await;
    
    assert!(result.is_err(), "Should reject connection without token");
    
    server.shutdown().await;
}
```

---

## 🏷️ Labels & Metadata

- **Epic:** Technical Debt - Test Quality
- **Type:** Test Implementation
- **Priority:** P3 (Low - can be deferred)
- **Complexity:** Medium
- **Risk:** Medium
- **Tech Stack:** Rust, Tokio, Tungstenite, Warp
- **Story Points:** 3 (M = 3-4 hours)
