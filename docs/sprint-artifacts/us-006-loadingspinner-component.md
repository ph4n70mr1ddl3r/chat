# Story 1.6: Implement LoadingSpinner Component (Slint)

**Status:** review
**Priority:** P1 (Essential Feedback)  
**Week:** 1  
**Owner:** Amelia (Developer)  
**Designer:** Sally  
**Reviewer:** Winston (Code Review Pending)  
**Created:** 2025-12-18

---

## 📋 Story

**As a** user  
**I want** to see a clear visual indicator when the application is busy loading data  
**So that** I know the app hasn't frozen and my requested action is being processed.

---

## 🎯 Acceptance Criteria

### AC1: Animated "Halo" Spinner Style (Rectangle-Native)
- **Constraint:** Use a native `Rectangle` with a partial/full rotating border. Do NOT use an `Image` icon for the halo to ensure visual precision.
- **Visual Style:** Full-border rotating halo (as defined in [DESIGN_TOKENS_REFERENCE.md](file:///docs/DESIGN_TOKENS_REFERENCE.md#L411)).
- **Size Variants:**
  - `"small"`: 20px × 20px
  - `"medium"` (default): 24px × 24px
  - `"large"`: 32px × 32px
- **Border-width:** 2px.
- **Test:** Verify dimensions and rotation animation in Slint preview.

### AC2: Respects `reduce_motion` Preference (WCAG 2.3.3)
- **CRITICAL:** If `reduce_motion=true`, the spinner MUST NOT use an `animate` block. It must render as a static circle.
- **Pattern:** Follow the "Correct Usage" pattern in [DESIGN_TOKENS_REFERENCE.md](file:///docs/DESIGN_TOKENS_REFERENCE.md#L431).
- **Test:** Toggle `reduce_motion` and verify animation stops completely without "instant" flickering.

### AC3: Optional Loading Message
- Support `message` prop (string).
- If provided: Text appears 8px (`Spacing.sm`) below the spinner.
- **Text Style:** 12px (`FONT_SIZE_CAPTION`), `NEUTRAL_MEDIUM` color, centered.
- **Test:** Provide message and verify layout alignment and spacing.

### AC4: Colorization Support
- Defaults to `currentColor` (inherits from parent text color).
- Supports direct color overrides via `color` prop (e.g., `Colors.primary`).
- **Test:** Verify spinner color matches the provided token.

### AC5: Accessibility (ARIA Live & Roles)
- `accessible-role`: `"status"`.
- `accessible-label`: If `message` is provided, use it. Otherwise use "Loading...".
- **Aria-Live Behavior:** The wrapper should signal to screen readers that it's a "polite" update region.
- **Test:** Verify NVDA announces the loading state when the component appears.

---

## 📝 Dev Context: LoadingSpinner Component Foundation

### Business Value
"Responsive Delight" (UX Spec 4.1) requires that users never feel uncertainty during async tasks. The LoadingSpinner is the primary feedback mechanism for conversation switches and message fetching.

### Technical Approach

**File Location:** `/src/frontend/components/loading_spinner.slint`

**Implementation Detail:**
Use a `Rectangle` with `border-width: 2px` and `border-radius: height / 2`. Tie the rotation to a `rotation-angle` property and use a `loop-count: infinite` in the `animate` block.

**Props (Rule 1 Compliance):**
```slint
// Data Props
in property <string> message;        // Optional message

// Style Props
in property <string> size: "medium"; // "small"|"medium"|"large"
in property <brush> color: Colors.neutral_medium;
in property <bool> reduce_motion: false;
```

### Dependencies
- **US-001 Tokens:** Uses `Colors`, `Spacing`, `Typography`, `Motion.slow` (400ms), and `EASE_LINEAR`.

---

## 🏗️ Architecture & Compliance

### Project Structure Notes
- Component in `src/frontend/components/loading_spinner.slint`.
- Uses `VerticalLayout` to stack spinner and optional message.

### References
- [Week 1 Definitions: Component 5](file:///docs/WEEK1_COMPONENT_DEFINITIONS.md#L651)
- [Design Tokens Reference: Spinner Design Standards](file:///docs/DESIGN_TOKENS_REFERENCE.md#L411)
- [Component API Standard](file:///docs/COMPONENT_API_STANDARD.md)

---

## 🔨 Tasks & Subtasks

### Task 1: Component Shell & Props
- [ ] Create `src/frontend/components/loading_spinner.slint`.
- [ ] Define standard props.
- [ ] Implement size mapping logic.

### Task 2: Visual Implementation (Native Rectangle)
- [ ] Implement `Rectangle` with 2px border.
- [ ] Tie `rotation-angle` to state.
- [ ] Implement `animate` block inside a conditional `reduce_motion` check.
- [ ] Add loading `Text` element.

### Task 3: Accessibility
- [ ] Set `accessible-role: "status"`.
- [ ] Implement `accessible-label` computed property.

### Task 4: Testing & Validation
- [ ] Create `tests/integration/loading_spinner_test.rs`.
- [ ] Test: `rotation-angle` is static when `reduce_motion=true`.
- [ ] Test: Message visibility.

### Task 5: Documentation
- [ ] Document usage in `docs/LOADINGSPINNER_COMPONENT_REFERENCE.md`.

---

## 📊 Definition of Done Checklist

- [x] **AC1:** Native Rectangle halo implemented.
- [x] **AC2:** NO animation executes if `reduce_motion=true`.
- [x] **AC3:** Optional message renders correctly.
- [x] **AC4:** Inherits color correctly.
- [x] **AC5:** Screen reader announces status.
- [x] **Tests:** Integration tests created.

---

## 🧪 Testing Strategy

### Automated Tests
- Verify `rotation-angle` static on `reduce_motion`.
- Verify `AccessibleRole` status.

### Manual Verification
- Visual smoothness check on high-DPI.
- NVDA update check.

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
- 2025-12-18: Improved with native Rectangle mandate for high-DPI "Halo" aesthetics.
- 2025-12-18: Implemented LoadingSpinner with rotating halo using native Rectangle and clipping. Added size variants and accessibility support.
