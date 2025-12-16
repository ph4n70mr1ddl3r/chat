# Sprint-Ready User Stories with DoR + DoD

**Derived from:** UX Design Specification (2,437 lines)  
**Created by:** Bob (Scrum Master) + Team  
**Date:** December 16, 2025  
**Total Stories:** 21 user stories  
**Sprint Distribution:** 6-week MVP roadmap  
**Reference:** DEFINITION_OF_READY_DONE.md

---

## Story Structure

Each story includes:
- **Story ID & Title:** US-### (reference in sprints)
- **Story Type:** Feature / Component / Integration / Bug Fix
- **Priority:** P0 (MVP Critical) | P1 (MVP High) | P2 (MVP Medium)
- **Week:** Which week this story ships
- **DoR Checklist:** Everything that must be true before coding starts
- **Acceptance Criteria:** 5+ testable requirements
- **DoD Checklist:** Everything that must be true before code merges
- **Estimation:** Size + Complexity + Risk
- **Dependencies:** Blocks / Blocked by / Related
- **Design References:** Where in UX spec / which components

---

## WEEK 1: Design Tokens & Base Components

### US-001: Implement Design Token Constants (Slint)

**Story Type:** Infrastructure / Component  
**Priority:** P0 (MVP Critical)  
**Week:** 1  
**Owner:** Amelia (Developer)  
**Designer:** Sally  
**Reviewer:** Winston

**As a** developer  
**I want** centralized design token constants in Slint (colors, typography, spacing, motion)  
**So that** all components use consistent values and design changes propagate automatically

---

#### Definition of Ready ✓

- [x] Design token values finalized by Sally
  - Colors: Fluent Blue, teal, grays, semantic (success/warning/error)
  - Typography: all sizes, weights, line heights
  - Spacing: 8px grid from 4px to 24px
  - Motion: durations (200/300/400/800ms), easing (ease-out/in-out/linear)

- [x] File location decided: `/src/frontend/design/tokens.slint`

- [x] Reference: UX Spec Section 7 (Visual System)

- [x] No blockers: this is first story (critical path)

- [x] Owner assigned: Amelia

#### Acceptance Criteria

- [ ] **AC1:** All color tokens defined and compile
  - Primary colors: Fluent Blue (#0078D4), Teal (#00A4EF)
  - Neutral colors: Dark (#333333), Medium (#666666), Light (#F5F5F5)
  - Semantic: Success (#107C10), Warning (#FFB900), Error (#E81123)
  - Test: All colors render correctly in simple component

- [ ] **AC2:** Typography tokens defined with all variants
  - Sizes: Display (48px), Headline (28px), Subheading (18px), Body (14px), Caption (12px)
  - Weights: 400 (regular), 500 (medium), 600 (semibold), 700 (bold)
  - Line heights: 1.2, 1.4, 1.6
  - Test: Typography applies correctly to text elements

- [ ] **AC3:** Spacing tokens defined on 8px grid
  - xs (4px), sm (8px), md (12px), lg (16px), xl (20px), xxl (24px)
  - Test: Spacing values are consistent and scale correctly

- [ ] **AC4:** Motion tokens defined
  - Durations: 200ms (quick), 300ms (standard), 400ms (slow), 800ms (very slow)
  - Easing: ease-out, ease-in-out, linear
  - Test: Motion values compile and can be used in animations

- [ ] **AC5:** Motion preference constants defined
  - `PREFERS_REDUCED_MOTION` boolean flag
  - Alternate token values for when reduce_motion=true
  - Test: Conditional tokens work correctly

- [ ] **AC6:** Token file compiles without errors
  - `cargo build --features slint-ui` passes
  - `cargo clippy` has zero warnings
  - Test output shows successful compilation

- [ ] **AC7:** Documentation complete
  - Design tokens reference guide: `/docs/DESIGN_TOKENS_REFERENCE.md`
  - Lists every token with its value and usage example
  - Test: Can find any token in reference

#### Definition of Done ✓

- [ ] **AC Verification:** All 7 AC pass ✓
  - Unit tests verify each token value
  - Compilation successful
  - Documentation complete

- [ ] **Unit Tests:** 100% passing
  - Test: All color values match specification
  - Test: All typography values apply correctly
  - Test: All spacing values calculate correctly
  - Test: Motion tokens work in animations
  - Test: Conditional tokens toggle correctly
  - Command: `cargo test --all` → 100% pass

- [ ] **Integration Tests:** Token usage in components
  - Test: Button component uses color tokens (not hardcoded)
  - Test: Text components use typography tokens (not hardcoded)
  - Test: Spacing is consistent across components using tokens

- [ ] **Accessibility Verified:**
  - Colors meet 4.5:1 contrast ratio (WCAG AA)
  - Motion tokens respect reduce_motion preference
  - No motion-based semantics (color + motion for meaning)

- [ ] **Code Review:** Winston approves
  - Code follows Rust + Slint conventions
  - Token organization is logical and maintainable
  - No dead code or unused tokens

- [ ] **Design Compliance:** Sally approves
  - All values match UX spec Section 7
  - Token names match design system naming
  - Colors rendered accurately

- [ ] **Documentation:** Complete in `/docs/`
  - Design tokens reference guide written
  - Example usage provided for each token type
  - Links provided to UX spec

- [ ] **Performance:** < 16ms render time ✓

- [ ] **Zero Warnings:** `cargo clippy` returns clean ✓

- [ ] **Merged to main:** PR #[N] merged ✓

#### Estimation

**Size:** M (3-5 days)  
**Complexity:** Low (straightforward token definition)  
**Risk:** Low (no external dependencies)  
**Time Estimate:** 20-25 hours (4h definition + 8h implementation + 6h testing + 3h documentation + 2h review)

#### Dependencies

- **Blocks:** US-002, US-003, US-004, US-005, US-006 (all Week 1 components use tokens)
- **Blocked by:** None (critical path)
- **Related:** UX Spec Section 7

#### Design References

- UX Spec Section 7.2 (Visual System - Colors)
- UX Spec Section 7.3 (Visual System - Typography)
- UX Spec Section 7.4 (Visual System - Spacing)
- UX Spec Section 7.5 (Visual System - Motion)

---

### US-002: Implement Button Component

**Story Type:** Component  
**Priority:** P0 (MVP Critical)  
**Week:** 1  
**Owner:** Amelia  
**Designer:** Sally  
**Reviewer:** Winston  

**As a** designer and developer  
**I want** a reusable Button component with 4 variants (primary, secondary, tertiary, danger) and 3 sizes  
**So that** all clickable elements in the app are consistent and accessible

---

#### Definition of Ready ✓

- [x] Component definition complete: WEEK1_COMPONENT_DEFINITIONS.md (Button section)
- [x] US-001 (Design Tokens) merged to main ✓
- [x] All AC verified by Sally
- [x] Test plan attached with unit + integration + accessibility tests

#### Acceptance Criteria

- [ ] **AC1:** Button renders with correct label and variants
  - Primary (Fluent Blue bg, white text)
  - Secondary (outline, blue text)
  - Tertiary (transparent, blue text)
  - Danger (red bg, white text)
  - Test: Visual comparison to UX spec

- [ ] **AC2:** All sizes render correctly
  - Small (28px height)
  - Medium (36px height)
  - Large (44px height)
  - Test: Measure dimensions, compare to spec

- [ ] **AC3:** on_clicked callback fires when clicked
  - Click button → callback fires
  - Test: Unit test verifies callback invoked

- [ ] **AC4:** Keyboard accessible
  - Tab to focus
  - Enter to activate
  - Space to activate
  - Test: Keyboard navigation + activation works

- [ ] **AC5:** Respects reduce_motion preference
  - When loading + reduce_motion=true: static spinner (no rotation)
  - When loading + reduce_motion=false: rotating spinner (400ms)
  - Test: Screenshot comparison reduce_motion true vs false

- [ ] **AC6:** Disabled state works correctly
  - is_disabled=true → button grayed out, clicks ignored
  - is_disabled=false → button interactive
  - Test: Clicks ignored when disabled

- [ ] **AC7:** Loading state works correctly
  - is_loading=true → spinner shows, label hidden
  - is_loading=false → label shows, spinner hidden
  - Test: State toggle shows/hides correctly

- [ ] **AC8:** Screen reader accessible
  - NVDA announces "Button: [label]"
  - Test: NVDA test successful

#### Definition of Done ✓

- [ ] All 8 AC pass ✓
- [ ] Unit tests 100% passing (8+ tests)
  - test_button_renders_primary_variant
  - test_button_renders_all_sizes
  - test_on_clicked_fires
  - test_keyboard_enter_activates
  - test_keyboard_space_activates
  - test_is_disabled_prevents_clicks
  - test_reduce_motion_disables_animation
  - test_loading_state_shows_spinner
- [ ] Integration tests with MessageInput parent ✓
- [ ] Accessibility tests passing (NVDA, keyboard, contrast) ✓
- [ ] Code review approved by Winston ✓
- [ ] Design compliance approved by Sally ✓
- [ ] Documentation complete ✓
- [ ] Performance: < 16ms ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** M (3-5 days)  
**Complexity:** Medium (variants, accessibility)  
**Risk:** Low (pattern established)  
**Time Estimate:** 25-30 hours

#### Dependencies

- **Blocks:** US-010, US-011, US-012, US-014, US-015 (all need Button)
- **Blocked by:** US-001 (Design Tokens)
- **Related:** COMPONENT_API_STANDARD.md

---

### US-003: Implement TextField Component

**Story Type:** Component  
**Priority:** P0 (MVP Critical)  
**Week:** 1  
**Owner:** Amelia  
**Designer:** Sally  
**Reviewer:** Winston  

**As a** user  
**I want** a text input field for composing messages and searching  
**So that** I can provide text input with proper validation and accessibility

---

#### Definition of Ready ✓

- [x] Component definition complete: WEEK1_COMPONENT_DEFINITIONS.md
- [x] US-001, US-002 complete and merged
- [x] All AC verified by Sally

#### Acceptance Criteria

- [ ] **AC1:** TextField renders with placeholder text when empty
- [ ] **AC2:** on_text_changed callback fires on every keystroke
- [ ] **AC3:** on_return_pressed callback fires when Enter pressed
- [ ] **AC4:** Error state displays red border + error message
- [ ] **AC5:** Keyboard accessible (Tab, all characters, Backspace, Arrow keys)
- [ ] **AC6:** has_error prop controls error appearance
- [ ] **AC7:** is_disabled prop prevents interaction

#### Definition of Done ✓

- [ ] All 7 AC pass ✓
- [ ] Unit tests 100% passing (7+ tests)
- [ ] Integration with MessageInput ✓
- [ ] Accessibility verified (NVDA, keyboard) ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Documentation complete ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** M (3-5 days)  
**Complexity:** Medium (text input events)  
**Risk:** Medium (first text input, pattern unfamiliar)  

#### Dependencies

- **Blocks:** US-010 (MessageInput uses TextField)
- **Blocked by:** US-001, US-002
- **Related:** COMPONENT_API_STANDARD.md

---

### US-004: Implement Icon Component

**Story Type:** Component  
**Priority:** P0 (MVP Critical)  
**Week:** 1  
**Owner:** Amelia  
**Designer:** Sally  
**Reviewer:** Winston  

**As a** designer  
**I want** a consistent Icon component for displaying SVG icons throughout the app  
**So that** all icons are sized, colored, and animated consistently

---

#### Definition of Ready ✓

- [x] Component definition complete
- [x] Icon library finalized (checkmark, spinner, close, send, user-profile, etc.)
- [x] All icons exist in `/assets/icons/`

#### Acceptance Criteria

- [ ] **AC1:** Icon renders all defined icons (checkmark, spinner, etc.)
- [ ] **AC2:** Size variants work (small 16px, medium 24px, large 32px, xlarge 48px)
- [ ] **AC3:** Color variants work (currentColor, blue, green, red, gray)
- [ ] **AC4:** Spinner icon rotates 400ms (or static if reduce_motion=true)
- [ ] **AC5:** alt_text accessible label working
- [ ] **AC6:** No accessibility violations

#### Definition of Done ✓

- [ ] All 6 AC pass ✓
- [ ] Unit tests 100% passing
- [ ] Integration with Button (spinner) ✓
- [ ] Accessibility verified ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** S (1-3 days)  
**Complexity:** Low  
**Risk:** Low  

#### Dependencies

- **Blocks:** US-005, US-006, US-010, US-011, US-012, US-014, US-015
- **Blocked by:** US-001

---

### US-005: Implement Chip Component

**Story Type:** Component  
**Priority:** P1 (MVP High)  
**Week:** 1  
**Owner:** Amelia  
**Designer:** Sally  
**Reviewer:** Winston  

**As a** designer  
**I want** a Chip component for displaying tags, labels, and status badges  
**So that** metadata is presented compactly and consistently

---

#### Definition of Ready ✓

- [x] Component definition complete
- [x] Variants finalized (default, primary, success, warning, error)

#### Acceptance Criteria

- [ ] **AC1:** Chip renders label correctly for all variants
- [ ] **AC2:** on_clicked callback works when chip is clickable
- [ ] **AC3:** on_dismissed callback works when is_dismissible=true
- [ ] **AC4:** X button appears only when is_dismissible=true
- [ ] **AC5:** is_disabled prop prevents interaction

#### Definition of Done ✓

- [ ] All 5 AC pass ✓
- [ ] Unit tests 100% passing
- [ ] Accessibility verified ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** S (1-3 days)  
**Complexity:** Low  
**Risk:** Low  

#### Dependencies

- **Blocks:** US-008, US-011, US-013
- **Blocked by:** US-001, US-004

---

### US-006: Implement LoadingSpinner Component

**Story Type:** Component  
**Priority:** P0 (MVP Critical)  
**Week:** 1  
**Owner:** Amelia  
**Designer:** Sally  
**Reviewer:** Winston  

**As a** user  
**I want** a loading spinner to indicate asynchronous operations are in progress  
**So that** I know the app is working and haven't encountered a hung state

---

#### Definition of Ready ✓

- [x] Component definition complete
- [x] All motion preferences documented

#### Acceptance Criteria

- [ ] **AC1:** Spinner rotates 400ms continuously (linear easing)
- [ ] **AC2:** All sizes render correctly (small/medium/large)
- [ ] **AC3:** Color variants work (currentColor, blue, gray)
- [ ] **AC4:** reduce_motion=true shows static icon (no rotation)
- [ ] **AC5:** Optional message displays below spinner
- [ ] **AC6:** aria-live="polite" for screen reader announcement

#### Definition of Done ✓

- [ ] All 6 AC pass ✓
- [ ] Unit tests 100% passing
- [ ] Accessibility verified ✓
- [ ] Performance: smooth 60 FPS rotation ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** S (1-3 days)  
**Complexity:** Low  
**Risk:** Low  

#### Dependencies

- **Blocks:** US-010, US-014, US-015
- **Blocked by:** US-001, US-004

---

## WEEK 2: Conversation & Message Components

### US-007: Implement MessageBubble Component

**Story Type:** Component  
**Priority:** P0 (MVP Critical)  
**Week:** 2  
**Owner:** Amelia  
**Designer:** Sally  
**Reviewer:** Winston  

**As a** user  
**I want** individual messages displayed in a conversation with delivery status, sender info, and timestamps  
**So that** I can see who sent what and when messages were delivered

---

#### Definition of Ready ✓

- [x] Week 1 components (Button, TextField, Icon, Chip, LoadingSpinner) merged ✓
- [x] Component definition complete: COMPONENT_COMPOSITION_RULES.md (MessageBubble pattern)
- [x] UX Spec Section 8.2 (MessageBubble component) reviewed

#### Acceptance Criteria

- [ ] **AC1:** Message content displays with sender name and timestamp
- [ ] **AC2:** Delivery status icon shown (pending spinner, sent checkmark, delivered double-checkmark, failed X)
- [ ] **AC3:** on_clicked callback fires when message tapped
- [ ] **AC4:** on_long_pressed callback fires on long press (2+ sec)
- [ ] **AC5:** Message actions menu appears on hover (Reply, Delete buttons)
- [ ] **AC6:** All delivery statuses render correctly
- [ ] **AC7:** Keyboard accessible (Tab, Enter to select, keyboard menu nav)

#### Definition of Done ✓

- [ ] All 7 AC pass ✓
- [ ] Unit tests 100% passing (7+ tests)
- [ ] Integration with MessageList parent ✓
- [ ] Accessibility verified (NVDA, keyboard) ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Documentation complete ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** M (3-5 days)  
**Complexity:** Medium  
**Risk:** Medium  

#### Dependencies

- **Blocks:** US-008, US-014, US-015
- **Blocked by:** US-001 through US-006 (Week 1 components)

---

### US-008: Implement ConversationItem Component

**Story Type:** Component  
**Priority:** P0 (MVP Critical)  
**Week:** 2  
**Owner:** Amelia  
**Designer:** Sally  
**Reviewer:** Winston  

**As a** user  
**I want** conversations listed compactly with last message preview, unread count, and presence indicator  
**So that** I can quickly find and switch between conversations

---

#### Definition of Ready ✓

- [x] Component definition complete
- [x] UX Spec Section 8.3 reviewed

#### Acceptance Criteria

- [ ] **AC1:** Conversation title displays with last message preview
- [ ] **AC2:** Unread count shown as badge
- [ ] **AC3:** User avatar displays with online/offline indicator
- [ ] **AC4:** Last message timestamp shown
- [ ] **AC5:** on_selected callback fires when tapped
- [ ] **AC6:** Hover state highlights conversation
- [ ] **AC7:** Keyboard accessible (Tab, Enter to select)

#### Definition of Done ✓

- [ ] All 7 AC pass ✓
- [ ] Unit tests 100% passing
- [ ] Integration with ConversationList parent ✓
- [ ] Accessibility verified ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** M (3-5 days)  
**Complexity:** Medium  
**Risk:** Low  

#### Dependencies

- **Blocks:** US-011, US-013
- **Blocked by:** US-001-006, US-007

---

### US-009: Implement Presence Indicator Component

**Story Type:** Component  
**Priority:** P1 (MVP High)  
**Week:** 2  
**Owner:** Barry (Quick Flow Dev)  
**Designer:** Sally  
**Reviewer:** Winston  

**As a** user  
**I want** to see if contacts are online, away, or offline  
**So that** I know if they're available to chat

---

#### Definition of Ready ✓

- [x] Component definition complete
- [x] Presence states defined (online/away/offline)

#### Acceptance Criteria

- [ ] **AC1:** Green dot for online, yellow for away, gray for offline
- [ ] **AC2:** Smooth pulse animation (if reduce_motion=false)
- [ ] **AC3:** Static icon (if reduce_motion=true)
- [ ] **AC4:** Tooltip shows "Online", "Away", or "Offline"
- [ ] **AC5:** Keyboard accessible

#### Definition of Done ✓

- [ ] All 5 AC pass ✓
- [ ] Unit tests 100% passing
- [ ] Accessibility verified ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** S (1-3 days)  
**Complexity:** Low  
**Risk:** Low  

#### Dependencies

- **Blocks:** US-012, US-013
- **Blocked by:** US-001-006

---

### US-010: Implement MessageInput Container

**Story Type:** Component + Integration  
**Priority:** P0 (MVP Critical)  
**Week:** 2  
**Owner:** Amelia  
**Designer:** Sally  
**Reviewer:** Winston  

**As a** user  
**I want** to compose and send messages with real-time feedback  
**So that** I can communicate with others in the app

---

#### Definition of Ready ✓

- [x] Week 1-2 components ready
- [x] UX Spec Section 8.4 (Message Input) reviewed
- [x] Architecture workshop completed (WebSocket + state management)
- [x] ARCHITECTURE_REALTIME_SYNC.md and WEBSOCKET_PROTOCOL.md ready

#### Acceptance Criteria

- [ ] **AC1:** TextField for message composition with placeholder
- [ ] **AC2:** Send button visible and clickable
- [ ] **AC3:** on_send callback fires with message text
- [ ] **AC4:** Message text clears after sending
- [ ] **AC5:** Button shows loading spinner while sending (up to 2 sec)
- [ ] **AC6:** Error displayed if send fails (server error)
- [ ] **AC7:** Keyboard shortcut: Ctrl+Enter sends message

#### Definition of Done ✓

- [ ] All 7 AC pass ✓
- [ ] Unit tests 100% passing (7+ tests)
- [ ] Integration with ConversationView ✓
- [ ] Accessibility verified ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Performance: message send < 2 seconds ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** M (3-5 days)  
**Complexity:** Medium (integration with messaging logic)  
**Risk:** Medium (state management coordination)  

#### Dependencies

- **Blocks:** US-014, US-015
- **Blocked by:** US-001-006, US-007

---

## WEEK 3: Real-Time & Message List

### US-011: Implement ConversationHeader Container

**Story Type:** Component  
**Priority:** P1 (MVP High)  
**Week:** 3  
**Owner:** Barry  
**Designer:** Sally  
**Reviewer:** Winston  

**As a** user  
**I want** to see conversation title, participant info, and action buttons  
**So that** I can navigate and manage conversations effectively

---

#### Definition of Ready ✓

- [x] Week 1-2 components ready
- [x] UX Spec Section 8.1 reviewed

#### Acceptance Criteria

- [ ] **AC1:** Conversation title displayed prominently
- [ ] **AC2:** Participant count shown
- [ ] **AC3:** Back button to exit conversation (mobile)
- [ ] **AC4:** Settings/more menu accessible
- [ ] **AC5:** Presence indicator for contact
- [ ] **AC6:** Last seen timestamp shown (if away)

#### Definition of Done ✓

- [ ] All 6 AC pass ✓
- [ ] Unit tests 100% passing
- [ ] Integration with ConversationView ✓
- [ ] Accessibility verified ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** M (3-5 days)  
**Complexity:** Medium  
**Risk:** Low  

#### Dependencies

- **Blocks:** US-014
- **Blocked by:** US-001-006, US-008, US-009

---

### US-012: Implement Presence Sync (Real-Time Updates)

**Story Type:** Integration / Real-Time  
**Priority:** P0 (MVP Critical)  
**Week:** 3  
**Owner:** Amelia  
**Designer:** Sally  
**Reviewer:** Winston + Murat  

**As a** user  
**I want** to see when contacts come online or go offline in real-time  
**So that** I know immediately who's available without refreshing

---

#### Definition of Ready ✓

- [x] ARCHITECTURE_REALTIME_SYNC.md completed
- [x] WEBSOCKET_PROTOCOL.md completed
- [x] Architecture workshop completed (state management for presence)

#### Acceptance Criteria

- [ ] **AC1:** WebSocket receives presence updates from server
- [ ] **AC2:** App state updates immediately (< 100ms)
- [ ] **AC3:** Presence indicator updates in ConversationList
- [ ] **AC4:** Presence indicator updates in ConversationHeader
- [ ] **AC5:** No duplicate updates (deduplication working)
- [ ] **AC6:** Handles network drop and reconnect gracefully

#### Definition of Done ✓

- [ ] All 6 AC pass ✓
- [ ] Unit tests 100% passing (presence sync logic)
- [ ] Integration tests with WebSocket mock ✓
- [ ] Performance: < 100ms presence update ✓
- [ ] Code review approved ✓
- [ ] Performance verified ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** L (5-8 days)  
**Complexity:** High (real-time logic, edge cases)  
**Risk:** High (first real-time feature, network edge cases)  

#### Dependencies

- **Blocks:** US-013, US-014, US-015
- **Blocked by:** US-001-006, US-008, US-009, US-010, US-011

---

### US-013: Implement Delivery Status Sync (Real-Time)

**Story Type:** Integration / Real-Time  
**Priority:** P0 (MVP Critical)  
**Week:** 3  
**Owner:** Amelia  
**Designer:** Sally  
**Reviewer:** Winston + Murat  

**As a** user  
**I want** to see when my messages are delivered and read by recipients  
**So that** I know my messages reached them

---

#### Definition of Ready ✓

- [x] ARCHITECTURE_REALTIME_SYNC.md completed
- [x] WEBSOCKET_PROTOCOL.md completed

#### Acceptance Criteria

- [ ] **AC1:** Message status changes from PENDING → SENT (< 500ms)
- [ ] **AC2:** Message status changes to DELIVERED when server confirms
- [ ] **AC3:** Delivery checkmark appears in MessageBubble
- [ ] **AC4:** Double-checkmark appears for DELIVERED
- [ ] **AC5:** Failed messages show error icon + retry button
- [ ] **AC6:** Handles duplicate receipts (no duplicate icons)

#### Definition of Done ✓

- [ ] All 6 AC pass ✓
- [ ] Unit tests 100% passing
- [ ] Integration with MessageList + MessageBubble ✓
- [ ] Performance: delivery confirmation < 500ms ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** L (5-8 days)  
**Complexity:** High (real-time sync, conflict handling)  
**Risk:** High (message ordering, deduplication)  

#### Dependencies

- **Blocks:** US-014, US-015
- **Blocked by:** US-001-006, US-007, US-010, US-012

---

## WEEK 4: Message List & Integration

### US-014: Implement MessageList Container (Display & Scroll)

**Story Type:** Component + Integration  
**Priority:** P0 (MVP Critical)  
**Week:** 4  
**Owner:** Amelia  
**Designer:** Sally  
**Reviewer:** Winston + Murat  

**As a** user  
**I want** to see a scrollable list of messages in a conversation  
**So that** I can read message history and follow the conversation

---

#### Definition of Ready ✓

- [x] MessageBubble (US-007) merged
- [x] Delivery status sync (US-013) working
- [x] Architecture for message ordering documented

#### Acceptance Criteria

- [ ] **AC1:** Messages display in chronological order
- [ ] **AC2:** Most recent message at bottom (chat style)
- [ ] **AC3:** Smooth scrolling to latest message on new arrival
- [ ] **AC4:** "Load more" button appears when history available
- [ ] **AC5:** Scroll-to-bottom button appears when user scrolls up
- [ ] **AC6:** Loading spinner shown while fetching messages
- [ ] **AC7:** Empty state shows when no messages yet

#### Definition of Done ✓

- [ ] All 7 AC pass ✓
- [ ] Unit tests 100% passing (message ordering, scroll logic)
- [ ] Integration with ConversationView ✓
- [ ] Performance: renders 100 messages smoothly (60 FPS) ✓
- [ ] Accessibility verified (keyboard scroll) ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** L (5-8 days)  
**Complexity:** High (scroll logic, performance optimization)  
**Risk:** High (performance with many messages)  

#### Dependencies

- **Blocks:** US-015
- **Blocked by:** US-001-006, US-007, US-010, US-012, US-013

---

### US-015: Implement Real-Time Message Arrival (WebSocket)

**Story Type:** Integration / Real-Time  
**Priority:** P0 (MVP Critical)  
**Week:** 4  
**Owner:** Amelia  
**Designer:** Sally  
**Reviewer:** Winston + Murat  

**As a** user  
**I want** to receive and display messages from other users in real-time  
**So that** conversations feel live and responsive

---

#### Definition of Ready ✓

- [x] ARCHITECTURE_REALTIME_SYNC.md completed
- [x] WEBSOCKET_PROTOCOL.md completed
- [x] MessageList (US-014) working
- [x] Architecture workshop covered message arrival scenarios

#### Acceptance Criteria

- [ ] **AC1:** New message from server appears in list immediately (< 200ms)
- [ ] **AC2:** Message inserts in correct chronological position
- [ ] **AC3:** Scroll auto-scrolls to new message if at bottom
- [ ] **AC4:** No scroll jump if user is reading history (scroll up)
- [ ] **AC5:** Message arrival notification shown (if reduce_motion=false)
- [ ] **AC6:** Handles out-of-order arrivals correctly (vector clocks)
- [ ] **AC7:** Handles duplicate messages (no duplicates in list)

#### Definition of Done ✓

- [ ] All 7 AC pass ✓
- [ ] Unit tests 100% passing (message arrival, ordering, dedup)
- [ ] Integration tests with WebSocket mock ✓
- [ ] Performance: < 200ms arrival to render ✓
- [ ] Edge case tests: network drop, message reordering ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** L (5-8 days)  
**Complexity:** High (real-time coordination, edge cases)  
**Risk:** High (network edge cases, message ordering)  

#### Dependencies

- **Blocks:** US-016, US-017
- **Blocked by:** US-001-006, US-007, US-010, US-014, US-013

---

## WEEK 5: Search & Animations

### US-016: Implement Conversation List Container

**Story Type:** Component + Integration  
**Priority:** P1 (MVP High)  
**Week:** 5  
**Owner:** Barry  
**Designer:** Sally  
**Reviewer:** Winston  

**As a** user  
**I want** a scrollable list of all my conversations  
**So that** I can easily find and switch between chats

---

#### Definition of Ready ✓

- [x] ConversationItem (US-008) merged
- [x] Presence sync (US-012) working

#### Acceptance Criteria

- [ ] **AC1:** Conversations sorted by last message time (newest first)
- [ ] **AC2:** Unread conversations highlighted
- [ ] **AC3:** Smooth scrolling with many conversations
- [ ] **AC4:** Search conversations by name/content (if US-017 ready)
- [ ] **AC5:** Tap conversation to open (navigates to chat)
- [ ] **AC6:** Long-press to show menu (archive, mute, etc.)

#### Definition of Done ✓

- [ ] All 6 AC pass ✓
- [ ] Unit tests 100% passing
- [ ] Integration with main view ✓
- [ ] Performance: smooth scroll with 50+ conversations ✓
- [ ] Accessibility verified ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** M (3-5 days)  
**Complexity:** Medium  
**Risk:** Low  

#### Dependencies

- **Blocks:** None
- **Blocked by:** US-008, US-012

---

### US-017: Implement Search Conversations (Full-Text)

**Story Type:** Feature + Integration  
**Priority:** P2 (MVP Medium)  
**Week:** 5  
**Owner:** Barry  
**Designer:** Sally  
**Reviewer:** Winston  

**As a** user  
**I want** to search for conversations by name, last message, or participant  
**So that** I can quickly find specific chats

---

#### Definition of Ready ✓

- [x] Search UX patterns documented (UX Spec Section 6)
- [x] Backend search API available

#### Acceptance Criteria

- [ ] **AC1:** Search input field visible in conversation list
- [ ] **AC2:** Type to search conversations in real-time
- [ ] **AC3:** Results filtered by name, last message, participants
- [ ] **AC4:** Clear button to reset search
- [ ] **AC5:** No results state shown when empty
- [ ] **AC6:** Search is performant (< 500ms response)

#### Definition of Done ✓

- [ ] All 6 AC pass ✓
- [ ] Unit tests 100% passing
- [ ] Integration with ConversationList ✓
- [ ] Performance: < 500ms search ✓
- [ ] Accessibility verified (keyboard search) ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** M (3-5 days)  
**Complexity:** Medium  
**Risk:** Low  

#### Dependencies

- **Blocks:** None
- **Blocked by:** US-016

---

### US-018: Implement Message Search (Full-Text)

**Story Type:** Feature + Integration  
**Priority:** P2 (MVP Medium)  
**Week:** 5  
**Owner:** Barry  
**Designer:** Sally  
**Reviewer:** Winston  

**As a** user  
**I want** to search for specific messages within conversations  
**So that** I can find important information quickly

---

#### Definition of Ready ✓

- [x] Search UX patterns documented
- [x] Backend search API available

#### Acceptance Criteria

- [ ] **AC1:** Search input in message list header
- [ ] **AC2:** Search across messages in current conversation
- [ ] **AC3:** Results highlighted in message list
- [ ] **AC4:** Jump to result with scroll
- [ ] **AC5:** Next/prev navigation between results
- [ ] **AC6:** Performance: < 1 second search on 1000+ messages

#### Definition of Done ✓

- [ ] All 6 AC pass ✓
- [ ] Unit tests 100% passing
- [ ] Integration with MessageList ✓
- [ ] Performance: < 1sec search ✓
- [ ] Accessibility verified ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** M (3-5 days)  
**Complexity:** Medium  
**Risk:** Low  

#### Dependencies

- **Blocks:** None
- **Blocked by:** US-014, US-015

---

## WEEK 6: Animations & Accessibility Testing

### US-019: Implement Message Animations & Transitions

**Story Type:** Polish  
**Priority:** P1 (MVP High)  
**Week:** 6  
**Owner:** Barry  
**Designer:** Sally  
**Reviewer:** Winston + Murat  

**As a** user  
**I want** smooth message animations when messages arrive and scroll  
**So that** the app feels polished and responsive

---

#### Definition of Ready ✓

- [x] Animation patterns documented in UX Spec Section 10
- [x] Motion preference patterns documented in Section 11
- [x] All animation tokens defined (US-001)

#### Acceptance Criteria

- [ ] **AC1:** New message fade-in animation (200ms, ease-out)
- [ ] **AC2:** Scroll-to-bottom animation (300ms, ease-out)
- [ ] **AC3:** Message selection animation (100ms highlight)
- [ ] **AC4:** All animations respect reduce_motion (instant if true)
- [ ] **AC5:** Typing indicator animation (if applicable)
- [ ] **AC6:** Presence indicator pulse animation (1s, if reduce_motion=false)

#### Definition of Done ✓

- [ ] All 6 AC pass ✓
- [ ] Unit tests 100% passing (animation timing)
- [ ] Visual tests: animation smoothness ✓
- [ ] Performance: 60 FPS during animations ✓
- [ ] Accessibility: reduce_motion respected ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** M (3-5 days)  
**Complexity:** Medium (animation timing)  
**Risk:** Low  

#### Dependencies

- **Blocks:** None
- **Blocked by:** US-007-015

---

### US-020: Implement Conversation Switching Animation

**Story Type:** Polish  
**Priority:** P1 (MVP High)  
**Week:** 6  
**Owner:** Barry  
**Designer:** Sally  
**Reviewer:** Winston + Murat  

**As a** user  
**I want** smooth transitions when switching between conversations  
**So that** navigation feels fluid

---

#### Definition of Ready ✓

- [x] Transition patterns documented (UX Spec Section 10)
- [x] Performance target: < 100ms switch (from UX Spec)

#### Acceptance Criteria

- [ ] **AC1:** Old message list fades out (200ms)
- [ ] **AC2:** New message list fades in (200ms)
- [ ] **AC3:** Total transition time < 100ms latency + animation
- [ ] **AC4:** Respects reduce_motion (instant switch if true)
- [ ] **AC5:** Scroll position resets to bottom of new conversation
- [ ] **AC6:** Header updates (title, participants) during transition

#### Definition of Done ✓

- [ ] All 6 AC pass ✓
- [ ] Unit tests 100% passing
- [ ] Performance: < 100ms latency + animation ✓
- [ ] Accessibility: reduce_motion respected ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** M (3-5 days)  
**Complexity:** Medium  
**Risk:** Low  

#### Dependencies

- **Blocks:** None
- **Blocked by:** US-014, US-015, US-016

---

### US-021: WCAG 2.1 AA Accessibility Testing & Fixes

**Story Type:** Quality Assurance  
**Priority:** P0 (MVP Critical)  
**Week:** 6  
**Owner:** Murat (Test Architect)  
**Designer:** Sally  
**Reviewer:** Winston  

**As a** user with accessibility needs  
**I want** the app to be fully accessible per WCAG 2.1 AA standards  
**So that** I can use the app regardless of disability

---

#### Definition of Ready ✓

- [x] WCAG 2.1 AA standards documented
- [x] All components built with a11y in mind
- [x] Accessibility testing tools installed (NVDA, contrast checker)
- [x] Manual test plan created

#### Acceptance Criteria

- [ ] **AC1:** NVDA screen reader reads all content correctly
- [ ] **AC2:** Keyboard navigation works for all interactive elements
- [ ] **AC3:** Focus ring visible on all focusable elements
- [ ] **AC4:** All text has 4.5:1 contrast ratio (WCAG AA)
- [ ] **AC5:** No keyboard traps
- [ ] **AC6:** Forms have accessible labels
- [ ] **AC7:** Error messages announced to screen readers
- [ ] **AC8:** No accessibility issues found in automated scan

#### Definition of Done ✓

- [ ] All 8 AC pass ✓
- [ ] Manual NVDA testing on all screens
- [ ] Manual keyboard testing (Tab, Enter, Escape, Arrows)
- [ ] Automated accessibility scan (0 errors)
- [ ] Color contrast verified (all elements)
- [ ] Focus management correct
- [ ] Documentation: accessibility checklist complete ✓
- [ ] Zero warnings ✓

#### Estimation

**Size:** L (5-8 days)  
**Complexity:** High (manual testing, cross-browser)  
**Risk:** Medium (findings may require component changes)  

#### Dependencies

- **Blocks:** US-022
- **Blocked by:** US-007-020

---

### US-022: Implement Motion Preferences (prefers-reduced-motion)

**Story Type:** Accessibility  
**Priority:** P0 (MVP Critical)  
**Week:** 6  
**Owner:** Amelia + Murat  
**Designer:** Sally  
**Reviewer:** Winston  

**As a** user with vestibular disorders or motion sensitivity  
**I want** the app to respect my motion preferences  
**So that** animations don't cause discomfort or disorientation

---

#### Definition of Ready ✓

- [x] Motion preference patterns documented (UX Spec Section 11)
- [x] 10 animation reductions documented
- [x] Slint implementation pattern provided
- [x] Testing strategy defined

#### Acceptance Criteria

- [ ] **AC1:** reduce_motion preference detected from system
- [ ] **AC2:** All animations disabled when reduce_motion=true (10 animations identified)
  - Spinner rotation
  - Message fade-in
  - Scroll animations
  - Presence pulse
  - Loading indicator
  - Others documented in UX Spec Section 11
- [ ] **AC3:** Static icons replace animated versions
- [ ] **AC4:** Transitions remain instant
- [ ] **AC5:** Motion preferences persist across sessions
- [ ] **AC6:** User can override in settings (if applicable)
- [ ] **AC7:** No animations trigger automatically on page load
- [ ] **AC8:** WCAG 2.3.3 & 2.3.4 compliance verified

#### Definition of Done ✓

- [ ] All 8 AC pass ✓
- [ ] Unit tests 100% passing (reduce_motion toggle)
- [ ] Integration tests: animations disable correctly ✓
- [ ] Manual testing with reduce_motion enabled ✓
- [ ] Visual comparison screenshots (with/without reduce_motion) ✓
- [ ] Accessibility verified (WCAG 2.3.3 + 2.3.4) ✓
- [ ] Code review approved ✓
- [ ] Design compliance verified ✓
- [ ] Documentation complete ✓
- [ ] Zero warnings ✓
- [ ] Merged to main ✓

#### Estimation

**Size:** L (5-8 days)  
**Complexity:** High (comprehensive animation audit)  
**Risk:** Medium (may require animation refactors across codebase)  

#### Dependencies

- **Blocks:** None (final polish)
- **Blocked by:** US-019, US-020, US-021

---

## SPRINT SUMMARY

### Velocity & Capacity

- **Total Stories:** 22 stories
- **Total Points:** ~150 points (estimated)
- **Sprint Duration:** 6 weeks
- **Expected Velocity:** 25 points / week
- **Buffer:** ~15% (for unknowns)

### Critical Path

**Longest dependency chain:**
```
US-001 (Design Tokens)
  → US-002, US-003, US-004, US-005, US-006 (Week 1 components)
  → US-007, US-008, US-009, US-010, US-011 (Week 2-3 containers)
  → US-012, US-013 (Week 3 real-time)
  → US-014, US-015 (Week 4 lists + arrival)
  → US-019, US-020 (Week 6 animations)
  → US-021, US-022 (Week 6 accessibility)
```

**Critical path length:** 22 stories  
**Estimated duration:** 6 weeks  

### High-Risk Stories

- **US-012:** Presence sync (real-time, network edge cases)
- **US-013:** Delivery status sync (real-time, message ordering)
- **US-014:** Message list performance (smooth scroll, 60 FPS)
- **US-015:** Real-time message arrival (ordering, deduplication)
- **US-021:** Accessibility testing (may find surprises)
- **US-022:** Motion preferences (comprehensive audit)

### Success Criteria

All 22 stories complete AND:
- ✅ All tests passing (unit + integration + accessibility)
- ✅ 0 bugs blocking MVP
- ✅ Performance targets met (< 2s send, < 100ms switch, 60 FPS)
- ✅ WCAG 2.1 AA accessibility verified
- ✅ Motion preferences respected
- ✅ Ready for QA + user testing

---

**Approved By:** Bob (Scrum Master), Amelia (Developer), Winston (Architect), Sally (UX Designer)  
**Status:** Ready for Week 1 Sprint  
**Next Review:** Dec 23, 2025 (end of Week 1)
