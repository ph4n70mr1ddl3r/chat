# Story 4.2: Implement PresenceIndicator Component (Slint)

**Status:** ready-for-dev
**Priority:** P1 (Essential)
**Week:** 2
**Owner:** Amelia (Developer)
**Designer:** Sally
**Reviewer:** Winston

## 📖 Story Description
**As a** chat user,
**I want** a clear visual presence indicator for online/offline status,
**So that** I can see at a glance whether the other participant is available.

## 🎯 Acceptance Criteria

- [ ] **AC1:** Distinct visual indicators for status:
    - **Online:** `SUCCESS` (Green) dot
    - **Offline:** `ERROR` (Red) or `NEUTRAL_MEDIUM` (Gray) border/dot (refer to Design Tokens)
    - **Away/Idle:** `WARNING` (Amber/Yellow) dot
- [ ] **AC2:** Accurate state reflection: The indicator correctly represents the state fetched from `AppState.presence_map` for a given `user_id`.
- [ ] **AC3:** Real-time updates: The UI updates immediately when a `PresenceChanged` event is processed by the backend handler and synced to the Slint state.
- [ ] **AC4:** Accessible Presentation:
    - Clear color contrast for all states.
    - Support for tooltips or descriptive labels ("Online", "Offline", "Away") when hovered or in high-detail views.
    - Shapes or secondary markers should be considered if color alone is insufficient for accessibility (best effort for MVP).
- [ ] **AC5:** Multiple Surfaces: The component should be reusable in the `ConversationList`, `ConversationHeader`, and `UserDetail` panels.

## 🛠️ Developer Context
- **Component Path:** `src/frontend/components/presence/PresenceIndicator.slint`
- **Base Components:** Use `Rectangle` for the dot, `Tokens` for colors and spacing.
- **Design System Rules:**
    - Use `SUCCESS` (#107C10) for Online.
    - Use `WARNING` (#FFB900) for Away/Idle.
    - Use `NEUTRAL_MEDIUM` (#666666) or `ERROR` for Offline (confirmed per `DESIGN_TOKENS_REFERENCE.md`).
- **State Integration:**
    - The `PresenceIndicator` should accept a `status` property ("online" | "offline" | "away").
    - Parent components will bind this property from `AppState.presence_map.get(user_id)`.
- **Accessibility:** Set `accessible-role: "img"` and `accessible-label` based on the status.

## 📝 Tasks

### Task 1: Component Setup
- [ ] Create `src/frontend/components/presence/PresenceIndicator.slint`.
- [ ] Define `in property <string> status`.
- [ ] Implement the visual dot with size (e.g., 8-10px) and border for contrast on various backgrounds.

### Task 2: State & Token Binding
- [ ] Map the `status` prop to `Tokens` color constants.
- [ ] Implement smooth color transitions using `DURATION_QUICK`.

### Task 3: Accessibility & Tooltips
- [ ] Add `accessible-label` mapping (e.g., "User is online").
- [ ] Implement a basic tooltip or hover state that shows the text status.

### Task 4: Integration & Verification
- [ ] Register the component in the appropriate module.
- [ ] Create basic integration test `tests/integration/presence_indicator_test.rs` to verify color mapping.
- [ ] Document in `docs/PRESENCEINDICATOR_COMPONENT_REFERENCE.md`.

## 📊 Definition of Done Checklist

- [ ] AC1-5 pass
- [ ] Component is reusable across 2+ surfaces
- [ ] Contrast ratios verified against design system standards
- [ ] Unit/Integration tests pass
- [ ] Code review approved

---

## 📈 Estimation
- **Size:** XS (1-2 days)
- **Complexity:** Low
- **Risk:** Low

## 🔗 Dependencies
- **Blocks:** Story 4.3 (Presence in Multiple Surfaces)
- **Blocked by:** Static Design Tokens (completed)

---

## 📅 Completion Notes List
- 2025-12-18: Story created by Bob (Scrum Master).
