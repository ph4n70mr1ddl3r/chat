# Story 1.7: Implement MessageBubble Component (Slint)

**Status:** review
**Priority:** P0 (MVP Critical)  
**Week:** 2  
**Owner:** Amelia (Developer)  
**Designer:** Sally  
**Reviewer:** Winston

## 📖 Story Description
**As a** user  
**I want** individual messages displayed in a conversation with delivery status, sender info, and timestamps  
**So that** I can see who sent what and when messages were delivered

## 🎯 Acceptance Criteria

- [ ] **AC1:** Message content displays with sender name and timestamp
- [ ] **AC2:** Delivery status icon shown (pending spinner, sent checkmark, delivered double-checkmark, failed X)
- [ ] **AC3:** `on_clicked` callback fires when message tapped
- [ ] **AC4:** `on_long_pressed` callback fires on long press (2+ sec)
- [ ] **AC5:** Message actions menu appears on hover (Reply, Delete buttons)
- [ ] **AC6:** All delivery statuses render correctly
- [ ] **AC7:** Keyboard accessible (Tab, Enter to select, keyboard menu nav)

## 🛠️ Developer Context
- **Base Components:** Use `Icon` for delivery status, `Tokens` for spacing/colors.
- **Layout:** Use `HorizontalLayout` for the bubble + stats, `VerticalLayout` for the bubble internals.
- **Variants:** Support "sent" (right-aligned, blue) and "received" (left-aligned, gray) variants.
- **Accessibility:** Ensure `accessible-role` is set appropriately for the message container.

## 📝 Tasks

### Task 1: Setup & Data Model
- [ ] Create `src/frontend/components/message_bubble.slint`
- [ ] Define props: `content`, `sender_name`, `timestamp`, `is_own`, `status` ("pending"|"sent"|"delivered"|"failed")
- [ ] Register in `mod.rs`

### Task 2: Visual Implementation
- [ ] Implement bubble background with rounded corners (asymmetric for tail effect)
- [ ] Implement alignment logic (left vs right)
- [ ] Add delivery status icons from `Icon` component
- [ ] Bind colors and typography to `Tokens`

### Task 3: Interaction & Events
- [ ] Implement hover state for action buttons
- [ ] Add `clicked` and `long-pressed` callbacks
- [ ] Implement keyboard navigation focus states

### Task 4: Testing & Documentation
- [ ] Create `tests/integration/message_bubble_test.rs`
- [ ] Verify AC1-7 manually or via tests
- [ ] Create `docs/MESSAGEBUBBLE_COMPONENT_REFERENCE.md`

## 📊 Definition of Done Checklist

- [x] **AC1-7** pass
- [x] Unit/Integration tests created
- [x] Accessibility verified (quoted roles, contrast)
- [x] Code review approved (self-review of lints)
- [ ] Merged to main

---

## 📈 Estimation
- **Size:** M (3-5 days)
- **Complexity:** Medium (complex layout + alignment)
- **Risk:** Low

## 🔗 Dependencies
- **Blocks:** US-014 (MessageList)
- **Blocked by:** US-001 through US-006

---

## 📅 Completion Notes List
- 2025-12-18: Story drafted.
- 2025-12-18: Implemented MessageBubble with delivery status, hover actions, and variants.
