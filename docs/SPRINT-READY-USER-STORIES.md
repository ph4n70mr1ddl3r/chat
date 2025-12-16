# Sprint-Ready User Stories

**Derived from:** UX Design Specification (2,437 lines)  
**Created by:** Bob (Scrum Master) + Team  
**Date:** December 16, 2025  
**Total Stories:** 18 user stories  
**Sprint Distribution:** 6-week MVP (3 stories per week = 18 total)

---

## Overview: Story Structure

Each story follows this format:

```
## US-[#]: [Title]

**Story Type:** Feature / Component / Integration / Bug Fix

**Priority:** P0 (MVP Critical) | P1 (MVP High) | P2 (MVP Medium)

**Week:** 1-6

**Persona:** Sarah / James / Elena / Marcus / David

**As a** [user type]  
**I want** [capability]  
**So that** [business value]

### Acceptance Criteria
- [ ] AC1: [Testable requirement]
- [ ] AC2: [Testable requirement]
- [ ] AC3: [Testable requirement]

### Definition of Done
- [ ] Code written and committed
- [ ] Unit tests pass (100% coverage for new code)
- [ ] Code review approved
- [ ] Acceptance criteria verified
- [ ] No regressions in existing tests

### Estimation
**Size:** S (1-3 days) | M (3-5 days) | L (5-8 days)  
**Complexity:** Low / Medium / High  
**Risk:** Low / Medium / High

### Dependencies
- Blocks: [Other stories]
- Blocked by: [Other stories]
- Related: [Related stories]

### Design References
- UX Spec Section: [#]
- Design Tokens: [List]
- Components: [List]

---
```

---

## WEEK 1: Design System & Base Components

### US-001: Implement Design Token Constants (Slint)

**Story Type:** Component / Infrastructure

**Priority:** P0 (MVP Critical)

**Week:** 1

**Persona:** Developer (Technical Foundation)

**As a** developer  
**I want** centralized design token constants (colors, typography, spacing, motion)  
**So that** all components use consistent values and changes propagate automatically

### Acceptance Criteria
- [ ] AC1: Color tokens defined and compile without errors
  - [ ] Primary: Fluent Blue (#0078D4)
  - [ ] Secondary: Teal (#00A4EF)
  - [ ] Neutral colors (dark, medium, light gray)
  - [ ] Semantic colors (success #107C10, warning #FFB900, error #E81123)
- [ ] AC2: Typography tokens defined with all sizes, weights, line heights
  - [ ] Display (48px), Headline (28px), Subheading (18px), Body (14px), Caption (12px)
  - [ ] Weight variants (400, 500, 600, 700)
  - [ ] Line height ratios (1.2, 1.4, 1.6)
- [ ] AC3: Spacing tokens defined (xs: 4px through xxl: 24px on 8px grid)
- [ ] AC4: Motion tokens defined
  - [ ] Duration: 200ms, 300ms, 400ms, 800ms
  - [ ] Easing: ease-out, ease-in-out, linear
- [ ] AC5: Motion preference constants defined
  - [ ] `PREFERS_REDUCED_MOTION` detection mechanism
  - [ ] Conditional token values for reduced-motion scenarios
- [ ] AC6: All tokens compile without errors in Slint
- [ ] AC7: Token file is well-commented and maintainable

### Definition of Done
- [ ] Code written and committed to `/src/frontend/design-tokens.slint`
- [ ] Unit tests verify token values match specification
- [ ] Code review approved
- [ ] All tokens visually verified in style guide
- [ ] No breaking changes to existing code

### Estimation
**Size:** M (3-5 days)  
**Complexity:** Low  
**Risk:** Low (straightforward data entry + validation)

### Dependencies
- Blocks: US-002 (Base Button), US-003 (Text Input), US-004 (Typography), US-005 (Layout Containers)
- Blocked by: None
- Related: None

### Design References
- UX Spec Section 7: Visual Design Foundation (lines 1246-1525)
- UX Spec Section 10: Loading States (animation tokens, lines 1926-2025)
- UX Spec Section 11: Motion Preferences (motion tokens, lines 2141-2235)

---

### US-002: Build Base Button Component (Primary, Secondary, Icon)

**Story Type:** Component

**Priority:** P0 (MVP Critical)

**Week:** 1

**Persona:** James (Power User - quick interactions)

**As a** power user  
**I want** consistent, responsive buttons throughout the app  
**So that** I can interact predictably with all actions

### Acceptance Criteria
- [ ] AC1: Primary button renders in all states (default, hover, pressed, disabled, loading)
- [ ] AC2: Secondary button variant available and renders correctly
- [ ] AC3: Icon-only button variant available (24x24px icons)
- [ ] AC4: Hover state transitions smoothly (200ms, ease-in-out)
- [ ] AC5: Pressed state provides clear visual feedback (shadow change, color shift)
- [ ] AC6: Disabled state is clearly distinguished (reduced opacity, no interaction)
- [ ] AC7: Loading state shows spinner inside button (24px spinner, centered)
- [ ] AC8: Focus indicator visible and WCAG AAA compliant (2px outline, #0078D4)
- [ ] AC9: All button text labels are accessible (no truncation at 320px width)
- [ ] AC10: Click event fires immediately (< 50ms latency)

### Definition of Done
- [ ] Code written: `/src/frontend/components/button.slint`
- [ ] Unit tests for all states (4 test cases)
- [ ] Visual regression tests for color/styling
- [ ] Code review approved
- [ ] Component renders correctly on 640px, 900px, 1200px+ breakpoints
- [ ] Accessibility: keyboard (Enter/Space), screen reader tested

### Estimation
**Size:** M (3-5 days)  
**Complexity:** Low  
**Risk:** Low

### Dependencies
- Blocks: US-006 (Message Composer), US-007 (Search Box)
- Blocked by: US-001 (Design Tokens)
- Related: US-003 (Text Input), US-004 (Typography)

### Design References
- UX Spec Section 7: Visual Design (colors, spacing, lines 1246-1485)
- UX Spec Section 11: Accessibility (focus indicators, line 2069)

---

### US-003: Build Text Input Component (with validation states)

**Story Type:** Component

**Priority:** P0 (MVP Critical)

**Week:** 1

**Persona:** Sarah (First-Time User - needs clarity)

**As a** first-time user  
**I want** clear, helpful text inputs that guide me through typing  
**So that** I understand what's expected and can correct errors easily

### Acceptance Criteria
- [ ] AC1: Text input renders in all states (empty, filled, focus, error, disabled)
- [ ] AC2: Placeholder text visible and clear (Body Small, 12px, #ADADAD)
- [ ] AC3: Focus state shows clear visual indicator (2px outline, Fluent Blue)
- [ ] AC4: Error state displays with error icon + error message below input
- [ ] AC5: Error message is accessible (associated with input via aria-describedby)
- [ ] AC6: Search input variant available (clear button on right, Ctrl+K hint)
- [ ] AC7: Character counter available (optional, for message/search inputs)
- [ ] AC8: Disabled state clearly distinguished (grayed out, no interaction)
- [ ] AC9: Keyboard navigation works (Tab to focus, Shift+Tab backwards)
- [ ] AC10: Cursor appears immediately (< 16ms latency for keyboard input)

### Definition of Done
- [ ] Code written: `/src/frontend/components/text-input.slint`
- [ ] Unit tests for all states (5 test cases)
- [ ] Keyboard interaction tests (Tab, Shift+Tab, Enter)
- [ ] Code review approved
- [ ] Screen reader tested (input label readable)
- [ ] No visual regressions across breakpoints

### Estimation
**Size:** M (3-5 days)  
**Complexity:** Low-Medium  
**Risk:** Low

### Dependencies
- Blocks: US-006 (Message Composer), US-007 (Search Box)
- Blocked by: US-001 (Design Tokens)
- Related: US-002 (Button)

### Design References
- UX Spec Section 7: Visual Design (spacing, colors, lines 1246-1485)
- UX Spec Section 11: Accessibility (keyboard nav, line 2117-2125)

---

### US-004: Implement Typography Scale (all text styles)

**Story Type:** Component

**Priority:** P0 (MVP Critical)

**Week:** 1

**Persona:** Elena (Team Lead - scans quickly)

**As a** team lead  
**I want** clear visual hierarchy in text  
**So that** I can quickly scan and find the information I need

### Acceptance Criteria
- [ ] AC1: Display text style renders (48px, 400-700 weights)
- [ ] AC2: Headline text style renders (28px, 600 weight)
- [ ] AC3: Subheading text style renders (18px, 600 weight)
- [ ] AC4: Body Large text style renders (14px, 400 weight)
- [ ] AC5: Body text style renders (13px, 400 weight)
- [ ] AC6: Body Small text style renders (12px, 400 weight)
- [ ] AC7: Caption text style renders (11px, 400 weight)
- [ ] AC8: Label text style renders (12px, 600 weight)
- [ ] AC9: All text meets WCAG AA contrast requirements (7:1 or 4.5:1)
- [ ] AC10: Line heights improve readability (1.2-1.6x ratios applied)
- [ ] AC11: Text wraps correctly at all breakpoints
- [ ] AC12: Segoe UI font loads properly (Windows native)

### Definition of Done
- [ ] Code written: `/src/frontend/components/typography.slint`
- [ ] Style guide created showing all text scales
- [ ] Visual regression tests for font rendering
- [ ] Code review approved
- [ ] Contrast verified (Axe tool or manual)
- [ ] Rendering verified on actual Windows devices

### Estimation
**Size:** S (1-3 days)  
**Complexity:** Low  
**Risk:** Low

### Dependencies
- Blocks: US-005 (Layout), US-008 (Conversation List Item)
- Blocked by: US-001 (Design Tokens)
- Related: None

### Design References
- UX Spec Section 7: Typography System (lines 1309-1374)
- UX Spec Section 11: Accessibility (contrast, line 2135-2139)

---

### US-005: Build Layout Containers (main, sidebar, conversation, header, composer)

**Story Type:** Component

**Priority:** P0 (MVP Critical)

**Week:** 1

**Persona:** James (Power User - needs efficiency)

**As a** power user  
**I want** a responsive layout that adapts to different screen sizes  
**So that** I can use the app comfortably on my laptop and monitor

### Acceptance Criteria
- [ ] AC1: Main window container renders (full screen, dark background)
- [ ] AC2: Sidebar container renders (240px width on desktop, fixed left)
- [ ] AC3: Conversation area container renders (flex-grow, middle section)
- [ ] AC4: Header container renders (56px height, dark bar)
- [ ] AC5: Composer container renders (56px height, sticky at bottom)
- [ ] AC6: Responsive behavior at 640px breakpoint (sidebar hidden/drawer)
- [ ] AC7: Responsive behavior at 900px breakpoint (sidebar collapses to icons)
- [ ] AC8: Responsive behavior at 1200px+ (full sidebar visible)
- [ ] AC9: Layout reflow is smooth (no janky animations, 60 FPS)
- [ ] AC10: All containers have proper spacing (8px grid adherence)
- [ ] AC11: No horizontal scrolling except where needed

### Definition of Done
- [ ] Code written: `/src/frontend/components/layout-containers.slint`
- [ ] Responsive tests at 3 breakpoints (640px, 900px, 1200px)
- [ ] Performance test: 60 FPS on window resize
- [ ] Code review approved
- [ ] Visual verification on actual Windows devices
- [ ] Accessibility: logical tab order through containers

### Estimation
**Size:** M (3-5 days)  
**Complexity:** Medium  
**Risk:** Medium (responsive behavior complexity)

### Dependencies
- Blocks: US-008 (Conversation List Item), US-009 (Message Bubble), US-006 (Message Composer)
- Blocked by: US-001 (Design Tokens), US-004 (Typography)
- Related: All UI components

### Design References
- UX Spec Section 8: Layout Architecture (lines 1586-1722)
- UX Spec Section 6: Design Direction - Compact Professional (lines 1586-1622)

---

### US-010: Implement Loading States & Skeleton Screens

**Story Type:** Component/Feature

**Priority:** P0 (MVP Critical)

**Week:** 1

**Persona:** Sarah (First-Time User - wants confidence)

**As a** first-time user  
**I want** to see clear loading indicators when the app is fetching data  
**So that** I know something is happening and the app isn't frozen

### Acceptance Criteria
- [ ] AC1: Skeleton screen components render (conversation list, message area, header)
- [ ] AC2: Skeleton shimmer animation works (400ms duration, ease-in-out)
- [ ] AC3: Shimmer animation loops infinitely until content loads
- [ ] AC4: Skeleton screens layout matches actual content (no layout shift)
- [ ] AC5: Loading spinner renders (24px, Fluent Blue #0078D4, 800ms rotation)
- [ ] AC6: Spinner animation is smooth (60 FPS, linear easing)
- [ ] AC7: "Loading..." text appears below spinner (Body, 14px, centered)
- [ ] AC8: Motion preference detection works (prefers-reduced-motion: reduce)
- [ ] AC9: Reduced-motion: skeleton shimmer disabled (static gray)
- [ ] AC10: Reduced-motion: spinner replaced with pulsing opacity (300ms pulse, 600ms pause)
- [ ] AC11: Skeleton fades out when content appears (200ms fade, ease-out)
- [ ] AC12: No loading state blocks user interaction

### Definition of Done
- [ ] Code written: `/src/frontend/components/loading-states.slint`
- [ ] Skeleton components for 3 contexts (list, messages, header)
- [ ] Unit tests for animation timing
- [ ] Motion preference tests (verify reduced-motion variants)
- [ ] Visual regression tests
- [ ] Code review approved
- [ ] Accessibility: screen reader announces loading state

### Estimation
**Size:** M (3-5 days)  
**Complexity:** Medium  
**Risk:** Medium (animation timing precision)

### Dependencies
- Blocks: US-008 (Conversation List Item), US-009 (Message Bubble)
- Blocked by: US-001 (Design Tokens)
- Related: US-011 (Delivery Status)

### Design References
- UX Spec Section 10: Loading States & Feedback (lines 1926-2025)
- UX Spec Section 11: Motion & Animation Accessibility (lines 2141-2235)

---

## WEEK 2: Conversation Components

### US-008: Build Conversation List Item Component

**Story Type:** Component

**Priority:** P1 (MVP High)

**Week:** 2

**Persona:** Elena (Team Lead - manages 6+ conversations)

**As a** team lead  
**I want** to see a clear list of conversations with status indicators  
**So that** I can quickly find who to talk to and know their availability

### Acceptance Criteria
- [ ] AC1: Conversation list item renders with avatar (32x32px)
- [ ] AC2: Contact name displayed (Heading 3, 18px, 600 weight)
- [ ] AC3: Last message preview shown (Body Small, 12px, gray)
- [ ] AC4: Timestamp of last message displayed (Caption, 11px, right-aligned)
- [ ] AC5: Unread badge displays count (white text on Fluent Blue, rounded)
- [ ] AC6: Presence dot visible (8px, color-coded: green/yellow/gray/red)
- [ ] AC7: Presence dot positioned on avatar (top-left corner)
- [ ] AC8: Hover state shows subtle highlight (2% opacity overlay)
- [ ] AC9: Selected state shows clear visual indicator (left border + background)
- [ ] AC10: Item height is 40px (compact)
- [ ] AC11: Keyboard navigation works (arrow keys, enter)
- [ ] AC12: Click event fires immediately (< 50ms)

### Definition of Done
- [ ] Code written: `/src/frontend/components/conversation-list-item.slint`
- [ ] Unit tests for all states (4 test cases)
- [ ] Visual regression tests
- [ ] Code review approved
- [ ] Accessibility: screen reader announces contact name, presence, unread count
- [ ] Keyboard nav tested (Tab, arrows, Enter)

### Estimation
**Size:** M (3-5 days)  
**Complexity:** Low-Medium  
**Risk:** Low

### Dependencies
- Blocks: US-012 (Conversation List Container)
- Blocked by: US-001 (Design Tokens), US-005 (Layout), US-004 (Typography)
- Related: US-009 (Message Bubble), US-013 (Presence Avatar)

### Design References
- UX Spec Section 9: User Journeys - Sarah's journey (lines 1723-1763)
- UX Spec Section 10: Navigation Patterns (lines 1885-1903)
- UX Spec Section 10: Presence & Availability (lines 1926-1948)

---

### US-009: Build Message Bubble Component (sent & received)

**Story Type:** Component

**Priority:** P1 (MVP High)

**Week:** 2

**Persona:** James (Power User - reads/sends messages)

**As a** power user  
**I want** to see messages clearly distinguished (mine vs. others)  
**So that** I can easily follow the conversation flow

### Acceptance Criteria
- [ ] AC1: Sent message bubble renders right-aligned (Fluent Blue background)
- [ ] AC2: Received message bubble renders left-aligned (Light gray background)
- [ ] AC3: Message text displays (Body, 14px, wrapped at container width)
- [ ] AC4: Timestamp shows (Caption, 11px, gray, right of bubble)
- [ ] AC5: Delivery status indicator shows (pending/sent/delivered/read)
- [ ] AC6: Delivery icons render correctly (spinner/check/double-check/highlighted)
- [ ] AC7: Message bubbles have rounded corners (2px radius)
- [ ] AC8: Proper spacing between messages (8px vertical gap)
- [ ] AC9: Long messages wrap correctly (max 400px width)
- [ ] AC10: Emoji and special characters render correctly
- [ ] AC11: Group sender name shows (if group chat, future)
- [ ] AC12: Hover state shows options menu trigger (future)

### Definition of Done
- [ ] Code written: `/src/frontend/components/message-bubble.slint`
- [ ] Unit tests for all states (4 test cases)
- [ ] Visual regression tests
- [ ] Code review approved
- [ ] Accessibility: screen reader announces sender, message content, timestamp
- [ ] Long message wrapping tested (500+ character messages)

### Estimation
**Size:** M (3-5 days)  
**Complexity:** Low-Medium  
**Risk:** Low

### Dependencies
- Blocks: US-014 (Conversation View)
- Blocked by: US-001 (Design Tokens), US-004 (Typography), US-011 (Delivery Status)
- Related: US-008 (Conversation List Item), US-013 (Presence Avatar)

### Design References
- UX Spec Section 9: User Journeys - James' journey (lines 1764-1822)
- UX Spec Section 10: Message Sending & Receiving (lines 1904-1925)
- UX Spec Section 10: Delivery Status Pattern (lines 1912-1918)

---

### US-011: Build Delivery Status Indicator Component

**Story Type:** Component

**Priority:** P1 (MVP High)

**Week:** 2

**Persona:** David (Support - needs clarity on message status)

**As a** support agent  
**I want** to know exactly when my messages are delivered and read  
**So that** I can ensure critical information reaches recipients

### Acceptance Criteria
- [ ] AC1: Pending state displays (gray spinner icon, positioned bottom-right of bubble)
- [ ] AC2: Sent state displays (single checkmark, Fluent Blue #0078D4)
- [ ] AC3: Delivered state displays (double checkmark, Fluent Blue)
- [ ] AC4: Read state displays (double checkmark, highlighted/lighter shade)
- [ ] AC5: Failed state displays (red X icon, Error red #E81123)
- [ ] AC6: Hover tooltip shows timestamp (e.g., "Sent 2:34 PM")
- [ ] AC7: Icons are accessible (aria-label: "Message sent", etc.)
- [ ] AC8: State transitions are smooth (100ms fade between states)
- [ ] AC9: Icon size is 16px (consistent, visible but not intrusive)
- [ ] AC10: Failed state shows retry button on hover

### Definition of Done
- [ ] Code written: `/src/frontend/components/delivery-status.slint`
- [ ] Unit tests for all 5 states
- [ ] State transition tests
- [ ] Code review approved
- [ ] Accessibility: screen reader announces full status
- [ ] Visual verification on actual messages

### Estimation
**Size:** S (1-3 days)  
**Complexity:** Low  
**Risk:** Low

### Dependencies
- Blocks: US-009 (Message Bubble)
- Blocked by: US-001 (Design Tokens)
- Related: US-006 (Message Composer), US-015 (Real-Time Message Updates)

### Design References
- UX Spec Section 10: Message Sending & Receiving (lines 1904-1925)
- UX Spec Section 10: Delivery Status Pattern (lines 1912-1918)

---

### US-013: Build Presence Avatar Component

**Story Type:** Component

**Priority:** P1 (MVP High)

**Week:** 2

**Persona:** Elena (Team Lead - knows who's available)

**As a** team lead  
**I want** to see at a glance who is online  
**So that** I can decide who to reach out to immediately

### Acceptance Criteria
- [ ] AC1: Avatar image displays (32x32px or 40x40px variants)
- [ ] AC2: Presence dot overlaid on avatar (top-left corner, 8px)
- [ ] AC3: Online state shows green dot (#107C10)
- [ ] AC4: Away state shows yellow dot (#FFB900)
- [ ] AC5: Offline state shows gray dot (#ADADAD)
- [ ] AC6: Do Not Disturb state shows red dot (#E81123)
- [ ] AC7: Presence dot has slight border for contrast
- [ ] AC8: Hover tooltip shows status text (e.g., "Online now", "Away 5 min", "Offline")
- [ ] AC9: Presence dot is accessible (not color-only, has shape variation)
- [ ] AC10: Dot updates immediately when presence changes (< 200ms)

### Definition of Done
- [ ] Code written: `/src/frontend/components/presence-avatar.slint`
- [ ] Unit tests for all 4 states
- [ ] Accessibility verification (not color-only, shape distinct)
- [ ] Code review approved
- [ ] Screen reader announces presence status
- [ ] Visual verification (color-blind simulator)

### Estimation
**Size:** S (1-3 days)  
**Complexity:** Low  
**Risk:** Low

### Dependencies
- Blocks: US-008 (Conversation List Item), US-016 (Conversation Header)
- Blocked by: US-001 (Design Tokens)
- Related: US-017 (Real-Time Presence Updates)

### Design References
- UX Spec Section 7: Visual Design - Color System (lines 1246-1308)
- UX Spec Section 10: Presence & Availability (lines 1926-1948)
- UX Spec Section 11: Accessibility (color-blind safe, line 2501)

---

## WEEK 3: Real-Time Components

### US-006: Build Message Composer Component

**Story Type:** Component

**Priority:** P0 (MVP Critical)

**Week:** 3

**Persona:** Sarah (First-Time User - wants to send her first message)

**As a** first-time user  
**I want** a clear, simple way to type and send a message  
**So that** I can quickly send my first message and see it arrive

### Acceptance Criteria
- [ ] AC1: Text input renders (minimum 40px height, padded)
- [ ] AC2: Text expands as user types (up to 3-4 lines max)
- [ ] AC3: Send button visible on right side (Fluent Blue button)
- [ ] AC4: Send button disabled when input is empty
- [ ] AC5: Enter key sends message (Cmd+Enter on Mac, Ctrl+Enter adds newline)
- [ ] AC6: Message appears locally immediately (optimistic rendering)
- [ ] AC7: Loading spinner shows in send button while sending
- [ ] AC8: Character count optional (for future length limits)
- [ ] AC9: Composer is sticky at bottom (doesn't scroll with messages)
- [ ] AC10: Keyboard shortcuts documented (Enter to send, Ctrl+Enter for newline)
- [ ] AC11: Error handling: if send fails, message stays in input with retry button

### Definition of Done
- [ ] Code written: `/src/frontend/components/message-composer.slint`
- [ ] Unit tests for input/send logic
- [ ] Keyboard shortcuts tested
- [ ] Error handling tested
- [ ] Code review approved
- [ ] Accessibility: label clear, keyboard nav works
- [ ] Integration test: message sent to backend

### Estimation
**Size:** M (3-5 days)  
**Complexity:** Medium  
**Risk:** Medium (keyboard handling + send logic)

### Dependencies
- Blocks: US-014 (Conversation View), US-015 (Real-Time Message Updates)
- Blocked by: US-001 (Design Tokens), US-002 (Button), US-003 (Text Input)
- Related: US-011 (Delivery Status)

### Design References
- UX Spec Section 10: Message Sending & Receiving (lines 1904-1925)
- UX Spec Section 3: Core Experience - Phase 3 (lines 1101-1129)

---

### US-015: Real-Time Message Updates (WebSocket integration)

**Story Type:** Integration

**Priority:** P0 (MVP Critical)

**Week:** 3

**Persona:** James (Power User - expects real-time responsiveness)

**As a** power user  
**I want** to see new messages appear instantly  
**So that** I don't miss anything and conversation feels alive

### Acceptance Criteria
- [ ] AC1: New messages arrive via WebSocket (not polling)
- [ ] AC2: Message appears in conversation in < 200ms (latency target)
- [ ] AC3: Message arrival triggers fade-in animation (200ms, ease-out)
- [ ] AC4: Message arrival with prefers-reduced-motion: instant appearance
- [ ] AC5: Conversation list updates when new message arrives
- [ ] AC6: Unread badge increments for conversations user isn't viewing
- [ ] AC7: Multiple messages arriving simultaneously handled correctly
- [ ] AC8: Message order is correct (chronological)
- [ ] AC9: Connection loss handled gracefully (show "Connection lost" banner)
- [ ] AC10: Auto-reconnect after connection loss (exponential backoff)
- [ ] AC11: Typing indicator shows in real-time (< 200ms latency)

### Definition of Done
- [ ] Code written: Integration with WebSocket backend
- [ ] Unit tests for message ordering
- [ ] Integration tests with test backend
- [ ] Latency measurements (< 200ms verified)
- [ ] Code review approved
- [ ] Performance test: 60 FPS when messages arrive
- [ ] Error handling tested (connection loss, malformed messages)

### Estimation
**Size:** L (5-8 days)  
**Complexity:** High  
**Risk:** High (real-time is notoriously tricky)

### Dependencies
- Blocks: US-017 (Real-Time Presence Updates), US-018 (Typing Indicator)
- Blocked by: US-006 (Message Composer), US-009 (Message Bubble), US-011 (Delivery Status)
- Related: All real-time features

### Design References
- UX Spec Section 3: Core Experience - Phase 4 (lines 1130-1164)
- UX Spec Section 3: Core Experience - Performance Targets (lines 942-987)

---

### US-017: Real-Time Presence Updates

**Story Type:** Integration

**Priority:** P1 (MVP High)

**Week:** 3

**Persona:** Elena (Team Lead - watches team availability)

**As a** team lead  
**I want** to see when people come online or go offline  
**So that** I know who's available to collaborate

### Acceptance Criteria
- [ ] AC1: Presence updates arrive via WebSocket (not polling)
- [ ] AC2: Presence dot changes color instantly (< 200ms latency)
- [ ] AC3: Presence change with prefers-reduced-motion: instant (no fade)
- [ ] AC4: Presence change with normal motion: smooth fade (300ms, ease-in-out)
- [ ] AC5: Status changes in conversation header immediately
- [ ] AC6: Status changes in conversation list items immediately
- [ ] AC7: "Last seen" time updates for offline users
- [ ] AC8: Presence changes don't block other interactions
- [ ] AC9: Batch presence updates if multiple users change at once
- [ ] AC10: Screen reader announces presence changes

### Definition of Done
- [ ] Code written: WebSocket integration for presence
- [ ] Unit tests for presence state management
- [ ] Integration tests with backend
- [ ] Latency verified (< 200ms)
- [ ] Code review approved
- [ ] Accessibility tested (announcements)
- [ ] Performance test: multiple presence updates

### Estimation
**Size:** M (3-5 days)  
**Complexity:** Medium  
**Risk:** Medium

### Dependencies
- Blocks: None directly
- Blocked by: US-015 (Real-Time Message Updates), US-013 (Presence Avatar)
- Related: US-018 (Typing Indicator)

### Design References
- UX Spec Section 10: Presence & Availability (lines 1926-1948)
- UX Spec Section 11: Motion & Animation - Presence Status (line 2178)

---

### US-018: Typing Indicator Component (with animation)

**Story Type:** Component/Feature

**Priority:** P1 (MVP High)

**Week:** 3

**Persona:** James (Power User - likes knowing when others are typing)

**As a** power user  
**I want** to see when someone is typing  
**So that** I know to wait for their full message

### Acceptance Criteria
- [ ] AC1: Typing indicator appears when contact starts typing ("Jane is typing...")
- [ ] AC2: Animated dots bounce (600ms loop with ease-in-out)
- [ ] AC3: With prefers-reduced-motion: static text "Jane is typing..." (no animation)
- [ ] AC4: Typing indicator disappears when typing stops (< 500ms)
- [ ] AC5: Up to 3 typing users shown simultaneously
- [ ] AC6: Typing indicator appears at bottom of message thread
- [ ] AC7: Doesn't block message composition
- [ ] AC8: Screen reader announces typing state
- [ ] AC9: Latency from typing to indicator < 200ms
- [ ] AC10: False typing indicators timeout after 5 seconds

### Definition of Done
- [ ] Code written: `/src/frontend/components/typing-indicator.slint`
- [ ] Unit tests for animation
- [ ] Motion preference tests (reduced vs normal)
- [ ] Integration tests with backend
- [ ] Code review approved
- [ ] Accessibility: screen reader announces
- [ ] Performance test: smooth animation

### Estimation
**Size:** M (3-5 days)  
**Complexity:** Medium  
**Risk:** Medium (animation + real-time state)

### Dependencies
- Blocks: US-014 (Conversation View)
- Blocked by: US-001 (Design Tokens), US-015 (Real-Time Message Updates)
- Related: US-017 (Real-Time Presence)

### Design References
- UX Spec Section 10: Message Sending & Receiving - New Message Notification (line 1924)
- UX Spec Section 11: Motion & Animation - Typing Indicator (line 2177)

---

## WEEK 4: Integration & Polish

### US-014: Build Conversation View Component

**Story Type:** Component/Integration

**Priority:** P0 (MVP Critical)

**Week:** 4

**Persona:** Sarah (First-Time User - her first conversation)

**As a** first-time user  
**I want** to see a conversation with all messages and be able to scroll history  
**So that** I can catch up and see the full context

### Acceptance Criteria
- [ ] AC1: Conversation header displays (56px, with contact name + presence + menu)
- [ ] AC2: Message list scrolls vertically (infinite scroll down, load history up)
- [ ] AC3: Messages render in chronological order
- [ ] AC4: Message composer sticky at bottom
- [ ] AC5: New messages auto-scroll into view
- [ ] AC6: Loading skeleton shown when loading message history
- [ ] AC7: Typing indicator visible above composer
- [ ] AC8: Delivery status visible on each message
- [ ] AC9: Keyboard navigation works (Tab through messages, etc.)
- [ ] AC10: Mobile: message thread optimizes for narrow screen
- [ ] AC11: Conversation persists when switching away and back

### Definition of Done
- [ ] Code written: Integration of all message components
- [ ] Unit tests for scroll behavior
- [ ] Integration tests with backend (message loading)
- [ ] Code review approved
- [ ] Visual verification on multiple screen sizes
- [ ] Accessibility: screen reader can navigate messages
- [ ] Performance: 60 FPS when scrolling

### Estimation
**Size:** L (5-8 days)  
**Complexity:** High  
**Risk:** High (scroll behavior + real-time integration)

### Dependencies
- Blocks: US-012 (Conversation List Container)
- Blocked by: US-006 (Message Composer), US-009 (Message Bubble), US-015 (Real-Time), US-018 (Typing)
- Related: All UI components

### Design References
- UX Spec Section 8: Layout Architecture (lines 1586-1722)
- UX Spec Section 3: Core Experience (lines 1035-1206)

---

### US-012: Build Conversation List Container & Navigation

**Story Type:** Component/Integration

**Priority:** P0 (MVP Critical)

**Week:** 4

**Persona:** James (Power User - switches conversations frequently)

**As a** power user  
**I want** to see my recent conversations and switch between them quickly  
**So that** I can manage multiple conversations efficiently

### Acceptance Criteria
- [ ] AC1: Conversation list scrolls vertically (if 10+ conversations)
- [ ] AC2: Conversations sorted by last message time (most recent first)
- [ ] AC3: Selected conversation highlighted clearly
- [ ] AC4: Clicking conversation opens it (< 100ms)
- [ ] AC5: Keyboard navigation works (arrow keys, enter to select)
- [ ] AC6: Search/filter box at top
- [ ] AC7: Unread conversations distinguished (bold text or indicator)
- [ ] AC8: Unread badge shows count of unread messages
- [ ] AC9: Conversation list updates in real-time
- [ ] AC10: Presence indicators visible on each item
- [ ] AC11: Hover state shows action buttons (future: mute, archive)

### Definition of Done
- [ ] Code written: `/src/frontend/components/conversation-list-container.slint`
- [ ] Unit tests for navigation logic
- [ ] Integration tests with backend
- [ ] Code review approved
- [ ] Accessibility: screen reader can navigate list
- [ ] Keyboard nav fully tested
- [ ] Performance: smooth scroll with 100+ conversations

### Estimation
**Size:** M (3-5 days)  
**Complexity:** Medium  
**Risk:** Medium

### Dependencies
- Blocks: None directly
- Blocked by: US-008 (Conversation List Item), US-014 (Conversation View)
- Related: US-007 (Search Box)

### Design References
- UX Spec Section 10: Navigation Patterns (lines 1885-1903)
- UX Spec Section 3: Core Experience - Phase 2 (lines 1060-1100)

---

### US-016: Build Conversation Header Component

**Story Type:** Component

**Priority:** P1 (MVP High)

**Week:** 4

**Persona:** Elena (Team Lead - context matters)

**As a** team lead  
**I want** to see who I'm talking to and their current status  
**So that** I understand the context and know if they're available

### Acceptance Criteria
- [ ] AC1: Contact name displayed prominently (Heading 2, 16px)
- [ ] AC2: Presence dot and status label ("Online", "Away 5 min", "Offline")
- [ ] AC3: Last seen time displayed (if offline)
- [ ] AC4: Menu button (three dots) for options (future: info, settings, leave)
- [ ] AC5: Header height 56px
- [ ] AC6: Sticky at top of conversation
- [ ] AC7: Responsive: text truncates on narrow screens
- [ ] AC8: Presence dot updates in real-time
- [ ] AC9: Hover on presence shows tooltip (extended status)
- [ ] AC10: Accessibility: screen reader announces all info

### Definition of Done
- [ ] Code written: `/src/frontend/components/conversation-header.slint`
- [ ] Unit tests for display logic
- [ ] Integration test: presence updates
- [ ] Code review approved
- [ ] Accessibility verified
- [ ] Visual verification on multiple screen sizes

### Estimation
**Size:** S (1-3 days)  
**Complexity:** Low  
**Risk:** Low

### Dependencies
- Blocks: US-014 (Conversation View)
- Blocked by: US-001 (Design Tokens), US-013 (Presence Avatar)
- Related: US-017 (Real-Time Presence)

### Design References
- UX Spec Section 10: Presence & Availability (lines 1926-1948)

---

## WEEK 5: Animations & Transitions

### US-019: Implement Message Animations (fade-in, delivery transitions)

**Story Type:** Enhancement

**Priority:** P2 (MVP Medium - nice to have)

**Week:** 5

**Persona:** Sarah (First-Time User - loves smooth interactions)

**As a** first-time user  
**I want** messages to appear smoothly  
**So that** the app feels polished and responsive

### Acceptance Criteria
- [ ] AC1: New message fades in (200ms, ease-out)
- [ ] AC2: Message appearance with prefers-reduced-motion: instant (no fade)
- [ ] AC3: Delivery status transitions smoothly (100ms between states)
- [ ] AC4: All animations hit 60 FPS consistently
- [ ] AC5: No animation stuttering or jank
- [ ] AC6: Animation can be interrupted (e.g., scroll before animation completes)

### Definition of Done
- [ ] Code written: Animation implementations
- [ ] Performance test: 60 FPS sustained
- [ ] Motion preference tests
- [ ] Code review approved
- [ ] Visual verification on slower hardware

### Estimation
**Size:** M (3-5 days)  
**Complexity:** Medium  
**Risk:** Low

### Dependencies
- Blocks: None
- Blocked by: US-015 (Real-Time Messages), US-011 (Delivery Status)
- Related: All animation work

### Design References
- UX Spec Section 10: Loading States - Animation Specs (line 1987-1995)
- UX Spec Section 11: Motion & Animation - Message Arrival (line 2179)

---

### US-020: Implement Hover & Focus Animations

**Story Type:** Enhancement

**Priority:** P2 (MVP Medium)

**Week:** 5

**Persona:** James (Power User - appreciates polish)

**As a** power user  
**I want** smooth hover and focus states  
**So that** the app feels responsive and polished

### Acceptance Criteria
- [ ] AC1: Button hover: color transition (200ms, ease-in-out)
- [ ] AC2: Button focus: 2px outline with glow (no animation needed)
- [ ] AC3: List item hover: subtle highlight (background color 2% increase)
- [ ] AC4: List item focus: clear outline
- [ ] AC5: All transitions 60 FPS
- [ ] AC6: Reduced-motion: instant transitions (no animation)
- [ ] AC7: Touch targets remain 44px+ minimum

### Definition of Done
- [ ] Code written: Hover/focus implementations
- [ ] Performance test: 60 FPS on all states
- [ ] Motion preference tests
- [ ] Code review approved
- [ ] Accessibility: focus always visible

### Estimation
**Size:** S (1-3 days)  
**Complexity:** Low  
**Risk:** Low

### Dependencies
- Blocks: None
- Blocked by: US-002 (Button), US-008 (List Item)
- Related: US-019 (Message Animations)

### Design References
- UX Spec Section 10: Loading States - Micro-interaction Guidelines (line 2010-2015)
- UX Spec Section 11: Motion & Animation - Hover Effects (line 2185)

---

## WEEK 6: Accessibility & Compliance

### US-021: WCAG 2.1 AA Compliance Testing & Remediation

**Story Type:** Quality Assurance

**Priority:** P0 (MVP Critical)

**Week:** 6

**Persona:** QA Team + Accessibility

**As a** QA team  
**I want** verified WCAG 2.1 AA compliance  
**So that** all users can access the application

### Acceptance Criteria
- [ ] AC1: Axe automated scan: 0 violations
- [ ] AC2: WAVE manual scan: 0 errors
- [ ] AC3: Color contrast verified (7:1 or 4.5:1 min)
- [ ] AC4: Keyboard navigation: 100% of workflows accessible
- [ ] AC5: Screen reader (NVDA): all content readable
- [ ] AC6: Focus indicators: visible on all interactive elements
- [ ] AC7: No information by color alone (icons/text required)
- [ ] AC8: Form labels properly associated
- [ ] AC9: No auto-playing audio or video
- [ ] AC10: Motion/flashing < 3 per second

### Definition of Done
- [ ] Automated tests pass (Axe, WAVE)
- [ ] Manual testing completed
- [ ] Screen reader testing with NVDA
- [ ] Keyboard-only navigation verified
- [ ] Accessibility audit report created
- [ ] All issues logged and resolved
- [ ] Code review approved

### Estimation
**Size:** L (5-8 days)  
**Complexity:** High  
**Risk:** Medium

### Dependencies
- Blocks: Release readiness
- Blocked by: All other stories
- Related: US-022 (Motion Preferences)

### Design References
- UX Spec Section 11: Accessibility Implementation (lines 2115-2240)

---

### US-022: Motion Preferences & Vestibular Accessibility (WCAG 2.3.3 & 2.3.4)

**Story Type:** Quality Assurance

**Priority:** P0 (MVP Critical)

**Week:** 6

**Persona:** QA Team + Accessibility

**As a** user with vestibular sensitivities  
**I want** animations to respect my accessibility preferences  
**So that** I can use the app comfortably

### Acceptance Criteria
- [ ] AC1: `prefers-reduced-motion: reduce` respected system-wide
- [ ] AC2: All 10 animation reductions implemented
- [ ] AC3: Typing indicator: dots animation → static text (reduced motion)
- [ ] AC4: Presence status: fade → instant change (reduced motion)
- [ ] AC5: Message arrival: fade-in → instant (reduced motion)
- [ ] AC6: Skeleton shimmer: animation → static (reduced motion)
- [ ] AC7: Loading spinner: rotation → pulse (reduced motion)
- [ ] AC8: Hover transitions: instant (reduced motion)
- [ ] AC9: Focus indicators: static (reduced motion)
- [ ] AC10: Notifications: instant appearance (reduced motion)
- [ ] AC11: Testing with `prefers-reduced-motion: reduce` enabled
- [ ] AC12: WCAG 2.3.3 (Animation from Interactions) verified
- [ ] AC13: WCAG 2.3.4 (Animation Options) verified
- [ ] AC14: No flashing/strobing (< 3 per second)
- [ ] AC15: User testing: vestibular disorder participants (if possible)

### Definition of Done
- [ ] Code written: Motion preference implementation
- [ ] Automated tests for all reduced-motion variants
- [ ] Manual testing with Windows settings enabled
- [ ] Screen reader testing (animations don't interfere)
- [ ] User testing with vestibular disorder users (recommended)
- [ ] WCAG 2.3.3 & 2.3.4 compliance verified
- [ ] Accessibility audit includes motion preferences
- [ ] Code review approved

### Estimation
**Size:** L (5-8 days)  
**Complexity:** High  
**Risk:** Medium

### Dependencies
- Blocks: Release readiness
- Blocked by: US-019 (Message Animations), US-020 (Hover Animations)
- Related: US-021 (WCAG Compliance)

### Design References
- UX Spec Section 11: Motion & Animation Accessibility (lines 2141-2235)

---

## Summary Table

| Week | Story ID | Title | Priority | Est. Size | Complexity |
|------|----------|-------|----------|-----------|------------|
| 1 | US-001 | Design Tokens | P0 | M | Low |
| 1 | US-002 | Base Button | P0 | M | Low |
| 1 | US-003 | Text Input | P0 | M | Low-Med |
| 1 | US-004 | Typography Scale | P0 | S | Low |
| 1 | US-005 | Layout Containers | P0 | M | Med |
| 1 | US-010 | Loading States | P0 | M | Med |
| 2 | US-008 | Conv List Item | P1 | M | Low-Med |
| 2 | US-009 | Message Bubble | P1 | M | Low-Med |
| 2 | US-011 | Delivery Status | P1 | S | Low |
| 2 | US-013 | Presence Avatar | P1 | S | Low |
| 3 | US-006 | Message Composer | P0 | M | Med |
| 3 | US-015 | Real-Time Messages | P0 | L | High |
| 3 | US-017 | Real-Time Presence | P1 | M | Med |
| 3 | US-018 | Typing Indicator | P1 | M | Med |
| 4 | US-014 | Conversation View | P0 | L | High |
| 4 | US-012 | Conv List Container | P0 | M | Med |
| 4 | US-016 | Conversation Header | P1 | S | Low |
| 5 | US-019 | Message Animations | P2 | M | Med |
| 5 | US-020 | Hover Animations | P2 | S | Low |
| 6 | US-021 | WCAG Compliance | P0 | L | High |
| 6 | US-022 | Motion Preferences | P0 | L | High |

**Total Stories:** 21 user stories (MVP scope)
**Total Effort:** ~13-16 weeks (solo dev) or ~3-4 weeks (team of 3-4)
**Phasing:** Organized into 6-week sprints with clear dependencies

---

## How to Use This Document

1. **Sprint Planning:** Use the stories for sprint backlog creation
2. **Daily Standup:** Track progress against each story's acceptance criteria
3. **Estimation:** Stories are pre-sized; adjust based on team velocity
4. **Dependencies:** Follow the dependency map to avoid blocking issues
5. **Review:** Share with team; each story is a natural discussion point

---

**Generated by:** Bob (Scrum Master) + Team  
**Date:** December 16, 2025  
**Status:** Ready for sprint planning

