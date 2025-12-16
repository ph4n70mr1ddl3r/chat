# Story 1.2: Implement Button Component (Slint)

**Status:** ready-for-dev  
**Priority:** P0 (MVP Critical)  
**Week:** 1  
**Owner:** Amelia (Developer)  
**Designer:** Sally  
**Reviewer:** Winston  
**Created:** 2025-12-16

---

## 📋 Story

**As a** designer and developer  
**I want** a reusable Button component with 4 variants (primary, secondary, tertiary, danger) and 3 sizes  
**So that** all clickable elements in the app are consistent and accessible

---

## 🎯 Acceptance Criteria

### AC1: Button Renders with Correct Variants ✓
- Primary variant: Fluent Blue (#0078D4) background, white text
  - Hover: Darker blue (#0063B1)
  - Active: Even darker blue (#004A94)
- Secondary variant: White background, blue text, outline border
  - Border: 1px Fluent Blue
  - Hover: Light blue background (#EFF6FC)
  - Active: Blue background (#F3F9FE)
- Tertiary variant: Transparent background, blue text, minimal style
  - Hover: Light blue background
  - Active: Blue background
- Danger variant: Red background (#A4373A), white text
  - Hover: Darker red (#8B2E31)
  - Active: Even darker red (#6B2327)
- Test: Visual comparison to UX spec, all variants render correctly

### AC2: All Sizes Render Correctly ✓
- Small: 28px height, compact padding
- Medium: 36px height, standard padding (default)
- Large: 44px height, generous padding
- Test: Measure button dimensions, verify against spec

### AC3: on_clicked Callback Fires When Clicked ✓
- Click button → callback fires
- Multiple clicks → callback fires each time
- Test: Unit test verifies callback invoked with correct timing

### AC4: Keyboard Accessible ✓
- Tab key: Focus moves to button
- Enter key: Activates button (fires on_clicked)
- Space key: Activates button (fires on_clicked)
- Test: Keyboard navigation + activation verified

### AC5: Respects reduce_motion Preference ✓
- When is_loading=true and reduce_motion=false: rotating spinner (400ms duration)
- When is_loading=true and reduce_motion=true: static spinner (no rotation)
- Test: Screenshot comparison, animation verification

### AC6: Disabled State Works Correctly ✓
- When is_disabled=true: button grayed out, clicks ignored
- When is_disabled=false: button interactive and clickable
- Test: Clicks prevented when disabled, allowed when enabled

### AC7: Loading State Works Correctly ✓
- When is_loading=true: spinner displays, label hidden
- When is_loading=false: label displays, spinner hidden
- Smooth transitions between states
- Test: State toggle shows/hides content correctly

### AC8: Screen Reader Accessible ✓
- NVDA announces: "Button: [label], [state if applicable]"
- All variants and states properly labeled
- Test: NVDA accessibility test

---

## 📝 Dev Context: Button Component Foundation

### Business Value
Buttons are the primary interactive element in the UI. A consistent, accessible Button component will:
- Enable all downstream UI work (AC6-AC8 components depend on Button)
- Ensure consistent user experience across all clickable elements
- Provide proper accessibility for users with disabilities
- Support loading states for async operations (message sending)
- Respect user motion preferences (WCAG 2.3.3)

### Technical Approach

**File Location:** `/src/frontend/components/button.slint`

This is a **Slint component** (not just constants) that:
1. Accepts input props (label, variant, size, disabled, loading, reduce_motion)
2. Handles click events and keyboard activation
3. Renders visual feedback for all states
4. Uses tokens from US-001 (colors, spacing, motion, typography)
5. Supports 4 variants × 3 sizes = 12 combinations

**Props:**
```slint
// Data props
label: string,
on_clicked: function(),

// Style props
variant: string,         // "primary", "secondary", "tertiary", "danger"
size: string,            // "small", "medium", "large"
is_disabled: bool,
is_loading: bool,
reduce_motion: bool,
```

**Why Slint:**
- Components handle state and interactivity
- Type-safe with prop validation
- Hot-reload for development
- Directly embeddable in other components

### Dependencies
- **US-001 Tokens:** Uses FLUENT_BLUE, SUCCESS, ERROR, DANGER colors; spacing tokens; motion tokens
- **No backward dependency:** Only depends on tokens

### AC → Implementation Mapping

| AC | Implementation | Test Method |
|----|----|---|
| AC1 | 4 variant branches in component (if-else on variant prop) | Visual test, unit test variant rendering |
| AC2 | 3 size branches (if-else on size prop, different dimensions) | Measure rendered heights; unit test |
| AC3 | on_clicked callback in root element | Unit test callback fires on click |
| AC4 | Keyboard event handlers (pressed-enter, pressed-space) | Keyboard test, e2e test |
| AC5 | MOTION_DURATION_REDUCED() helper for spinner animation | Screenshot comparison |
| AC6 | Root Rectangle pointer-events condition on is_disabled | Click test when disabled |
| AC7 | Conditional rendering based on is_loading (show spinner or label) | State toggle test |
| AC8 | accessible-label and accessible-role properties | NVDA test |

---

## 🏗️ Architecture & Compliance

### File Structure
```
src/frontend/
├── components/
│   ├── button.slint              ← This story creates this
│   ├── typography_test.slint     ← From US-001 (reference for patterns)
│   └── ... (other components)
├── design/
│   └── tokens.slint              ← From US-001 (uses these tokens)
└── ui.slint
```

### Component Pattern (Reference)
```slint
// src/frontend/components/button.slint
import { FLUENT_BLUE, ERROR, ... } from "../design/tokens.slint";

export component Button {
    // Props
    in property <string> label;
    in property <function()> on_clicked;
    in property <string> variant: "primary";  // default
    in property <string> size: "medium";      // default
    in property <bool> is_disabled: false;
    in property <bool> is_loading: false;
    in property <bool> reduce_motion: false;
    
    // Internal state
    private property <bool> hovered: false;
    private property <bool> pressed: false;
    
    // Layout
    Rectangle {
        // Color based on variant + state
        background: get_background_color(variant, hovered, pressed, is_disabled);
        
        // Size based on size prop
        height: get_height(size);
        
        // Disable pointer events when disabled
        pointer-events: is_disabled ? none : auto;
        
        // Keyboard handlers
        key-pressed(event) => {
            if (event.text == " " || event.text == Key.Return) {
                on_clicked();
            }
        }
        
        // Content: label or spinner
        if is_loading {
            LoadingSpinner { ... }  // Will use MOTION_DURATION_REDUCED()
        } else {
            Text {
                text: label;
            }
        }
    }
}

function get_background_color(variant, hovered, pressed, disabled) -> color {
    if (disabled) return NEUTRAL_LIGHT;
    if (pressed) return get_active_color(variant);
    if (hovered) return get_hover_color(variant);
    return get_base_color(variant);
}
```

### Naming Conventions
- Component name: `Button` (PascalCase)
- Props: `is_disabled`, `is_loading` (snake_case with `is_` prefix for booleans)
- Functions: `get_background_color()` (snake_case)
- Color references: Use token names (FLUENT_BLUE, not rgb values)

---

## 🔨 Tasks & Subtasks

### Task 1: Define Button Component Structure (AC1, AC2)
- [ ] Create `/src/frontend/components/button.slint`
- [ ] Define all props (label, variant, size, is_disabled, is_loading, reduce_motion)
- [ ] Implement variant logic (if-else branches for 4 variants)
- [ ] Implement size logic (if-else branches for 3 sizes)
- [ ] Define color functions (get_base_color, get_hover_color, get_active_color, get_disabled_color)
- [ ] Test: Verify component compiles without errors

### Task 2: Implement Click Handling (AC3)
- [ ] Add click event handler to root Rectangle
- [ ] Implement on_clicked callback invocation
- [ ] Test: Unit test verifies callback fires on click

### Task 3: Implement Keyboard Accessibility (AC4)
- [ ] Add keyboard event handlers (Enter key, Space key)
- [ ] Route keyboard events to on_clicked callback
- [ ] Add focus visuals (outline on focus)
- [ ] Test: Keyboard navigation test (Tab, Enter, Space)

### Task 4: Implement Loading State (AC7)
- [ ] Add conditional rendering (is_loading ? spinner : label)
- [ ] Create simple spinner (rotating element)
- [ ] Test: State toggle test (show/hide label and spinner)

### Task 5: Implement Motion Preference Support (AC5)
- [ ] Import MOTION_DURATION_REDUCED from tokens
- [ ] Use in spinner animation: animate rotation with MOTION_DURATION_REDUCED(DURATION_SLOW)
- [ ] When reduce_motion=true: animation duration → 0ms (static)
- [ ] When reduce_motion=false: animation duration → 400ms (rotating)
- [ ] Test: Screenshot comparison reduce_motion true vs false

### Task 6: Implement Screen Reader Support (AC8)
- [ ] Add accessible-label property
- [ ] Add accessible-role: "button"
- [ ] Update label text based on state (add "[Loading...]" suffix if loading)
- [ ] Test: NVDA test (announces "Button: [label], [state]")

### Task 7: Create Unit Tests
- [ ] Create `/tests/integration/button_test.rs` (Slint components are tested via integration tests)
- [ ] test_button_renders_primary_variant
- [ ] test_button_renders_all_sizes
- [ ] test_on_clicked_fires
- [ ] test_keyboard_enter_activates
- [ ] test_keyboard_space_activates
- [ ] test_is_disabled_prevents_clicks
- [ ] test_reduce_motion_disables_animation
- [ ] test_loading_state_shows_spinner
- [ ] Test: All tests passing

### Task 8: Create Component Documentation
- [ ] Create `/docs/BUTTON_COMPONENT_REFERENCE.md`
- [ ] Document all props with examples
- [ ] Document all variants and their use cases
- [ ] Document keyboard behavior
- [ ] Document accessibility features
- [ ] Provide code examples for each variant
- [ ] Test: Documentation complete and linked from main docs

---

## 📊 Definition of Done Checklist

- [ ] **AC1 - Variants:** All 4 variants render correctly with proper colors
- [ ] **AC2 - Sizes:** All 3 sizes render correctly with proper dimensions
- [ ] **AC3 - Click:** on_clicked callback fires when clicked
- [ ] **AC4 - Keyboard:** Tab/Enter/Space work correctly
- [ ] **AC5 - Motion:** Loading animation respects reduce_motion preference
- [ ] **AC6 - Disabled:** is_disabled=true prevents all interaction
- [ ] **AC7 - Loading:** is_loading shows spinner, hides label
- [ ] **AC8 - A11y:** Screen reader announces button with state
- [ ] **Unit Tests:** 8+ tests created and 100% passing
- [ ] **Integration Tests:** Component integrates with parent components
- [ ] **Accessibility:** NVDA test passes, keyboard navigation works
- [ ] **Code Review:** Winston approves (Slint conventions, accessibility)
- [ ] **Design Review:** Sally approves (matches UX spec colors/sizes)
- [ ] **Documentation:** Reference guide complete with examples
- [ ] **Performance:** Component renders < 16ms
- [ ] **Zero Warnings:** No build warnings or clippy issues
- [ ] **PR Merged:** Code merged to main branch

---

## 🧪 Testing Strategy

### Unit Tests
```rust
#[test]
fn test_button_renders_primary_variant() {
    // Verify primary variant renders with FLUENT_BLUE background
}

#[test]
fn test_button_renders_all_sizes() {
    // Verify small (28px), medium (36px), large (44px) heights
}

#[test]
fn test_on_clicked_fires() {
    // Click button → verify callback invoked
}

#[test]
fn test_keyboard_enter_activates() {
    // Focus button, press Enter → callback fires
}

#[test]
fn test_keyboard_space_activates() {
    // Focus button, press Space → callback fires
}

#[test]
fn test_is_disabled_prevents_clicks() {
    // is_disabled=true → clicks ignored
}

#[test]
fn test_reduce_motion_disables_animation() {
    // reduce_motion=true → spinner doesn't rotate
}

#[test]
fn test_loading_state_shows_spinner() {
    // is_loading=true → spinner visible, label hidden
}
```

### Integration Tests
- Button in MessageInput component
- Button in Dialog footer
- Button with different text lengths
- Button with icons (future)

### Accessibility Tests
- NVDA screen reader test
- Keyboard navigation (Tab, Enter, Space)
- Color contrast verification
- Focus visible test

### Visual Tests
- Variants comparison (4 × 3 = 12 screenshots)
- State comparison (normal, hover, active, disabled, loading)
- Motion preference comparison

---

## 📈 Estimation

**Size:** M (3-5 days)  
**Complexity:** Medium (multiple variants, accessibility)  
**Risk:** Low (pattern established from US-001 tokens)  
**Time Breakdown:**
- Component structure & variants: 6 hours
- Click & keyboard handling: 4 hours
- Loading state: 3 hours
- Motion preferences: 2 hours
- Accessibility: 3 hours
- Testing: 5 hours
- Documentation: 2 hours
- **Total: 25-30 hours**

---

## 🔗 Dependencies & Relationships

### Blocks (These stories depend on this)
- US-010: MessageInput Container (needs Button for send button)
- US-011: ConversationHeader Container (needs Button for settings)
- US-012: Presence Sync Real-time (needs Button for controls)
- US-014: MessageList Container (needs Button for actions)
- US-015: Real-time Message Arrival (needs Button feedback)

### Blocked By
- **US-001:** Design Tokens ✅ (COMPLETE - unblocks US-002)

### Related Stories
- US-003: TextField Component (uses Button for form submissions)
- US-004: Icon Component (used with Button for icon buttons)

---

## 💾 File References

### Source Files to Create
- **Create:** `/src/frontend/components/button.slint` (new file)
- **Create:** `/tests/integration/button_test.rs` (new file)
- **Create:** `/docs/BUTTON_COMPONENT_REFERENCE.md` (new file)
- **Reference:** `/src/frontend/design/tokens.slint` (uses tokens from US-001)
- **Reference:** `/docs/ux-design-specification.md` Section 6.1 (Button spec)

### Reference Documents
- **UX Spec:** `/docs/ux-design-specification.md` Section 6.1 (Button styling)
- **Component Standard:** `/docs/COMPONENT_API_STANDARD.md`
- **Design Tokens:** `/docs/DESIGN_TOKENS_REFERENCE.md`
- **Week 1 Defs:** `/docs/WEEK1_COMPONENT_DEFINITIONS.md`

---

## 🎬 Next Steps After Completion

1. ✅ **This story complete** → Merge PR to main
2. ⏭️ **Next story:** US-003 (TextField Component) - unblocked
3. 📋 **Update sprint status:** Change `us-002-button-component` from `ready-for-dev` → `in-progress`
4. 🔄 **Developer workflow:** Dev agent runs `*dev-story` with this story context

---

## 🏷️ Labels & Metadata

- **Epic:** Week 1 - Design Tokens & Base Components
- **Type:** Component / Base UI
- **Priority:** P0 (MVP Critical)
- **Complexity:** Medium
- **Risk:** Low
- **Owner:** Amelia
- **Tech Stack:** Slint, Rust 1.75+
- **Story Points:** 5 (M = 3-5 days)

---

**Document Version:** 1.0  
**Last Updated:** 2025-12-16  
**Status:** ✅ Ready for Development

