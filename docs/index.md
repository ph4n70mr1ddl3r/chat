# Chat Application - Project Documentation

**Project**: Private Chat Application  
**Type**: Brownfield - Rust backend + Slint desktop frontend  
**Repository**: Rust workspace with 3 crates  
**Generated**: 2025-12-16T08:05:00Z  
**Scan Mode**: Exhaustive  

## Quick Summary

The **chat** application is a Rust-based private messaging system featuring:

- **Backend**: WebSocket-based real-time messaging server with JWT authentication
- **Frontend**: Native desktop UI using Slint framework
- **Database**: SQLite (MVP) with PostgreSQL migration path for production
- **Architecture**: Monorepo workspace with backend, frontend, and shared libraries
- **Status**: Active development with production-ready features

### Key Statistics

| Metric | Value |
|--------|-------|
| **Backend Source Files** | 33 |
| **Backend Lines of Code** | 8,142 |
| **Frontend Source Files** | 26 |
| **Frontend Lines of Code** | 3,766 |
| **Shared Library Files** | 3 |
| **Shared Library Lines** | 272 |
| **Test Files** | 16 |
| **Test Lines of Code** | 3,197 |
| **Total Project Lines** | ~15,377 |

## Project Structure

```
chat/
├── src/
│   ├── backend/        # WebSocket server, auth, handlers
│   ├── frontend/       # Slint UI application
│   └── shared/         # Protocol definitions, common types
├── tests/              # Integration and contract tests
├── specs/              # Feature specifications
└── docs/               # Project documentation
```

## Technology Stack

### Backend
- **Runtime**: Tokio (async)
- **Web**: Warp, Tungstenite (WebSocket)
- **Database**: SQLx with SQLite/PostgreSQL
- **Auth**: JWT tokens, bcrypt password hashing
- **Serialization**: Serde, JSON Schema validation
- **Logging**: Tracing with structured logs

### Frontend
- **Framework**: Slint (version 1.5)
- **Build**: Rust compile to native binary
- **HTTP**: Reqwest for API calls
- **UI**: Declarative .slint components

### Shared
- **Protocol**: Message envelope schemas
- **Errors**: Unified error handling
- **Models**: Common data structures

## Architecture Overview

### Multi-Part Structure

This is a **monorepo** with 3 logical parts:

#### 1. **Backend** - WebSocket Messaging Server
- **Path**: `src/backend/`
- **Entry**: `main.rs` (server binary)
- **Responsibilities**:
  - User authentication and session management
  - WebSocket connection handling
  - Message routing and delivery
  - Database persistence
  - Rate limiting and security

#### 2. **Frontend** - Desktop GUI Application
- **Path**: `src/frontend/`
- **Entry**: `main.rs` (UI application)
- **Responsibilities**:
  - User interface (Slint-based)
  - Connection management to backend
  - Message display and composition
  - User presence and status
  - Cross-platform native UI

#### 3. **Shared** - Protocol Library
- **Path**: `src/shared/`
- **Library**: Common types and schemas
- **Responsibilities**:
  - Message envelope definitions
  - Error types
  - Protocol constants
  - Validation schemas

## Features & Capabilities

### Completed Features
- User registration and login
- JWT-based authentication
- Real-time messaging via WebSocket
- One-to-one private conversations
- Message persistence
- User deletion with message anonymization
- Offline message queueing
- Rate limiting (5 failed logins per 15 min/IP)
- Structured logging and observability
- Contract-based API schemas

### Design Decisions

Key architectural choices:
- **One-to-one conversations**: No group chat (scope constraint)
- **Immutable messages**: No edit/delete after send
- **WebSocket protocol**: Bidirectional real-time communication
- **SQLite MVP**: Path to PostgreSQL for production scale
- **Stateless auth**: JWT tokens (scalable, no session state)
- **Immediate delivery**: Best-effort with offline queue
- **Self-hosted**: Single-server deployment model

## Existing Documentation

### Project Specs (`specs/001-private-chat/`)
- **spec.md** - Complete feature specification with clarifications
- **data-model.md** - Database schema and relationships
- **plan.md** - Implementation strategy
- **research.md** - Domain and technical research
- **contracts/** - API schema contracts
  - `websocket-protocol.md` - WebSocket message format
  - `server-contract.md` - Server API contract

### Operations Docs (`docs/`)
- **API.md** - REST/WebSocket API reference
- **DEPLOYMENT.md** - Deployment procedures and architecture
- **DEPLOYMENT_POSTGRES_MIGRATION.md** - Database migration strategy
- **PRIVACY.md** - Privacy and data handling policies
- **WINDOWS_CLIENT_GUIDE.md** - Windows client setup
- **CODE_COVERAGE_REPORT.md** - Test coverage metrics
- **TROUBLESHOOTING.md** - Common issues and solutions

## Code Organization

### Backend Structure (`src/backend/`)

**Main Entry Points**
- `main.rs` - Server startup and initialization
- `server.rs` - WebSocket server setup
- `lib.rs` - Library exports

**Key Modules**
- `handlers/` - HTTP and WebSocket message handlers
  - `auth_with_rate_limit.rs` - Authentication with rate limiting
  - `message_handler.rs` - Message routing
  - `presence_handler.rs` - User presence tracking
  - `search_handler.rs` - Message search
  - `websocket_handler.rs` - WebSocket connection lifecycle

- `middleware/` - Request processing middleware
  - `auth.rs` - JWT token extraction and validation
  - `cors.rs` - Cross-origin resource sharing
  - `logging.rs` - Request/response logging

- `services/` - Business logic
  - `auth_service.rs` - User authentication
  - `message_service.rs` - Message handling
  - `presence_service.rs` - Presence tracking
  - `search_service.rs` - Message search
  - `user_service.rs` - User management

- `db/` - Database access layer
  - `migrations/` - SQL migration files
  - `connection.rs` - Database connection pooling
  - `schema.rs` - Database schema definitions

- `models/` - Domain models
- `validators/` - Input validation schemas
- `tests/` - Backend tests

**Dependencies**
- Tokio, Warp (async HTTP)
- Tungstenite (WebSocket)
- SQLx (database)
- Jsonwebtoken, Bcrypt (security)
- Serde, Jsonschema (serialization)
- Tracing (observability)

### Frontend Structure (`src/frontend/`)

**Main Entry Points**
- `main.rs` - Application startup
- `lib.rs` - Library exports
- `ui.rs` - UI orchestration
- `ui.slint` - Declarative UI (main layout)

**UI Components** (`components/`)
- `message_bubble.slint` - Individual message display
- `message_input.slint` - Message composition
- `user_list.slint` - User/conversation list
- `chat_screen.slint` - Main chat interface
- `login_screen.slint` - Authentication screen

**Screens** (`screens/`)
- `chat_screen.rs` - Chat screen logic
- `login_screen.rs` - Login logic
- `user_search_screen.rs` - User discovery
- `settings_screen.rs` - Application settings

**Services** (`services/`)
- `http_client.rs` - API communication
- `websocket_service.rs` - WebSocket connection
- `auth_service.rs` - Authentication state
- `message_service.rs` - Message management

**Key Features**
- Native Slint UI (cross-platform capable)
- Async message handling
- Connection persistence
- Real-time message updates

### Shared Library (`src/shared/`)

**Exports**
- `lib.rs` - Public API
- `errors/` - Error types
- `protocol/` - Message envelope and schemas

**Contents**
- `MessageEnvelope` - Standard message format
- `Error` - Unified error handling
- Validation schemas for contracts

## Testing Architecture

### Test Organization (`tests/`)

**Integration Tests**
- `conversation_test.rs` - Multi-user conversations
- `deletion_test.rs` - User deletion and anonymization
- `e2e_test.rs` - End-to-end workflows
- `logout_test.rs` - Session management
- `message_delivery_test.rs` - Message persistence
- `presence_test.rs` - User presence tracking
- `presence_latency_test.rs` - Presence update timing
- `search_test.rs` - Message search functionality
- `user_search_test.rs` - User discovery
- `websocket_handshake_test.rs` - Connection establishment

**Contract Tests**
- `message_schema_test.rs` - Message envelope validation
- `schema_validator.rs` - JSON schema validation

**Unit Tests**
- `message_validation_test.rs` - Input validation
- `models_test.rs` - Domain model behavior
- `property_tests.rs` - Property-based testing

**Performance Tests**
- `load/locustfile.py` - Load testing scenarios
- `performance_test.rs` - Benchmark tests

## Database Schema

### Core Tables
- **users** - User accounts and credentials
- **conversations** - One-to-one conversation metadata
- **messages** - Message content and metadata
- **user_sessions** - Active WebSocket sessions
- **presence** - User online/offline status

### Key Design Features
- Message anonymization on user deletion
- Conversation-scoped message storage
- Session tracking for WebSocket connections
- Presence state machine
- Audit timestamps on all records

## Deployment Configuration

### Current Setup
- **Target**: Windows binary executable
- **Database**: SQLite (MVP)
- **Server**: Single-server self-hosted
- **Observability**: Structured logs + error tracking

### Production Ready
- PostgreSQL migration path documented
- Deployment procedures documented
- Windows client guide provided
- Rate limiting and security measures in place

## Next Steps in BMM Workflow

Based on this documentation, the recommended workflow path is:

1. ✅ **Document Project** (completed - you are here)
2. **Research** (Discovery phase)
   - Domain analysis of chat applications
   - Competitive landscape analysis
   - Technical trends and patterns
3. **PRD** (Planning phase)
   - Define new features or improvements
   - Align with business goals
4. **Create Architecture** (Solutioning phase)
   - Design integration points
   - Specify new system components
5. **Create Epics & Stories** (Solutioning phase)
   - Break into implementable work items
6. **Test Design** (Solutioning phase)
   - Plan testing strategy
7. **Sprint Planning** (Implementation phase)
   - Coordinate team execution

## Key Findings

### Strengths
✅ Well-structured monorepo with clear separation of concerns  
✅ Comprehensive test coverage (16 test files, 3,197 lines)  
✅ Production-ready deployment documentation  
✅ Detailed API contracts and schemas  
✅ Security measures (rate limiting, JWT, password hashing)  

### Areas for Enhancement
- UI/UX improvements through design system
- Additional feature development or refinements
- Performance optimization at scale
- Expanded test automation

## Scan Details

**Scan Level**: Exhaustive  
**Files Analyzed**:
- All Rust source files (`.rs`)
- All Slint UI files (`.slint`)
- All configuration files
- All documentation

**Excluded**:
- Build artifacts (`target/`)
- Dependencies (`Cargo.lock`)
- Generated files

---

*For detailed deep-dives into specific areas, see sub-documentation or run deep-dive mode on specific modules.*
