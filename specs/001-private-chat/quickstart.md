# Quick Start Guide: Private Chat Application

**Date**: 2025-12-15  
**Status**: Development Setup  

This guide gets developers up and running with the chat application in under 15 minutes.

---

## Prerequisites

- **Rust**: 1.75+ (install from [rustup.rs](https://rustup.rs/))
- **WSL2** (Windows Subsystem for Linux; Ubuntu 20.04+): For server development
- **Windows 11/10**: For client development
- **Git**: For cloning the repository
- **SQLite3**: Usually included; verify with `sqlite3 --version`

---

## Project Structure

```
chat-app/
├── server/          # Rust WebSocket server (runs in WSL)
├── client/          # Slint Windows GUI client
├── shared/          # Shared types (Message, User, Conversation)
├── specs/           # Feature specifications & contracts
└── Cargo.toml       # Workspace root
```

---

## Phase 1: Set Up Development Environment

### 1.1 Clone Repository

```bash
git clone https://github.com/your-org/chat-app.git
cd chat-app
```

### 1.2 Initialize Cargo Workspace

Verify workspace structure:

```bash
# From repo root
cargo --version   # Verify Rust installed
cargo build --workspace  # Build all crates
```

**Expected output**:
```
   Compiling chat-server v0.1.0
   Compiling chat-client v0.1.0
   Compiling chat-shared v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 45.32s
```

---

## Phase 2: Run the Server (WSL)

### 2.1 Start the Server

From WSL bash:

```bash
cd chat-app/server
cargo run -- --port 8080 --db-path ./chat.db --log-level debug
```

**Expected output**:
```
2025-12-15T10:30:00Z INFO chat_server: Starting server on port 8080
2025-12-15T10:30:00Z INFO sqlx: Initializing database at ./chat.db
2025-12-15T10:30:00Z INFO chat_server: Waiting for connections...
```

**Port Mappings**:
- `8080`: WebSocket server (ws://localhost:8080/socket)
- HTTP endpoints: `http://localhost:8080/auth/signup`, etc.

### 2.2 Verify Server Health

In another WSL terminal:

```bash
curl http://localhost:8080/health
```

**Expected response**:
```json
{
  "status": "healthy",
  "timestamp": 1702657890000,
  "uptime_seconds": 15
}
```

---

## Phase 3: Create Test Accounts

### 3.1 Sign Up User A

```bash
curl -X POST http://localhost:8080/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "password": "TestPass123"
  }'
```

**Response**:
```json
{
  "userId": "user-550e8400-e29b-41d4-a716-446655440000",
  "username": "alice",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expiresIn": 3600
}
```

**Save the token** for next step: `export TOKEN_ALICE="<token>"`

### 3.2 Sign Up User B

```bash
curl -X POST http://localhost:8080/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "bob",
    "password": "TestPass456"
  }'
```

**Save token**: `export TOKEN_BOB="<token>"`

### 3.3 Verify Users

```bash
curl -H "Authorization: Bearer $TOKEN_ALICE" http://localhost:8080/user/me
```

**Expected response**:
```json
{
  "userId": "user-550e8400-e29b-41d4-a716-446655440000",
  "username": "alice",
  "createdAt": 1702657890000,
  "isOnline": false,
  "lastSeenAt": null
}
```

---

## Phase 4: Test WebSocket Connection

### 4.1 Connect with WebSocket Client Tool

Use `websocat` (install: `cargo install websocat`):

```bash
# Terminal 1: Alice connects
websocat "ws://localhost:8080/socket?token=$TOKEN_ALICE"

# Terminal 2: Bob connects (in another terminal)
websocat "ws://localhost:8080/socket?token=$TOKEN_BOB"
```

### 4.2 Send Test Message from Alice to Bob

In Alice's terminal, send:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "type": "message",
  "timestamp": 1702657890000,
  "data": {
    "recipientId": "user-660e8400-e29b-41d4-a716-446655440001",
    "content": "Hello Bob!"
  }
}
```

**Expected response in Alice's terminal** (ACK):
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "type": "ack",
  "timestamp": 1702657890124,
  "data": {
    "status": "sent",
    "conversationId": "conv-789",
    "messageId": "550e8400-e29b-41d4-a716-446655440001",
    "serverTimestamp": 1702657890123
  }
}
```

**Expected message in Bob's terminal**:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "type": "message",
  "timestamp": 1702657890123,
  "data": {
    "senderId": "user-550e8400-e29b-41d4-a716-446655440000",
    "senderUsername": "alice",
    "recipientId": "user-660e8400-e29b-41d4-a716-446655440001",
    "content": "Hello Bob!",
    "conversationId": "conv-789",
    "status": "delivered"
  }
}
```

---

## Phase 5: Run the Windows Client

### 5.1 Build Slint Client

From Windows PowerShell (or WSL with Windows cross-compilation):

```bash
cd chat-app/client
cargo build --release
```

**Output**: `target/release/chat-client.exe`

### 5.2 Run Client

```bash
# Double-click the .exe or run from terminal
./target/release/chat-client.exe
```

**Expected behavior**:
- Windows window opens (Material 3 design)
- Login screen displayed
- Input fields for username/password

### 5.3 Login and Chat

1. Click **Sign Up** tab
2. Enter: Username=`charlie`, Password=`TestPass789`
3. Click **Create Account**
4. Login screen appears
5. Click **Search Users** → Type `alice` → Click result
6. **Start Chat** button activates
7. Type message in input field
8. Click **Send** or press Enter
9. Message appears in conversation

---

## Phase 6: Run Tests

### 6.1 Unit Tests (Server)

```bash
cd server
cargo test --lib
```

**Expected output**:
```
running 12 tests

test models::tests::test_user_creation ... ok
test handlers::tests::test_message_validation ... ok
...

test result: ok. 12 passed; 0 failed
```

### 6.2 Integration Tests (Server)

```bash
cd server
cargo test --test "*"
```

**Expected output**:
```
running 5 tests

test websocket_integration_test::test_message_delivery ... ok
test auth_integration_test::test_signup_flow ... ok
...

test result: ok. 5 passed; 0 failed
```

### 6.3 Client UI Tests

```bash
cd client
cargo test --lib
```

**Note**: Slint UI tests are automated; no manual interaction needed.

---

## Phase 7: Debug Mode

### 7.1 Server with Verbose Logging

```bash
# Set log level to trace (most verbose)
cd server
RUST_LOG=trace cargo run -- --log-level trace
```

**Enables**:
- Full request/response logging
- Database query logs
- WebSocket frame inspection
- Performance metrics

### 7.2 Client with Debug Output

Slint client includes built-in debugging:

```bash
# Enable Slint debugging
cd client
SLINT_DEBUG=1 cargo run
```

Opens debug window showing:
- Property values
- Component hierarchy
- Event log
- Performance profiler

---

## Common Tasks

### Run Full Integration Test Suite

```bash
cargo test --workspace
```

### Check Code Style

```bash
cargo fmt --check
cargo clippy --all
```

### Fix Formatting

```bash
cargo fmt --all
```

### Build for Production

```bash
# Server (WSL)
cd server
cargo build --release

# Client (Windows)
cd client
cargo build --release
```

### Check Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cd server
cargo tarpaulin --out Html --output-dir ./coverage
```

---

## Troubleshooting

### Issue: Port 8080 Already in Use

```bash
# Find process using port 8080 (WSL)
lsof -i :8080

# Kill process
kill -9 <PID>

# Or use different port
cargo run -- --port 8081
```

### Issue: WebSocket Connection Refused

```bash
# Verify server is running
curl http://localhost:8080/health

# Check WSL networking (ensure localhost:8080 accessible from Windows)
# In WSL: netstat -tulpn | grep 8080
```

### Issue: JWT Token Invalid

```bash
# Tokens expire after 1 hour; create new account
curl -X POST http://localhost:8080/auth/signup \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "password": "TestPass123"}'
```

### Issue: SQLite Database Locked

```bash
# Close all connections; remove lock file
rm chat.db-wal chat.db-shm

# Restart server
cargo run -- --db-path ./chat.db
```

### Issue: Slint Client Won't Build

```bash
# Ensure Rust target installed for Windows
rustup target add x86_64-pc-windows-msvc

# Clear build cache
cargo clean

# Rebuild
cargo build --release
```

---

## Next Steps

✅ **Local Development**: Server + Client running locally  
→ **Phase 2**: Implement user stories (TDD priority: P1 stories first)  
→ **Phase 3**: Integration testing (server + client)  
→ **Phase 4**: Deploy to staging (WSL on shared server)  
→ **Phase 5**: Load testing (100 msgs/sec target)  

---

## Useful Commands Cheat Sheet

```bash
# Start server
cd server && cargo run -- --port 8080 --log-level debug

# Start client
cd client && cargo run

# Run all tests
cargo test --workspace

# Run specific test
cargo test test_message_delivery -- --nocapture

# Check for issues
cargo clippy --all

# Format code
cargo fmt --all

# Build release (optimized)
cargo build --release

# Connect to WebSocket (websocat)
websocat "ws://localhost:8080/socket?token=$TOKEN"

# Query server health
curl http://localhost:8080/health

# Create account
curl -X POST http://localhost:8080/auth/signup \
  -H "Content-Type: application/json" \
  -d '{"username": "test", "password": "Test1234"}'

# Get current user
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/user/me

# Get conversation history
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/conversations/conv-id/messages?limit=10"
```

---

## Documentation References

- **Feature Spec**: `specs/001-private-chat/spec.md` (user stories, requirements)
- **Data Model**: `specs/001-private-chat/data-model.md` (schema, entities)
- **WebSocket Protocol**: `specs/001-private-chat/contracts/websocket-protocol.md`
- **Server API**: `specs/001-private-chat/contracts/server-contract.md`
- **Message Schema**: `specs/001-private-chat/contracts/message-schema.json` (JSON schema)

---

## Support

For issues, questions, or suggestions:
- **GitHub Issues**: Report bugs and feature requests
- **Discord**: Real-time discussion with team
- **Email**: dev@chat-app.local

---

**Status**: ✅ Ready for Phase 2 (Implementation)
