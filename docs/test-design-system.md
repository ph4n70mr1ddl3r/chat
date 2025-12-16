# System-Level Test Design & Testability Review

**Date:** 2025-12-17  
**Author:** Riddler (TEA - Test Architect)  
**Project:** chat - Modernized Desktop Chat Application  
**Phase:** 3 - Solutioning Gate Check  
**Status:** Complete  
**Mode:** System-Level Testability Review

---

## Executive Summary

This document presents a comprehensive testability assessment of the chat application architecture in preparation for the implementation phase. The architecture demonstrates **STRONG TESTABILITY** across all three critical dimensions (Controllability, Observability, Reliability), with clear patterns for quality validation.

**Key Findings:**

- ✅ **Controllability**: PASS - Architecture supports comprehensive test control through dependency injection, mocking, and stateless design
- ✅ **Observability**: PASS - Extensive logging, metrics, and trace integration enable thorough validation
- ✅ **Reliability**: PASS - Component isolation, async/await patterns, and database transaction management support robust testing

**Test Strategy:**
- **Unit Tests**: 50% (fast feedback on business logic, error handling)
- **Integration Tests**: 35% (service boundaries, database operations, WebSocket contract)
- **E2E Tests**: 15% (critical user journeys, end-to-end workflows)

**Architecturally Significant Requirements (ASRs):** 12 identified, all PASS
**Critical Testability Concerns:** None identified
**Gate Recommendation:** ✅ **PASS** - Architecture is ready for implementation with strong testing foundation

---

## Section 1: Testability Assessment

### 1.1 Controllability (Can we control system state for testing?)

**Overall Rating: ✅ PASS**

#### 1.1.1 Test State Seeding and Database Reset

**Finding**: EXCELLENT - Backend provides comprehensive control over test state

**Evidence from Architecture:**

1. **Dependency Injection Pattern** (Architecture Decision #3)
   - Handler layer receives injected database connections, repositories, and services
   - Enables test instances to use in-memory SQLite or transaction rollback strategies
   - Example: `CreateMessageHandler` receives `DatabaseService` → testable with mock db

2. **Stateless Service Design** (Architecture Pattern #2)
   - Services accept input parameters without side effects
   - Each operation is functionally isolated and repeatable
   - Example: `MessageService::send_message(msg, sender_id, recipient_id)` returns Result, no state coupling

3. **Factory Pattern for Test Data**
   - Middleware and handlers accept configuration objects
   - Can inject test configurations (JWT secrets, rate limits, connection pools)
   - Database migrations enable clean slate between tests

**Testability Score:** 9/10
**Blocker Issues:** None
**Implementation Guidance:**
- Use SQLite transactions with automatic rollback for integration tests
- Inject test configurations via constructor parameters
- Pre-populate test data using SQL migrations before each test suite

---

#### 1.1.2 External Dependency Mockability

**Finding**: EXCELLENT - Clear boundaries enable comprehensive mocking

**Evidence from Architecture:**

1. **WebSocket Protocol Abstraction** (Architecture Pattern #5)
   - `TungsteniteWebSocket` interface enables mock implementations
   - Test socket can simulate connection failures, delays, malformed messages
   - Example: `MockWebSocket` can return canned messages or trigger error scenarios

2. **Repository Pattern** (Pattern #1)
   - `UserRepository`, `MessageRepository`, `ConversationRepository` are traits
   - Can implement mock versions returning test data
   - Enables testing handlers without database

3. **Middleware Composition** (Pattern #3)
   - Auth middleware injects JWT validator
   - Test version can inject mock validator accepting any token
   - Enables testing authorization logic in isolation

**Testability Score:** 9/10
**Blocker Issues:** None
**Implementation Guidance:**
- Create `MockRepository` implementations for each repository trait
- Inject mock WebSocket for connection testing
- Use `mockito` or `wiremock` for HTTP service mocks

---

#### 1.1.3 Error Condition Triggering

**Finding**: STRONG - Architecture supports error scenario testing

**Evidence from Architecture:**

1. **Error Handling Pattern** (Pattern #6)
   - All handlers return `Result<Response, ServiceError>`
   - Services define complete error variants (AuthFailed, NotFound, Conflict, etc.)
   - Test can trigger each error path by controlled repository responses

2. **Network Resilience** (Architecture Decision #7)
   - Connection manager implements retry logic with exponential backoff
   - Can be tested by simulating:
     - Connection timeouts → mock WebSocket returns Timeout error
     - Partial messages → mock WebSocket returns incomplete frames
     - Server disconnect → mock WebSocket returns ConnectionClosed

3. **Message Validation** (Decision #2)
   - Validator accepts message content and performs format checks
   - Can test invalid input: oversized messages, malformed JSON, missing fields
   - Injection point enables replacing validator with one that rejects predictably

**Testability Score:** 8/10
**Blocker Issues:** None
**Implementation Guidance:**
- Create test harness that can inject specific error responses
- Use deliberate delays/failures in mock services
- Validate error propagation through entire stack

---

### 1.2 Observability (Can we inspect system state and validate outcomes?)

**Overall Rating: ✅ PASS**

#### 1.2.1 System State Inspection

**Finding**: EXCELLENT - Architecture provides multiple inspection points

**Evidence from Architecture:**

1. **Comprehensive Logging** (Decision #8)
   - All handlers log request/response with structured JSON
   - Services log state transitions (message created, user connected, conversation updated)
   - Error conditions include full context and stack traces
   - Test can capture logs and verify expected events occurred

2. **Database Query Transparency**
   - SQLite connection can log all SQL statements
   - Can verify database was modified correctly (message inserted, presence updated)
   - Transaction boundaries clear → can inspect state at each step

3. **WebSocket Message Flow**
   - Each message transmitted is logged with timestamp and content
   - Presence updates, message delivery confirmations logged
   - Test harness can observe message ordering and timing

**Testability Score:** 10/10
**Blocker Issues:** None
**Implementation Guidance:**
- Enable structured logging in test environments
- Capture logs for assertion verification (e.g., "should contain log entry for message creation")
- Use database snapshots to verify state changes

---

#### 1.2.2 Test Result Determinism

**Finding**: STRONG - Architecture minimizes non-determinism

**Evidence from Architecture:**

1. **Stateless Request Handling** (Pattern #2)
   - Requests produce deterministic responses given same input
   - No global state or thread-local variables
   - Clock-dependent tests use injected time source

2. **WebSocket Message Ordering** (Pattern #5)
   - Tokio runtime guarantees message delivery order per connection
   - Test harness can control delivery sequence
   - Presence updates maintain causal ordering

3. **Race Condition Prevention** (Architecture Decision #5)
   - Presence updates use `Arc<Mutex<>>` for thread-safe state
   - Conversation sync uses database transactions
   - Multi-message handling is serialized per conversation

**Testability Score:** 9/10
**Blocker Issues:** None
**Concern**: Timestamp generation could be non-deterministic (partially mitigated by injection)
**Implementation Guidance:**
- Inject system clock for timestamp generation in tests
- Use fixed seed for any randomization (e.g., message IDs)
- Control async task execution order using `tokio::task` spawning

---

#### 1.2.3 NFR Validation Capability

**Finding**: EXCELLENT - Architecture enables comprehensive NFR testing

**Evidence from Architecture:**

1. **Performance Metrics** (Decision #4)
   - Handlers measure request processing time
   - WebSocket latency tracked for message delivery
   - Memory usage observable via Rust instrumentation

2. **Security Validation** (Decision #6)
   - All API endpoints protected by JWT validation
   - Authorization logic in middleware → can inspect tokens
   - Password hashing uses bcrypt with deterministic rounds → reproducible in tests

3. **Reliability Metrics**
   - Connection success/failure rates logged
   - Message delivery confirmation tracked
   - Offline message queue size observable

**Testability Score:** 10/10
**Blocker Issues:** None
**Implementation Guidance:**
- Measure handler latency using `std::time::Instant`
- Validate JWT tokens programmatically in tests
- Verify bcrypt hashes using `bcrypt::verify`

---

### 1.3 Reliability (Can tests run in isolation and reproduce failures?)

**Overall Rating: ✅ PASS**

#### 1.3.1 Test Isolation

**Finding**: EXCELLENT - Architecture naturally supports isolated testing

**Evidence from Architecture:**

1. **Component Isolation** (Pattern #1 - Repository Pattern)
   - Each repository is a trait with test implementation
   - Tests don't share database connections
   - Each test instance gets its own service graph

2. **Stateless Design** (Pattern #2)
   - No static variables or globals
   - Each request creates fresh context
   - Tests can run in parallel without interference

3. **Resource Cleanup** (Pattern #4)
   - Database connections explicitly closed in Drop trait
   - WebSocket connections close on disconnect
   - Test framework can ensure cleanup between tests

**Testability Score:** 10/10
**Blocker Issues:** None
**Implementation Guidance:**
- Use Rust's test framework `#[test]` with separate test modules
- Each test spawns isolated Tokio runtime
- Use test-specific databases (separate SQLite files per test)

---

#### 1.3.2 Failure Reproducibility

**Finding**: STRONG - Architecture supports failure reproduction

**Evidence from Architecture:**

1. **Deterministic Error Handling** (Pattern #6)
   - Errors include full context (user ID, message ID, timestamp)
   - Stack traces captured in production logs
   - Test can reproduce exact sequence that triggered failure

2. **Message Replay Capability**
   - All messages logged with exact timestamp and sender
   - Can replay messages in test to reproduce state
   - Presence state can be rebuilt from logs

3. **Network Condition Simulation**
   - Mock WebSocket can reproduce exact timing delays
   - Can simulate packet loss patterns
   - Connection state transitions reproducible

**Testability Score:** 9/10
**Blocker Issues:** None
**Implementation Guidance:**
- Implement HAR (HTTP Archive) format recording for WebSocket traffic
- Store test failure logs with full message history
- Create "failure reproduction" test cases from production logs

---

#### 1.3.3 Parallel Test Safety

**Finding**: EXCELLENT - Architecture supports parallel test execution

**Evidence from Architecture:**

1. **Stateless Services** (Pattern #2)
   - No shared state between requests
   - Each test thread gets independent service instances
   - Database transactions provide ACID isolation

2. **Async/Await Isolation** (Pattern #2)
   - Tokio runtime per test
   - Channel communication doesn't leak across tests
   - Task spawning scoped to test runtime

3. **Resource Allocation**
   - Database connections per test instance
   - Unique port allocation for test servers
   - No global resource contention

**Testability Score:** 10/10
**Blocker Issues:** None
**Implementation Guidance:**
- Use `#[tokio::test]` for async test functions
- Allocate unique database files per test
- Use port 0 (OS-assigned) for test server binding

---

## Section 2: Architecturally Significant Requirements (ASRs)

### 2.1 ASR Analysis and Risk Scoring

**Methodology**: Each ASR is scored using Probability (1-3) × Impact (1-3) = Score (1-9)
- Scores 1-3: Monitor (low risk)
- Scores 4-5: Document (manageable risk)
- Scores 6-8: Mitigate (high priority)
- Scores 9: Block (critical - must resolve)

---

### ASR-001: Real-Time Message Delivery (<2 sec end-to-end)

**Source**: PRD NFR2-3 (Performance)  
**Architecture Pattern**: WebSocket (Pattern #5), Message Queue (Pattern #7)  
**Testability Risk**: Medium

| Factor | Assessment |
|--------|------------|
| **Probability (p)** | 2 - Possible (network delays, high load) |
| **Impact (i)** | 3 - Critical (degrades core UX) |
| **Risk Score** | 6 - HIGH PRIORITY |
| **Testability** | ✅ Observable - can measure E2E latency with mock delays |

**Mitigation Strategy:**
- Load test with simulated network latency (100ms-500ms)
- Measure WebSocket round-trip time per message
- Monitor queue depth under load
- P0 Test: "Send message with 100ms network latency - verify delivery within 2s"

**Validation**: Load testing with k6, latency measurement in handlers

---

### ASR-002: Conversation Switching (<100ms UI response)

**Source**: PRD NFR2-2a (Performance)  
**Architecture Pattern**: Repository Pattern (Pattern #1), Caching (Decision #4)  
**Testability Risk**: Low

| Factor | Assessment |
|--------|------------|
| **Probability (p)** | 1 - Unlikely (simple query operation) |
| **Impact (i)** | 2 - Degraded (noticeable lag) |
| **Risk Score** | 2 - MONITOR |
| **Testability** | ✅ Measurable - can instrument query execution time |

**Mitigation Strategy:**
- Index conversations by user_id for fast lookup
- Cache recent conversations in memory
- Measure query time with mock database
- P1 Test: "Retrieve 200 conversations - verify under 50ms"

**Validation**: Database performance profiling

---

### ASR-003: 60 FPS UI Rendering

**Source**: PRD NFR2-1 (Performance)  
**Architecture Pattern**: Slint Rendering (external)  
**Testability Risk**: Medium (UI framework responsibility)

| Factor | Assessment |
|--------|------------|
| **Probability (p)** | 2 - Possible (backend delays impact frontend) |
| **Impact (i)** | 2 - Degraded (noticeable jank) |
| **Risk Score** | 4 - DOCUMENT |
| **Testability** | ✅ Indirectly testable - verify backend doesn't block rendering |

**Mitigation Strategy:**
- Backend handlers complete within 16ms (for 60 FPS)
- Slint rendering tests verify message injection doesn't stall
- Async handlers prevent UI thread blocking
- P1 Test: "Message update completes within 16ms"

**Validation**: Slint performance profiling, frame rate monitoring

---

### ASR-004: JWT Authentication (Secure Token Validation)

**Source**: PRD NFR1-3 (Security)  
**Architecture Pattern**: Middleware (Pattern #3), JWT Validation (Decision #6)  
**Testability Risk**: Low

| Factor | Assessment |
|--------|------------|
| **Probability (p)** | 1 - Unlikely (proven JWT library) |
| **Impact (i)** | 3 - Critical (security vulnerability) |
| **Risk Score** | 3 - DOCUMENT |
| **Testability** | ✅ High - JWT validation is deterministic and injectable |

**Mitigation Strategy:**
- Validate token signature with known key
- Reject expired tokens (test with past timestamp)
- Reject tampered tokens (modify payload)
- P0 Test: "Reject request with invalid JWT token"
- P0 Test: "Reject request with expired token"
- P0 Test: "Reject request with modified token payload"

**Validation**: JWT unit tests, token inspection

---

### ASR-005: Bcrypt Password Hashing (Secure Password Storage)

**Source**: PRD NFR1-4 (Security)  
**Architecture Pattern**: Password Hashing (Decision #6)  
**Testability Risk**: Low

| Factor | Assessment |
|--------|------------|
| **Probability (p)** | 1 - Unlikely (proven library) |
| **Impact (i)** | 3 - Critical (credential compromise) |
| **Risk Score** | 3 - DOCUMENT |
| **Testability** | ✅ High - deterministic with fixed cost factor |

**Mitigation Strategy:**
- Use bcrypt cost factor 4 in tests (fast, deterministic)
- Verify hash matches original password
- Reject incorrect passwords
- P0 Test: "Hash password and verify match"
- P0 Test: "Reject password that doesn't match hash"

**Validation**: Bcrypt unit tests with fixed seeds

---

### ASR-006: SQLite MVP → PostgreSQL Production Migration

**Source**: PRD Scalability (NFR3-1, NFR3-2)  
**Architecture Pattern**: Repository Abstraction (Pattern #1)  
**Testability Risk**: Medium

| Factor | Assessment |
|--------|------------|
| **Probability (p)** | 2 - Possible (DB-specific behavior differences) |
| **Impact (i)** | 2 - Degraded (data consistency issues) |
| **Risk Score** | 4 - DOCUMENT |
| **Testability** | ✅ Observable - repository abstraction enables alternate implementations |

**Mitigation Strategy:**
- Repository tests use in-memory SQLite
- Integration tests verify SQL compatibility
- Migration scripts tested with data samples
- P1 Test: "Create user in SQLite and PostgreSQL - verify identical results"

**Validation**: Repository abstraction tests, migration testing

---

### ASR-007: Offline Message Queuing (Connection Resilience)

**Source**: PRD NFR5-2 (Reliability)  
**Architecture Pattern**: Message Queue (Pattern #7), Retry Logic (Decision #7)  
**Testability Risk**: Medium

| Factor | Assessment |
|--------|------------|
| **Probability (p)** | 2 - Possible (network disconnects happen) |
| **Impact (i)** | 2 - Degraded (message loss) |
| **Risk Score** | 4 - DOCUMENT |
| **Testability** | ✅ Observable - can simulate disconnection and verify queue |

**Mitigation Strategy:**
- Queue persists messages to local SQLite when offline
- Reconnection triggers queue flush
- Test verifies message order preservation
- P1 Test: "Simulate disconnect - verify messages queued"
- P1 Test: "Reconnect - verify queued messages sent in order"

**Validation**: Queue integration tests, replay verification

---

### ASR-008: Multi-Conversation Sync (State Consistency)

**Source**: PRD FR33-FR40 (Multi-Conversation Management)  
**Architecture Pattern**: Conversation Manager (Pattern #4), Database Transactions (Decision #5)  
**Testability Risk**: High

| Factor | Assessment |
|--------|=========|
| **Probability (p)** | 2 - Possible (race conditions in high-load scenarios) |
| **Impact (i)** | 3 - Critical (message loss, duplicate messages) |
| **Risk Score** | 6 - HIGH PRIORITY |
| **Testability** | ✅ Observable - can inject concurrent requests and verify consistency |

**Mitigation Strategy:**
- Use database transactions for atomic updates
- Test concurrent message delivery to same conversation
- Verify no message duplication
- Verify no message loss
- P0 Test: "Send 100 concurrent messages to same conversation - verify all received, no duplicates"
- P0 Test: "Close connection mid-transaction - verify rollback, no partial state"

**Validation**: Concurrent stress testing, transaction verification

---

### ASR-009: Presence Awareness (Real-Time Status)

**Source**: PRD NFR2-6d (Performance - Presence throttling)  
**Architecture Pattern**: Presence Manager (Pattern #4), Arc<Mutex> (Decision #5)  
**Testability Risk**: Medium

| Factor | Assessment |
|--------|------------|
| **Probability (p)** | 2 - Possible (timing-dependent behavior) |
| **Impact (i)** | 1 - Minor (cosmetic if outdated) |
| **Risk Score** | 2 - MONITOR |
| **Testability** | ✅ Observable - can control clock and verify throttling |

**Mitigation Strategy:**
- Presence updates throttled to prevent spam
- Test verifies updates batched correctly
- Monitor Arc<Mutex> contention under load
- P1 Test: "Send 1000 presence updates in 1s - verify throttled to ≤100 updates"

**Validation**: Clock injection tests, load testing

---

### ASR-010: WCAG AA Accessibility Compliance

**Source**: PRD NFR4-10 (Accessibility)  
**Architecture Pattern**: Keyboard Navigation (frontend), Clean API (backend)  
**Testability Risk**: Low (backend supports, frontend verifies)

| Factor | Assessment |
|--------|------------|
| **Probability (p)** | 1 - Unlikely (accessibility library responsibility) |
| **Impact (i)** | 2 - Degraded (excludes users) |
| **Risk Score** | 2 - MONITOR |
| **Testability** | ✅ Partial - backend doesn't implement UI accessibility, Slint does |

**Mitigation Strategy:**
- Backend provides structured responses (no unescaped content)
- API returns semantic data (clear status codes, descriptive errors)
- Frontend (Slint) implements keyboard navigation
- P1 Test: "Verify all API error messages are descriptive and actionable"

**Validation**: Accessibility audit tools, manual testing

---

### ASR-011: Rate Limiting (API Protection)

**Source**: PRD NFR1-5 (Security - API security)  
**Architecture Pattern**: Middleware (Pattern #3), Rate Limit Handler (Decision #6)  
**Testability Risk**: Medium

| Factor | Assessment |
|--------|=========|
| **Probability (p)** | 1 - Unlikely (if properly configured) |
| **Impact (i)** | 2 - Degraded (DoS vulnerability) |
| **Risk Score** | 2 - MONITOR |
| **Testability** | ✅ Observable - can send rapid requests and verify throttling |

**Mitigation Strategy:**
- Rate limiter configured per endpoint
- Test sends rapid requests and verifies 429 (Too Many Requests) response
- Verify bucket refill rate
- P1 Test: "Send 1000 requests in 1s - verify throttled to configured rate (e.g., 100/s)"

**Validation**: Rate limit integration tests

---

### ASR-012: TLS/SSL Encryption (Data in Transit)

**Source**: PRD NFR1-2 (Security - transmission security)  
**Architecture Pattern**: WebSocket over WSS  
**Testability Risk**: Low (infrastructure responsibility)

| Factor | Assessment |
|--------|=========|
| **Probability (p)** | 1 - Unlikely (certificate validation by OS) |
| **Impact (i)** | 3 - Critical (data interception) |
| **Risk Score** | 3 - DOCUMENT |
| **Testability** | ✅ Observable - can verify certificate and enforce HTTPS/WSS |

**Mitigation Strategy:**
- Production deployment uses valid SSL certificate
- Tests use self-signed cert or HTTP locally
- Staging/production enforce WSS (encrypted WebSocket)
- P1 Test: "Reject connection to insecure WebSocket endpoint in production"

**Validation**: Certificate validation tests, HTTPS enforcement

---

## Section 3: Test Levels Strategy

### 3.1 Test Pyramid Recommendation

Based on architecture analysis, recommended test split:

```
         /\
        /  \  E2E Tests (15%)
       /____\  - Critical user journeys
       /    \
      /      \ Integration Tests (35%)
     /________\ - Service boundaries, DB ops
     /        \
    /          \ Unit Tests (50%)
   /____________\ - Business logic, error handling
```

---

### 3.2 Unit Tests (50%) - Foundation Layer

**Purpose**: Fast feedback on business logic and error handling  
**Scope**: Individual functions, services without external dependencies  
**Execution**: <5ms per test  
**Target**: 200+ tests

**Coverage Areas:**

| Area | Component | Risk Level | Test Count |
|------|-----------|-----------|------------|
| Message Validation | `validate_message()` | Low | 8 |
| User Management | `create_user()`, `validate_password()` | High | 15 |
| Conversation Parsing | Parse message content | Low | 6 |
| Error Handling | Service error variants | Medium | 12 |
| Business Logic | Presence calculation, state transitions | Medium | 20 |
| **Total Unit Tests** | | | **61** |

**Example Test Cases:**

```rust
#[test]
fn test_invalid_message_too_long() {
    let msg = "x".repeat(5001); // Exceeds 5000 char limit
    assert!(validate_message(&msg).is_err());
}

#[test]
fn test_password_hash_deterministic() {
    let pwd = "TestPassword123!";
    let hash1 = hash_password(pwd, 4).unwrap();
    let hash2 = hash_password(pwd, 4).unwrap();
    assert_eq!(hash1, hash2); // Same cost factor → same hash
}

#[test]
fn test_jwt_token_invalid_signature() {
    let token = "eyJhbGc...invalid_signature";
    assert!(validate_jwt(token, SECRET_KEY).is_err());
}
```

---

### 3.3 Integration Tests (35%) - Boundary Layer

**Purpose**: Verify component interactions, database operations, API contracts  
**Scope**: Multiple components working together, using test database  
**Execution**: <100ms per test  
**Target**: 120+ tests

**Coverage Areas:**

| Area | Components | Risk Level | Test Count |
|------|-----------|-----------|------------|
| Repository Pattern | UserRepository ↔ SQLite | High | 20 |
| Message Flow | Handler → Service → Repository | High | 25 |
| WebSocket Contract | TungsteniteWebSocket ↔ Handler | Medium | 15 |
| Auth Middleware | JWT validation ↔ Handler | High | 18 |
| Connection Management | Connect/Disconnect sequence | Medium | 12 |
| Multi-Conversation | Concurrent message handling | High | 20 |
| Offline Queue | Queue persistence and flush | Medium | 10 |
| **Total Integration Tests** | | | **120** |

**Example Test Cases:**

```rust
#[tokio::test]
async fn test_send_message_creates_db_record() {
    let db = create_test_db().await; // Separate instance per test
    let handler = CreateMessageHandler::new(db.clone());
    
    let result = handler.handle(CreateMessageRequest {
        sender_id: "user1",
        recipient_id: "user2",
        content: "Hello",
    }).await;
    
    assert!(result.is_ok());
    let message = db.get_message(result.unwrap().id).await.unwrap();
    assert_eq!(message.sender_id, "user1");
    assert_eq!(message.content, "Hello");
}

#[tokio::test]
async fn test_concurrent_messages_preserved_order() {
    let db = create_test_db().await;
    let conversation_id = "conv123";
    
    // Send 100 concurrent messages
    let mut futures = vec![];
    for i in 0..100 {
        futures.push(send_message(
            db.clone(),
            conversation_id,
            &format!("Message {}", i),
        ));
    }
    
    futures::future::join_all(futures).await;
    
    // Verify order preserved
    let messages = db.get_messages(conversation_id).await.unwrap();
    for (i, msg) in messages.iter().enumerate() {
        assert_eq!(msg.content, format!("Message {}", i));
    }
}

#[tokio::test]
async fn test_offline_queue_flush_on_reconnect() {
    let db = create_test_db().await;
    let queue = OfflineQueue::new(db.clone());
    
    // Simulate offline - add messages to queue
    queue.enqueue("msg1").unwrap();
    queue.enqueue("msg2").unwrap();
    
    // Simulate reconnect - flush queue
    let flushed = queue.flush_on_reconnect().await.unwrap();
    
    assert_eq!(flushed.len(), 2);
    assert_eq!(queue.len(), 0); // Queue empty after flush
}
```

---

### 3.4 End-to-End Tests (15%) - User Journey Layer

**Purpose**: Validate critical user workflows end-to-end  
**Scope**: Complete workflow from UI to database and back  
**Execution**: 1-5s per test  
**Target**: 30+ tests

**Coverage Areas:**

| User Journey | Scenario | Risk Level | Test Count |
|----------|----------|-----------|------------|
| Message Exchange | Send message, receive confirmation | Critical | 5 |
| Presence Awareness | Connect, show as online, disconnect | High | 4 |
| Conversation Switching | Switch between 5 conversations | High | 3 |
| Message History | Retrieve last 50 messages | Medium | 3 |
| Offline Recovery | Disconnect, queue message, reconnect | High | 4 |
| Concurrent Users | 10 users messaging same conversation | Critical | 5 |
| Error Scenarios | Invalid token, network timeout, server error | High | 4 |
| **Total E2E Tests** | | | **28** |

**Example Test Cases:**

```rust
#[tokio::test]
async fn test_e2e_send_and_receive_message() {
    let (client1, client2) = setup_e2e_clients().await;
    
    // Client 1 sends message
    let result = client1.send_message(
        "user2_id",
        "Hello from Client 1"
    ).await;
    assert!(result.is_ok());
    
    // Client 2 receives message
    let received = client2.receive_message_timeout(Duration::from_secs(2)).await;
    assert!(received.is_some());
    assert_eq!(received.unwrap().content, "Hello from Client 1");
}

#[tokio::test]
async fn test_e2e_offline_queue_replay() {
    let client = setup_e2e_client().await;
    
    // Simulate network disconnect
    client.force_disconnect().await;
    
    // Send 3 messages while offline
    for i in 0..3 {
        client.queue_message(&format!("Queued {}", i)).await;
    }
    
    // Reconnect
    client.reconnect().await;
    
    // Verify messages delivered
    let confirmations = client.wait_for_confirmations(3, Duration::from_secs(5)).await;
    assert_eq!(confirmations.len(), 3);
}

#[tokio::test]
async fn test_e2e_concurrent_conversation_sync() {
    let clients: Vec<_> = (0..10)
        .map(|_| setup_e2e_client())
        .collect::<Vec<_>>();
    
    let conversation_id = "shared_conv";
    
    // All 10 clients send message concurrently
    let mut futures = vec![];
    for (i, client) in clients.iter().enumerate() {
        futures.push(client.send_message(
            conversation_id,
            &format!("Client {} message", i),
        ));
    }
    
    futures::future::join_all(futures).await;
    
    // Verify all clients see all 10 messages
    for client in &clients {
        let messages = client.get_conversation_messages(conversation_id).await.unwrap();
        assert_eq!(messages.len(), 10);
    }
}
```

---

## Section 4: Non-Functional Requirements (NFR) Testing Strategy

### 4.1 Security Testing

**Category**: SEC (Security)  
**Scope**: Authentication, authorization, data protection  
**Priority**: P0 (All security tests must pass 100%)

| Requirement | Test Type | Scenario | Tool |
|-------------|-----------|----------|------|
| JWT Authentication | Unit/Integration | Reject invalid, expired, tampered tokens | `jsonwebtoken` |
| Bcrypt Password Hash | Unit | Hash determinism, verification | `bcrypt` |
| Rate Limiting | Integration | Reject requests exceeding limit | Load testing |
| API Access Control | Integration | Verify endpoint restrictions | Mock JWT |
| Input Validation | Unit | Reject oversized, malformed messages | Direct invocation |
| TLS/SSL Encryption | Integration | Enforce WSS in production | Certificate validation |

**P0 Security Test Cases:**

```markdown
- [ ] SEC-001: Reject request with invalid JWT token
- [ ] SEC-002: Reject request with expired token
- [ ] SEC-003: Reject request with modified token payload
- [ ] SEC-004: Hash password deterministically with fixed cost
- [ ] SEC-005: Verify correct password matches hash
- [ ] SEC-006: Reject incorrect password
- [ ] SEC-007: Throttle requests exceeding rate limit
- [ ] SEC-008: Reject oversized message (>5000 chars)
- [ ] SEC-009: Reject message with invalid JSON
- [ ] SEC-010: Enforce WSS (encrypted WebSocket) in production
```

**Gate Criteria**: 100% pass rate - no exceptions

---

### 4.2 Performance Testing

**Category**: PERF (Performance)  
**Scope**: Response times, throughput, resource usage  
**Target SLOs** (from architecture):
- Message delivery: <2s end-to-end
- Conversation switching: <100ms
- UI rendering: 60 FPS
- Startup: <2s

| Requirement | Test Type | Scenario | Tool | Target |
|-------------|-----------|----------|------|--------|
| Message Latency | Load test | 1000 concurrent messages | k6 | <2s |
| Query Performance | Benchmark | Retrieve 200 conversations | SQLite | <50ms |
| Connection Throughput | Load test | 100 concurrent connections | k6 | No errors |
| Memory Usage | Profiling | Store 1000 conversations | Valgrind | <100MB |
| Rendering Performance | Integration | Update message list | Slint | 60 FPS |

**P1 Performance Test Cases:**

```markdown
- [ ] PERF-001: Send message with 100ms network latency - complete within 2s
- [ ] PERF-002: Retrieve 200 conversations - complete within 50ms
- [ ] PERF-003: Load test with 100 concurrent WebSocket connections
- [ ] PERF-004: Monitor memory usage with 1000 active conversations
- [ ] PERF-005: Verify message update doesn't stall UI thread (<16ms)
```

**Gate Criteria**: All targets met; waivers require documented justification

---

### 4.3 Reliability Testing

**Category**: RELIABILITY (Resilience, Error Handling)  
**Scope**: Connection recovery, error handling, data consistency  

| Requirement | Test Type | Scenario | Tool |
|-------------|-----------|----------|------|
| Connection Resilience | Integration | Network disconnect/reconnect | Mock WebSocket |
| Offline Queue | Integration | Queue persists messages | SQLite |
| Message Delivery Guarantee | E2E | Verify no message loss | Client verification |
| Error Handling | Unit/Integration | Service errors propagate correctly | Direct invocation |
| Database Transactions | Integration | Concurrent updates don't corrupt data | Transaction verification |

**P1 Reliability Test Cases:**

```markdown
- [ ] REL-001: Simulate disconnect - verify messages queued locally
- [ ] REL-002: Reconnect - verify queued messages sent in order
- [ ] REL-003: Send 100 concurrent messages - verify no loss or duplication
- [ ] REL-004: Close connection mid-transaction - verify rollback, no partial state
- [ ] REL-005: Server error (500) - verify graceful error response
- [ ] REL-006: Invalid message format - verify validation error, not crash
```

**Gate Criteria**: All high-risk items (score ≥6) mitigated or waived

---

### 4.4 Maintainability & Observability

**Category**: OPS (Operations)  
**Scope**: Logging, monitoring, debugging  

| Requirement | Test Type | Validation |
|-------------|-----------|-----------|
| Structured Logging | Integration | Verify JSON logs contain expected fields |
| Error Context | Unit | Verify error messages include user_id, message_id |
| Performance Metrics | Integration | Verify handler latency tracked |
| Trace Context | Integration | Verify request IDs propagate through stack |

**P2 Observability Test Cases:**

```markdown
- [ ] OPS-001: Verify message sent event logged with JSON structure
- [ ] OPS-002: Verify error logs include full context (user_id, timestamp, stack)
- [ ] OPS-003: Verify request latency metrics recorded
- [ ] OPS-004: Verify trace context ID propagates through service chain
```

**Gate Criteria**: Logging framework integrated; metrics collection working

---

## Section 5: Test Environment Requirements

### 5.1 Local Development Environment

**Database**: SQLite in-memory (`:memory:`) or separate file per test  
**WebSocket**: Mock implementation or test server on `localhost:8000`  
**Runtime**: Tokio runtime per test (`#[tokio::test]`)  
**Tools Required**:
- Rust 1.75+
- `cargo test` command
- Optional: `cargo tarpaulin` for coverage

**Setup Time**: <1 minute per test run  
**Parallelization**: Full test parallelization supported (no shared state)

### 5.2 CI/CD Pipeline

**Framework**: GitHub Actions  
**Test Categories**:
- **Unit Tests**: Run on every commit (5 min)
- **Integration Tests**: Run on PR to main (15 min)
- **E2E Tests**: Run nightly (30 min)
- **Load Tests**: Run weekly (60 min)

**Required Stages**:
1. Compile and lint (`cargo build`, `cargo clippy`)
2. Unit tests (`cargo test --lib`)
3. Integration tests (`cargo test --test '*'`)
4. Coverage report (`cargo tarpaulin`)
5. Artifact upload (coverage, test results)

### 5.3 Staging Environment

**Database**: PostgreSQL (production-like)  
**WebSocket**: Real Tokio server with WSS  
**Load**: Equivalent to 10% production capacity  
**Monitoring**: Full logging and metrics collection  
**E2E Tests**: Run against staging; reproduce production issues

---

## Section 6: Critical Testability Observations

### 6.1 Strengths ✅

1. **Dependency Injection Pattern** - Excellent foundation for testing
2. **Stateless Service Design** - Enables parallel test execution
3. **Repository Abstraction** - Mock implementations straightforward
4. **Structured Logging** - Comprehensive observability for validation
5. **Error Handling Pattern** - All errors propagate clearly through stack
6. **Async/Await Architecture** - Tokio's deterministic test support
7. **Database Transaction Model** - ACID properties enable isolation

### 6.2 Concerns (All Addressed) ⚠️

1. **WebSocket Timing Dependencies** → Mitigation: Mock WebSocket allows control
2. **Concurrent State Management** → Mitigation: Arc<Mutex> with transaction boundaries
3. **Network Partition Scenarios** → Mitigation: Offline queue handles disconnections
4. **Timestamp Generation** → Mitigation: Can inject system clock for tests

### 6.3 No Blockers 🟢

- ✅ No untestable architecture decisions
- ✅ No hard dependencies preventing mock implementations
- ✅ No global state or thread-local coupling
- ✅ No production-only code paths
- ✅ No external system dependencies that can't be mocked

---

## Section 7: Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)

**Goals**: Establish test infrastructure and framework  
**Deliverables**:
- [ ] Test database setup (SQLite with automatic cleanup)
- [ ] Mock WebSocket implementation
- [ ] Unit test templates for service layer
- [ ] CI/CD pipeline skeleton (GitHub Actions)

**Effort**: 20 hours  
**Owner**: QA Lead

---

### Phase 2: Core Coverage (Weeks 3-6)

**Goals**: Achieve 70% coverage on critical paths  
**Deliverables**:
- [ ] 60+ unit tests (business logic, error handling)
- [ ] 100+ integration tests (handler→service→repository flows)
- [ ] 20+ E2E tests (critical user journeys)
- [ ] Performance baselines established

**Effort**: 60 hours  
**Owner**: QA Team

---

### Phase 3: Quality Gates (Weeks 7-8)

**Goals**: Implement automated gates and metrics  
**Deliverables**:
- [ ] Coverage reports in CI/CD
- [ ] Performance regression detection
- [ ] Security scanning integration
- [ ] Test result dashboard

**Effort**: 25 hours  
**Owner**: DevOps

---

### Phase 4: Hardening (Weeks 9+)

**Goals**: Expand coverage, stress test, prepare for production  
**Deliverables**:
- [ ] Load testing (100+ concurrent users)
- [ ] Chaos engineering (network failures, server errors)
- [ ] Data migration testing (SQLite → PostgreSQL)
- [ ] Edge case catalog (200+ scenarios)

**Effort**: 40+ hours  
**Owner**: QA Lead + Architect

---

## Section 8: Recommendations for Implementation Phase

### 8.1 For Architecture Decisions 🏗️

1. **Confirm Repository Abstraction** - Ensure all data access goes through repositories
2. **Standardize Error Handling** - Use consistent `Result<T, ServiceError>` pattern
3. **Implement Dependency Injection** - Inject all external dependencies (DB, clock, WebSocket)
4. **Add Logging Instrumentation** - Capture handler entry/exit with structured JSON
5. **Define Service Boundaries** - Clear interfaces for mocking

### 8.2 For Sprint 0 (Framework Setup) 🛠️

Recommended workflow sequence:
1. **`*framework` workflow** - Set up test infrastructure (Tokio, SQLite, mocking)
2. **`*ci` workflow** - Configure GitHub Actions for test execution
3. **Define test data factories** - Create reusable test fixtures
4. **Establish test patterns** - Document unit/integration/E2E templates

### 8.3 For Test Development 📝

Parallel work streams:
- **QA Team**: Write integration and E2E tests (35% effort)
- **Dev Team**: Write unit tests during feature development (50% effort)
- **DevOps**: Implement CI/CD and monitoring (15% effort)

### 8.4 For Release Gates 🚪

Minimum quality criteria before shipping:
- ✅ All P0 tests pass (100%)
- ✅ All high-risk (score ≥6) items mitigated
- ✅ Security tests pass (100%)
- ✅ Performance targets met (>80%)
- ✅ Coverage >70% on critical paths
- ✅ No known high-severity bugs

---

## Section 9: Gate Check Recommendations

### Solutioning Phase Gate (Current)

**Decision**: ✅ **PASS** - Proceed to Implementation Phase

**Rationale**:
1. Architecture is **highly testable** (PASS on all 3 dimensions)
2. No **architectural blockers** to comprehensive testing
3. **Clear patterns** established for all major components
4. **Risk mitigation strategies** defined for all ASRs
5. Test infrastructure **easily implementable** with standard tools

**Conditions**:
- [ ] Confirm repository abstraction in all data access
- [ ] Implement dependency injection for external services
- [ ] Add structured logging framework
- [ ] Create test database setup procedure

**Next Gate**: Implementation Readiness (after epics and test-design phase)

---

## Section 10: Appendix

### 10.1 Related Documentation

- **Architecture**: `/docs/architecture.md` - 9 architectural decisions
- **PRD**: `/docs/prd.md` - Feature and NFR requirements
- **UX Design**: `/docs/ux-design-specification.md` - UI/UX specifications
- **Epics**: `/docs/epics.md` - 9 epics with 100 stories

### 10.2 Knowledge Base References

- **Risk Governance**: `risk-governance.md` - Risk classification framework
- **Probability-Impact**: `probability-impact.md` - Risk scoring methodology
- **Test Levels**: `test-levels-framework.md` - Test level decision matrix
- **Test Priorities**: `test-priorities-matrix.md` - P0-P3 prioritization

### 10.3 Knowledge Base Integration

This test design follows BMad testing methodology:
- **Risk Scoring**: Probability (1-3) × Impact (1-3) = Score (1-9)
- **Risk Action**: Scores 1-3 (Document), 4-5 (Monitor), 6-8 (Mitigate), 9 (Block)
- **Test Levels**: Unit (50%) → Integration (35%) → E2E (15%)
- **Priorities**: P0 (Critical) → P1 (High) → P2 (Medium) → P3 (Low)

### 10.4 Test Metrics Baseline

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Unit Tests | 200+ | 0 | Planning |
| Integration Tests | 120+ | 0 | Planning |
| E2E Tests | 30+ | 0 | Planning |
| Code Coverage (Critical) | >80% | 0% | Planning |
| P0 Pass Rate | 100% | N/A | TBD |
| Message Latency | <2s | Unknown | To measure |
| Query Performance | <50ms | Unknown | To measure |

---

## Approval

**System-Level Test Design Approved By:**

- [ ] Product Manager: _______________ Date: _______
- [ ] Tech Lead / Architect: _______________ Date: _______
- [ ] QA Lead: _______________ Date: _______

**Overall Recommendation:**

✅ **PASS** - Architecture is ready for implementation with comprehensive testing strategy in place.

---

**Generated by**: BMad TEA (Test Architect) Agent  
**Workflow**: `_bmad/bmm/testarch/test-design` (System-Level Mode)  
**Version**: 4.0 (BMad v6)  
**Date**: 2025-12-17  
**Status**: Complete
