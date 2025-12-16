# Project Context: chat

**Project:** chat - Private Chat Application  
**Type:** Desktop Application UI/UX Modernization (Brownfield)  
**Technologies:** Rust 1.75+, Slint, Tokio, Warp, SQLite → PostgreSQL  
**Status:** Architecture Complete, Ready for Implementation  
**Last Updated:** 2025-12-17

---

## 🎯 Quick Start for AI Agents

### Before You Start Implementing

1. **Read the complete architecture document:** `/home/riddler/chat/docs/architecture.md`
2. **Understand the 9 core decisions** (lines 150-571)
3. **Learn the 9 mandatory patterns** (lines 573-1280)
4. **Study the project structure** (lines 1282-1903)
5. **Reference this file for quick decisions**

### What We're Building

A modernized Slint-based desktop chat application with:
- Real-time messaging (WebSocket)
- Multi-conversation management
- Presence awareness
- Modern, minimal UI design
- Professional aesthetic positioning

**Key Constraint:** Maintain existing backend protocols unchanged (brownfield modernization)

---

## 🏗️ Architecture at a Glance

### 9 Core Architectural Decisions

| # | Decision | Choice | Why |
|---|----------|--------|-----|
| 1 | Component Organization | Domain-Based | Maps to user journeys; parallel development |
| 2 | Design System | Centralized Tokens | Single source of truth; theme switching |
| 3 | State Management | Centralized AppState | Slint-native reactive bindings |
| 4 | Message Loading | On-Demand (50 msgs) | Memory efficient; supports any size |
| 5 | Backend Integration | Command/Event Pattern | Type-safe; versionable; extensible |
| 6 | Error Resilience | Automatic Retry+Backoff | Better UX; handles transient errors |
| 7 | Message Rendering | Virtual Lists | 60+ FPS performance |
| 8 | Startup Performance | Progressive Loading | ~1 second perceived startup |
| 9 | Animations | Calculated Effects | Professional + performant |

### Domain Organization

All backend handlers and services organized by domain:

```
Backend Domains:
  • auth           → authentication, tokens, JWT
  • messages       → message storage, delivery, history
  • presence       → user status, online/offline
  • conversation   → conversation management, participants
  • user           → user discovery, search, profiles
  • system_prefs   → application settings

Frontend Domains:
  • messaging      → message display, composition, lists
  • presence       → online indicators, status displays
  • discovery      → conversation finding, switching
  • settings       → user preferences
  • layouts        → screen containers
  • shared         → reusable primitives
```

---

## 🎨 Design System Rules

### Design Tokens (Single Source of Truth)

**File:** `src/frontend/design/tokens.slint`

Every visual element MUST use a token. No hardcoded colors/fonts/spacing anywhere.

```slint
// ✅ CORRECT
background: Tokens.palette-primary-light;
font-size: Tokens.font-size-base;
padding: Tokens.spacing-md;

// ❌ WRONG
background: #FFFFFF;
font-size: 14px;
padding: 16px;
```

### Theme Switching

- Detect Windows dark/light mode on startup
- Store current theme in `AppState.theme` ("light" | "dark")
- All components conditionally use tokens based on theme
- Use: `theme == "light" ? Tokens.color-light : Tokens.color-dark`

### Color Palette

- **Primary Light:** #FFFFFF (background)
- **Primary Dark:** #1F1F1F (background)
- **Accent:** #0078D4 (Fluent Blue - actions)
- **Text Light:** #000000 (on light background)
- **Text Dark:** #FFFFFF (on dark background)

### Typography

- **Family:** "Segoe UI", system-ui, sans-serif
- **Sizes:** xs(12px), sm(13px), base(14px), lg(16px), xl(18px)
- **Weights:** regular(400), medium(500), semibold(600)

### Spacing Scale (8px base)

- xs: 4px
- sm: 8px
- md: 16px
- lg: 24px
- xl: 32px

### Animations

- Fast: 100ms
- Base: 200ms
- Slow: 300ms

**All animations must be calculated (not hardcoded waits). Adapt to frame rate.**

---

## 📁 Project Structure Rules

### Backend Organization

```
src/backend/
├── handlers/              # HTTP request handlers (domain-organized)
│   ├── auth.rs            # POST /auth/login, /auth/signup
│   ├── messages.rs        # Message endpoints + WebSocket
│   ├── conversation.rs    # GET /conversations, POST /conversations
│   ├── dispatcher.rs      # Route WebSocket messages to handlers
│   └── [other domains]
├── services/              # Business logic (one per domain)
│   ├── auth_service.rs
│   ├── message_service.rs
│   ├── presence.rs
│   ├── conversation_service.rs
│   ├── user_service.rs
│   └── [domain-specific services]
├── middleware/            # Cross-cutting concerns
│   ├── auth.rs            # JWT validation
│   └── rate_limit.rs      # Rate limiting
├── db/                    # Database layer
│   ├── migrations/        # SQL migration files
│   └── queries/           # Database query functions
└── models/                # Data models
```

**Rule:** Everything is domain-organized. No mixed concerns.

### Frontend Organization

```
src/frontend/
├── screens/               # Full-screen components (one .slint + one .rs per screen)
│   ├── login_screen.slint + login_screen.rs
│   ├── chat_screen.slint + chat_screen.rs
│   ├── settings_screen.slint + settings_screen.rs
│   └── [other screens]
├── components/            # Reusable component library
│   ├── messaging/         # Message-related components
│   │   ├── MessageBubble.slint
│   │   ├── MessageList.slint
│   │   ├── MessageComposer.slint
│   │   └── MessageSearch.slint
│   ├── presence/          # Presence-related components
│   ├── discovery/         # Conversation discovery
│   ├── settings/          # Settings components
│   ├── layouts/           # Layout containers
│   └── shared/            # Primitives (Button, Input, etc)
├── design/                # Design system
│   └── tokens.slint       # All design tokens (single source of truth)
├── services/              # Frontend services
│   ├── http_client.rs     # REST API calls
│   ├── websocket_client.rs # WebSocket connection
│   └── session.rs         # Session/user management
└── ui.slint               # Main entry point
```

**Rules:**
- One component per file
- PascalCase file names (MessageBubble.slint)
- Domain-based subdirectories
- No generic names (use specific domain)

### Shared Code

```
src/shared/
├── protocol/              # Command/Event definitions
│   └── mod.rs
├── errors/                # Error types
│   └── mod.rs
└── lib.rs
```

---

## 🔤 Naming Conventions

### Frontend Components

- **File names:** PascalCase (MessageBubble.slint)
- **Component names:** PascalCase (export component MessageBubble)
- **Properties:** snake-case (in property <string> sender-name)
- **Files:** One component per file

```slint
// ✅ CORRECT: src/frontend/components/messaging/MessageBubble.slint
export component MessageBubble {
  in property <string> sender-name;
  in property <string> text;
  // ...
}

// ❌ WRONG
message-bubble.slint           // Wrong: kebab-case
messageBubble.slint            // Wrong: camelCase
MessageBubbleItem.slint        // Wrong: too generic
```

### Backend Handlers

- **File names:** snake_case (messages.rs)
- **Function names:** snake_case (fn handle_send_message)
- **File location:** Centralized in handlers/ directory
- **Organization:** By domain (handlers/messages.rs, handlers/auth.rs)

```rust
// ✅ CORRECT: src/backend/handlers/messages.rs
pub async fn handle_send_message(
    state: &mut AppState,
    cmd: SendMessageCommand
) -> Result<MessageSentEvent, Error> { ... }

// ❌ WRONG
fn SendMessage() { }           // Wrong: PascalCase
fn send_message_handler() { }  // Wrong: redundant suffix
```

### Backend Services

- **File names:** snake_case (message_service.rs)
- **Struct names:** PascalCase (MessageService)
- **Method names:** snake_case (send_message)
- **File location:** Centralized in services/ directory

```rust
// ✅ CORRECT: src/backend/services/message_service.rs
pub struct MessageService { ... }
impl MessageService {
    pub fn send_message(&self, msg: Message) -> Result<(), Error> { ... }
}

// ❌ WRONG
pub struct messageservice { }  // Wrong: lowercase
pub fn SendMessage() { }       // Wrong: PascalCase function
```

---

## 📨 WebSocket Protocol Rules

### Message Format

All WebSocket messages follow strict structure:

```json
{
  "id": "uuid-string",          // Unique message ID
  "command": "CommandName",     // ← Sent by frontend
  "event": "EventName",         // ← Sent by backend
  "payload": { ... }            // Command/event-specific data
}
```

### Naming Conventions

- **Commands** (frontend → backend): PascalCase
  - SendMessage, FetchMessages, SetPresence, TypingIndicator
  
- **Events** (backend → frontend): PascalCase
  - MessageSent, MessageReceived, PresenceChanged, Error
  
- **JSON fields in payload**: camelCase
  - conversationId, senderId, messageId, userId

```json
// ✅ CORRECT
{
  "id": "cmd-123",
  "command": "SendMessage",
  "payload": {
    "conversationId": "conv-456",
    "text": "Hello"
  }
}

// ❌ WRONG
{
  "command": "send_message",      // Wrong: snake_case command
  "payload": {
    "conversation_id": "conv-456" // Wrong: snake_case JSON
  }
}
```

### Type Serialization

All types must use Serde with consistent camelCase:

```rust
// ✅ CORRECT
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageCommand {
    pub conversation_id: String,
    pub text: String,
}
// Serializes to: { "conversationId": "...", "text": "..." }

// ❌ WRONG
#[derive(Serialize)]
pub struct SendMessageCommand {
    pub conversation_id: String,  // Will serialize as "conversation_id"!
}
```

---

## 🔄 State Management Rules

### AppState Structure

**File:** Backend state passed to frontend via events

```rust
#[derive(Debug, Clone)]
pub struct AppState {
  pub theme: String,                                    // "light" | "dark"
  pub connection_status: String,                        // "connected" | "disconnected" | "connecting"
  pub current_user: User,
  pub conversations: Vec<Conversation>,
  pub current_conversation_id: Option<String>,
  pub presence_map: HashMap<String, UserPresence>,     // Who's online
  pub message_cache: HashMap<String, Vec<Message>>,    // Message cache by conversation
}
```

### State Update Rules

- **Always use mutable references** for efficiency
- **Never clone AppState** (inefficient!)
- **Update in-place** whenever possible

```rust
// ✅ CORRECT: Mutable reference, efficient in-place update
fn handle_message_received(state: &mut AppState, msg: Message) {
    if let Some(messages) = state.message_cache.get_mut(&msg.conversation_id) {
        messages.push(msg);
    }
}

// ❌ WRONG: Cloning entire state
fn handle_message_received(state: AppState, msg: Message) -> AppState {
    let mut new_state = state.clone();  // ❌ INEFFICIENT!
    // ...
    new_state
}
```

### Message Cache Pattern

```rust
// On-demand loading: 50 messages at a time
// Cache by conversation_id
// Clear on app close
pub struct AppState {
    pub message_cache: HashMap<String, Vec<Message>>,  // conversation_id → messages
}

// When user opens conversation:
// 1. Frontend sends: FetchMessages { conversation_id, before_id: None }
// 2. Backend loads 50 messages from database
// 3. Returns: HistoryLoaded { messages: [...] }
// 4. Frontend caches: state.message_cache.insert(conv_id, messages)
// 5. Slint reactively renders message list
```

### Reactive Binding

- Slint automatically updates UI when AppState changes
- **Don't manually update UI** (use AppState updates instead)
- **All state changes flow through AppState**

```slint
// ✅ CORRECT: Reactive binding
export component ChatScreen {
    in property <AppState> state;
    
    for msg in state.messages: MessageBubble {
        text: msg.text;
        sender-name: msg.sender-name;
    }
    // Auto-updates when state.messages changes!
}

// ❌ WRONG: Manual UI updates
export component ChatScreen {
    property <[Message]> local-messages: [];
    // ❌ This won't stay in sync with backend state!
}
```

---

## ❌ Error Handling Rules

### Errors as Events (NOT Exceptions)

Errors are communicated as events, matching offline-first strategy:

```json
{
  "id": "evt-123",
  "event": "Error",
  "payload": {
    "code": "NETWORK_ERROR",
    "message": "Failed to send message",
    "details": "connection timeout"
  }
}
```

### Error Event Structure

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEvent {
    pub code: String,                 // e.g., "NETWORK_ERROR", "AUTH_ERROR"
    pub message: String,              // User-friendly message
    pub details: Option<String>,      // Optional technical details
}
```

### Frontend Error Handling

```rust
// ✅ CORRECT: Handle errors as events
fn handle_error_event(state: &mut AppState, error: ErrorEvent) {
    state.last_error = Some(error);
    state.connection_status = "disconnected".to_string();
    // UI reactively displays error via AppState.last_error
}

// ❌ WRONG: Throwing exceptions
match send_message() {
    Err(e) => return Err(e);  // ❌ Wrong pattern!
}
```

### Retry Logic

- Automatic retry: 100ms → 200ms → 400ms (3 attempts max)
- Only notify user after max retries
- Manual reconnect button available during disconnection

```rust
// Backend automatically retries transient errors
async fn send_with_retry(cmd: Command, max_retries: u32) -> Result<Event> {
    let mut backoff = 100; // milliseconds
    for attempt in 0..max_retries {
        match execute_command(&cmd).await {
            Ok(event) => return Ok(event),
            Err(e) if is_transient(&e) => {
                sleep(Duration::from_millis(backoff)).await;
                backoff *= 2;  // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## 📊 Loading State Management

### Per-Domain Tracking

Track loading state separately for each domain in AppState:

```rust
pub struct AppState {
    pub loading_conversations: bool,      // Conversations list
    pub loading_messages: bool,           // Message history
    pub loading_presence: bool,           // Presence updates
    pub loading_authentication: bool,     // Auth operations
}
```

### Loading State Pattern

```slint
// ✅ CORRECT: Per-domain loading indicator
export component ChatScreen {
    in property <AppState> state;
    
    if state.loading-messages {
        LoadingSpinner { }
    } else {
        MessageList { 
            messages: state.messages;
        }
    }
}

// ❌ WRONG: Global loading flag
export component ChatScreen {
    property <bool> is-loading: false;  // ❌ Too vague, what's loading?
}
```

---

## 🧪 Testing Organization

### Test Location

**COLOCATED:** Tests live in the same file as source code, not separate directories:

```rust
// ✅ CORRECT: src/backend/services/message_service.rs
pub fn send_message(msg: Message) -> Result<MessageSentEvent> {
    // implementation
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_send_message_success() {
        // test implementation
    }
}

// ❌ WRONG: Separate tests directory
// tests/services/message_service_test.rs  ❌ Hard to navigate!
```

### Test Types

1. **Unit Tests** (src/backend/**/*_test.rs in module)
   - Fast, isolated, no I/O
   - Mock external dependencies
   - Test one function/behavior

2. **Integration Tests** (tests/integration/**/*.rs)
   - With real systems (database, WebSocket)
   - Full workflows
   - API + backend integration

3. **Contract Tests** (tests/contract/**/*.rs)
   - API schema validation
   - Protocol compliance
   - Message format verification

4. **Load Tests** (tests/load/locustfile.py)
   - Performance benchmarks
   - Stress testing
   - 60+ FPS verification

### Test Naming

- Unit tests: `test_function_name` in module
- Integration tests: `fn test_feature_workflow()` 
- Contract tests: `fn test_message_schema_validation()`

---

## ⚙️ Configuration & Settings

### Settings in AppState

All configuration stored in AppState with getter methods:

```rust
impl AppState {
    pub fn get_message_batch_size(&self) -> usize {
        50  // Messages per fetch
    }
    
    pub fn get_retry_backoff_ms(&self, attempt: u32) -> u64 {
        100 * (2 ^ attempt)  // Exponential backoff
    }
    
    pub fn get_animation_duration(&self) -> Duration {
        Duration::from_millis(200)  // Base duration
    }
}
```

### System Preferences Service

Windows theme integration in `src/backend/services/system_preferences.rs`:

```rust
pub fn detect_windows_theme() -> Theme {
    // Returns "light" or "dark" based on Windows settings
}
```

---

## 🎯 Performance Targets

### Must Achieve

- **60+ FPS** UI rendering (maintained via virtual lists)
- **<100ms** conversation switching (AppState property change only)
- **<2 seconds** startup time (progressive loading)
- **<50ms** message delivery (WebSocket overhead only)

### Implementation Strategies

1. **Virtual Lists** → Render only visible messages
2. **On-Demand Loading** → Load messages when needed (50 at a time)
3. **Progressive Startup** → Load critical features first
4. **Lazy Loading** → Load assets/data on demand
5. **Caching** → Keep current conversation messages cached

---

## 🔒 Security Patterns

### Authentication

- JWT tokens with expiration
- Bcrypt password hashing
- Token refresh endpoint
- Rate limiting on auth endpoints (3 attempts per minute)

```rust
// ✅ CORRECT: Rate-limited auth
pub async fn handle_login(
    rate_limiter: &RateLimiter,
    credentials: LoginCredentials
) -> Result<LoginResponse> {
    rate_limiter.check_limit("login", &user_id)?;
    // authenticate...
}
```

### Middleware

- `middleware/auth.rs` validates JWT on all endpoints
- `middleware/rate_limit.rs` prevents brute force attacks
- Applied to all HTTP routes

```rust
// ✅ CORRECT: Middleware applied
app.route("/messages", post(handle_messages).layer(auth_middleware))
    .layer(rate_limit_middleware);
```

### Protocol Security

- WebSocket over WSS (TLS/SSL)
- Commands signed with user JWT
- Events include user_id for validation

---

## 🚨 Common Mistakes to Avoid

### Frontend

❌ **Hardcoding colors**
```slint
background: #FFFFFF;  // ❌ Use tokens instead!
```
✅ Use design tokens: `background: Tokens.palette-primary-light;`

❌ **Component per feature**
```
components/
  ├── UserProfiles/        // ❌ Wrong organization
  ├── MessageSending/      // ❌ Wrong organization
```
✅ Organize by domain: `components/messaging/`, `components/presence/`

❌ **Separate test files**
```
src/components/Button.slint
tests/components/button_test.slint  // ❌ Wrong!
```
✅ Colocate tests: Tests in same file as component

❌ **Manual state management**
```slint
property <[Message]> local-messages: [];  // ❌ Loses sync!
```
✅ Use reactive AppState binding

### Backend

❌ **Cloning AppState**
```rust
let mut new_state = state.clone();  // ❌ Inefficient!
```
✅ Use mutable references: `fn handle(&mut self, state: &mut AppState)`

❌ **Hardcoding JSON field names**
```rust
pub struct User {
    pub user_id: String,  // ❌ Will serialize as "user_id"!
}
```
✅ Use Serde: `#[serde(rename_all = "camelCase")]`

❌ **Mixing concerns in handlers**
```rust
// handlers/messages.rs
fn send_message() { /* ALL the logic */ }  // ❌ Should call service!
```
✅ Keep handlers thin, put logic in services:
```rust
// handlers/messages.rs
fn handle_send_message(service: &MessageService, cmd: Command) {
    service.send_message(cmd)?;
}

// services/message_service.rs
fn send_message(&self, cmd: Command) { /* business logic */ }
```

❌ **Throwing errors instead of events**
```rust
return Err("Network error".to_string());  // ❌ Wrong!
```
✅ Send as event: `{ event: "Error", payload: { ... } }`

❌ **Global flags for loading state**
```rust
pub loading: bool,  // ❌ What's loading?
```
✅ Per-domain flags: `pub loading_messages: bool, pub loading_presence: bool`

### Both

❌ **Vague naming**
```rust
fn process_data() { }       // ❌ What data?
fn handle_thing() { }       // ❌ What thing?
```
✅ Specific naming: `fn handle_send_message_command()`, `fn parse_websocket_message()`

❌ **No examples in code**
```rust
// TODO: implement this
```
✅ Provide concrete examples showing correct patterns

---

## 🔗 Dependencies & Versions

### Backend

- **Rust:** 1.75+ (stable)
- **Tokio:** 1.x (async runtime)
- **Warp:** 0.3+ (web framework)
- **Tungstenite:** 0.21+ (WebSocket)
- **Serde:** 1.0+ (serialization)
- **SQLite:** Embedded (via rusqlite or sqlx)
- **Bcrypt:** For password hashing

### Frontend

- **Rust:** 1.75+ (stable)
- **Slint:** Latest stable (UI framework)
- **Tokio:** 1.x (async runtime for services)
- **Serde:** 1.0+ (serialization)

### Shared

- **Serde:** 1.0+
- **Tokio:** 1.x
- **Uuid:** For unique IDs

---

## 📋 Implementation Checklist

### Before Starting

- [ ] Read complete architecture document
- [ ] Understand all 9 core decisions
- [ ] Learn all 9 mandatory patterns
- [ ] Review project structure layout
- [ ] Check naming conventions
- [ ] Review this file for quick reference

### While Implementing

- [ ] Follow domain-based organization
- [ ] Use design tokens (never hardcode colors)
- [ ] Use centralized AppState pattern
- [ ] Apply retry logic to network calls
- [ ] Track loading states per-domain
- [ ] Use error-as-event pattern
- [ ] Organize tests colocated
- [ ] Follow naming conventions exactly
- [ ] Use Serde with camelCase
- [ ] Maintain 60+ FPS performance

### Before Submitting

- [ ] All components use design tokens
- [ ] All WebSocket messages follow protocol
- [ ] All errors are communicated as events
- [ ] All tests are colocated and passing
- [ ] All naming follows conventions
- [ ] Code follows all patterns exactly
- [ ] No hardcoded values (use AppState/config)
- [ ] Performance targets met (60+ FPS, <100ms switching)

---

## 🎓 Key Files to Reference

| File | Purpose |
|------|---------|
| `/docs/architecture.md` | Complete architectural decisions & patterns |
| `src/frontend/design/tokens.slint` | Design system (single source of truth) |
| `src/shared/protocol/mod.rs` | WebSocket command/event definitions |
| `src/backend/handlers/dispatcher.rs` | WebSocket message routing |
| `src/backend/services/` | Business logic for each domain |
| `/tests/integration/` | Integration test examples |

---

## 🤝 Questions & Clarifications

### Q: Should I read the full architecture document?
**A:** Yes, at least once. Reference it often during implementation.

### Q: What if I find a conflict in the architecture?
**A:** Update this file and the architecture document to clarify. All implementations should reference the latest version.

### Q: What if performance targets aren't met?
**A:** Check: 1) Virtual lists enabled? 2) On-demand loading? 3) Progressive startup? Refer to architecture section 7-8.

### Q: How do I add new components?
**A:** 1) Create file in correct domain (src/frontend/components/domain/) 2) Use PascalCase filename 3) Use design tokens 4) Follow component patterns

### Q: Can I deviate from the patterns?
**A:** No. Patterns are mandatory to prevent conflicts. If a pattern doesn't work, update the architecture and this file, then notify the team.

---

## ✅ You're Ready!

This project context provides everything needed for consistent implementation following your architecture.

**Start with:** Reading the complete architecture document, then reference this file for quick decisions while coding.

**Questions about specifics:** Check the architecture document for complete explanations and examples.

**Implementation ready:** Follow all patterns exactly, use this file as a quick reference, and maintain consistency across all components.

---

**Project Status:** ✅ Architecture Complete, Implementation Ready

**Last Updated:** 2025-12-17  
**Architecture Version:** Complete (8/8 steps)  
**Validation:** 100% pass (all requirements covered, zero critical gaps)

