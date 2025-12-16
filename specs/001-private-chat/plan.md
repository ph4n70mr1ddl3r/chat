# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]
**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build a secure, real-time private chat application with one-to-one conversations only. Users create accounts with JWT-based authentication, communicate via WebSocket with automatic offline message queuing (indefinite retry with exponential backoff), and have access to persistent conversation history. Backend: Rust with WebSocket server. Frontend: Rust + Slint native desktop for Windows. Database: SQLite (MVP) → PostgreSQL (production). Target: 100 msg/sec throughput, sub-2 second latency, 99% uptime, up to 10k concurrent users.

## Technical Context

**Language/Version**: Rust 1.75+ (stable) - both backend and frontend
**Primary Dependencies**: 
  - Backend: tokio (async runtime), tungstenite (WebSocket), serde (serialization), sqlx (database), jsonwebtoken (JWT)
  - Frontend: Slint (UI framework), tokio (async runtime), tungstenite (WebSocket client)
**Storage**: SQLite for MVP (single-server MVP deployment), with migration path to PostgreSQL for production scaling
**Testing**: cargo test (unit + integration testing); Rust native testing framework
**Target Platform**: Backend: Linux server (self-hosted single-server); Frontend: Windows native binary (Rust + Slint)
**Project Type**: Multi-project with shared contracts (Backend API server + Desktop GUI frontend)
**Performance Goals**: 100 messages/sec peak throughput; sub-2 second latency for online delivery; support 10,000 concurrent users
**Constraints**: 5000 character max message length; rate-limiting: 5 failed logins per IP per 15 minutes; 99% uptime target; WebSocket automatic reconnection with idempotent message IDs
**Scale/Scope**: 10,000 concurrent users; ~1,000 messages/conversation average; 10 conversations/user average; 250 bytes/message typical; no per-user quota; disk utilization alert at 80%

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**I. Library-First**: ✅ PASS
- Backend exposes core chat functionality as library (message delivery, authentication, conversation management)
- Frontend Slint application consumes the client library via shared WebSocket contract
- Both are independently testable and have clear boundaries

**II. CLI Interface**: ⚠️ CONDITIONAL PASS (requires implementation)
- Backend: CLI endpoint for admin operations (user management, message inspection, server health)
- Frontend: Desktop GUI (CLI not applicable for Slint native app; satisfies intent via graphical interface)
- Shared contract: JSON protocol over WebSocket for both CLI and GUI clients

**III. Test-First (NON-NEGOTIABLE)**: ✅ PASS - Commitment required
- Unit tests for all business logic (authentication, message validation, conversation rules, delivery retry logic)
- Integration tests for WebSocket contract compliance and message delivery flow
- Tests must be written before implementation begins

**IV. Integration Testing**: ✅ PASS - Commitment required
- Contract tests: WebSocket message schema validation (server ↔ client, offline queuing)
- Inter-service tests: Authentication → Message Delivery → Storage flow
- Shared schema: `message.json`, `auth.json`, `conversation.json` in contracts/

**V. Observability**: ✅ PASS - Commitment required
- Structured JSON logging: authentication events, message delivery state transitions, connection lifecycle
- Metrics: latency histograms (send → deliver), message throughput, error rates, queue depth
- Centralized error tracking for failed deliveries, auth violations, storage failures
- **Implementation approach**: 
  - MVP: File-based JSON logs in `./logs/` directory (rotation via logrotate or app-level rotation)
  - Production: Can integrate with external service (Datadog, Sentry, etc.) via adapter pattern without MVP complexity
  - Error tracking includes: stack traces, request context, user ID (if applicable), timestamp, severity level
  - Failed message delivery logged with: message ID, recipient ID, retry attempt count, last error reason
  - Retention: Keep 30 days of logs on disk; archive to cold storage if needed

**VI. Versioning & Breaking Changes**: ✅ PASS - Commitment required
- MAJOR.MINOR.BUILD versioning for WebSocket protocol
- Contract versioning in `/contracts/` directory
- Migration guide for protocol updates

**VII. Simplicity**: ✅ PASS - YAGNI enforced
- No group chat; no edit/delete; no file sharing; no external notifications
- Single server deployment (no distributed consensus complexity)
- Append-only message model (no transaction complexity)

**Summary**: All gates PASS. Feature is aligned with constitution. Ready for Phase 0 research.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
# Selected: Option 2 - Backend + Desktop Frontend (Multi-project)

src/
├── backend/
│   ├── lib.rs                  # Core chat library (auth, messaging, storage)
│   ├── models/                 # User, Message, Conversation, Session
│   ├── services/               # AuthService, MessageService, ConversationService
│   ├── handlers/               # WebSocket message handlers
│   ├── db/                     # Database migrations and queries
│   ├── cli/                    # Server CLI (admin operations, diagnostics)
│   └── main.rs                 # Server entry point
│
├── frontend/
│   ├── lib.rs                  # Frontend library (UI components, WebSocket client)
│   ├── components/             # Slint UI components (login, chat, user list)
│   ├── screens/                # Screen layouts (LoginScreen, ChatScreen)
│   ├── services/               # WebSocket client service, auth client
│   └── main.rs                 # Desktop GUI entry point
│
├── shared/
│   ├── lib.rs                  # Shared types and utilities
│   ├── protocol/               # WebSocket message types, contract definitions
│   └── errors/                 # Shared error types
│
tests/
├── contract/                   # WebSocket protocol contract tests
│   └── message_schema_test.rs  # Validates message/auth/conversation schemas
├── integration/                # Backend + Frontend integration tests
│   └── message_delivery_test.rs # End-to-end offline delivery with reconnection
└── unit/                       # Backend unit tests (business logic)
    └── auth_test.rs
    └── message_validation_test.rs
    └── conversation_rules_test.rs

Cargo.workspace.toml            # Workspace configuration for backend, frontend, shared
```

**Structure Decision**: Multi-project workspace (3 sub-crates: `backend`, `frontend`, `shared`) with shared contracts. This enables:
- Shared protocol definitions without tight coupling
- Independent compilation and testing of frontend/backend
- Clear separation of concerns (UI vs. business logic vs. communication protocol)
- Easy migration of communication layer (WebSocket → gRPC if needed)

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
