# Story 6.5: Delivery Status Sync with Real-Time Updates

**Status:** pending
**Priority:** P1 (MVP High - Connection Resilience)
**Week:** 3 (Real-Time & Message List)
**Epic:** Epic 6 - Connection & Sync Resilience
**Story Number:** 13

---

## 📖 Story Description

**As a** chat user,
**I want** delivery status (sent, delivered, read) for my messages to sync automatically after reconnection,
**So that** I have confidence my messages reached recipients even during connectivity issues.

**User Value:** Users can trust message delivery without manual intervention. If the connection drops and restores, delivery confirmations automatically sync, maintaining accurate message state.

---

## 🎯 Acceptance Criteria

- [ ] **AC1: Delivery Status Tracking** - Messages sent during disconnection maintain a delivery queue that persists across app restarts.
- [ ] **AC2: Automatic Sync on Reconnection** - When connection is restored, all queued delivery status updates are sent to backend in order.
- [ ] **AC3: No Duplicate Deliveries** - Backend correctly handles idempotent delivery status updates (duplicates ignored, not creating duplicate state changes).
- [ ] **AC4: User Visibility** - Message delivery status (pending/sent/delivered/read) updates are visible in the UI as they sync.
- [ ] **AC5: Reliable State** - User state remains consistent between client and backend after reconnection (no stale delivery states).
- [ ] **AC6: Performance Target** - Delivery status updates sync and UI reflects changes within 500ms of reconnection.

---

## 🛠️ Developer Context

### Files & Components Involved

**Frontend:**
- `src/frontend/handlers/delivery_handlers.rs` (New - delivery status handlers)
- `src/frontend/handlers/connection_handlers.rs` (Existing - connection lifecycle)
- `src/frontend/components/messaging/MessageBubble.slint` (Update - show delivery status)
- `src/frontend/state/app_state.rs` (Update - delivery queue support)

**Backend:**
- `src/backend/services/message_service.rs` (Delivery status updates)
- `src/backend/handlers/messages.rs` (Handle delivery sync commands)
- `src/backend/handlers/dispatcher.rs` (Route delivery commands)
- `src/shared/protocol/mod.rs` (Delivery command/event types)

**Testing:**
- `tests/integration/delivery_status_sync_test.rs` (New)
- `tests/contract/delivery_schema_test.rs` (New)

### Architecture Patterns to Follow

**1. Command/Event Pattern (Mandatory)**
   - Frontend sends: `SyncDeliveryStatus` command
   - Backend responds with: `DeliveryStatusUpdated` events
   - Format: `{ id, command/event, payload }`
   - Serialization: serde with `rename_all = "camelCase"`

**2. State Management (Mandatory)**
   - Use `AppState` as single source of truth
   - Delivery queue stored in: `AppState.delivery_status_queue`
   - Update via mutable references, NOT clones
   - React via Slint property bindings

**3. Error Handling (Mandatory)**
   - Errors communicated as `Error` events (not exceptions)
   - Standardized error structure with `code`, `message`, `recoverable`, `retryable`
   - Retry failed syncs with exponential backoff (100ms → 200ms → 400ms, max 3 attempts)

**4. Component Organization (Mandatory)**
   - Domain-based: all delivery-related in `handlers/delivery_handlers.rs`
   - File naming: PascalCase components (MessageBubble.slint)
   - One component per file
   - All visual elements use design tokens

### Technical Specifications

**WebSocket Protocol - Delivery Sync Commands:**

```json
// Frontend → Backend (New - during reconnection)
{
  "id": "cmd-delivery-sync-123",
  "command": "SyncDeliveryStatus",
  "payload": {
    "deliveryUpdates": [
      {
        "messageId": "msg-456",
        "status": "sent"
      },
      {
        "messageId": "msg-457",
        "status": "sent"
      }
    ]
  }
}
```

**WebSocket Protocol - Delivery Status Events:**

```json
// Backend → Frontend (Existing pattern, verify consistency)
{
  "id": "evt-delivery-123",
  "event": "DeliveryStatusUpdated",
  "payload": {
    "messageId": "msg-456",
    "status": "delivered",
    "timestamp": "2025-12-18T10:30:00Z"
  }
}

// Backend → Frontend (Multiple updates)
{
  "id": "evt-delivery-batch-123",
  "event": "DeliveryStatusBatchUpdated",
  "payload": {
    "updates": [
      { "messageId": "msg-456", "status": "delivered", "timestamp": "2025-12-18T10:30:00Z" },
      { "messageId": "msg-457", "status": "delivered", "timestamp": "2025-12-18T10:30:01Z" }
    ]
  }
}
```

**Backend Delivery Sync Service:**

```rust
// In src/backend/services/message_service.rs
pub fn sync_delivery_status(
    &self,
    user_id: &str,
    updates: Vec<DeliveryStatusUpdate>
) -> Result<Vec<Message>, Error> {
    // For each update:
    // 1. Verify message belongs to sender or recipient
    // 2. Update message status in database (idempotent)
    // 3. Return list of updated messages for client confirmation
}

// In src/backend/handlers/messages.rs
pub async fn handle_sync_delivery_status(
    state: &mut AppState,
    cmd: SyncDeliveryStatusCommand
) -> Result<DeliveryStatusBatchUpdatedEvent, ErrorEvent> {
    // Route to message_service.sync_delivery_status
    // Broadcast to affected recipients
    // Return confirmation event
}
```

**Frontend Delivery Queue Management:**

```rust
// In src/frontend/state/app_state.rs
#[derive(Debug, Clone)]
pub struct AppState {
    // ... existing fields
    pub delivery_status_queue: Vec<PendingDeliveryUpdate>,
    pub is_syncing_delivery: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingDeliveryUpdate {
    pub message_id: String,
    pub status: String,  // "sent", "delivered", "read"
    pub queued_at: i64,  // timestamp for ordering
}
```

**Frontend Handler Pattern:**

```rust
// In src/frontend/handlers/delivery_handlers.rs
pub fn handle_delivery_status_updated(
    state: &mut AppState,
    event: DeliveryStatusUpdatedEvent,
) {
    // Update message status in message_cache
    if let Some(messages) = state.message_cache.get_mut(&event.conversation_id) {
        if let Some(msg) = messages.iter_mut().find(|m| m.id == event.message_id) {
            msg.delivery_status = event.status;
            msg.delivery_timestamp = Some(event.timestamp);
        }
    }
    
    // Remove from delivery queue if present
    state.delivery_status_queue.retain(|u| u.message_id != event.message_id);
}

pub fn handle_connection_restored(state: &mut AppState) {
    // Trigger delivery sync if queue not empty
    if !state.delivery_status_queue.is_empty() {
        state.is_syncing_delivery = true;
        // Queue sync command to send on next WebSocket write
    }
}
```

**Idempotency Pattern (Critical):**

```rust
// Backend: Idempotent delivery status update
pub fn update_delivery_status_idempotent(
    &self,
    message_id: &str,
    new_status: &str,
) -> Result<Message, Error> {
    // 1. Load current message
    let current = self.get_message(message_id)?;
    
    // 2. Determine if update is valid (don't downgrade status)
    let status_order = ["pending", "sent", "delivered", "read"];
    let current_idx = status_order.iter().position(|s| *s == current.status)?;
    let new_idx = status_order.iter().position(|s| *s == new_status)?;
    
    // 3. Only update if new status is equal or higher in order
    if new_idx >= current_idx {
        self.db.update_message_status(message_id, new_status)?;
        return self.get_message(message_id);
    }
    
    // 4. Return existing message unchanged (idempotent)
    Ok(current)
}
```

---

## 📝 Implementation Tasks

### **Task 1: Backend Delivery Sync Service** (Day 1-2)

**Objective:** Implement backend support for delivery status synchronization with idempotency.

**Subtasks:**
- [ ] In `src/backend/services/message_service.rs`:
  - [ ] Add `sync_delivery_status` method that accepts batch updates
  - [ ] Implement idempotent status updates (don't downgrade status)
  - [ ] Verify sender/recipient authorization for each message
  - [ ] Return list of updated messages for confirmation
  
- [ ] In `src/backend/handlers/messages.rs`:
  - [ ] Add `handle_sync_delivery_status` handler
  - [ ] Route command from dispatcher
  - [ ] Broadcast updated messages to all recipients in affected conversations
  - [ ] Send confirmation event back to sender with sync timestamp
  
- [ ] In `src/shared/protocol/mod.rs`:
  - [ ] Add `SyncDeliveryStatusCommand` type
  - [ ] Add `DeliveryStatusUpdatedEvent` type
  - [ ] Add `DeliveryStatusBatchUpdatedEvent` type
  - [ ] Ensure Serde derives with `rename_all = "camelCase"`

**Definition of Done:**
- [ ] Backend handlers accept and process delivery sync commands
- [ ] All updates are idempotent
- [ ] Authorization verified for each message
- [ ] Unit tests pass (idempotency verification)
- [ ] Integration test passes (multi-user sync scenario)

---

### **Task 2: Frontend State & Queue Management** (Day 1-2)

**Objective:** Implement client-side delivery queue that persists and syncs on reconnection.

**Subtasks:**
- [ ] In `src/frontend/state/app_state.rs`:
  - [ ] Add `delivery_status_queue: Vec<PendingDeliveryUpdate>` field
  - [ ] Add `is_syncing_delivery: bool` flag
  - [ ] Implement queue persistence (store on disk or in session)
  
- [ ] In `src/frontend/handlers/delivery_handlers.rs` (New file):
  - [ ] Implement `handle_delivery_status_updated` handler
  - [ ] Implement `handle_delivery_status_batch_updated` handler
  - [ ] Update `AppState.message_cache` with new delivery status
  - [ ] Remove synced messages from delivery queue
  
- [ ] In `src/frontend/handlers/connection_handlers.rs`:
  - [ ] Add reconnection hook to trigger delivery sync
  - [ ] Queue `SyncDeliveryStatus` command on reconnection
  - [ ] Set `is_syncing_delivery` flag during sync
  - [ ] Clear flag when sync completes or fails

**Definition of Done:**
- [ ] Delivery queue persists across app restarts
- [ ] Queue automatically syncs on reconnection
- [ ] Synced messages removed from queue
- [ ] Delivery status updates reflected in UI immediately
- [ ] Unit tests pass (queue management)

---

### **Task 3: Message Status UI Display** (Day 2)

**Objective:** Update message bubble to show delivery status with visual indicators.

**Subtasks:**
- [ ] In `src/frontend/components/messaging/MessageBubble.slint`:
  - [ ] Add `delivery-status: String` property
  - [ ] Add conditional display for status indicators
  - [ ] Use design tokens for status colors (sent=gray, delivered=blue, read=green)
  - [ ] Show tooltip with timestamp on hover
  
- [ ] Status Display Rules (Use Design Tokens):
  - [ ] "pending" → Gray spinner icon + "Sending..." text
  - [ ] "sent" → Gray checkmark icon + "Sent" text
  - [ ] "delivered" → Blue checkmark icon + "Delivered" text
  - [ ] "read" → Blue double-checkmark icon + "Read" text
  - [ ] Failed/error → Red icon + "Failed - Tap to retry"

**Definition of Done:**
- [ ] All delivery statuses display with correct visual indicators
- [ ] Indicators use design system tokens
- [ ] Status updates immediately when delivery status changes
- [ ] Timestamps display on hover
- [ ] Visual tests pass

---

### **Task 4: Connection Lifecycle Integration** (Day 1)

**Objective:** Hook delivery sync into connection restoration events.

**Subtasks:**
- [ ] In `src/backend/handlers/websocket.rs`:
  - [ ] Verify `handle_client_connected` broadcasts connection status
  - [ ] Verify `handle_client_disconnected` marks user offline (related to presence, Task 5)
  
- [ ] In `src/frontend/handlers/connection_handlers.rs`:
  - [ ] On `ConnectionRestored` event:
    - [ ] Check if `delivery_status_queue` is not empty
    - [ ] If not empty: queue `SyncDeliveryStatus` command
    - [ ] Set `is_syncing_delivery = true`
  - [ ] On `SyncDeliveryStatusCompleted` event:
    - [ ] Set `is_syncing_delivery = false`
    - [ ] Log successful sync

**Definition of Done:**
- [ ] Delivery sync triggered on reconnection
- [ ] Queue persists during disconnect
- [ ] Sync completes without blocking UI
- [ ] Integration tests pass

---

### **Task 5: Error Handling & Retry Logic** (Day 2-3)

**Objective:** Handle sync failures gracefully with automatic retry.

**Subtasks:**
- [ ] In `src/frontend/handlers/delivery_handlers.rs`:
  - [ ] Implement retry logic with exponential backoff (100ms → 200ms → 400ms)
  - [ ] Max 3 retry attempts
  - [ ] After max retries, notify user with error message
  - [ ] Allow manual retry via UI button
  
- [ ] Error Event Handling:
  - [ ] Handle `DeliveryStatusSyncFailed` error event
  - [ ] Update `AppState.last_error` for UI display
  - [ ] Keep delivery queue for retry
  - [ ] Show error toast with "Retry" button

**Definition of Done:**
- [ ] Failed syncs retry automatically
- [ ] Max retries respected
- [ ] User can manually retry
- [ ] Error messages clear and actionable
- [ ] Tests pass (retry behavior)

---

### **Task 6: Integration Testing** (Day 2-3)

**Objective:** Verify end-to-end delivery status sync behavior.

**Subtasks:**
- [ ] In `tests/integration/delivery_status_sync_test.rs` (New):
  - [ ] Test 1: Simulate disconnect → multiple messages sent → reconnect → verify status sync
  - [ ] Test 2: Multiple delivery updates queued → single sync command sent → all processed
  - [ ] Test 3: Duplicate delivery updates on reconnect → verify idempotency (no duplicates)
  - [ ] Test 4: Failed sync with retry → verify exponential backoff and max attempts
  - [ ] Test 5: Delivery sync during active messaging → verify no message loss
  
- [ ] In `tests/contract/delivery_schema_test.rs` (New):
  - [ ] Verify `SyncDeliveryStatus` command schema
  - [ ] Verify `DeliveryStatusUpdated` event schema
  - [ ] Verify JSON field names (camelCase)
  - [ ] Verify required fields present

**Definition of Done:**
- [ ] All integration tests pass
- [ ] Contract tests pass
- [ ] Manual smoke test: Send message → disconnect → reconnect → verify status updates

---

## 🔧 Key Implementation Details

### Frontend Delivery Queue Implementation

```rust
// Add to AppState
pub delivery_status_queue: Vec<PendingDeliveryUpdate>,
pub is_syncing_delivery: bool,

// When sending a message (from message_handlers.rs):
pub fn handle_send_message_command(state: &mut AppState, cmd: SendMessageCommand) {
    let message = Message {
        id: generate_uuid(),
        text: cmd.text,
        delivery_status: "pending",  // Start as pending
        // ... other fields
    };
    
    // Add to cache with pending status
    state.message_cache.entry(cmd.conversation_id).or_insert_with(Vec::new).push(message.clone());
    
    // Queue for delivery sync if offline
    if state.connection_status == "disconnected" {
        state.delivery_status_queue.push(PendingDeliveryUpdate {
            message_id: message.id,
            status: "sent",  // Will be sent when connection restores
            queued_at: now(),
        });
    }
}

// When connection restores:
pub fn handle_connection_restored(state: &mut AppState) {
    if !state.delivery_status_queue.is_empty() {
        // Automatically send sync command
        state.is_syncing_delivery = true;
        // Queue will be processed by WebSocket handler
    }
}
```

### Backend Idempotency Implementation

```rust
// Status upgrade hierarchy: pending < sent < delivered < read
// Never downgrade status, only upgrade

pub fn update_delivery_status_idempotent(
    message_id: &str,
    new_status: &str,
) -> Result<Message> {
    let msg = db.get_message(message_id)?;
    
    // Check if new status is upgrade or same
    let status_weights = hashmap! {
        "pending" => 0,
        "sent" => 1,
        "delivered" => 2,
        "read" => 3,
    };
    
    if status_weights.get(new_status)? >= status_weights.get(&msg.status)? {
        db.update(message_id, new_status)?;
    }
    
    db.get_message(message_id)  // Return current state (unchanged or updated)
}
```

### Connection Lifecycle Hooks

**Backend (in handlers/websocket.rs):**
```rust
pub async fn on_client_connected(user_id: &str, connection: ClientConnection) {
    // 1. Mark user online in presence service
    presence_service.mark_online(user_id).await;
    
    // 2. Broadcast presence update to all connections
    broadcast_presence_changed(user_id, "online");
    
    // 3. Client can now sync any pending delivery statuses
}

pub async fn on_client_disconnected(user_id: &str) {
    // 1. Mark user offline in presence service
    presence_service.mark_offline(user_id).await;
    
    // 2. Broadcast presence update to all connections
    broadcast_presence_changed(user_id, "offline");
}
```

**Frontend (in handlers/connection_handlers.rs):**
```rust
pub fn handle_connection_restored(state: &mut AppState) {
    state.connection_status = "connected";
    
    // Trigger delivery status sync if queue not empty
    if !state.delivery_status_queue.is_empty() {
        state.is_syncing_delivery = true;
        // Command will be sent by WebSocket dispatcher
    }
}
```

---

## 📊 Validation & Quality Checklist

- [ ] **AC1:** Messages sent during disconnect are queued (storage verified)
- [ ] **AC2:** All queued updates synced on reconnection (test verified)
- [ ] **AC3:** Backend correctly handles idempotent updates (duplicate test passed)
- [ ] **AC4:** Message status visible in UI with correct indicators (visual test passed)
- [ ] **AC5:** Client/backend state consistent after sync (integration test passed)
- [ ] **AC6:** Sync completes within 500ms (performance test passed)
- [ ] Protocol fields use camelCase JSON (schema validation passed)
- [ ] All components use design tokens (visual audit passed)
- [ ] Error handling as events (not exceptions) (code review passed)
- [ ] Handler naming follows patterns: `handle_*` (naming audit passed)
- [ ] State updates use mutable references (not clones) (efficiency review passed)
- [ ] Retry logic with exponential backoff implemented (behavior test passed)
- [ ] Integration tests cover multi-user scenarios (test coverage passed)
- [ ] Code review approved by architecture reviewer
- [ ] No hardcoded values (all in AppState or config)

---

## 🔗 Dependencies & Blocking

**Blocks:**
- Story 6.6 (Send Failure Messaging and Manual Retry)
- Full offline support testing

**Blocked By:**
- Story 6.1 (Connection Status Indicator - needed for UX)
- Story 6.3 (Manual Reconnect Action - needed for testing)
- Story 4.5 (Real-Time Presence Updates - parallel sync pattern)

**Related Stories:**
- Story 3.4 (Read Receipts Display) - uses same delivery status pattern
- Story 2.6 (Delivery Feedback) - shares delivery status enum

---

## 📈 Estimation & Complexity

- **Story Points:** 8 (Medium-High)
- **Estimated Duration:** 2-3 days
- **Complexity:** Medium
  - Idempotency logic requires careful state management
  - Connection lifecycle integration requires coordination with presence
  - Queue persistence adds persistence layer
- **Risk Level:** Medium
  - Idempotency bugs could cause duplicate status updates
  - Lifecycle timing issues could cause race conditions
  - Queue persistence across restarts needs careful testing

---

## 🎓 Learning Points for Developer

**Critical Concepts:**
1. **Idempotent Operations:** Status updates must never create duplicates or downgrade state
2. **Queue Persistence:** Delivery queue must survive app crashes
3. **Connection Lifecycle:** Coordinate with presence updates during reconnection
4. **Batch Operations:** Optimize by syncing multiple updates in single command
5. **Error Resilience:** Graceful handling of network failures with retry

**Architecture Patterns Applied:**
- Command/Event pattern for clear semantics
- Centralized AppState for single source of truth
- Mutable references for efficient state updates
- Design tokens for consistent UI
- Domain-organized handlers for maintainability
- Error-as-events for offline-first design

---

## 📅 Completion Log

- **Created:** 2025-12-18 (by create-story workflow)
- **Story Status:** pending → **in-progress** (code review fixes applied)
- **Approved:** Yes (workflow validation passed)
- **Code Review Date:** 2025-12-18
- **Review Outcome:** 6 CRITICAL + 4 MEDIUM issues found and auto-fixed

### Code Review Fixes Applied (AI-Assisted)

**CRITICAL FIXES:**
- ✅ **CRITICAL #1:** Updated story status from `ready-for-dev` → `pending` (story not yet started)
- ✅ **CRITICAL #2:** Added `Read` status variant to `MessageStatus` enum (src/shared/protocol/mod.rs:7-14)
- ✅ **CRITICAL #3:** Added `read_at: Option<i64>` field to Message model (src/backend/models/mod.rs:97)
- ✅ **CRITICAL #4:** Created `/src/frontend/handlers/` directory with base module structure
  - Created `delivery_handlers.rs` with delivery event types
  - Created `connection_handlers.rs` with connection event types
  - Created `mod.rs` handler module registry
- ✅ **CRITICAL #5:** Added `handle_sync_delivery_status` async function to MessageHandler (src/backend/handlers/messages.rs:240-345)
  - Implements idempotent status update logic with hierarchy validation
  - Broadcasts delivery status updates to conversation participants
- ✅ **CRITICAL #6:** Added protocol types to shared/protocol/mod.rs (lines 178-234):
  - `DeliveryStatusUpdate`
  - `SyncDeliveryStatusCommand`
  - `DeliveryStatusUpdatedEvent`
  - `DeliveryStatusBatchUpdatedEvent`
  - `SyncDeliveryStatusCompletedEvent`
  - `DeliveryStatusSyncFailedEvent`

**MEDIUM FIXES:**
- ✅ **MEDIUM #1:** Enhanced message_bubble.slint status color mapping (src/frontend/components/message_bubble.slint:112-127)
  - Added "read" status handling with checkmark-double icon
  - Implemented status-based color scheme: pending/sent=gray, delivered/read=blue, failed=red
- ✅ **MEDIUM #2:** Created `tests/integration/delivery_status_sync_test.rs` (comprehensive test suite)
  - US-013-001: Multi-message sync on reconnection
  - US-013-002: Batch delivery update processing
  - US-013-003: Idempotent delivery update handling
  - US-013-004: Status downgrade prevention
  - US-013-005: Performance target verification (500ms)
  - US-013-006: Unauthorized update prevention
  - US-013-007: Read status timestamp tracking
- ✅ **MEDIUM #3:** Added `sync_delivery_status` method to MessageService (src/backend/services/message_service.rs:282-370)
  - Implements idempotent batch status updates with authorization checks
  - Returns updated messages for client confirmation
- ✅ **MEDIUM #4:** Added camelCase-serialized protocol types (covered in CRITICAL #6)

### File Changes Summary

| File | Changes | Status |
|------|---------|--------|
| docs/sprint-artifacts/us-013-delivery-status-sync-realtime.md | Story status updated to pending | ✅ |
| src/shared/protocol/mod.rs | Added Read variant + 6 new protocol types | ✅ |
| src/backend/models/mod.rs | Added read_at field + is_read() helper | ✅ |
| src/frontend/handlers/mod.rs | **NEW** - Base handler module | ✅ |
| src/frontend/handlers/delivery_handlers.rs | **NEW** - Delivery event types | ✅ |
| src/frontend/handlers/connection_handlers.rs | **NEW** - Connection event types | ✅ |
| src/backend/handlers/messages.rs | Added handle_sync_delivery_status() | ✅ |
| src/backend/services/message_service.rs | Added sync_delivery_status() | ✅ |
| src/frontend/components/message_bubble.slint | Updated status colors/icons | ✅ |
| tests/integration/delivery_status_sync_test.rs | **NEW** - 7 AC-focused tests | ✅ |

---

## 🚀 Ready for Implementation

This story now provides:
1. ✅ Clear acceptance criteria with measurable outcomes
2. ✅ Detailed technical requirements with code patterns
3. ✅ Task breakdown with clear subtasks
4. ✅ Foundation code (protocol types, models, handlers)
5. ✅ Error handling strategies
6. ✅ Comprehensive integration test suite
7. ✅ Performance verification (500ms target)
8. ✅ Security/idempotency considerations

**Developer can now proceed with implementation - all blocking issues resolved.**

### Next Steps for Developer
1. Implement frontend state management (AppState delivery_queue)
2. Implement frontend handlers for delivery event processing
3. Connect WebSocket to delivery sync commands
4. Run integration tests: `cargo test delivery_status_sync`
5. Verify 500ms performance target under load

