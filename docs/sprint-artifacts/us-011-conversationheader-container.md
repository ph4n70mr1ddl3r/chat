# Story 4.1: Implement ConversationHeader Container (Slint + Rust)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a Team Lead (Elena),
I want to see who I'm talking to and their current status,
so that I understand the context and know if they're available for immediate collaboration.

## Acceptance Criteria

1. [ ] **AC1: Visual Header Layout:**
    - Header height is fixed at 56px.
    - Displays contact name prominently using `Heading 2` (16px) per `Tokens`.
    - Integrated `PresenceAvatar` component shown to the left of the name.
2. [ ] **AC2: Presence & Status Display:**
    - Displays real-time presence dot and status label ("Online", "Away", "Offline").
    - Shows "Last seen" time for users in "Offline" state.
    - Hovering on the presence section reveals a tooltip with extended status info.
3. [ ] **AC3: Action Menu:**
    - Includes a standard "three dots" menu button on the far right for options.
4. [ ] **AC4: Sticky & Responsive Behavior:**
    - Header remains sticky at the top of the conversation view during scrolling.
    - Contact name and status label truncate elegantly on narrow screens to prevent layout breakage.
5. [ ] **AC5: Real-time Integration:**
    - Status and presence information updates immediately (< 200ms) when backend events arrive.
6. [ ] **AC6: Accessibility:**
    - Screen reader announces the contact name and full status on focus.
    - Interactive elements (menu) have accessible labels.

## Tasks / Subtasks

- [ ] **Task 1: UI Implementation (Slint) (AC: 1, 3, 5)**
  - [ ] Create `src/frontend/components/presence/ConversationHeader.slint`.
  - [ ] Implement the 56px sticky container layout using `Tokens`.
  - [ ] Integrate `PresenceAvatar` and action menu button.
- [ ] **Task 2: Logic & Real-time Sync (AC: 2, 5)**
  - [ ] Bind component properties to `AppState` (current conversation participants and presence map).
  - [ ] Implement status label formatting logic (Active, Away [time], Offline [last seen]).
- [ ] **Task 3: Responsive & Polish (AC: 4)**
  - [ ] Apply truncation rules for text elements.
  - [ ] Ensure smooth transitions for status changes (unless reduced motion is active).
- [ ] **Task 4: Verification (AC: 1-6)**
  - [ ] Create integration test `tests/integration/conversation_header_test.rs`.
  - [ ] Verify sticky behavior and truncation manually across breakpoints (640px+).

## Dev Notes

- **Relevant patterns:** Domain-based organization (Presence domain), Centralized AppState, Command/Event pattern.
- **Source tree components:** 
    - `src/frontend/components/presence/ConversationHeader.slint`
    - `src/frontend/screens/chat_screen.slint` (to host the header)
- **Testing standards:** Integration tests for presence update reactivity and manual verification for sticky CSS/Slint logic.

### Project Structure Notes

- Alignment with `src/frontend/components/presence/` directory for presence-related UI.
- Uses `src/frontend/design/tokens.slint` for all styling constants.

### References

- [UX Spec: Presence & Availability](file:///wsl.localhost/riddler/home/riddler/chat/docs/ux-design-specification.md#L1926-1948)
- [Architecture: Domain-Based Organization](file:///wsl.localhost/riddler/home/riddler/chat/docs/architecture.md#L182-231)
- [Design Tokens Reference](file:///wsl.localhost/riddler/home/riddler/chat/docs/DESIGN_TOKENS_REFERENCE.md)

## Dev Agent Record

### Agent Model Used

Antigravity v1 (Scrum Master Persona)

### Debug Log References

### Completion Notes List

### File List
- `docs/sprint-artifacts/us-011-conversationheader-container.md`
