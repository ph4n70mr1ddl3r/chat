# Feature Specification: Private Chat Application

**Feature Branch**: `001-private-chat`  
**Created**: Mon Dec 15 2025  
**Status**: Draft  
**Input**: User description: "build a chat application that allows users to chat privately"

## Clarifications

### Session 2025-12-15

- Q: Which real-time protocol should the message delivery system use? → A: WebSocket (persistent connection; bidirectional; standard for real-time chat)
- Q: Which authentication method should the system use for session management? → A: JWT tokens (stateless; scalable; standard for modern SPAs)
- Q: Are chats strictly one-to-one or can users start multiple threads with the same person? → A: Strict one-to-one (only ONE active conversation between any two users at a time)
- Q: Should there be a per-user message storage quota or retention policy? → A: Unlimited retention indefinitely per conversation
- Q: How should the system handle message delivery for offline users? → A: Indefinite retry with exponential backoff via an offline message queue until user comes online or deletes account
- Q: What is the expected message throughput (messages per second) the system should handle at peak load? → A: 100 messages/sec
- Q: What is the maximum message length (in characters) a single message can be? → A: 5000 characters
- Q: What password strength requirements should be enforced? → A: Minimum 8 characters, at least one uppercase, one lowercase, one digit
- Q: How long should the system retry delivering a message to an offline user before giving up? → A: Indefinitely until user comes back online or deletes account
- Q: What should happen when a user deletes their account? → A: Anonymize (messages remain but sender info replaced with "Deleted User"; account cannot be reactivated)
- Q: When a user sends a message and it's queued for offline delivery, what should the UI display? → A: Show a "pending" indicator (e.g., clock icon or spinner) while message awaits delivery; change to "delivered" checkmark once confirmed
- Q: What observability and monitoring strategy should the system implement for production? → A: Basic operational metrics (latency histograms, message throughput, error rates) + structured logs (auth events, message delivery state transitions) + centralized error tracking
- Q: Which database system should the system use for data persistence? → A: SQLite for MVP, plan PostgreSQL migration for production at scale
- Q: What security threats should the system defend against, and what rate-limiting is required? → A: Brute-force login attacks, password reuse, SQL injection; implement 5 failed login attempts per IP per 15 minutes rate-limiting
- Q: How should the system handle message metadata when a user account is deleted? → A: Add `deleted_at` timestamp and `is_anonymized` boolean flag to messages table; anonymization applied at deletion time (not query time); anonymized messages display only content, timestamp, and "Deleted User" label
- Q: What are the expected storage and memory scaling constraints for the system? → A: Assume ~1,000 messages/conversation, 10 conversations/user average, 250 bytes/message; no per-user storage quota; production alert threshold at 80% disk capacity utilization
- Q: What frontend framework and deployment target should the client application use? → A: Rust + Slint for Windows binary executable (native desktop GUI; strong type safety; cross-platform potential)
- Q: Can users edit or delete individual messages after sending? → A: No; messages are immutable after send (no edit/delete; simplifies data model; preserves conversation continuity and audit trail)
- Q: What is the deployment and hosting model for the system? → A: Self-hosted on single server (full control; lower cost; manual scaling; operational responsibility)
- Q: How should the system notify offline users of incoming messages? → A: In-app notifications only (message visible in chat window when user opens app; no external email/SMS alerts)
- Q: Is the one-to-one conversation constraint permanent or a temporary MVP limitation? → A: Permanent (no group chat; application scope fixed to private bilateral conversations only)

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
  
  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - User Registration and Account Creation (Priority: P1)

A new user can create an account with a username and password to access the chat application. This is the foundational prerequisite for using the platform.

**Why this priority**: Without account creation, users cannot access any chat features. This is the critical first step for any new user.

**Independent Test**: Can be fully tested by creating an account and verifying the user can log in successfully.

**Acceptance Scenarios**:

1. **Given** a user is on the signup page, **When** they enter a unique username and password meeting strength requirements (min 8 chars: uppercase, lowercase, digit), **Then** their account is created successfully
2. **Given** a user enters a username that already exists, **When** they attempt to create an account, **Then** they see an error message and account creation fails
3. **Given** a user enters an invalid password (too short or missing required character types), **When** they attempt to create an account, **Then** they see validation feedback

---

### User Story 2 - User Authentication and Login (Priority: P1)

A registered user can log in with their credentials to access their account and chat history. This ensures secure access to the application.

**Why this priority**: Critical for protecting user data and ensuring only authorized users access the application. Tied directly with account creation as part of core authentication flow.

**Independent Test**: Can be tested by logging in with valid credentials and confirming access to the chat interface.

**Acceptance Scenarios**:

1. **Given** a registered user with valid credentials, **When** they enter their username and password, **Then** they are logged in and see the chat interface
2. **Given** a user enters incorrect credentials, **When** they attempt to log in, **Then** they see an authentication error
3. **Given** a user is logged in, **When** they close and reopen the application, **Then** their session persists or they remain logged in

---

### User Story 3 - Start a Private One-on-One Chat (Priority: P1)

A logged-in user can initiate a private conversation with another specific user by selecting or searching for them. This is the core feature of the application.

**Why this priority**: This is the primary user interaction—starting a private chat. Without this, the app has no value.

**Independent Test**: Can be tested by starting a chat with another user and verifying the conversation interface appears correctly.

**Acceptance Scenarios**:

1. **Given** two registered users are logged in, **When** User A searches for and selects User B, **Then** a private chat window opens between them
2. **Given** User A initiates a chat with User B, **When** User B is online, **Then** User B sees a notification or indication of the new chat
3. **Given** a user is viewing their chats list, **When** they click on an existing chat, **Then** the conversation history loads

---

### User Story 4 - Send and Receive Messages (Priority: P1)

Users can send text messages to their private chat partner and receive messages in real-time. The conversation history is preserved.

**Why this priority**: This is the core functionality—without the ability to send and receive messages, there is no chat application. This is what users need the app for.

**Independent Test**: Can be tested by sending a message from one user and verifying it appears in the other user's chat window and is saved to history.

**Acceptance Scenarios**:

1. **Given** two users are in an active private chat, **When** User A types and sends a message, **Then** the message appears instantly in User B's chat window
2. **Given** a message is sent, **When** the recipient receives it, **Then** the message includes a timestamp and sender information
3. **Given** a user closes and reopens a chat conversation, **When** the conversation loads, **Then** previous messages are visible in chronological order
4. **Given** a user sends a message, **When** the recipient is offline, **Then** the message is stored and delivered when they come online

---

### User Story 5 - View Online Status and Availability (Priority: P2)

Users can see whether their chat partners are currently online or offline. This helps manage expectations about response times.

**Why this priority**: Enhances user experience by providing context about availability, but not essential for basic messaging functionality.

**Independent Test**: Can be tested by checking presence indicators update when users log in/out.

**Acceptance Scenarios**:

1. **Given** a user is in a private chat, **When** they view the conversation, **Then** they see the current online/offline status of their chat partner
2. **Given** a chat partner goes online, **When** the status updates, **Then** the online indicator changes in real-time for the user

---

### User Story 6 - Search Chat History (Priority: P2)

Users can search through their previous messages to quickly find past conversations or specific information shared in chats.

**Why this priority**: Improves usability for users with long chat histories, but not critical for basic chat functionality.

**Independent Test**: Can be tested by searching for a keyword and verifying matching messages are returned.

**Acceptance Scenarios**:

1. **Given** a user has messages in a private chat, **When** they search for a keyword, **Then** relevant messages are highlighted or displayed
2. **Given** no messages match the search, **When** a user performs a search, **Then** they see a "no results" message

---

### User Story 7 - Logout and Session Management (Priority: P2)

Users can safely log out of their account and terminate their session. This ensures security and privacy.

**Why this priority**: Important for security but typically a secondary feature after core chat functionality.

**Independent Test**: Can be tested by logging out and verifying the user is redirected to the login page.

**Acceptance Scenarios**:

1. **Given** a user is logged in, **When** they select logout, **Then** their session ends and they are redirected to the login page
2. **Given** a user has logged out, **When** they try to access chat without logging in again, **Then** they are redirected to the login page

---

### Edge Cases

- What happens when a user tries to start a chat with themselves? **Resolution**: System MUST prevent users from initiating conversations with their own account (validation rule on conversation creation).
- How does the system handle messages sent while the recipient is offline? **Resolution**: Messages are stored and retried indefinitely with exponential backoff until the recipient comes online or deletes their account; no deadline-based expiry (message is never discarded unless user account is deleted).
- What occurs if the connection drops mid-message transmission? **Resolution**: WebSocket reconnection is automatic; idempotent message IDs prevent duplicates on retry.
- How are messages handled if a user is blocked or deletes their account? **Resolution**: When a user deletes their account, their account is marked for permanent deletion and cannot be reactivated. All messages from that user remain in conversation history but sender information is anonymized (replaced with "Deleted User"). The other chat participant can still view the conversation history with anonymized messages.
- What happens if two users try to send messages simultaneously? **Resolution**: All messages have unique IDs and timestamps; no special conflict resolution needed (messages are append-only).
- How should the UI signal message delivery status? **Resolution**: Messages show a "pending" indicator (e.g., clock icon or spinner) while awaiting delivery to an offline recipient. Once the recipient comes online and the message is delivered, the indicator changes to a "delivered" checkmark. This provides users with clear feedback about message status without requiring explicit retry UI.
- How does the system handle very long messages or special characters? **Deferred to planning phase** (requires max message length and character encoding specification).
- Can users edit or delete sent messages? **Resolution**: Messages are immutable after send. No edit or delete functionality is provided. This simplifies the data model, preserves conversation continuity, and maintains a clear audit trail.
- How are offline users notified of incoming messages? **Resolution**: Notifications are in-app only. When a user opens the application, they will see any pending messages in the chat interface. No external notifications (email, SMS, desktop toasts) are sent while the user is offline.

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: System MUST allow users to create an account with a unique username and secure password (minimum 8 characters including uppercase, lowercase, and digit)
- **FR-002**: System MUST authenticate users with their credentials and maintain secure sessions using JWT tokens
- **FR-003**: System MUST allow authenticated users to search for and select other users to start a private chat
- **FR-004**: System MUST enable users to send text messages (max 5000 characters) in a private chat conversation via WebSocket connection
- **FR-005**: System MUST deliver messages to the recipient in real-time via WebSocket if they are online, or queue them in an offline message queue for indefinite retry with exponential backoff when they come online (no expiry; messages remain queued until user comes online or deletes account)
- **FR-006**: System MUST preserve and display complete message history for each private conversation
- **FR-007**: System MUST show timestamps and sender information for each message
- **FR-008**: System MUST display online/offline status for chat partners with real-time updates; status changes must propagate to conversation participants within 1 second of login/logout
- **FR-009**: System MUST allow users to search through their message history by keywords
- **FR-010**: System MUST allow users to log out and terminate their session securely
- **FR-011**: System MUST enforce authorization checks on all conversation queries. Users can only retrieve conversations where they are one of two participants. Unauthorized users receive 403 Forbidden response.
- **FR-012**: System MUST store and protect user data according to privacy standards. Specifically: (a) passwords hashed and salted using bcrypt; (b) JWT tokens used for session authentication (1-hour expiration); (c) MVP deployment uses plaintext SQLite stored locally on single server (no cloud); (d) Production deployments MUST enable full-disk encryption (e.g., LUKS on Linux, BitLocker on Windows). See T138_A for encryption implementation scope.
- **FR-013**: System MUST reject messages exceeding 5000 characters with a user-facing validation error
- **FR-014**: System MUST support account deletion; deleted accounts cannot be reactivated
- **FR-015**: System MUST anonymize messages from deleted users (replace sender info with "Deleted User") while preserving message content for conversation continuity
- **FR-016**: System MUST display message delivery status indicators (pending/clock icon while queued for offline delivery; checkmark once confirmed delivered)
- **FR-017**: System MUST enforce rate-limiting on authentication endpoints: maximum 5 failed login attempts per IP address per 15-minute window

### Key Entities

- **User**: Represents a person using the application with attributes including username, password (hashed/salted; minimum 8 characters with uppercase, lowercase, and digit), account creation date, and online/offline status
- **Message**: Represents a single message sent in a conversation with attributes including content (max 5000 characters), timestamp, sender, recipient, `deleted_at` (timestamp when sender account was deleted; NULL if sender still active), and `is_anonymized` (boolean flag; true if sender account deleted, false otherwise)
- **Conversation**: Represents a private chat between exactly two users (strict one-to-one), containing all messages exchanged between them. Only one active conversation can exist between any pair of users.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: New users can create an account and log in within 2 minutes
- **SC-002**: Users can start a private chat with another user within 30 seconds of logging in
- **SC-003**: Messages are delivered to online recipients in real-time (under 2 seconds latency)
- **SC-004**: System maintains 99% uptime during operation
- **SC-005**: Users can retrieve their complete message history without delays or data loss
- **SC-006**: 95% of first-time users successfully send their first message without assistance
- **SC-007**: Chat interface loads in under 3 seconds on standard internet connections
- **SC-008**: System securely prevents users from accessing other users' private conversations

### Non-Functional Quality Attributes

- **Performance**: 
  - Message delivery latency: p95 ≤ 1.5s, p99 ≤ 2s (for online recipients)
  - Peak throughput: ≥ 100 messages/sec sustained
  - WebSocket handshake: p95 ≤ 100ms
  - Conversation history query: p95 ≤ 100ms (50-message response)
  - User search query: p95 ≤ 200ms
  - Measurement window: 1-hour rolling window with per-percentile tracking
- **Scalability**: 
  - Designed for 10,000 concurrent users
  - Target throughput: 100 msgs/sec at peak load
  - Storage: ~1,000 msgs/conversation, 10 conversations/user, 250 bytes/msg average
  - No per-user storage quota
  - Operations alert threshold: 80% disk utilization (trigger proactive cleanup/expansion)
  - Database: SQLite (MVP), PostgreSQL (production) with horizontal read replicas
- **Reliability**: 
  - 99% uptime SLA (max 43 minutes downtime/month during operation)
  - Observation window: calendar month
  - Automatic WebSocket reconnection with exponential backoff (0.5s → 30s max)
  - Idempotent message IDs prevent duplicate delivery on retry
  - Message acknowledgment required before marking delivered
- **Observability**: 
  - **Metrics** (emit every 60 seconds):
    - Latency histograms: send → delivery (p50, p95, p99)
    - Message throughput: msgs/sec (current, 5-min rolling avg)
    - Error rate: failed messages/sec
    - Active connections: WebSocket count
    - Queue depth: pending offline messages
  - **Logs** (structured JSON):
    - Authentication events: signup, login, failed attempts, token refresh
    - Message delivery state transitions: pending → sent → delivered / failed
    - Connection lifecycle: connect, disconnect, reconnect, auth handshake
    - Error events: database failures, validation errors, timeout
  - **Centralized tracking**:
    - On-server file-based logs (no external services in MVP)
    - Error aggregation: failed delivery attempts logged with retry count
    - Production: can integrate with external service (e.g., Datadog, Sentry) via adapter
- **Security**: 
  - All passwords hashed (bcrypt) with per-user salt; minimum 8 chars with uppercase, lowercase, digit
  - JWT tokens: 1-hour expiration, RS256 signing (if using public key infrastructure) or HS256 (symmetric for MVP)
  - Private conversations: authorization check enforced on all queries (403 Forbidden for non-participants)
  - Defends against: brute-force login (5 attempts/15min per IP), SQL injection (parameterized queries), password reuse (not allowed across users)
  - Rate limiting: 5 failed logins per IP per 15 minutes; 1000 requests/min per user (other endpoints)
  - CORS: configured to match deployment host only; no wildcard

## Assumptions

- Users have standard internet connectivity (broadband or mobile)
- Users are comfortable with basic authentication (username/password)
- Initial deployment targets a moderate user base (up to 10,000 concurrent users)
- Messages are text-based; no file sharing, images, or rich media in MVP
- Users expect near-instant message delivery; "real-time" is acceptable as sub-2 second latency
- Privacy is a core requirement; all private chats remain between the two participants only
- System will implement standard password security practices (hashing, salting)
- Message retention is unlimited; all messages are stored indefinitely per conversation
- **Data Retention & Archival**:
  - Message retention is unlimited; no automatic deletion unless user account is deleted (soft delete + message anonymization)
  - Disk capacity alert (80%) triggers operational review; consider:
    - Extend storage capacity (add disk)
    - Implement message archival (move old messages to cold storage; SQL view over archived data)
    - Enable database compression (WAL, query optimization)
    - Audit user accounts; clean up deleted accounts if desired (app-level only; data never auto-purged)
  - No per-user storage quota enforced; users can send unlimited messages
- Only one active conversation exists between any pair of users at any time
- **Scope**: Application is permanently limited to one-to-one private conversations. Group chat, channels, or multi-user conversations are explicitly out-of-scope and will not be added in future versions.
- **Data Persistence**: MVP uses SQLite for single-server deployment; production scaling will migrate to PostgreSQL with async replication
- **Technology Stack**: 
  - **Backend**: Rust 1.75+ (stable) with WebSocket server for real-time communication
  - **Frontend**: Rust + Slint for Windows native desktop binary executable
  - **Database**: SQLite (MVP), PostgreSQL (production)
- **Hosting & Deployment**: Self-hosted single-server model (full operational control; manual scaling responsibility; suitable for MVP and initial deployment phases)
