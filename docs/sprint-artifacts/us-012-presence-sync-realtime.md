# Story 4.5: Real-Time Presence Synchronization (WebSockets)

**Status:** ready-for-dev
**Priority:** P1 (MVP High)
**Week:** 3
**Owner:** Amelia (Developer)
**Designer:** Sally
**Reviewer:** Winston

## 📖 Story Description
**As a** chat user,
**I want** presence changes to update in real time and see last-seen timestamps,
**So that** I have accurate, timely context about availability without manual refreshing.

## 🎯 Acceptance Criteria

- [ ] **AC1: Real-time Updates:** When a participant's presence state changes (online/offline/away), other users sharing a conversation see the status update in the UI immediately (< 200ms latency target).
- [ ] **AC2: Reliable Connection Lifecycle:** 
    - Closing the app or losing connection marks the user offline for others.
    - Reconnecting/Opening the app restores the online status and broadcasts it.
- [ ] **AC3: Presence Data Consistency:** The `AppState.presence_map` reflects the latest state received from the server for all relevant contacts.
- [ ] **AC4: Last Seen Support:** Offline users display their last-seen timestamp (if available) in the UI, formatted according to design standards.
- [ ] **AC5: Multi-session Awareness:** If a user has multiple sessions (e.g., two desktop instances), they appear online if at least one session is active.

## 🛠️ Developer Context
- **Component Path:** `src/frontend/handlers/presence_handlers.rs` (New)
- **Shared Protocol:** `chat_shared::protocol::PresenceData`
- **Backend Service:** `src/backend/services/presence.rs` (Partially implemented)
- **Frontend State:** `AppState.presence_map` (HashMap<String, UserPresence>)

### Technical Requirements
- **WebSocket Protocol:** 
    - Message Type: `"presence"`
    - Payload: `PresenceData { user_id: String, username: String, is_online: bool, last_seen_at: u64 }`
- **Backend Integration:**
    - Ensure `PresenceService::mark_online` is called in `src/backend/handlers/websocket.rs` during the connection handshake.
    - Ensure `PresenceService::mark_offline` is called during connection closed/dropped.
- **Frontend Integration:**
    - Implement `handle_presence_changed` in the frontend event dispatcher.
    - Update `AppState` reactively so that `PresenceIndicator` components (Story 4.2) re-render automatically.

### Performance & Security
- **Latency:** Broadcast should occur immediately after state change.
- **Scaling:** Backend `broadcast_presence` currently iterates through conversations; verify efficiency for users with high conversation counts.

## 📝 Tasks

### Task 1: Backend Connection Lifecycle Integration
- [ ] In `src/backend/handlers/websocket.rs`, hook `PresenceService::mark_online` into the successful registration of a new `ClientConnection`.
- [ ] Hook `PresenceService::mark_offline` into the `unregister` flow.
- [ ] Verify `broadcast_presence` correctly identifies all participants in shared conversations.

### Task 2: Frontend WebSocket Handler
- [ ] In `src/frontend/lib.rs` (or handler module), add a case for the `"presence"` message type in the WebSocket event loop.
- [ ] Implement `handle_presence_changed` in `src/frontend/handlers/presence_handlers.rs` to update the global `AppState`.

### Task 3: AppState & UI Binding
- [ ] Ensure `AppState` properly manages the `presence_map`.
- [ ] Connect `PresenceIndicator` (from `us-009`) to the live data from `AppState`.

### Task 4: Integration Testing
- [ ] Create `tests/integration/presence_sync_test.rs`.
- [ ] Mock two WebSocket connections and verify that toggling user A's status updates user B's client state.

## 📊 Definition of Done Checklist
- [ ] AC1-5 pass.
- [ ] WebSocket handler for `"presence"` type implemented.
- [ ] Backend lifecycle hooks verified (Connect/Disconnect).
- [ ] Integration tests pass for multi-user scenarios.
- [ ] Code review approved.

---

## 📈 Estimation
- **Size:** S (2-3 days)
- **Complexity:** Medium (Real-time state sync)
- **Risk:** Medium (WebSocket lifecycle edge cases)

## 🔗 Dependencies
- **Blocks:** Story 4.3 (Presence in Multiple Surfaces)
- **Blocked by:** us-009 (PresenceIndicator Component), Backend WebSocket Foundation.

---

## 📅 Completion Notes List
- 2025-12-18: Story created by Bob (Scrum Master).
