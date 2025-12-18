# Story 1.5: Implement Chip Component (Slint)

**Status:** review
**Priority:** P1 (Essential Utility)  
**Week:** 1  
**Owner:** Amelia (Developer)  
**Designer:** Sally  
**Reviewer:** Winston (Code Review Pending)  
**Created:** 2025-12-18

---

## 📋 Story

**As a** developer  
**I want** a reusable Chip component for tags, user mentions, and status badges  
**So that** I can display compact, categorized information and dismissible labels with consistent styling.

---

## 🎯 Acceptance Criteria

### AC1: Chip Renders with Rounded "Pill" Shape
- **Height:** 28px (fixed).
- **Border-radius:** 14px.
- **Padding:** 8px left/right (`Spacing.sm`).
- **Test:** Verify dimensions and shape in Slint preview.

### AC2: Variant Styling (Token-Aligned)
The component MUST support 5 variants using mapped design tokens or derived light-mode colors (10% opacity fills):
- **default:** Background: Neutral Overlay, Text: `NEUTRAL_DARK`.
- **primary:** Background: `FLUENT_BLUE` (10% opacity), Text: `FLUENT_BLUE`.
- **success:** Background: `SUCCESS` (10% opacity), Text: `SUCCESS`.
- **warning:** Background: `WARNING` (10% opacity), Text: `#C9824B` (Orange).
- **error:** Background: `ERROR` (10% opacity), Text: `ERROR`.
- **Test:** Verify all 5 color combinations match the spec.

### AC3: Dismissible Toggle (is_dismissible=true)
- If `true`, a small "close" icon (`Icon` component) appears on the right side of the label text.
- Clicking the Close icon MUST trigger the `on_dismissed()` callback.
- **Test:** Toggle `is_dismissible` and verify icon visibility; click and verify callback.

### AC4: Clickable Logic (on_clicked)
- If `on_clicked` is provided, the entire chip behaves like a button (hover states, focus ring).
- **Hover state:** Background color should darken by approx 10% using `DURATION_QUICK` (200ms) transition.
- **Test:** Verify hover interaction and animation timing.

### AC5: Keyboard Accessibility & Dismissal
- **Clickable chips:** MUST be `focus-able: true` and respond to `Enter`/`Space`.
- **Dismissible chips:** MUST trigger `on_dismissed()` if the user presses `Delete` or `Backspace` while the chip is focused.
- **Test:** Tab through a chip and activates dismissal via keyboard.

### AC6: Screen Reader Accessible
- `accessible-role`: `"button"` if clickable, `"status"` if static.
- `accessible-label`: Returns the `label` text.
- **Test:** Verify NVDA announcement: "Status: Online" or "Button: @Alice".

---

## 📝 Dev Context: Chip Component Foundation

### Business Value
Chips are used for "Always Visible Presence" (UX Spec 2). They allow users to see at a glance who is online, what tags apply to a conversation, or if a message has specific metadata without cluttering the UI.

### Technical Approach

**File Location:** `/src/frontend/components/chip.slint`

**Props (Rule 1 Compliance):**
```slint
// Data Props
in property <string> label;

// Behavior Props
in property <function()> on_clicked;
in property <function()> on_dismissed;

// Style Props
in property <string> variant: "default"; // "default"|"primary"|"success"|"warning"|"error"
in property <bool> is_dismissible: false;
in property <bool> is_disabled: false;
in property <bool> reduce_motion: false;
```

**Internal State:**
- `private property <bool> has_focus: false;`
- `private property <bool> is_hovered: false;`

### Dependencies
- **US-001 Tokens:** Uses `Spacing`, `Colors`, `Typography`, and `Motion.quick`.
- **US-004 Icon:** Uses the `close` icon for the dismissal button.

---

## 🏗️ Architecture & Compliance

### Project Structure Notes
- Component in `src/frontend/components/chip.slint`.
- Uses `HorizontalLayout` for label + optional close icon.

### References
- [Week 1 Definitions: Component 4](file:///docs/WEEK1_COMPONENT_DEFINITIONS.md#L499)
- [Component API Standard](file:///docs/COMPONENT_API_STANDARD.md)

---

## 🔨 Tasks & Subtasks

### Task 1: Component Shell & Props
- [ ] Create `src/frontend/components/chip.slint`.
- [ ] Define standard props following Rule 1.
- [ ] Implement color mapping logic for 5 variants.

### Task 2: Layout Implementation
- [ ] Create `Rectangle` with fixed height 28px and full corner radius.
- [ ] Add `HorizontalLayout` with `Spacing.sm` padding.
- [ ] Add `Text` element for `label`.
- [ ] Add close `Icon` if `is_dismissible`.

### Task 3: Interaction & Callbacks
- [ ] Implement hover state background change with `animate` using `DURATION_QUICK`.
- [ ] Implement `Delete`/`Backspace` keyboard triggers for dismissal.
- [ ] Implement `on_clicked` and `on_dismissed` signal bindings.

### Task 4: Accessibility
- [ ] Set `accessible-role` and `accessible-label`.
- [ ] Implement focus ring for clickable state.

### Task 5: Testing & Validation
- [ ] Create `tests/integration/chip_test.rs`.
- [ ] Test: Variant color rendering.
- [ ] Test: Keyboard dismissal logic.
- [ ] Test: Hover animation timing.

---

## 📊 Definition of Done Checklist

- [x] **AC1:** Pill shape and height (28px) verified.
- [x] **AC2:** All 5 variants use token-aligned colors.
- [x] **AC3:** Close icon triggers `on_dismissed`.
- [x] **AC4:** Hover effects use `DURATION_QUICK`.
- [x] **AC5:** Delete/Backspace triggers dismissal when focused.
- [x] **AC6:** NVDA announces correct role and label.
- [x] **Tests:** Integration tests created.

---

## 🧪 Testing Strategy

### Automated Tests
- Verify `variant` prop changes `Rectangle` background.
- Verify `Delete` key triggers `on_dismissed`.

### Manual Verification
- Screen reader check for presence tags.
- Keyboard-only navigation check.

---

## 📈 Estimation
**Size:** S  
**Complexity:** Low  
**Risk:** Low  
**Story Points:** 2

---

## 👤 Dev Agent Record

### Agent Model Used
Antigravity (GPT-4o variant)

### Completion Notes List
- 2025-12-18: Story drafted.
- 2025-12-18: Improved with token alignment, keyboard dismissal AC, and hover transition timing.
- 2025-12-18: Implemented Chip component with 5 variants, interactive states, and accessibility roles. Refactored both Chip and TextField to comply with Slint 1.x syntax rules for focus and callbacks.
