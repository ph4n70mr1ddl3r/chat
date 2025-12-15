# Phase 0 Research: Private Chat Application Design

**Date**: 2025-12-15  
**Status**: Complete  

All technology decisions resolved. All clarifications from feature spec researched and validated.

**Reference Documents**:
- Backend patterns: `/home/riddler/chat/RUST_REALTIME_CHAT_GUIDE.md` (comprehensive WebSocket, auth, database, error handling patterns)
- Frontend patterns: `/home/riddler/chat/DESKTOP_CHAT_ARCHITECTURE.md` (Slint component architecture, threading, state management)
- Architecture diagrams: `/home/riddler/chat/ARCHITECTURE_DIAGRAMS.txt` (visual explanations)

---

## 1. Rust Backend + SQLite Architecture

**Decision**: Tokio + sqlx + SQLite (WAL mode)

**Rationale**:
- **Tokio**: Proven async runtime for real-time applications; battle-tested in production chat systems
- **sqlx**: Compile-time query verification with type safety; native async support (tokio::Postgres/SQLite)
- **SQLite WAL mode**: Enables concurrent reads while maintaining single-writer semantics; adequate for MVP (10k users); simpler than PostgreSQL for initial deployment

**Alternatives Considered**:
- **rusqlite**: Synchronous-only, incompatible with async WebSocket handlers → rejected
- **Diesel ORM**: Over-engineered for SQLite; added complexity → rejected  
- **SeaORM**: Immature SQLite support; better for PostgreSQL → rejected
- **PostgreSQL**: Overkill for MVP; SQLite sufficient for 100k messages + 10k concurrent users; migrate later if needed → deferred

**Implementation Path**:
```rust
// Cargo.toml dependencies
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite"] }
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

---

## 2. WebSocket: tokio-tungstenite + Warp Router

**Decision**: tokio-tungstenite (client/server) + warp (HTTP routing)

**Rationale**:
- **tokio-tungstenite**: Minimal overhead; pure Tokio integration (no competing runtimes)
- **warp**: Lightweight router; filter-based composition; excellent for WebSocket upgrade path
- **Combined**: Enables clean separation of HTTP (REST health checks, auth) and WebSocket (message streaming)

**Alternatives Considered**:
- **actix-web-ws**: Full framework, overkill for chat; adds dependencies → rejected
- **tonic (gRPC)**: Wrong paradigm for browser clients; overkill for text messages → rejected
- **quinn (QUIC)**: Immature ecosystem; browser support limited → rejected

**Implementation Pattern**:
```rust
// Server: HTTP upgrade to WebSocket via warp
let ws_route = warp::path("socket")
    .and(warp::ws())
    .map(|ws: warp::ws::Ws| ws.on_upgrade(handle_connection));

// Client: tokio-tungstenite for Slint integration
let (ws_stream, _) = tokio_tungstenite::connect_async(uri).await?;
```

---

## 3. JSON Serialization: serde + serde_json

**Decision**: serde (derive) + serde_json

**Rationale**:
- **De facto Rust standard**: Widest ecosystem support; every library expects serde
- **Compile-time codegen**: Zero runtime overhead; type-safe serialization
- **Human-readable**: Essential for debugging; facilitates protocol versioning
- **Performance**: ~99% efficient vs. binary formats for text messages (200 bytes typical)

**Alternatives Considered**:
- **protobuf**: Binary, not human-readable; complicates debugging → rejected
- **bincode**: Custom Rust format; not interoperable with other languages → rejected
- **MessagePack**: Less ecosystem integration; serde_json sufficient → rejected

**Message Structure**:
```json
{
  "id": "msg-uuid-v4",
  "type": "message|typing|read|ack",
  "senderId": "user123",
  "recipientId": "user456",
  "content": "Hello, world!",
  "timestamp": 1702657890000,
  "metadata": {}
}
```

---

## 4. CLI & Configuration: clap (derive) + config crate

**Decision**: clap 4.x (derive macros) + config crate (layered)

**Rationale**:
- **clap**: Most maintained CLI parser; derive macros eliminate boilerplate; built-in help/version
- **config crate**: Supports CLI args → env vars → config files (precedence order)
- **Combined**: Flexible deployment (WSL with args, Docker with env, config files in production)

**Alternatives Considered**:
- **structopt**: Deprecated in favor of clap (last version 0.3.26) → rejected
- **argh**: Too minimal; lacks config file support → rejected

**Usage**:
```bash
# WSL command-line
./chat-server --port 8080 --db-path ./chat.db --log-level debug

# Configuration file equivalent
[server]
port = 8080
db_path = "./chat.db"
log_level = "debug"
```

---

## 5. Slint GUI for Windows Client

**Decision**: Slint 1.14+ (declarative DSL + Winit backend)

**Rationale**:
- **Native Windows**: Compiles to x86-64 executable; Winit handles Windows windowing system
- **Material 3 Design**: Professional UI out-of-box; expected by Windows users
- **Lightweight**: ~300 KiB runtime; 50-80 MiB memory usage (vs. Electron 150-200 MiB)
- **Reactive Properties**: Automatic UI sync when data changes; ideal for real-time messages
- **Proven in Production**: Used by SK Signet, WesAudio, OTIV

**Alternatives Considered**:
- **Qt/PyQt**: 100+ MiB footprint; licensing complexity → rejected
- **Electron**: 150-200 MiB baseline; Web security model limits OS integration → rejected
- **Flutter**: Mobile-first; desktop support immature → rejected
- **Tauri**: Web-based approach; Slint's native compilation avoids browser complexity → rejected
- **Druid/Iced**: Immature ecosystem; Slint has better Material Design → rejected

**Threading Model** (Critical for Windows):
- Main thread: Slint event loop (Windows requirement; handles windowing)
- Worker thread: Tokio async runtime (WebSocket I/O)
- Communication: tokio::sync::mpsc channels + `invoke_from_event_loop()` for UI updates

```rust
// Spawn Tokio on background thread; Slint runs on main
std::thread::spawn(|| {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        // WebSocket loop
    })
});

// Update UI from Tokio via channel + invoke_from_event_loop()
slint::invoke_from_event_loop(move || {
    ui.set_messages(messages.clone());
}).ok();
```

---

## 6. Message Delivery & Offline Handling

**Decision**: At-least-once delivery with UUID deduplication; exponential backoff reconnection

**Rationale**:
- **At-least-once**: Simpler than exactly-once (no distributed state needed); idempotent for chat
- **UUID dedup**: Client assigns message ID; server deduplicates by ID
- **Exponential backoff**: RFC 6455 recommendation; prevents thundering herd; 0.5-60s range

**Alternative Rejected**:
- **Exactly-once**: Requires transaction logs, consensus, distributed state → unnecessary complexity
- **Fire-and-forget**: Message loss unacceptable for chat
- **Fixed delay**: Synchronization issues on mass reconnection

**Reconnection Strategy**:
```
Attempt 1: 0.5-1.5s (random jitter)
Attempt 2: 1.5-3.5s
Attempt 3: 3-7s
Attempt 4: 7-15s
Attempt 5: 15-30s
Attempt 6+: 30-60s (capped)
```

---

## 7. Session Management: JWT + Ping-Pong Keepalive

**Decision**: JWT tokens at handshake; ping-pong keepalive (25s/5s)

**Rationale**:
- **JWT at handshake**: Single authentication point during HTTP upgrade; stateless (no session storage needed)
- **Ping-pong**: RFC 6455 native control frames; detects dead connections without application overhead
- **25s/5s interval**: Socket.IO standard; proven in production; <1% bandwidth overhead

**Token Structure**:
```json
{
  "sub": "user-id",
  "aud": "chat-app",
  "iat": 1702657890,
  "exp": 1702661490,  // 1 hour expiry
  "scopes": ["send", "receive"]
}
```

**Alternatives Rejected**:
- **Cookies**: Not applicable to WebSocket (set after handshake)
- **Re-auth per message**: Excessive overhead
- **Client-side heartbeat**: Application-level inefficiency vs. RFC-compliant ping

---

## 8. Rate Limiting & Security

**Decision**: Token-bucket rate limiting (100 msgs/min, 10 KB/msg) + WSS (TLS 1.3) + input validation

**Rationale**:
- **Token-bucket**: Fair across users; burst allowance (5 msgs/sec); prevents abuse
- **WSS**: Encrypts tokens and content in transit; required for production
- **Input validation**: Length, UTF-8 validity, schema compliance; reject on fail (close connection)

**Rate Limit Thresholds**:
- 100 messages per minute per user (typical chat)
- 10 KB max message size
- 5 message burst allowance per second
- Penalty: 429 status code; close on repeated violation

**Input Validation Checklist**:
- ✓ Length: 1-5,000 characters (from spec)
- ✓ UTF-8 validity (RFC 3629)
- ✓ JSON schema: must match Message structure
- ✓ Recipient authorization: verify sender can message recipient
- ✗ Masking: RFC 6455 mandates (handled by protocol)

---

## 9. Testing Strategy: Pyramid (Unit → Integration → E2E)

**Decision**: tokio::test + mockall + in-memory SQLite + Slint testing framework

**Rationale**:
- **Fast unit tests**: Mocks external dependencies; ~100ms per test
- **Integration tests**: Real SQLite (in-memory); contracts validated
- **E2E tests**: Full server + client; real WebSocket connections
- **TDD mandatory**: Tests written first, then implementation (red-green-refactor)

**Test Stack**:
```toml
[dev-dependencies]
tokio = { version = "1", features = ["test-util"] }
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }
mockall = "0.12"
proptest = "1.0"
```

**Coverage Targets**:
- Unit: 80%+ code coverage
- Integration: All API contracts
- E2E: Critical user paths (P1 stories)

---

## 10. Scaling & Deployment

**Decision**: Stateless server design (Redis pub/sub for multi-node); SQLite→PostgreSQL migration path

**Rationale**:
- **Stateless**: No in-process message queues; enables horizontal scaling
- **Redis pub/sub**: Low-latency message distribution between nodes; room-based subscriptions
- **SQLite→PostgreSQL**: SQLite sufficient for MVP; defined upgrade path when needed

**Multi-Node Architecture**:
```
Client A ←→ Server 1 ──┐
                        ├─→ Redis Pub/Sub ←─ Server 2 ←→ Client B
Client C ←→ Server 1 ──┘     (channel: chat:dm:{user1}:{user2})
```

**Sticky Sessions** (if long-polling fallback added):
- Cookie-based: server sets `io=server-id`
- Load balancer: `hash $remote_addr` or connection affinity

---

## Summary Table

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **Language** | Rust 1.75+ | Systems language; async/await; type safety |
| **Async Runtime** | Tokio 1.x | Battle-tested; minimal overhead; WebSocket-native |
| **Database** | SQLite (WAL) | Simple, fast, ACID; adequate for MVP |
| **WebSocket** | tokio-tungstenite + warp | Minimal, pure Tokio integration |
| **JSON** | serde + serde_json | Standard; human-readable; debuggable |
| **CLI** | clap + config crate | Flexible; supports args, env, config files |
| **Client GUI** | Slint 1.14+ | Native Windows; lightweight; Material 3 design |
| **Client WebSocket** | tokio-tungstenite | Async Rust client; easy Slint integration |
| **Authentication** | JWT + ping-pong | Stateless; RFC 6455 compliant |
| **Delivery** | At-least-once + UUID dedup | Simple, correct semantics for chat |
| **Rate Limit** | Token-bucket | Fair; burst-aware; prevents abuse |
| **Testing** | Unit+Integration+E2E | TDD mandatory; pyramid approach |
| **Scaling** | Redis pub/sub (future) | Stateless; horizontal scaling ready |

---

## Deferred Decisions (For Phase 1+)

- **Database migrations**: Simple append-only initially; migrate to schema versioning if needed
- **Metrics/Observability**: Tracing infrastructure defined; Prometheus integration deferred
- **Account deletion UX**: Spec defines anonymization; detailed flow deferred to Phase 1
- **Persistence layer abstraction**: Direct SQLx queries acceptable for MVP; repository pattern if complexity grows

---

## Next Steps

✅ Phase 0 complete: All research decisions made and justified.  
→ **Phase 1**: Generate data-model.md, API contracts, quickstart.md  
→ **Phase 2** (follow-up): `/speckit.tasks` generates task list for implementation
