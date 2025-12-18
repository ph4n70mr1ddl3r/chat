# Story 2.1: Implement MessageInput Container (Slint + Rust)

**Status:** ready-for-dev
**Priority:** P0 (Critical)
**Week:** 2
**Owner:** Amelia (Developer)
**Designer:** Sally
**Reviewer:** Winston

## 📖 Story Description
**As a** chat user,
**I want** a dedicated message composer with clear send readiness feedback,
**So that** I can confidently compose messages and know when they can be sent.

## 🎯 Acceptance Criteria

- [ ] **AC1: Visual Composition UI:**
    - Minimal, professional design following Fluent principles.
    - Spacious text input area (multi-line support, expanding up to 4 lines).
    - Clear placeholder text ("Type a message...").
- [ ] **AC2: Send Readiness Feedback:**
    - Send button is **disabled** when the input is empty or whitespace-only.
    - Send button becomes **enabled** as soon as text is entered.
    - Visual feedback (hover/pressed states) follows `Tokens`.
- [ ] **AC3: Keyboard Interaction:**
    - `Enter` key triggers the `send` callback/command.
    - `Ctrl+Enter` (or `Shift+Enter`) inserts a line break.
    - `Esc` clears the currently focused selection or returns focus to the message list.
- [ ] **AC4: Sending State:**
    - When sending is in progress (`is_sending` property), the input is disabled.
    - The send button displays a `LoadingSpinner` or "Sending..." state.
- [ ] **AC5: Error Handling:**
    - If sending fails (e.g., offline), the message text is preserved in the input.
    - A clear error message is displayed below the input area.
- [ ] **AC6: Typing Notifications:**
    - Emits a `typing(true)` event when the user starts typing.
    - Emits `typing(false)` after a short period of inactivity (debounced) or when the message is sent.

## 🛠️ Developer Context
- **Component Path:** `src/frontend/components/messaging/MessageComposer.slint`
- **Container Path:** `src/frontend/containers/message_composer.rs` (if using container pattern) or integrated into the Screen handler.
- **Base Components:** 
    - Wraps/Uses `src/frontend/components/shared/Input.slint` or a specialized `MessageInput.slint`.
    - Uses `Button` and `LoadingSpinner` from shared components.
- **Design System Rules:**
    - Follows 8px grid for padding/spacing.
    - Uses `ACCENT` (#0078D4) for the active send button.
    - Font: `Tokens.font-family`, Size: `Tokens.font-size-base`.
- **State Integration:**
    - Bind `text` property to the local draft state in `AppState`.
    - Trigger `SendMessage` command on send action.
    - Handle `MessageSent` or `Error` events to update UI state.

## 📝 Tasks

### Task 1: UI Implementation (Slint)
- [ ] Create/Refactor `src/frontend/components/messaging/MessageComposer.slint`.
- [ ] Implement multi-line `TextEdit` or `LineEdit` with expansion logic.
- [ ] Add `is_sending`, `error_text`, and `draft_text` properties.
- [ ] Implement `send` and `typing` callbacks.

### Task 2: Logic & Keyboard Handling
- [ ] Implement `Enter` vs `Ctrl+Enter` logic in Slint or Rust handler.
- [ ] Add debounced typing notification logic (emit `TypingIndicator` command).
- [ ] Implement draft preservation logic (update `AppState.message_drafts` on change).

### Task 3: Backend Integration
- [ ] Map `send` callback to `handle_send_message` in Rust.
- [ ] Send `SendMessage` command via WebSocket.
- [ ] Handle server response to clear input on success or show error on failure.

### Task 4: Verification & Testing
- [ ] Create `tests/integration/message_composer_test.rs`.
- [ ] Verify enter-to-send and empty-state-disabled behavior.
- [ ] Manual test: Verify line breaks and multi-line expansion.

## 📊 Definition of Done Checklist

- [ ] AC1-6 pass
- [ ] Follows PascalCase naming for .slint files
- [ ] Uses centralized `Tokens` for all styling
- [ ] No hardcoded strings (use configuration or constants)
- [ ] Unit/Integration tests pass
- [ ] Code review approved

---

## 📈 Estimation
- **Size:** M (3-5 days)
- **Complexity:** Medium
- **Risk:** Low-Medium (Keyboard handling edge cases)

## 🔗 Dependencies
- **Blocks:** Story 2.2 (Send Message Confirmation)
- **Blocked by:** Design Tokens (US-001), Button Component (US-002)

---

## 📅 Completion Notes List
- 2025-12-18: Story created by Bob (Scrum Master).
