---
stepsCompleted: [1, 2, 3, 4]
inputDocuments:
  - /home/riddler/chat/docs/prd.md
  - /home/riddler/chat/docs/architecture.md
  - /home/riddler/chat/docs/ux-design-specification.md
---

# chat - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for chat, decomposing the requirements from the PRD, UX Design if it exists, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: Users can view a list of all conversations they are currently participating in
FR2: Users can search for specific conversations by participant name or display name
FR3: Users can filter conversations to show only active (unread) conversations
FR4: Users can pin/favorite specific conversations to keep them at the top of the list
FR5: Users can quickly switch between conversations with a single interaction
FR6: The system preserves scroll position and context when users switch between conversations
FR7: Users can see visual indication of unread vs. read conversations
FR8: Users can view a preview/snippet of the most recent message in each conversation
FR9: Users can compose text messages in a dedicated message input interface
FR10: Users can send a composed message and receive confirmation that it was sent successfully
FR11: Users can see real-time feedback while composing (e.g., character count, send button state)
FR12: Users can insert line breaks within messages (Ctrl+Enter or equivalent)
FR13: The system preserves unsent message text if users navigate away before sending
FR14: The system shows an error message when a user attempts to send while offline (with message text preserved)
FR15: Users can clear the message composition box after a successful send
FR16: The system provides clear visual feedback confirming message delivery
FR17: Users can view an ordered history of messages in a conversation (newest last)
FR18: Users can scroll through message history to view past conversations
FR19: Users can search within a conversation to find specific past messages
FR20: The system displays message metadata (sender name, timestamp) with each message
FR21: Users can see visual distinction between their own messages and received messages
FR22: Users can see when a message has been read by the recipient (read receipts)
FR23: Users can see when a recipient is actively typing in a conversation (typing indicators)
FR24: The system maintains message history durability across application restarts
FR25: Users can see the online/offline status of conversation participants
FR26: The system shows presence status with a visual indicator (green for online, red for offline)
FR27: Users can see presence status in multiple places (conversation list, conversation header, user lists)
FR28: The system indicates away/idle status separately from offline
FR29: Users can see when presence status changes in real-time
FR30: Users can disable presence sharing in settings (post-MVP)
FR31: The system sends presence updates to other users when local user comes online/goes offline
FR32: The system maintains presence consistency across the user's session
FR33: Users can actively manage 5+ conversations simultaneously
FR34: Users can see which conversations have unread messages across all active conversations
FR35: Users can navigate between multiple conversations without losing their place
FR36: The system shows conversation metadata (participant name, last message time) for quick scanning
FR37: Users can see visual indicators distinguishing active conversations from inactive ones
FR38: Users can organize conversations through search, filtering, and pinning
FR39: The system prevents accidental loss of context when switching rapidly between conversations
FR40: Users can view total unread message count across all conversations
FR41: The system displays a clear, always-visible connection status indicator
FR42: Users can see whether the app is Connected, Disconnected, or Connecting
FR43: The system indicates reason for disconnection (no internet, server unavailable, etc.) where available
FR44: Users can manually trigger reconnection when disconnected
FR45: When users trigger reconnection, the system attempts to restore the connection
FR46: Upon successful reconnection, the system syncs any pending state changes
FR47: The system shows clear error messages when send operations fail due to connectivity
FR48: The system queues presence updates for retry when connection is restored
FR49: New users can create an account with minimal friction (< 2 minutes)
FR50: New users can log in after account creation
FR51: The onboarding flow guides users to find their first conversation partner
FR52: The onboarding experience demonstrates the key capabilities (search, message, send)
FR53: New users receive confirmation when they've sent their first message
FR54: The application provides clear explanations for each interface element during onboarding
FR55: Users can skip onboarding steps if they prefer
FR56: The application remembers onboarding completion state per user
FR57: Administrators can view a list of all registered users
FR58: Administrators can search for specific users by name or identifier
FR59: Administrators can view user status and activity information
FR60: Administrators can reset user passwords
FR61: Administrators can deactivate/delete user accounts
FR62: Administrators can view system activity logs
FR63: The system records audit trail of admin actions
FR64: Administrators can access system health and performance metrics
FR65: Support staff can look up user accounts by name or ID
FR66: Support staff can view user conversation history (with appropriate privacy controls)
FR67: Support staff can see message delivery status for specific messages
FR68: Support staff can view user login history and session information
FR69: Support staff can access knowledge base or troubleshooting guides
FR70: The system provides clear error messages that support staff can reference with users
FR71: The system logs errors and exceptions for support investigation
FR72: Support staff can initiate assistance or bug reporting workflows
FR73: The application uses consistent typography across all screens
FR74: The application uses consistent color palette reflecting Fluent Design System
FR75: All buttons have consistent styling and interactive behavior
FR76: All input fields (text boxes, search bars) have consistent appearance and behavior
FR77: All conversation items display with consistent layout and spacing
FR78: All messages display with consistent formatting and styling
FR79: The application uses consistent spacing and padding throughout
FR80: Hover, focus, and active states are consistent across interactive elements
FR81: Users can navigate all core workflows using keyboard only (Tab navigation)
FR82: Users can activate buttons and controls using Enter or Space
FR83: Users can move between conversations using keyboard shortcuts (Tab to cycle)
FR84: Users can send messages using keyboard (Enter to send, Ctrl+Enter for line break)
FR85: The application maintains visible focus indicators for keyboard navigation
FR86: All text meets minimum contrast ratios for accessibility (WCAG AA)
FR87: The application supports screen readers with proper semantic labels
FR88: Users can navigate dialogs and modals using keyboard
FR89: The application runs on Windows 10 and Windows 11
FR90: The application respects the Windows system dark/light theme setting
FR91: The application sends Windows notifications for new messages
FR92: Windows notifications display message preview and sender information
FR93: Clicking a notification brings the application window to focus and shows relevant conversation
FR94: The application window can be resized and repositioned on screen
FR95: The application maintains window state across application restarts
FR96: The application supports standard Windows window controls (minimize, maximize, close)
FR97: The application layout adapts to different window sizes (minimum 640x480)
FR98: Conversation list remains accessible regardless of window width
FR99: Message composition area remains functional at minimum supported size
FR100: Presence indicators remain visible at all window sizes
FR101: Connection status indicator remains visible at all window sizes
FR102: The application prevents UI elements from overlapping or hiding at edge cases
FR103: Scrollbars appear only when content exceeds available space
FR104: All text remains readable at supported window sizes
FR105: The application starts up and becomes ready for use within 2 seconds
FR106: Switching between conversations is instantaneous (< 100ms)
FR107: Messages appear in the conversation immediately after sending (< 500ms)
FR108: Presence updates appear in real-time (< 1 second)
FR109: UI interactions remain responsive during message receiving
FR110: The application handles large message histories (100+ messages) without degradation
FR111: The application continues functioning if backend connection is slow
FR112: The application recovers gracefully from temporary connection loss


### NonFunctional Requirements

NFR1-1a: All user login attempts are authenticated using JWT (JSON Web Tokens) issued by the backend after credential validation
NFR1-1b: Failed login attempts are rate-limited: 5 failed attempts from a single IP address trigger a 15-minute lockout
NFR1-1c: JWT tokens include expiration timestamps; expired tokens automatically require re-authentication
NFR1-1d: User sessions are associated with unique session identifiers stored on the backend
NFR1-1e: All API requests to the backend include valid JWT tokens; requests without tokens are rejected with 401 Unauthorized
NFR1-1f: Users cannot access conversations or messages they are not members of; all conversation queries are filtered by user membership
NFR1-2a: All WebSocket connections to the backend use secure protocols (WSS - WebSocket Secure)
NFR1-2b: WebSocket handshakes validate JWT tokens; unauthenticated connection attempts are rejected
NFR1-2c: Message payloads transmitted over WebSocket include message integrity checks (HMAC or similar)
NFR1-2d: Client-server communication enforces TLS 1.2 minimum (TLS 1.3 preferred for new deployments)
NFR1-2e: All HTTPS/WSS connections use certificates from trusted Certificate Authorities
NFR1-3a: User credentials (passwords) are hashed using industry-standard algorithms (bcrypt, Argon2) and never stored in plaintext
NFR1-3b: Database connections from the application to SQLite include connection validation and error handling to prevent injection attacks
NFR1-3c: Local SQLite database files on the desktop client include file-system level protections (user-level ownership, restricted permissions)
NFR1-3d: Sensitive data in transit between backend and desktop (authentication tokens, message content) may be encrypted at rest in future Phase 2 deployment with full-disk encryption
NFR1-4a: User sessions automatically expire after 30 minutes of inactivity
NFR1-4b: Users can manually logout, which invalidates their current JWT token and session on the backend
NFR1-4c: Multiple simultaneous logins from the same user account are permitted (no single-session limit)
NFR1-4d: Session invalidation (logout, expiry, or lockout) prevents further API access immediately
NFR1-4e: Session state is stored on the backend; the client cannot manipulate or forge session information
NFR1-5a: All user input (usernames, messages, search queries) is validated for type, length, and content before processing
NFR1-5b: Message content is sanitized to prevent XSS (Cross-Site Scripting) attacks when displayed in the UI
NFR1-5c: Database queries use parameterized statements (prepared statements) to prevent SQL injection
NFR1-5d: API endpoints validate request payloads against defined schemas; malformed requests are rejected with 400 Bad Request
NFR1-5e: File uploads (if implemented) are validated for file type, size, and content scanning
NFR1-6a: Passwords must be at least 8 characters in length
NFR1-6b: Passwords should include a mix of uppercase, lowercase, numbers, and special characters (recommended but not enforced in MVP)
NFR1-6c: Passwords are never transmitted in plaintext; only hashes are stored server-side
NFR1-6d: Users cannot reuse the same password for at least 5 previous password changes
NFR1-7a: All API endpoints require valid JWT authentication tokens
NFR1-7b: API rate limiting enforces per-user request quotas to prevent brute-force attacks and resource exhaustion
NFR1-7c: API responses do not leak sensitive information (e.g., full error stack traces, internal system details)
NFR1-7d: CORS (Cross-Origin Resource Sharing) is restricted to approved frontend domains only
NFR1-7e: API endpoints log failed authentication attempts and suspicious activity for monitoring
NFR1-8a: Error messages displayed to users are user-friendly and do not reveal internal system details
NFR1-8b: Security-sensitive errors (authentication failures, authorization denials) are logged server-side but not detailed to the client
NFR1-8c: Application crash logs do not contain sensitive data (passwords, tokens, personal information)
NFR1-8d: Debug logs are not enabled in production deployments
NFR1-9a: All external dependencies (Rust crates, libraries) are managed via dependency lock files (Cargo.lock)
NFR1-9b: Regular security audits of dependencies are performed to identify known vulnerabilities
NFR1-9c: Vulnerable dependencies are patched or replaced within 30 days of notification
NFR1-10a: User data is retained only as long as necessary for service delivery
NFR1-10b: Users can request deletion of their account and all associated data
NFR1-10c: Private conversation data is never shared with third parties without explicit user consent
NFR1-10d: Privacy policy is clearly communicated and accessible to all users
NFR2-1a: Application startup time from launch to ready-for-use state is ≤ 2 seconds on systems with typical hardware (Intel i5-8th gen or equivalent, 8GB RAM)
NFR2-1b: Initial UI rendering (login screen or conversation list if already authenticated) completes within 1 second
NFR2-1c: Network connection is established to the backend within 1.5 seconds of startup
NFR2-1d: Previously cached user data (conversations, recent messages) is loaded from local database before fetching updates from backend
NFR2-2a: Conversation switching (list click → message history display) completes within 100ms (perceived instant)
NFR2-2b: Message composition input (typing) responds to keyboard input with < 50ms latency
NFR2-2c: Button clicks and menu interactions provide visual feedback within 100ms
NFR2-2d: Scrolling through message history is smooth (60 FPS, no frame drops)
NFR2-2e: UI remains responsive while receiving incoming messages or presence updates
NFR2-2f: Window resizing and layout reflow adapt smoothly without UI freezing
NFR2-3a: Message sent by user appears in local UI within 500ms (immediate local display)
NFR2-3b: Incoming messages from other users appear in the conversation UI within 1 second of server transmission
NFR2-3c: Presence updates (user online/offline status) appear in the UI within 1 second
NFR2-3d: Typing indicators (when implemented) appear and disappear within 500ms
NFR2-4a: Application memory footprint at steady state is ≤ 300MB on typical systems
NFR2-4b: Memory usage remains stable when message history exceeds 1,000+ messages per conversation
NFR2-4c: Application does not leak memory during extended sessions (24+ hours of continuous use)
NFR2-4d: CPU usage remains < 5% during idle state (no user interaction, backend connection active)
NFR2-4e: CPU usage remains < 15% during normal operation (active messaging, presence updates, UI interaction)
NFR2-5a: Local SQLite database queries (message history retrieval, conversation list) complete within 100ms for typical database sizes (50+ conversations, 10,000+ messages)
NFR2-5b: Database is indexed on frequently queried columns (conversation_id, user_id, timestamp) for fast retrieval
NFR2-5c: Database file size grows predictably with message count (~1KB per message for typical content)
NFR2-5d: Database vacuum operations do not block UI interactions; performed asynchronously if needed
NFR2-6a: WebSocket connections remain open and reuse connections (no unnecessary reconnections)
NFR2-6b: Message payloads are optimized to minimize bandwidth (compression, binary protocols where applicable)
NFR2-6c: Initial conversation list load transmits only essential data (conversation ID, name, last message preview, unread count)
NFR2-6d: Presence updates are throttled to prevent excessive traffic (max 1 update per second per user)
NFR2-7a: Rendering of conversation list with 100+ items completes within 200ms
NFR2-7b: Rendering of message history with 500+ messages completes within 300ms (virtualized/lazy loading)
NFR2-7c: UI maintains 60 FPS during smooth scrolling and animations
NFR2-7d: Slint framework rendering does not exceed GPU or CPU limits on lower-end hardware
NFR2-8a: Backend API responses for conversation list queries complete within 500ms under normal load
NFR2-8b: Backend API responses for message history queries complete within 1 second for typical conversation sizes (100+ messages)
NFR2-8c: WebSocket message relay from backend to all conversation members completes within 500ms
NFR3-1a: System is designed to support 10,000 Monthly Active Users (MAU) without functional degradation
NFR3-1b: System is designed to support 2,000 Daily Active Users (DAU) concurrently online without performance degradation
NFR3-1c: Single-server deployment (SQLite) is viable for up to 2K DAU; scaling beyond requires PostgreSQL migration
NFR3-1d: User database records (profiles, settings) scale linearly with user growth; no architectural bottlenecks expected at 100K+ users
NFR3-2a: Backend can process and store 500,000+ messages/day (average for 2K DAU × 10 messages/user × 25 conversations)
NFR3-2b: Message delivery throughput is ≥ 100 messages/second at peak load
NFR3-2c: Message history queries support conversations with 100,000+ messages without performance degradation
NFR3-2d: Message search queries (when implemented) complete within 2 seconds on full conversation history
NFR3-3a: Backend WebSocket server accepts and maintains 2,000 simultaneous WebSocket connections
NFR3-3b: Connection state is efficiently managed; idle connections do not consume excessive resources
NFR3-3c: Connection pool on backend is sized to handle 2K concurrent users + 20% headroom (2,400 total)
NFR3-4a: Users can participate in 100+ simultaneous conversations without UI or backend performance degradation
NFR3-4b: Conversation list displays smoothly even with 200+ conversations (via pagination or virtualization)
NFR3-4c: Creating and deleting conversations scales linearly up to 1,000 conversations per user
NFR3-5a: SQLite database can grow to 10GB+ without performance degradation (single-server MVP limit)
NFR3-5b: Beyond 10GB, migration to PostgreSQL is recommended for horizontal scalability
NFR3-5c: Backup and restore operations on 10GB database complete within 30 minutes
NFR3-5d: Database replication (future Phase 2) enables geographic distribution for low-latency access
NFR3-6a: Backend architecture (Tokio/Warp) supports vertical scaling (multi-core CPU utilization)
NFR3-6b: Client application is stateless and can scale horizontally with load balancing
NFR3-6c: Session state is managed server-side, enabling stateless client scaling
NFR3-6d: Future Phase 2 can add database replicas and read-only followers without client changes
NFR3-7a: Frequently accessed data (user profiles, conversation metadata) is cached in memory on backend (TTL: 5 minutes)
NFR3-7b: Client-side caching of message history reduces backend query load on repeated access
NFR3-7c: Cache invalidation is triggered by real-time updates (WebSocket events) to maintain data freshness
NFR3-8a: System undergoes load testing for 1.5x peak capacity (3K concurrent users) to validate headroom
NFR3-8b: Load testing verifies no message loss, latency degradation, or connection failures under stress
NFR3-8c: Load testing simulates realistic message patterns (bursts, idle periods, presence updates)
NFR3-8d: System is designed to gracefully degrade under overload (queuing, prioritization) rather than failing
NFR4-1a: All core workflows are completable using keyboard only (no mouse required)
NFR4-1b: Tab key navigates through all interactive elements in logical reading order
NFR4-1c: Shift+Tab reverses navigation order
NFR4-1d: Enter or Space activates buttons and links
NFR4-1e: Arrow keys navigate within lists, dropdowns, and radio button groups
NFR4-1f: Escape key closes dialogs, modals, and menus
NFR4-1g: Focus indicators are always visible, with sufficient contrast (≥ 3:1)
NFR4-2a: Tab switches between next/previous conversation
NFR4-2b: Shift+Tab switches between previous/next conversation
NFR4-2c: Enter sends message composition (Ctrl+Enter creates new line in message body)
NFR4-2d: Ctrl+A selects all text in message composition area
NFR4-2e: Ctrl+Z/Ctrl+Y undo/redo in message composition
NFR4-2f: Alt+Tab (OS standard) switches application windows
NFR4-3a: All text labels are associated with form inputs (semantic HTML/accessibility framework)
NFR4-3b: Button purposes are clearly described (not just "Click here")
NFR4-3c: Conversation list items are announced with sender name, message preview, and unread status
NFR4-3d: Message timestamps are accessible (e.g., "3:45 PM" announced, not hidden in hover)
NFR4-3e: Icons are paired with text labels or have alt-text descriptions
NFR4-3f: Dialogs are announced when opened; focus moves to dialog content
NFR4-4a: All text meets WCAG AA contrast ratio of ≥ 4.5:1 for normal text, ≥ 3:1 for large text (18pt+)
NFR4-4b: Status indicators do not rely solely on color (e.g., online/offline uses color + icon or text)
NFR4-4c: Error messages and warnings are not conveyed by color alone; include text or icons
NFR4-4d: Links are distinguishable from body text by color + underline (not color alone)
NFR4-4e: Text is readable on both light and dark backgrounds
NFR4-5a: Application supports text resizing up to 200% without loss of functionality
NFR4-5b: Layout reflows smoothly when text is enlarged; no content is cut off or hidden
NFR4-5c: Horizontal scrolling is not required when text is enlarged at 200% (except for pre-formatted code)
NFR4-5d: Buttons, form fields, and interactive elements scale proportionally with text size
NFR4-6a: Animations are subtle and non-distracting (recommended duration: 200-300ms)
NFR4-6b: Users can disable animations if needed (future Phase 2 OS accessibility setting support)
NFR4-6c: No content moves or auto-plays without user control (unless essential to functionality)
NFR4-7a: Focus is visible on all interactive elements
NFR4-7b: Focus order follows logical reading order (left-to-right, top-to-bottom)
NFR4-7c: Focus does not become trapped; users can tab out of any control
NFR4-7d: When dialogs open, focus moves to the dialog; when closed, focus returns to the triggering control
NFR4-8a: Error messages are clear and specific (not just "Error")
NFR4-8b: Error messages suggest how to fix the problem
NFR4-8c: Form validation errors are associated with the relevant input field
NFR4-8d: Help text is available for complex workflows (e.g., conversation creation, settings)
NFR4-9a: Application scales to minimum 640x480 without loss of core functionality
NFR4-9b: Touch targets are at least 44x44 pixels (recommended) or 48x48 pixels (optimal)
NFR4-9c: Text remains readable at all supported screen sizes
NFR4-10a: Application meets WCAG 2.1 Level AA accessibility standards
NFR4-10b: Automated accessibility scanning (Axe, WAVE) finds no Level AA violations
NFR4-10c: Manual testing with screen readers (NVDA, JAWS) validates user workflows
NFR4-10d: Testing with keyboard only validates all features are accessible without mouse
NFR5-1a: Application detects network disconnections within 5 seconds
NFR5-1b: Upon disconnection, user is shown a clear status indicator ("Reconnecting..." or "Offline")
NFR5-1c: Application automatically attempts to reconnect to backend (exponential backoff: 1s, 2s, 4s, 8s, max 30s)
NFR5-1d: Manual "Reconnect" button is provided in offline state for user control
NFR5-1e: Conversations and messages sent during disconnection are queued locally and resent when connection is restored
NFR5-2a: Messages sent by user during disconnection are stored locally (offline queue)
NFR5-2b: Queue persists across application restarts (survives app crash or forced shutdown)
NFR5-2c: Max queue size is 100 messages; oldest messages are dropped if exceeded
NFR5-2d: Queued messages are automatically sent in order upon reconnection
NFR5-2e: User is notified if message send fails after multiple retry attempts
NFR5-3a: If backend becomes temporarily unavailable (< 5 min), application queues messages and continues functioning
NFR5-3b: If backend outage extends beyond 30 minutes, user is notified with estimated recovery time (if available)
NFR5-3c: Upon backend recovery, application automatically resyncs queued messages and conversation state
NFR5-3d: No messages are lost if backend crashes with messages in-flight (transaction logs or write-ahead logs)
NFR5-4a: Message delivery is idempotent; duplicate delivery to backend does not create duplicate messages
NFR5-4b: Conversation state is consistent between client and backend; client re-syncs state on reconnection
NFR5-4c: User data (conversations, memberships) is never corrupted by concurrent updates
NFR5-4d: Database transactions ensure atomic writes; partial updates do not leave data in inconsistent state
NFR5-5a: Application does not crash under normal operation or expected error conditions
NFR5-5b: Application handles out-of-memory conditions gracefully (clear error message, controlled shutdown)
NFR5-5c: Application handles invalid/corrupted database state with automatic recovery or user guidance
NFR5-5d: Unhandled exceptions are logged but do not crash the application; user-friendly error dialog is shown
NFR5-6a: Application logs warnings and errors to a local log file (location configurable)
NFR5-6b: Log entries include timestamp, severity level, and descriptive message
NFR5-6c: Sensitive information (passwords, tokens) is never logged
NFR5-6d: Log files are rotated and retained for 30 days (older logs are archived or deleted)
NFR5-6e: Debug logging can be enabled for troubleshooting; disabled by default in production
NFR5-7a: Application provides "Report Error" functionality to send diagnostics to support
NFR5-7b: Users can export/backup local conversation history
NFR5-7c: Users can clear local cache and re-sync from backend without data loss
NFR5-7d: If UI crashes, application restarts cleanly; user remains logged in
NFR5-8a: Target backend availability is 99% uptime (monthly: max 7.2 hours downtime)
NFR5-8b: Planned maintenance is scheduled during low-activity windows (e.g., 2-4 AM UTC)
NFR5-8c: Maintenance windows are announced to users at least 48 hours in advance
NFR5-8d: During maintenance, users are shown a maintenance message; current connections are gracefully disconnected
NFR6-1a: Rust code follows standard conventions (clippy lint passing, formatting via rustfmt)
NFR6-1b: Code is organized into logical modules; duplication is minimized
NFR6-1c: Functions are documented with rustdoc comments explaining purpose and parameters
NFR6-1d: Complex algorithms include inline comments explaining logic
NFR6-2a: Unit tests cover core business logic (authentication, message validation, state management)
NFR6-2b: Integration tests validate end-to-end workflows (login → send message → receive message)
NFR6-2c: Target code coverage is ≥ 70% for backend core logic
NFR6-2d: All security-sensitive functions (authentication, authorization, encryption) have 100% test coverage
NFR6-2e: UI components have visual regression tests to prevent unintended design changes
NFR6-3a: Application builds reproducibly (same source → same binary hash)
NFR6-3b: Build process is automated via CI/CD (GitHub Actions or equivalent)
NFR6-3c: Build artifacts are versioned and tagged in version control
NFR6-3d: Deployment process is documented and repeatable (runbook for new developers)
NFR6-3e: Rollback to previous version is possible if deployment fails
NFR6-4a: Application configuration is externalized (not hardcoded in source code)
NFR6-4b: Sensitive configuration (API keys, database passwords) is stored in secure vaults or environment variables
NFR6-4c: Configuration changes do not require code recompilation; can be applied at runtime
NFR6-4d: Configuration is version-controlled (with secrets excluded from version control)
NFR6-5a: Application exposes metrics via standard format (e.g., Prometheus metrics endpoint)
NFR6-5b: Metrics include: active connection count, message throughput, API response times, error rates
NFR6-5c: Alerts are configured for critical metrics (connection failures, high error rates, resource exhaustion)
NFR6-5d: Logs are centralized and searchable (future Phase 2 with ELK stack or similar)
NFR6-5e: Structured logging (JSON format) enables easy parsing and analysis
NFR6-6a: Architecture documentation explains design decisions and component interactions
NFR6-6b: API documentation describes all endpoints, request/response formats, and error codes
NFR6-6c: Deployment documentation includes prerequisites, installation steps, and troubleshooting
NFR6-6d: Component library documentation includes usage examples and best practices
NFR6-6e: Runbooks exist for common operational tasks (backup, restore, log rotation)
NFR6-7a: All external dependencies are tracked in lock files (Cargo.lock)
NFR6-7b: Dependency updates are tested before promotion to production
NFR6-7c: Known vulnerabilities are tracked and patched within 30 days of notification
NFR6-7d: Dependency tree is kept minimal; unused dependencies are removed
NFR6-8a: All code changes go through version control (Git)
NFR6-8b: Commits include meaningful messages explaining the "why" not just the "what"
NFR6-8c: Pull requests require code review before merge (at least 1 reviewer)
NFR6-8d: Branch protection rules prevent direct commits to main branch
NFR6-9a: Releases follow semantic versioning (MAJOR.MINOR.PATCH)
NFR6-9b: Release notes document new features, bug fixes, and breaking changes
NFR6-9c: Release tags are created in version control for each production release
NFR6-9d: Release binaries are signed and include checksums for integrity verification
NFR6-10a: Performance regressions are detected via automated benchmarks in CI/CD
NFR6-10b: Profiling tools are used to identify performance bottlenecks
NFR6-10c: Optimization decisions are data-driven (measure before and after)
NFR6-10d: Technical debt is tracked and addressed in regular maintenance sprints


### Additional Requirements

- Maintain existing backend protocols unchanged (brownfield constraint)
- Keep user account model unchanged (brownfield constraint)
- Organize backend and frontend by domain (handlers/services/components organized by user journey)
- Centralize design tokens (colors/typography/spacing/motion) as the single source of truth; theme switching via AppState
- Use centralized `AppState` for UI state with reactive bindings; update state via events (not ad-hoc globals)
- Use Command/Event pattern for backend integration over WebSocket (versionable, type-safe)
- Treat errors as events with standardized structure; include retryability metadata to guide UX
- Implement automatic retry + backoff for transient failures and expose connection status in state
- Use on-demand message loading and virtual lists to preserve performance with large histories
- Follow Serde naming conventions consistently (camelCase on the wire)
- Keep tests colocated with source and runnable via `cargo test` (Rust-native harness)


### FR Coverage Map

### FR Coverage Map

FR1: Epic 1 - Users can view a list of all conversations they are currently participating in
FR2: Epic 1 - Users can search for specific conversations by participant name or display name
FR3: Epic 1 - Users can filter conversations to show only active (unread) conversations
FR4: Epic 1 - Users can pin/favorite specific conversations to keep them at the top of the list
FR5: Epic 1 - Users can quickly switch between conversations with a single interaction
FR6: Epic 1 - The system preserves scroll position and context when users switch between conversations
FR7: Epic 1 - Users can see visual indication of unread vs. read conversations
FR8: Epic 1 - Users can view a preview/snippet of the most recent message in each conversation
FR9: Epic 2 - Users can compose text messages in a dedicated message input interface
FR10: Epic 2 - Users can send a composed message and receive confirmation that it was sent successfully
FR11: Epic 2 - Users can see real-time feedback while composing (e.g., character count, send button state)
FR12: Epic 2 - Users can insert line breaks within messages (Ctrl+Enter or equivalent)
FR13: Epic 2 - The system preserves unsent message text if users navigate away before sending
FR14: Epic 2 - The system shows an error message when a user attempts to send while offline (with message text preserved)
FR15: Epic 2 - Users can clear the message composition box after a successful send
FR16: Epic 2 - The system provides clear visual feedback confirming message delivery
FR17: Epic 3 - Users can view an ordered history of messages in a conversation (newest last)
FR18: Epic 3 - Users can scroll through message history to view past conversations
FR19: Epic 3 - Users can search within a conversation to find specific past messages
FR20: Epic 3 - The system displays message metadata (sender name, timestamp) with each message
FR21: Epic 3 - Users can see visual distinction between their own messages and received messages
FR22: Epic 3 - Users can see when a message has been read by the recipient (read receipts)
FR23: Epic 3 - Users can see when a recipient is actively typing in a conversation (typing indicators)
FR24: Epic 3 - The system maintains message history durability across application restarts
FR25: Epic 4 - Users can see the online/offline status of conversation participants
FR26: Epic 4 - The system shows presence status with a visual indicator (green for online, red for offline)
FR27: Epic 4 - Users can see presence status in multiple places (conversation list, conversation header, user lists)
FR28: Epic 4 - The system indicates away/idle status separately from offline
FR29: Epic 4 - Users can see when presence status changes in real-time
FR30: Epic 4 - Users can disable presence sharing in settings (post-MVP)
FR31: Epic 4 - The system sends presence updates to other users when local user comes online/goes offline
FR32: Epic 4 - The system maintains presence consistency across the user's session
FR33: Epic 5 - Users can actively manage 5+ conversations simultaneously
FR34: Epic 5 - Users can see which conversations have unread messages across all active conversations
FR35: Epic 5 - Users can navigate between multiple conversations without losing their place
FR36: Epic 5 - The system shows conversation metadata (participant name, last message time) for quick scanning
FR37: Epic 5 - Users can see visual indicators distinguishing active conversations from inactive ones
FR38: Epic 5 - Users can organize conversations through search, filtering, and pinning
FR39: Epic 5 - The system prevents accidental loss of context when switching rapidly between conversations
FR40: Epic 5 - Users can view total unread message count across all conversations
FR41: Epic 6 - The system displays a clear, always-visible connection status indicator
FR42: Epic 6 - Users can see whether the app is Connected, Disconnected, or Connecting
FR43: Epic 6 - The system indicates reason for disconnection (no internet, server unavailable, etc.) where available
FR44: Epic 6 - Users can manually trigger reconnection when disconnected
FR45: Epic 6 - When users trigger reconnection, the system attempts to restore the connection
FR46: Epic 6 - Upon successful reconnection, the system syncs any pending state changes
FR47: Epic 6 - The system shows clear error messages when send operations fail due to connectivity
FR48: Epic 6 - The system queues presence updates for retry when connection is restored
FR49: Epic 7 - New users can create an account with minimal friction (< 2 minutes)
FR50: Epic 7 - New users can log in after account creation
FR51: Epic 7 - The onboarding flow guides users to find their first conversation partner
FR52: Epic 7 - The onboarding experience demonstrates the key capabilities (search, message, send)
FR53: Epic 7 - New users receive confirmation when they've sent their first message
FR54: Epic 7 - The application provides clear explanations for each interface element during onboarding
FR55: Epic 7 - Users can skip onboarding steps if they prefer
FR56: Epic 7 - The application remembers onboarding completion state per user
FR57: Epic 8 - Administrators can view a list of all registered users
FR58: Epic 8 - Administrators can search for specific users by name or identifier
FR59: Epic 8 - Administrators can view user status and activity information
FR60: Epic 8 - Administrators can reset user passwords
FR61: Epic 8 - Administrators can deactivate/delete user accounts
FR62: Epic 8 - Administrators can view system activity logs
FR63: Epic 8 - The system records audit trail of admin actions
FR64: Epic 8 - Administrators can access system health and performance metrics
FR65: Epic 9 - Support staff can look up user accounts by name or ID
FR66: Epic 9 - Support staff can view user conversation history (with appropriate privacy controls)
FR67: Epic 9 - Support staff can see message delivery status for specific messages
FR68: Epic 9 - Support staff can view user login history and session information
FR69: Epic 9 - Support staff can access knowledge base or troubleshooting guides
FR70: Epic 9 - The system provides clear error messages that support staff can reference with users
FR71: Epic 9 - The system logs errors and exceptions for support investigation
FR72: Epic 9 - Support staff can initiate assistance or bug reporting workflows
FR73: Epic 10 - The application uses consistent typography across all screens
FR74: Epic 10 - The application uses consistent color palette reflecting Fluent Design System
FR75: Epic 10 - All buttons have consistent styling and interactive behavior
FR76: Epic 10 - All input fields (text boxes, search bars) have consistent appearance and behavior
FR77: Epic 10 - All conversation items display with consistent layout and spacing
FR78: Epic 10 - All messages display with consistent formatting and styling
FR79: Epic 10 - The application uses consistent spacing and padding throughout
FR80: Epic 10 - Hover, focus, and active states are consistent across interactive elements
FR81: Epic 11 - Users can navigate all core workflows using keyboard only (Tab navigation)
FR82: Epic 11 - Users can activate buttons and controls using Enter or Space
FR83: Epic 11 - Users can move between conversations using keyboard shortcuts (Tab to cycle)
FR84: Epic 11 - Users can send messages using keyboard (Enter to send, Ctrl+Enter for line break)
FR85: Epic 11 - The application maintains visible focus indicators for keyboard navigation
FR86: Epic 11 - All text meets minimum contrast ratios for accessibility (WCAG AA)
FR87: Epic 11 - The application supports screen readers with proper semantic labels
FR88: Epic 11 - Users can navigate dialogs and modals using keyboard
FR89: Epic 12 - The application runs on Windows 10 and Windows 11
FR90: Epic 12 - The application respects the Windows system dark/light theme setting
FR91: Epic 12 - The application sends Windows notifications for new messages
FR92: Epic 12 - Windows notifications display message preview and sender information
FR93: Epic 12 - Clicking a notification brings the application window to focus and shows relevant conversation
FR94: Epic 12 - The application window can be resized and repositioned on screen
FR95: Epic 12 - The application maintains window state across application restarts
FR96: Epic 12 - The application supports standard Windows window controls (minimize, maximize, close)
FR97: Epic 13 - The application layout adapts to different window sizes (minimum 640x480)
FR98: Epic 13 - Conversation list remains accessible regardless of window width
FR99: Epic 13 - Message composition area remains functional at minimum supported size
FR100: Epic 13 - Presence indicators remain visible at all window sizes
FR101: Epic 13 - Connection status indicator remains visible at all window sizes
FR102: Epic 13 - The application prevents UI elements from overlapping or hiding at edge cases
FR103: Epic 13 - Scrollbars appear only when content exceeds available space
FR104: Epic 13 - All text remains readable at supported window sizes
FR105: Epic 14 - The application starts up and becomes ready for use within 2 seconds
FR106: Epic 14 - Switching between conversations is instantaneous (< 100ms)
FR107: Epic 14 - Messages appear in the conversation immediately after sending (< 500ms)
FR108: Epic 14 - Presence updates appear in real-time (< 1 second)
FR109: Epic 14 - UI interactions remain responsive during message receiving
FR110: Epic 14 - The application handles large message histories (100+ messages) without degradation
FR111: Epic 14 - The application continues functioning if backend connection is slow
FR112: Epic 14 - The application recovers gracefully from temporary connection loss


## Epic List

## Epic List

### Epic 1: Conversation Discovery & Management
Users can find, organize, and switch between conversations confidently.
**FRs covered:** FR1–FR8

### Epic 2: Message Composition & Sending
Users can draft and send messages with clear feedback (including offline handling).
**FRs covered:** FR9–FR16

### Epic 3: Message Reading, History & In-Conversation Search
Users can read messages with metadata and search within a conversation; history persists.
**FRs covered:** FR17–FR24

### Epic 4: Presence & Status Awareness
Users can see who's online/away and react to presence changes.
**FRs covered:** FR25–FR32

### Epic 5: Multi-Conversation Management
Users can manage multiple conversations smoothly without losing context/state.
**FRs covered:** FR33–FR40

### Epic 6: Connection & Sync Resilience
Users experience reliable behavior through disconnects/reconnects and sync issues.
**FRs covered:** FR41–FR48

### Epic 7: Onboarding & First-Time Experience
New users can understand and successfully complete key workflows.
**FRs covered:** FR49–FR56

### Epic 8: Admin User Management & Auditability
Admins can manage users and view logs/health signals safely.
**FRs covered:** FR57–FR64

### Epic 9: Support & Troubleshooting Workflows
Support can investigate issues and guide users with actionable diagnostics.
**FRs covered:** FR65–FR72

### Epic 10: Visual Design System & UI Consistency
Users get a cohesive, modern UI with consistent components and interactions.
**FRs covered:** FR73–FR80

### Epic 11: Accessibility & Keyboard Navigation
Users can operate core workflows via keyboard and assistive tech.
**FRs covered:** FR81–FR88

### Epic 12: Windows Integration & Platform Support
Windows users get native expectations (notifications, window controls, theme respect).
**FRs covered:** FR89–FR96

### Epic 13: Responsive Layout & UI Adaptation
Users can use the app comfortably across window sizes without layout breakage.
**FRs covered:** FR97–FR104

### Epic 14: Performance & Reliability Targets
Users get fast startup, fast switching, and stable performance at scale.
**FRs covered:** FR105–FR112


## Epic 1: Conversation Discovery & Management

Users can find, organize, and switch between conversations confidently.

### Story 1.0: Brownfield Integration Checkpoint (WebSocket + SQLite)

**Implements:** (Derived)
**Derived Requirement:** Brownfield compatibility with existing WebSocket protocol and SQLite schema/migrations

As a chat user,
I want the modernized app to remain compatible with existing backend protocols and stored data,
So that I don’t lose access to conversations or experience regressions.

**Acceptance Criteria:**

**Given** I upgrade from an existing deployment with a known WebSocket protocol and an existing SQLite database  
**When** I launch the modernized application and sign in  
**Then** the app can connect using the existing WebSocket protocol and load my conversation list successfully  
**And** the app can open an existing conversation and send/receive messages without protocol errors

**Given** the existing SQLite schema requires changes for new features  
**When** the app starts with a database that is on an older schema version  
**Then** migrations are applied safely and incrementally  
**And** existing data remains readable after migration

### Story 1.1: Conversation List With Unread + Preview

**Implements:** FR1, FR7, FR8, FR36

As a chat user,
I want to view my conversations with a preview and unread indicator,
So that I can quickly decide which conversation to open next.

**Acceptance Criteria:**

**Given** I am authenticated and have one or more conversations  
**When** I open the conversation list  
**Then** I see a list of my conversations showing participant/display name, last message time, and most recent message preview  
**And** conversations with unread messages are visually distinct from read conversations

### Story 1.2: Conversation Switching With Context Preservation

**Implements:** FR5, FR6

As a chat user,
I want to switch between conversations without losing my place,
So that I can multitask across chats efficiently.

**Acceptance Criteria:**

**Given** I have opened a conversation from the list  
**When** I switch to a different conversation and then return to the original conversation  
**Then** the app preserves my context for the original conversation (e.g., where I was in the UI)  
**And** switching requires a single interaction from the conversation list

### Story 1.3: Conversation List Keyboard Navigation (Basics)

**Implements:** FR81, FR82, FR83, FR85

As a chat user,
I want to navigate the conversation list using the keyboard,
So that I can operate core workflows without relying on the mouse.

**Acceptance Criteria:**

**Given** the application window is focused  
**When** I move keyboard focus to the conversation list (e.g., via Tab navigation)  
**Then** a visible focus indicator is shown on the focused list item  
**And** I can change the selected conversation using keyboard navigation and open it using Enter

### Story 1.4: Search Conversations By Participant/Display Name

**Implements:** FR2

As a chat user,
I want to search my conversations by participant name or display name,
So that I can find the right conversation quickly.

**Acceptance Criteria:**

**Given** I have multiple conversations with different participants  
**When** I enter a search query in the conversation search input  
**Then** the conversation list filters to show matching conversations by participant/display name  
**And** clearing the search restores the full conversation list

### Story 1.5: Filter Conversations To Active/Unread

**Implements:** FR3

As a chat user,
I want to filter my conversation list to only show active/unread conversations,
So that I can focus on what needs my attention.

**Acceptance Criteria:**

**Given** I have a mix of read and unread conversations  
**When** I apply the “active/unread” filter  
**Then** the list shows only conversations with unread messages  
**And** the UI indicates that a filter is active and can be cleared

### Story 1.6: Pin/Favorite Conversations

**Implements:** FR4

As a chat user,
I want to pin/favorite conversations,
So that important chats stay easy to access.

**Acceptance Criteria:**

**Given** I have one or more conversations in the list  
**When** I pin a conversation  
**Then** that conversation is visually marked as pinned and remains at the top of the list  
**And** I can unpin it to return it to normal ordering

<!-- Repeat for each additional epic (N = 2, 3, 4...) -->

## Epic 2: Message Composition & Sending

Users can draft and send messages with clear feedback (including offline handling).

### Story 2.1: Message Composer UI + Send Button State

**Implements:** FR9, FR11

As a chat user,
I want a dedicated message composer with clear send readiness feedback,
So that I can confidently compose messages and know when they can be sent.

**Acceptance Criteria:**

**Given** I am viewing an active conversation  
**When** I type into the message input  
**Then** I can compose a text message in the dedicated composer  
**And** the UI provides real-time feedback while composing (e.g., character count and send button enabled/disabled state)

### Story 2.2: Send Message + Sent Confirmation

**Implements:** FR10, FR15

As a chat user,
I want to send a composed message and get immediate confirmation it was sent,
So that I trust the system accepted my message.

**Acceptance Criteria:**

**Given** I have composed a non-empty message within the allowed character limit  
**When** I send the message  
**Then** the system accepts the message and shows confirmation that it was sent successfully  
**And** the composer clears after successful send

**Given** I attempt to send a message while connected  
**When** the send fails for a non-connectivity reason (e.g., validation error, server error)  
**Then** the system shows a clear error message  
**And** the unsent message content remains available for correction or retry (per defined rules)

### Story 2.3: Keyboard Message Composition (Enter-to-Send, Ctrl+Enter Line Break)

**Implements:** FR12, FR82, FR84

As a chat user,
I want keyboard shortcuts for sending and adding line breaks,
So that I can compose efficiently without losing message formatting.

**Acceptance Criteria:**

**Given** I am typing in the message composer  
**When** I press Ctrl+Enter  
**Then** a line break is inserted into the message without sending it  
**And** when I press Enter (with send enabled) the message is sent

### Story 2.4: Draft Preservation Per Conversation

**Implements:** FR13

As a chat user,
I want my unsent draft preserved per conversation when I navigate away,
So that I can switch chats without losing what I was writing.

**Acceptance Criteria:**

**Given** I have typed a draft message in Conversation A but have not sent it  
**When** I switch to Conversation B and then return to Conversation A  
**Then** the draft message for Conversation A is preserved in the composer  
**And** drafts are tracked per conversation (Conversation B has its own draft state)

### Story 2.5: Offline Send Error With Draft Preserved

**Implements:** FR14

As a chat user,
I want a clear error message when I attempt to send while offline,
So that I understand why the send failed and can retry later.

**Acceptance Criteria:**

**Given** I have a composed message draft and the app is offline/disconnected  
**When** I attempt to send the message  
**Then** the system shows an offline send error to the user  
**And** the draft message text remains preserved in the composer for retry

### Story 2.6: Delivery Feedback (Delivered/Read Semantics)

**Implements:** FR16

As a chat user,
I want message delivery feedback that distinguishes “sent”, “delivered”, and “read”,
So that I know whether my message reached the recipient and was seen.

**Acceptance Criteria:**

**Given** I have sent a message in a conversation  
**When** the system receives delivery and read status updates for that message  
**Then** the message UI updates to reflect delivered and read states distinctly  
**And** status changes are associated with the correct message and conversation

<!-- Repeat for each additional epic (N = 3, 4, 5...) -->

## Epic 3: Message Reading, History & In-Conversation Search

Users can read messages with metadata and search within a conversation; history persists.

### Story 3.1: Message List Rendering (Own vs Received) + Metadata

**Implements:** FR17, FR20, FR21

As a chat user,
I want messages rendered with clear sender/recipient styling and metadata,
So that I can quickly understand who said what and when.

**Acceptance Criteria:**

**Given** I am viewing a conversation with messages from me and the other participant  
**When** the message list is displayed  
**Then** my messages are visually distinct from received messages  
**And** each message shows sender name (or label) and timestamp metadata

### Story 3.2: Ordered Message History (Newest Last) + Lazy Scrollback

**Implements:** FR18

As a chat user,
I want to read the message history in order and load older messages as I scroll,
So that the app remains fast even for long conversations.

**Acceptance Criteria:**

**Given** a conversation has more messages than can fit on screen  
**When** I open the conversation  
**Then** messages are shown in chronological order (newest last)  
**And** older messages load lazily when I scroll upward (without blocking the UI)

### Story 3.3: In-Conversation Search With Highlight + Jump

**Implements:** FR19

As a chat user,
I want to search within a conversation and jump to matching messages,
So that I can find information quickly without manual scrolling.

**Acceptance Criteria:**

**Given** a conversation contains messages matching a keyword  
**When** I search for that keyword within the conversation  
**Then** matching messages are highlighted in the message list  
**And** I can jump to a specific match result to bring that message into view

### Story 3.4: Read Receipts Display

**Implements:** FR22

As a chat user,
I want to see when my message has been read by the recipient,
So that I understand whether it was seen.

**Acceptance Criteria:**

**Given** I have sent a message in a conversation  
**When** the recipient reads that message and the system reports a read receipt  
**Then** the message UI updates to indicate it has been read  
**And** the read receipt is associated with the correct message

### Story 3.5: Typing Indicators Display

**Implements:** FR23

As a chat user,
I want to see when the other participant is typing,
So that I can anticipate an incoming reply.

**Acceptance Criteria:**

**Given** I am viewing an active conversation  
**When** the other participant starts typing and a typing event is received  
**Then** the UI displays a typing indicator for that participant  
**And** the indicator clears when typing stops or after a reasonable timeout

### Story 3.6: Message History Durability Across Restarts

**Implements:** FR24

As a chat user,
I want my conversation history to persist across app restarts,
So that I can pick up where I left off.

**Acceptance Criteria:**

**Given** I have exchanged messages in a conversation  
**When** I close and reopen the application and return to that conversation  
**Then** previously sent and received messages are still available in history  
**And** the conversation view loads successfully without requiring manual recovery steps

<!-- Repeat for each additional epic (N = 4, 5, 6...) -->

## Epic 4: Presence & Status Awareness

Users can see who's online/away and react to presence changes.

### Story 4.1: Presence Basics (Online/Offline) in Conversation View

**Implements:** FR25

As a chat user,
I want to see whether the other participant is online or offline,
So that I know whether I should expect an immediate response.

**Acceptance Criteria:**

**Given** I am viewing an active conversation  
**When** the conversation header is displayed  
**Then** I can see the other participant’s online/offline status  
**And** the status reflects the most recently known state from the system

### Story 4.2: Presence Indicator Colors + Legend

**Implements:** FR26

As a chat user,
I want a clear visual presence indicator for online/offline status,
So that status is easy to understand at a glance.

**Acceptance Criteria:**

**Given** presence status is shown in the UI  
**When** a participant is online or offline  
**Then** the UI uses distinct visual indicators (e.g., green for online, red for offline)  
**And** the meaning of the indicators is discoverable (e.g., via tooltip/legend)

### Story 4.3: Presence Shown in Multiple Surfaces

**Implements:** FR27

As a chat user,
I want presence status shown consistently across the app,
So that I don’t have to open a conversation to know who is available.

**Acceptance Criteria:**

**Given** I can see participants in multiple app surfaces  
**When** the conversation list, conversation header, and user lists are displayed  
**Then** each surface shows the participant’s presence indicator consistently  
**And** presence rendering follows the same rules across surfaces

### Story 4.4: Away/Idle vs Offline Distinction (Auto + Manual)

**Implements:** FR28

As a chat user,
I want “away/idle” to be distinct from “offline”, and to be set automatically or manually,
So that presence reflects reality without ambiguity.

**Acceptance Criteria:**

**Given** I am authenticated  
**When** I am inactive for a configured period  
**Then** my status transitions to away/idle automatically  
**And** I can manually set my status (e.g., online/away) to override the automatic state (within defined rules)

### Story 4.5: Real-Time Presence Updates + Last Seen

**Implements:** FR29, FR31, FR32

As a chat user,
I want presence changes to update in real time and see last-seen timestamps,
So that I have accurate, timely context about availability.

**Acceptance Criteria:**

**Given** two users share a conversation  
**When** one user comes online, goes away/idle, or goes offline  
**Then** the other user sees the status change reflected in the UI without a manual refresh  
**And** when a user is offline, the UI can display a “last seen” timestamp (if available)

### Story 4.6: Disable Presence Sharing (Settings)

**Implements:** FR30

As a chat user,
I want to disable presence sharing in settings,
So that I can control whether others can see my online/away status.

**Acceptance Criteria:**

**Given** I am authenticated  
**When** I disable presence sharing in settings  
**Then** other users no longer see my live presence updates (per defined rules)  
**And** I can re-enable presence sharing to resume normal presence behavior

<!-- Repeat for each additional epic (N = 5, 6, 7...) -->

## Epic 5: Multi-Conversation Management

Users can manage multiple conversations smoothly without losing context/state.

### Story 5.1: Multi-Conversation Tabs (Open/Recent)

**Implements:** FR33, FR35, FR37

As a chat user,
I want multiple conversations to be available as tabs (or an equivalent multi-open model),
So that I can switch between active chats quickly.

**Acceptance Criteria:**

**Given** I have recently opened multiple conversations  
**When** I view the app’s conversation navigation area  
**Then** I can see multiple open/recent conversations represented as tabs  
**And** I can switch between them directly  
**And** the currently active conversation is visually distinct from inactive conversations

### Story 5.2: Preserve Per-Conversation UI State Across Switching

**Implements:** FR6, FR39

As a chat user,
I want per-conversation UI state preserved when switching between conversations,
So that I don’t lose my place or context in each conversation.

**Acceptance Criteria:**

**Given** I have scrolled within Conversation A and have a draft in its composer  
**When** I switch to Conversation B and then back to Conversation A  
**Then** Conversation A restores its prior UI state (e.g., scroll position)  
**And** Conversation A’s draft state is preserved independently of Conversation B

### Story 5.3: Unread Counts and Cross-Conversation Badging

**Implements:** FR34, FR40

As a chat user,
I want unread counts/badges per conversation,
So that I can prioritize which conversations need attention.

**Acceptance Criteria:**

**Given** I have unread messages in one or more conversations  
**When** I view the conversation list and/or tabs  
**Then** each conversation shows an unread indicator and count (when applicable)  
**And** unread indicators update when I read the messages  
**And** the UI can show a total unread count across all conversations (where applicable)

### Story 5.4: Conversation Sorting Rules (Pinned + Recent + Unread)

**Implements:** FR38

As a chat user,
I want predictable ordering of conversations,
So that important chats remain easy to find.

**Acceptance Criteria:**

**Given** I have pinned conversations and a mix of recent and unread conversations  
**When** the conversation list is displayed  
**Then** pinned conversations appear first  
**And** remaining conversations follow consistent ordering rules (e.g., recent activity and/or unread status)

### Story 5.5: Conversation-Level Muting (In-App Only)

**Implements:** (Derived)
**Derived Requirement:** UX-driven distraction control (not explicitly listed as a PRD FR)

As a chat user,
I want to mute a conversation within the app,
So that I can reduce distractions for that conversation without leaving it.

**Acceptance Criteria:**

**Given** I am viewing a conversation  
**When** I mute the conversation  
**Then** in-app notification signals for that conversation are suppressed (per defined rules)  
**And** I can unmute the conversation to restore normal in-app notification behavior

### Story 5.6: Multi-Conversation Search & Jump

**Implements:** FR2, FR38

As a chat user,
I want to search across conversations and jump to the correct conversation,
So that I can find relevant threads quickly.

**Acceptance Criteria:**

**Given** I have multiple conversations and a search input for conversations  
**When** I search for a participant/display name and select a result  
**Then** the app navigates to the selected conversation  
**And** the conversation becomes active in the conversation navigation (list/tabs)

<!-- Repeat for each additional epic (N = 6, 7, 8...) -->

## Epic 6: Connection & Sync Resilience

Users experience reliable behavior through disconnects/reconnects and sync issues.

### Story 6.1: Always-Visible Connection Status Indicator

**Implements:** FR41, FR42

As a chat user,
I want an always-visible connection status indicator,
So that I can quickly understand whether the app is connected.

**Acceptance Criteria:**

**Given** the app is running  
**When** I view the top app bar  
**Then** I can see a connection status indicator at all times  
**And** it reflects one of: Connected, Connecting, Disconnected

### Story 6.2: Disconnection Reasons and Clear Connection Messaging

**Implements:** FR43

As a chat user,
I want to understand why I’m disconnected when possible,
So that I can take the right action to restore service.

**Acceptance Criteria:**

**Given** the app transitions to Disconnected  
**When** a likely cause is available (e.g., no internet, server unavailable, auth expired)  
**Then** the UI shows the best-known reason alongside the disconnected status  
**And** if no reason is available, the UI shows a generic disconnected message

### Story 6.3: Manual Reconnect Action and Attempt Feedback

**Implements:** FR44, FR45

As a chat user,
I want to manually trigger reconnection when disconnected,
So that I can restore connectivity when I’m ready.

**Acceptance Criteria:**

**Given** the app is Disconnected  
**When** I trigger a reconnect action  
**Then** the app attempts to restore the connection  
**And** the indicator updates to Connecting during the attempt  
**And** on success the indicator updates to Connected  
**And** on failure the indicator returns to Disconnected with an updated best-known reason (when available)

**Given** the app is Connecting due to a prior reconnect attempt  
**When** I attempt to trigger reconnection again  
**Then** the app prevents duplicate attempts (e.g., disables the reconnect control)  
**And** the UI continues to show Connecting until success or failure

### Story 6.4: Sync Pending State Changes After Reconnection

**Implements:** FR46

As a chat user,
I want pending state changes to sync after reconnection,
So that my activity is consistent across devices and participants.

**Acceptance Criteria:**

**Given** I experienced a disconnect and later reconnect successfully  
**When** the connection is restored  
**Then** the app syncs any pending state changes created during the disconnect (e.g., receipts and local UI signals such as typing)  
**And** pending state changes are cleared once confirmed synced

### Story 6.5: Presence Updates Are Queued and Retried After Connectivity Restores

**Implements:** FR48

As a chat user,
I want my presence updates to be queued during disconnects,
So that my availability is corrected when the connection returns.

**Acceptance Criteria:**

**Given** the app is Disconnected and my presence state changes locally  
**When** the connection is restored  
**Then** the app sends queued presence updates to the server  
**And** the UI reflects the latest local presence state while disconnected

### Story 6.6: Send Failure Messaging and Manual Retry

**Implements:** FR47

As a chat user,
I want clear error messages when sending fails due to connectivity,
So that I can understand what happened and retry intentionally.

**Acceptance Criteria:**

**Given** I attempt to send a message while disconnected (or during a failed connection attempt)  
**When** the send fails due to connectivity  
**Then** the message is marked as failed with a clear error message  
**And** the message remains failed until I manually retry sending it  
**And** retry attempts are reflected in the UI (e.g., returning to sending state)

**Given** a message is in a failed state due to connectivity  
**When** I retry while the app is still disconnected  
**Then** the message remains failed  
**And** the UI provides clear feedback that connectivity is still required before the retry can succeed

## Epic 7: Onboarding & First-Time Experience

New users can understand and successfully complete key workflows.

### Story 7.1: Fast Account Creation (Username + Password)

**Implements:** FR49

As a new user,
I want to create an account quickly with a username and password,
So that I can start using the app with minimal friction.

**Acceptance Criteria:**

**Given** I am not signed in  
**When** I create an account with the required fields (username + password)  
**Then** the account creation flow completes in a minimal number of steps  
**And** the app confirms account creation successfully

**Given** I attempt to create an account with invalid or conflicting information  
**When** the username is already taken or the password does not meet requirements  
**Then** the app shows a clear error message explaining what to fix  
**And** the form remains available to correct and resubmit

### Story 7.2: Login After Account Creation

**Implements:** FR50

As a new user,
I want to log in after creating my account,
So that I can access my conversations.

**Acceptance Criteria:**

**Given** I have created an account  
**When** I enter my username and password and submit  
**Then** I am signed in successfully  
**And** I am taken to the primary app experience

**Given** I enter an incorrect username or password  
**When** I attempt to sign in  
**Then** the app shows a clear authentication error  
**And** I remain on the login screen to retry

### Story 7.3: Guided Onboarding Checklist (Skippable)

**Implements:** FR55, FR56

As a new user,
I want a guided onboarding checklist I can skip,
So that I can either learn by doing or jump straight into the app.

**Acceptance Criteria:**

**Given** this is my first time signing in  
**When** I enter the app  
**Then** I see an onboarding checklist that guides me through initial steps  
**And** each step is optional and can be skipped  
**And** if I skip onboarding, I can proceed to use the app normally

### Story 7.4: Find First Conversation Partner During Onboarding

**Implements:** FR51

As a new user,
I want onboarding to guide me to find my first conversation partner,
So that I can start chatting quickly.

**Acceptance Criteria:**

**Given** I am in the onboarding flow  
**When** I reach the “find a partner” step  
**Then** I am guided to search/select a person to start a conversation with  
**And** selecting a person results in an opened/created conversation with that partner

### Story 7.5: Onboarding Demonstrates Key Capabilities With Explanations

**Implements:** FR52, FR54

As a new user,
I want onboarding to demonstrate the key capabilities with clear explanations,
So that I understand what the main interface elements do.

**Acceptance Criteria:**

**Given** I am in the onboarding flow  
**When** I progress through onboarding steps  
**Then** onboarding demonstrates the key capabilities (search, compose, send)  
**And** the UI provides clear explanations for relevant interface elements during those steps

### Story 7.6: First Message Confirmation and Onboarding Completion State

**Implements:** FR53, FR56

As a new user,
I want confirmation when I send my first message and for onboarding to remember completion,
So that I know I succeeded and I’m not repeatedly prompted.

**Acceptance Criteria:**

**Given** I am a new user and have not yet completed onboarding  
**When** I send my first message successfully  
**Then** I receive a clear confirmation that my first message was sent  
**And** my onboarding completion state is saved per user  
**And** on subsequent sign-ins, onboarding does not reappear as incomplete

## Epic 8: Admin User Management & Auditability

Admins can manage users and view logs/health signals safely.

### Story 8.1: Admin-Only Access to Admin UI

**Implements:** NFR1-1e, NFR1-1f
**Derived Requirement:** Role-based access control for administrative tooling

As an administrator,
I want a dedicated admin interface accessible only to admins,
So that administrative capabilities are protected from non-admin users.

**Acceptance Criteria:**

**Given** I am signed in as a non-admin user  
**When** I attempt to access the admin UI  
**Then** access is denied

**Given** I am signed in as an admin user  
**When** I access the admin UI  
**Then** I can access administrative features

### Story 8.2: View and Search Registered Users

**Implements:** FR57, FR58

As an administrator,
I want to view and search the list of registered users,
So that I can find accounts quickly for management tasks.

**Acceptance Criteria:**

**Given** I am in the admin UI  
**When** I view the user management area  
**Then** I can see a list of registered users

**Given** I am in the admin UI with many users  
**When** I search by a user name or identifier  
**Then** the results include matching users

### Story 8.3: View User Status and Activity Information

**Implements:** FR59

As an administrator,
I want to view user status and activity information,
So that I can assess whether an account is active and healthy.

**Acceptance Criteria:**

**Given** I am viewing a user in the admin UI  
**When** user status and activity information is available  
**Then** I can see it for that user (e.g., active/deactivated, recent activity)

### Story 8.4: Reset User Passwords (Temporary Password + Forced Change)

**Implements:** FR60

As an administrator,
I want to reset a user’s password by setting a temporary password,
So that the user can regain access securely.

**Acceptance Criteria:**

**Given** I am an admin viewing a user account  
**When** I set a temporary password for the user  
**Then** the user can log in with that temporary password  
**And** the user is required to change their password on next login

**Given** I attempt to reset a password for a user that does not exist or is deactivated  
**When** I submit the password reset action  
**Then** the system rejects the action with a clear error message  
**And** no password change is applied

### Story 8.5: Deactivate (Soft-Delete) User Accounts

**Implements:** FR61

As an administrator,
I want to deactivate user accounts,
So that I can remove access without deleting historical records.

**Acceptance Criteria:**

**Given** I am an admin viewing a user account  
**When** I deactivate the account  
**Then** the user can no longer sign in  
**And** the account is marked as deactivated in the admin UI

**Given** I attempt to deactivate an account that is already deactivated  
**When** I perform the deactivation action  
**Then** the system does not create inconsistent state  
**And** the UI clearly indicates the account is already deactivated

### Story 8.6: Admin Activity Logs and Audit Trail

**Implements:** FR62, FR63

As an administrator,
I want system activity logs and an audit trail of admin actions,
So that I can investigate issues and ensure accountability.

**Acceptance Criteria:**

**Given** I am an admin  
**When** I perform an administrative action (e.g., password reset, deactivate account)  
**Then** the system records an audit trail entry for that action

**Given** I am an admin  
**When** I view the system activity logs  
**Then** I can see recent system/admin activity entries

### Story 8.7: View System Health and Performance Metrics

**Implements:** FR64

As an administrator,
I want to view system health and performance metrics,
So that I can identify problems early.

**Acceptance Criteria:**

**Given** I am an admin  
**When** I view the admin health/metrics area  
**Then** I can see high-level health and performance metrics (as defined for MVP)

## Epic 9: Support & Troubleshooting Workflows

Support can investigate issues and guide users with actionable diagnostics.

### Story 9.1: Support-Only Access to Support UI

**Implements:** NFR1-1e, NFR1-1f
**Derived Requirement:** Role-based access control for support tooling

As a support staff member,
I want a dedicated support interface accessible only to support staff,
So that support capabilities are protected from regular users.

**Acceptance Criteria:**

**Given** I am signed in as a non-support user  
**When** I attempt to access the support UI  
**Then** access is denied

**Given** I am signed in as a support user  
**When** I access the support UI  
**Then** I can access support features

### Story 9.2: Look Up User Accounts by Name or ID

**Implements:** FR65

As a support staff member,
I want to look up user accounts by name or ID,
So that I can quickly find the account related to an issue.

**Acceptance Criteria:**

**Given** I am in the support UI  
**When** I search for a user by name or identifier  
**Then** I can find matching user accounts  
**And** I can open a user detail view for a selected user

**Given** I search for a user that does not exist  
**When** I submit the search  
**Then** the UI shows “no results” clearly  
**And** suggests how to refine the search (as defined for MVP)

### Story 9.3: View User Conversation History (Including Message Bodies)

**Implements:** FR66

As a support staff member,
I want to view a user’s conversation history (including message bodies) with appropriate privacy controls,
So that I can investigate issues reported by users.

**Acceptance Criteria:**

**Given** I am a support user viewing a specific user account  
**When** I open the user’s conversation history  
**Then** I can view the user’s conversations and associated message bodies  
**And** access is controlled by support authorization rules (as defined for MVP)

**Given** privacy controls prevent me from viewing a user’s message bodies  
**When** I attempt to access the conversation history  
**Then** access is denied with a clear message  
**And** the UI does not reveal restricted message content

### Story 9.4: View Delivery Status for Specific Messages

**Implements:** FR67

As a support staff member,
I want to view message delivery status for specific messages,
So that I can confirm whether and when a message was delivered and read.

**Acceptance Criteria:**

**Given** I am viewing a message in support tools  
**When** delivery/read status information is available  
**Then** I can see that status for the message

### Story 9.5: View User Login History and Session Information

**Implements:** FR68

As a support staff member,
I want to view user login history and session information,
So that I can troubleshoot access and authentication problems.

**Acceptance Criteria:**

**Given** I am viewing a user in the support UI  
**When** login history and session information is available  
**Then** I can view relevant recent entries (as defined for MVP)

### Story 9.6: Knowledge Base / Troubleshooting Guides and Referenceable Errors

**Implements:** FR69, FR70

As a support staff member,
I want access to troubleshooting guides and referenceable error messages,
So that I can guide users through resolving problems efficiently.

**Acceptance Criteria:**

**Given** I am in the support UI  
**When** I open troubleshooting guides/knowledge base content  
**Then** I can view and follow support guidance

**Given** a user experiences an error  
**When** the error is shown in the app  
**Then** the error message is clear and support can reference it consistently

### Story 9.7: Logged Errors and In-App Bug Reporting With Diagnostics

**Implements:** FR71, FR72

As a support staff member,
I want errors/exceptions to be logged and for users to be able to report problems with diagnostics attached,
So that investigations have enough detail to reproduce and fix issues.

**Acceptance Criteria:**

**Given** the application encounters an error/exception  
**When** it occurs  
**Then** the system records a log entry suitable for support investigation

**Given** a user wants to report a problem  
**When** they submit an in-app report  
**Then** the report includes relevant diagnostics (as defined for MVP)  
**And** the report is available to support for follow-up

## Epic 10: Visual Design System & UI Consistency

Users get a cohesive, modern UI with consistent components and interactions.

### Story 10.1: Fluent-First Visual Design Baseline

**Implements:** FR73, FR74, FR78, FR79

As a user,
I want the UI to follow a Fluent-first design baseline,
So that the app feels consistent with platform expectations and visually cohesive.

**Acceptance Criteria:**

**Given** I navigate across core app screens  
**When** I observe typography, spacing, and visual patterns  
**Then** the UI follows a consistent Fluent-aligned baseline across screens  
**And** core visual tokens (typography/spacing) are applied consistently

### Story 10.2: Theme Support (Manual Light/Dark)

**Implements:** (Derived)
**Derived Requirement:** UX-driven preference to override OS theme (Architecture supports manual toggle; PRD FR covers OS-theme respect)

As a user,
I want to manually switch between light and dark themes,
So that I can use the app comfortably in different environments.

**Acceptance Criteria:**

**Given** the app provides theme selection  
**When** I switch between light and dark themes  
**Then** the UI updates to the selected theme  
**And** the selection persists across app restarts

### Story 10.3: Consistent Buttons, Inputs, and Search Components

**Implements:** FR75, FR76, FR80

As a user,
I want buttons, inputs, and search components to behave consistently,
So that interactions feel predictable across the app.

**Acceptance Criteria:**

**Given** I use buttons, text inputs, and search fields in different parts of the app  
**When** I interact with them  
**Then** they have consistent styling, sizing, and interaction states (hover/focus/disabled)  
**And** validation and error presentation is consistent where applicable

### Story 10.4: Consistent Conversation List Item Layouts

**Implements:** FR77, FR79

As a user,
I want conversation list items to use a consistent layout and spacing,
So that scanning and navigation feels familiar.

**Acceptance Criteria:**

**Given** I view the conversation list in different states (empty, populated, mixed)  
**When** conversation items are displayed  
**Then** item layout, spacing, and typography are consistent  
**And** important metadata (name, last message time, unread state) is presented consistently

### Story 10.5: Standardized Empty, Loading, and Error States

**Implements:** FR73, FR74, FR79

As a user,
I want empty, loading, and error states to be consistent and informative,
So that I always know what’s happening and what to do next.

**Acceptance Criteria:**

**Given** a screen is loading, has no data, or encounters an error  
**When** the state is displayed  
**Then** the UI uses standardized patterns for empty/loading/error states  
**And** error states provide clear, actionable messaging where applicable

### Story 10.6: Consistent Iconography and Sizing Rules

**Implements:** FR74, FR79

As a user,
I want icons to be used consistently with clear sizing rules,
So that the UI looks polished and easy to interpret.

**Acceptance Criteria:**

**Given** icons are used for actions, status, and navigation  
**When** I view icons across the app  
**Then** icon style and sizing are consistent  
**And** icons convey meaning consistently with accompanying labels/tooltips where needed

## Epic 11: Accessibility & Keyboard Navigation

Users can operate core workflows via keyboard and assistive tech.

### Story 11.1: WCAG 2.1 AA (Best-Effort) for Core Flows

**Implements:** FR86

As a user,
I want the app to meet WCAG 2.1 AA accessibility expectations (best effort) for key screens,
So that the app is usable for a broad set of needs.

**Acceptance Criteria:**

**Given** I use the app’s core flows (navigate conversations, read, compose, send)  
**When** I interact with the UI  
**Then** text contrast, focus visibility, and interactive element semantics are implemented to a WCAG 2.1 AA best-effort standard for MVP

### Story 11.2: Keyboard Access for Key Flows

**Implements:** FR81, FR82, FR83, FR84

As a keyboard user,
I want to complete key flows with the keyboard,
So that I can use the app efficiently without a mouse.

**Acceptance Criteria:**

**Given** I am using only the keyboard  
**When** I navigate key workflows (switch conversation, search, compose, send)  
**Then** every required control is reachable via keyboard focus  
**And** focus order is predictable and does not trap me unintentionally

### Story 11.3: Accessible Labels and Announcements for Key UI Elements

**Implements:** FR87

As a user who relies on a screen reader,
I want key UI elements to have accessible labels and descriptions,
So that the interface is understandable and operable.

**Acceptance Criteria:**

**Given** I encounter icon-only buttons, inputs, and status indicators  
**When** they are rendered  
**Then** they have accessible names/labels and appropriate descriptions/tooltips as needed

### Story 11.4: Standardized Focus Treatment and Visual Feedback

**Implements:** FR85

As a keyboard user,
I want consistent focus styling and visible feedback,
So that I always know where I am in the UI.

**Acceptance Criteria:**

**Given** I move focus through interactive elements  
**When** focus changes  
**Then** the focused element is clearly indicated  
**And** focus styling is consistent across component types

### Story 11.5: Validation and Error Messaging Accessible via Keyboard

**Implements:** FR81, FR82

As a user,
I want validation and error messages to be accessible and keyboard-friendly,
So that I can recover from errors without confusion.

**Acceptance Criteria:**

**Given** a form/input error occurs  
**When** the error is presented  
**Then** the error message is visible and associated with the relevant field/control  
**And** I can reach and address the error using the keyboard

### Story 11.6: Dialog and Modal Keyboard Navigation

**Implements:** FR88

As a keyboard user,
I want to navigate dialogs and modals using the keyboard,
So that I can complete tasks without a mouse.

**Acceptance Criteria:**

**Given** a dialog or modal is open  
**When** I use Tab/Shift+Tab and Enter/Space  
**Then** I can move focus between controls in the dialog/modal and activate them  
**And** I can dismiss the dialog/modal using the keyboard (per defined rules)

## Epic 12: Windows Integration & Platform Support

Windows users get native expectations (notifications, window controls, theme respect).

### Story 12.1: Runs on Windows 10 and Windows 11

**Implements:** FR89

As a Windows user,
I want the application to run on Windows 10 and Windows 11,
So that I can use the app on supported Windows versions.

**Acceptance Criteria:**

**Given** I am using Windows 10 or Windows 11  
**When** I install and launch the application  
**Then** the app starts successfully and core workflows are usable (sign in, open conversation, send message)

### Story 12.2: Respect Windows System Dark/Light Theme Setting

**Implements:** FR90

As a Windows user,
I want the app to respect the Windows system dark/light theme setting,
So that the app matches my OS theme preference.

**Acceptance Criteria:**

**Given** I have not explicitly chosen a theme in the app  
**When** Windows is set to light or dark mode  
**Then** the app uses the corresponding theme by default

**Given** Windows theme changes while the app is running  
**When** I have not explicitly chosen a theme in the app  
**Then** the app updates to match the new system theme

### Story 12.3: Windows Notifications for New Messages (Preview + Focus + Deep Link)

**Implements:** FR91, FR92, FR93

As a Windows user,
I want Windows notifications for new messages,
So that I can notice messages when I’m not actively watching the app.

**Acceptance Criteria:**

**Given** I receive a new message  
**When** the app triggers a Windows notification  
**Then** the notification includes the sender and a message preview (as permitted)  
**And** clicking the notification focuses the app window and opens the relevant conversation

### Story 12.4: Muting Only Affects In-App Notification Signals

**Implements:** (Derived)
**Derived Requirement:** Clarifies scope of conversation muting vs. OS notifications (not explicitly listed as a PRD FR)

As a Windows user,
I want conversation muting to affect only in-app notification signals,
So that muting behavior is predictable and contained within the app.

**Acceptance Criteria:**

**Given** a conversation is muted  
**When** a new message arrives for that conversation  
**Then** in-app notification signals for that conversation are suppressed per the mute rules  
**And** OS-level notification behavior remains controlled by the app’s Windows notification rules

### Story 12.5: Standard Window Controls + Window State Persistence

**Implements:** FR94, FR95, FR96

As a Windows user,
I want standard window controls and window state to persist across restarts,
So that the app behaves like a normal desktop application.

**Acceptance Criteria:**

**Given** I use standard window controls (minimize, maximize/restore, close)  
**When** I interact with the window  
**Then** the app responds as expected for a desktop application

**Given** I resize or reposition the window and close the app  
**When** I relaunch the app  
**Then** the app restores the last used window size and position (where supported)

## Epic 13: Responsive Layout & UI Adaptation

Users can use the app comfortably across window sizes with preserved context.

### Story 13.1: Responsive Layout Breakpoints (Narrow/Medium/Wide)

**Implements:** FR97, FR99, FR100, FR101, FR102, FR103, FR104

As a user,
I want the UI to adapt at defined breakpoints (narrow, medium, wide),
So that it remains usable and readable across window sizes.

**Acceptance Criteria:**

**Given** I resize the app window across supported sizes (minimum 640x480) and across narrow, medium, and wide widths  
**When** the layout changes  
**Then** core UI regions (conversation list, message view, composer) remain usable at the minimum supported size  
**And** presence indicators and the connection status indicator remain visible at all supported window sizes  
**And** the layout adapts without overlapping or inaccessible controls  
**And** scrollbars appear only when content exceeds available space  
**And** all text remains readable at supported window sizes

### Story 13.2: Collapsible Conversation List Drawer on Narrow Widths

**Implements:** FR98

As a user on a narrow window,
I want the conversation list to be accessible via a collapsible drawer,
So that I can focus on the current conversation while still being able to switch.

**Acceptance Criteria:**

**Given** the app is in a narrow layout  
**When** I open the conversation list drawer  
**Then** I can select a conversation and navigate to it  
**And** the drawer can be closed to return focus to the conversation view

### Story 13.3: Preserve Message List Anchor/Scroll on Window Resize

**Implements:** FR102

As a user,
I want my position in the message list preserved when the window is resized,
So that I don’t lose context while reading.

**Acceptance Criteria:**

**Given** I am reading messages at a particular scroll position  
**When** I resize the app window causing reflow  
**Then** the app preserves my reading position using a stable anchor/scroll strategy  
**And** the message list does not jump unexpectedly away from the current context

### Story 13.4: Responsive Typography and Spacing Adjustments

**Implements:** FR104

As a user,
I want typography and spacing to adapt appropriately across layouts,
So that content remains legible and visually balanced.

**Acceptance Criteria:**

**Given** I view the app in narrow, medium, and wide layouts  
**When** content is displayed  
**Then** typography and spacing remain legible and consistent with the design system  
**And** the UI avoids cramped or excessively sparse layouts

## Epic 14: Performance Targets & Responsiveness

Users experience fast, responsive interactions with measurable targets.

### Story 14.1: Define MVP Performance Targets (Start + Open Conversation)

**Implements:** FR105, FR106

As a user,
I want the app to feel fast for the most common actions,
So that I can start chatting without delays.

**Acceptance Criteria:**

**Given** I cold start the application  
**When** the app becomes ready for use  
**Then** it is ready within 2 seconds

**Given** I switch between two conversations  
**When** the conversation view changes  
**Then** the switch completes in under 100ms

### Story 14.2: Basic In-App Timing Logs for Key Actions

**Implements:** FR107, FR108, FR109

As a product/team member,
I want basic in-app timing logs for key actions,
So that we can measure performance and regressions during development.

**Acceptance Criteria:**

**Given** the app runs in a development or diagnostics mode  
**When** I cold start the app, switch conversations, send a message, or receive presence updates  
**Then** the app records basic timing metrics for those actions  
**And** the metrics can be reviewed for troubleshooting and tuning

**Given** I send a message successfully  
**When** the UI reflects the sent message in the conversation  
**Then** the message appears within 500ms

**Given** a presence update is received  
**When** the UI reflects the new presence state  
**Then** the update appears within 1 second

### Story 14.3: Responsive UI Under Low Connectivity (Progress + Error States)

**Implements:** FR110, FR111, FR112

As a user,
I want the UI to remain responsive under low connectivity,
So that I can understand what’s happening and continue working where possible.

**Acceptance Criteria:**

**Given** connectivity is slow, intermittent, or unavailable  
**When** I perform common actions (e.g., navigate, open a conversation, send)  
**Then** the UI remains responsive and communicates progress and/or errors clearly  
**And** the app uses best-effort behavior rather than freezing or becoming unresponsive

**Given** a conversation contains 100+ messages  
**When** I open the conversation and scroll through history  
**Then** the app remains usable without noticeable degradation (per MVP targets)

**Given** the backend connection is slow  
**When** I use the app  
**Then** the app continues functioning with appropriate loading/progress states

**Given** the connection drops temporarily and then returns  
**When** connectivity is restored  
**Then** the app recovers gracefully and resumes normal operation without requiring a restart
