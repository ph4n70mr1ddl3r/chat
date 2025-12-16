# Week 1 Component Definitions

**Version:** 1.0  
**Date:** December 16, 2025  
**Week:** Week 1 (Design Tokens & Base Components)  
**Status:** Ready for Implementation  
**Related:** `COMPONENT_API_STANDARD.md`

---

This document defines the 5 base components that Amelia will implement in Week 1. Each component follows the Component API Standard and is ready for development.

---

## Component 1: Button

**Story:** US-002 (Base Components - Buttons)

### Purpose

Primary interactive element for all clickable actions: sending messages, confirming dialogs, navigation, etc. Must be keyboard accessible and support multiple variants.

### Category

Base Component

### Input Props

| Prop | Type | Default | Required | Category | Description |
|------|------|---------|----------|----------|-------------|
| label | string | - | ✓ | Data | Button text label |
| on_clicked | function() | - | ✓ | Behavior | Triggered when user clicks or presses Enter/Space |
| variant | string | "primary" | ✗ | Style | One of: "primary", "secondary", "tertiary", "danger" |
| size | string | "medium" | ✗ | Style | One of: "small", "medium", "large" |
| is_disabled | bool | false | ✗ | Style | If true, button is non-interactive and grayed out |
| is_loading | bool | false | ✗ | Style | If true, show spinner instead of label (for async actions) |
| reduce_motion | bool | false | ✗ | Style | Respect user motion preferences (disable spinner animation) |

### Callbacks

| Callback | Args | When Triggered |
|----------|------|-----------------|
| on_clicked | none | User clicks button or presses Enter/Space while focused |

### Visual Variants

#### variant prop

- **primary** (default): Fluent Blue background, white text, primary call-to-action
  - Hover: Darker blue (#0063B1)
  - Active: Even darker blue (#004A94)
  - Disabled: Light gray background, light gray text

- **secondary**: White background, blue text, outline border, secondary actions
  - Border: 1px Fluent Blue
  - Hover: Light blue background (#EFF6FC)
  - Active: Blue background (#F3F9FE)
  - Disabled: Light gray text, no border

- **tertiary**: Transparent background, blue text, minimal style
  - Hover: Light blue background
  - Active: Blue background
  - Disabled: Light gray text

- **danger**: Red background (#A4373A), white text, destructive actions (delete, logout)
  - Hover: Darker red (#8B2E31)
  - Active: Even darker red (#6B2327)
  - Disabled: Light gray

#### size prop

- **small**: 28px height, 12px font, 8px padding left/right, for compact UIs (inline actions)
- **medium** (default): 36px height, 14px font, 12px padding left/right, standard buttons
- **large**: 44px height, 16px font, 16px padding left/right, primary CTAs, mobile-friendly

#### is_disabled prop

- **false** (default): Interactive, responds to clicks, mouse pointer
- **true**: Grayed out, cursor: not-allowed, clicks ignored, no hover effects

#### is_loading prop

- **false** (default): Show label text normally
- **true**: Hide label, show spinner icon (24px, rotating)
  - If reduce_motion=true: static spinner icon (no rotation)
  - Disabled interaction during loading

### Accessibility

- **Role:** button
- **Label:** The button label (e.g., "Send Message", "Cancel")
- **Keyboard:** 
  - Tab to focus
  - Enter to activate (same as click)
  - Space to activate (same as click)
- **Motion:** When is_loading=true and reduce_motion=true, spinner is static (no rotation animation)
- **Contrast:** WCAG AA (4.5:1 contrast ratio for all variants)

### States Over Time

1. **Idle:** Button displays label, no interaction
2. **Hovered:** User moves mouse over, background color changes
3. **Focused:** User tabs to button, focus ring appears (2px outline, offset 2px)
4. **Active/Pressed:** User clicks or presses Enter/Space
5. **Loading** (if is_loading=true): Spinner replaces label, no further clicks accepted
6. **Disabled:** Button grayed out, no hover, clicks ignored

### Example Usage

```slint
// Primary button
Button {
    label: "Send Message";
    on_clicked() => {
        app.dispatch(Action::SendMessage);
    }
}

// Danger button, disabled
Button {
    label: "Delete";
    variant: "danger";
    is_disabled: true;
    on_clicked() => {
        app.dispatch(Action::DeleteConversation);
    }
}

// Loading button (async action)
Button {
    label: "Sending...";
    is_loading: app.is_sending_message;
    on_clicked() => {
        // Only called if not already loading
        app.dispatch(Action::SendMessage);
    }
}
```

### Testing

- **Unit tests:**
  - Button renders with correct label
  - on_clicked fires when clicked
  - on_clicked fires when Enter key pressed
  - on_clicked fires when Space key pressed
  - is_disabled prevents clicks
  - is_loading shows spinner, hides label
  - is_disabled + is_loading: spinner is static (not rotating) if reduce_motion=true
  - All variants render with correct colors
  - All sizes render with correct dimensions

- **Integration tests:**
  - Button inside a form dispatches form action
  - Multiple buttons in same container focus correctly (Tab navigation)

- **Accessibility tests:**
  - Screen reader announces "Button: [label]"
  - Tab key navigates to button
  - Enter activates button
  - Space activates button
  - Focus ring is visible
  - Color contrast ≥ 4.5:1 for all variants

- **Visual tests:**
  - Compare all variant + size combinations
  - Compare reduce_motion=true vs false (loading state)

### Related Components

- **Uses:** Icon (for loading spinner)
- **Used by:** MessageInput (send button), ConversationHeader (action buttons), Dialogs
- **Composes with:** Text (for label)

### Known Limitations

- Button does not support icons + text (use Icon separately if needed)
- Button does not support multi-line labels (truncates or wraps)
- Button width is automatic based on label; use parent container to control width

---

## Component 2: TextField

**Story:** US-003 (Base Components - Text Input)

### Purpose

Single-line text input for user-generated content: message composition, search, login credentials. Must support validation, error states, and placeholder text.

### Category

Base Component (Input)

### Input Props

| Prop | Type | Default | Required | Category | Description |
|------|------|---------|----------|----------|-------------|
| value | string | "" | ✓ | Data | Current text content |
| placeholder | string | - | ✗ | Data | Placeholder text when empty |
| on_text_changed | function(string) | - | ✓ | Behavior | Called when user types (fires on every character) |
| on_return_pressed | function() | - | ✗ | Behavior | Called when user presses Enter key |
| on_focus | function() | - | ✗ | Behavior | Called when input gains focus |
| on_blur | function() | - | ✗ | Behavior | Called when input loses focus |
| is_disabled | bool | false | ✗ | Style | If true, input is non-interactive |
| has_error | bool | false | ✗ | Style | If true, show error state (red border, error icon) |
| error_message | string | - | ✗ | Data | Error text displayed below input |
| reduce_motion | bool | false | ✗ | Style | Respect motion preferences |

### Callbacks

| Callback | Args | When Triggered |
|----------|------|-----------------|
| on_text_changed | new_text: string | Every time user modifies text (after each keystroke) |
| on_return_pressed | none | User presses Enter key |
| on_focus | none | TextField gains keyboard focus |
| on_blur | none | TextField loses keyboard focus |

### Visual Variants

#### Focus States

- **Unfocused:** 1px gray border (#D0D0D0), light gray background (#F7F7F7)
- **Focused:** 2px blue border (#0078D4), white background, focus ring
- **Disabled:** Light gray border, very light gray background (#F0F0F0), gray text

#### Error State (has_error=true)

- **Border:** 2px red (#C50F1F)
- **Text Color:** Red (#C50F1F)
- **Error Icon:** Red X icon appears to the right of text
- **Error Message:** Red text below input showing error_message

#### Placeholder

- **Color:** Light gray (#A8A8A8)
- **Font Weight:** Regular
- **Visible when:** value is empty AND input not focused

### Accessibility

- **Role:** textbox
- **Label:** Should be associated with a label element or aria-label
- **Keyboard:** 
  - Tab to focus/blur
  - All characters typeable
  - Backspace to delete
  - Enter to submit (if on_return_pressed provided)
  - Arrow keys to navigate text
- **Motion:** No motion/animation involved
- **Contrast:** 4.5:1 for text, 3:1 for borders

### States Over Time

1. **Empty & Unfocused:** Placeholder visible, border gray
2. **Empty & Focused:** Placeholder still visible, border blue, cursor blinking
3. **User Types:** Placeholder disappears, text appears, on_text_changed called
4. **User Focuses Away:** on_blur called, border returns to gray
5. **Error State:** If has_error=true, border red, error_message shown
6. **Disabled:** Background very light gray, no cursor, no interaction

### Example Usage

```slint
// Message composition input
TextField {
    value: app.current_message;
    placeholder: "Type a message...";
    on_text_changed(text) => {
        app.dispatch(Action::UpdateMessageText { text: text });
    }
    on_return_pressed() => {
        app.dispatch(Action::SendMessage);
    }
}

// Search input with validation
TextField {
    value: app.search_query;
    placeholder: "Search conversations...";
    has_error: app.search_error != "";
    error_message: app.search_error;
    on_text_changed(text) => {
        app.dispatch(Action::UpdateSearchQuery { text: text });
    }
}

// Disabled input
TextField {
    value: "You cannot edit this";
    is_disabled: true;
}
```

### Testing

- **Unit tests:**
  - Renders with correct placeholder when empty
  - on_text_changed fires with new text after each keystroke
  - on_return_pressed fires when Enter key pressed
  - on_focus fires when input focused
  - on_blur fires when input unfocused
  - is_disabled prevents interaction
  - has_error shows red border and error_message
  - Value updates correctly

- **Integration tests:**
  - TextField in a form submits on Enter
  - Typing in TextField updates parent state and re-renders
  - Error state shows and clears correctly

- **Accessibility tests:**
  - Screen reader announces role and value
  - Tab key navigates to/from input
  - All characters typeable
  - Enter key works as expected
  - Focus ring visible

- **Visual tests:**
  - Empty vs filled state
  - Focus vs unfocused state
  - Error state displays correctly
  - Placeholder styling correct

### Related Components

- **Uses:** Text (for placeholder, error message)
- **Used by:** MessageInput (main composition field), SearchBar, LoginForm
- **Composes with:** Error message display, optional icons

### Known Limitations

- Single-line only (no multi-line textarea)
- No built-in validation (parent component handles)
- No character counter (parent adds if needed)
- No input masking (e.g., phone numbers)

---

## Component 3: Icon

**Story:** US-004 (Base Components - Icons)

### Purpose

Wrapper for SVG icons used throughout the app: checkmarks, spinners, profile pictures, status indicators. Consistent sizing, colors, and accessibility.

### Category

Base Component

### Input Props

| Prop | Type | Default | Required | Category | Description |
|------|------|---------|----------|----------|-------------|
| name | string | - | ✓ | Data | Icon name/path (e.g., "checkmark", "spinner", "user-profile") |
| size | string | "medium" | ✗ | Style | One of: "small" (16px), "medium" (24px), "large" (32px), "xlarge" (48px) |
| color | string | "currentColor" | ✗ | Style | Color hex code or color name (e.g., "#0078D4", "blue", "red") |
| alt_text | string | - | ✗ | Data | Alt text for screen readers (required for semantic icons) |
| reduce_motion | bool | false | ✗ | Style | If true, disable icon animations (e.g., spinner rotation) |

### Callbacks

None. Icon is purely presentational.

### Visual Variants

#### name prop (Icon Library)

Week 1 icons needed:
- **checkmark**: Single checkmark icon (message delivery)
- **checkmark-double**: Double checkmark (message delivered to all)
- **spinner**: Animated loading spinner (loading states)
- **close**: X icon for close/delete
- **send**: Arrow/paper plane for send message
- **settings**: Gear icon for settings
- **search**: Magnifying glass for search
- **user-profile**: Circle with person silhouette
- **online-dot**: Small circle for presence (online)
- **offline-dot**: Grayed dot for presence (offline)

#### size prop

- **small**: 16px × 16px, for inline icons or compact UIs
- **medium** (default): 24px × 24px, standard size
- **large**: 32px × 32px, for prominent icons
- **xlarge**: 48px × 48px, for hero images/avatars

#### color prop

- **currentColor** (default): Inherits text color from parent
- **blue**: Fluent Blue (#0078D4) for primary actions
- **green**: Fluent Green (#107C10) for success/online
- **red**: Fluent Red (#C50F1F) for errors/danger
- **gray**: Light gray (#A8A8A8) for disabled/secondary
- Custom hex: Any valid color code

#### reduce_motion prop (for animated icons)

- **false** (default): Animations play normally
  - Spinner: 400ms rotation loop (linear)
  - Others: Static (no animation)
  
- **true**: Animations disabled
  - Spinner: Static image (no rotation)
  - Others: Same (no animation)

### Accessibility

- **Role:** If alt_text provided: "img", otherwise "presentation"
- **Label:** alt_text (e.g., "Message delivered", "Loading messages")
- **Keyboard:** None (icon is non-interactive)
- **Motion:** Respects reduce_motion for animations
- **Contrast:** Inherits color (parent ensures contrast)

### States Over Time

1. **Static Icon:** Renders and does not change
2. **Animated Icon (spinner):** Rotates continuously (if reduce_motion=false)
3. **Color Change:** Icon color changes based on parent state (parent prop changes color)

### Example Usage

```slint
// Delivery status icons
Icon {
    name: "checkmark";
    size: "small";
    color: "green";
    alt_text: "Message sent";
}

Icon {
    name: "checkmark-double";
    size: "small";
    color: "green";
    alt_text: "Message delivered";
}

// Loading spinner
Icon {
    name: "spinner";
    size: "medium";
    reduce_motion: app.reduce_motion;
    alt_text: "Loading messages";
}

// Presence indicator
Icon {
    name: "online-dot";
    size: "small";
    color: "green";
    alt_text: "User online";
}
```

### Testing

- **Unit tests:**
  - Icon renders with correct name/file
  - Icon renders with correct size (16/24/32/48px)
  - Icon renders with correct color
  - Animated icons (spinner) rotate when reduce_motion=false
  - Animated icons are static when reduce_motion=true
  - Icon is non-interactive (no on_click)

- **Integration tests:**
  - Icon used in Button renders correctly
  - Icon used in MessageBubble renders correctly
  - Icon color changes when parent prop changes

- **Accessibility tests:**
  - Screen reader announces alt_text if provided
  - Icon is marked as "presentation" if no alt_text
  - No focus/keyboard interaction needed

- **Visual tests:**
  - Compare all icon names at all sizes
  - Compare all colors
  - Compare reduce_motion=true vs false (for spinner)
  - Compare on different backgrounds (light, dark, high contrast)

### Related Components

- **Uses:** SVG assets from `/assets/icons/`
- **Used by:** Button (loading spinner), MessageBubble (delivery status), ConversationHeader (settings icon), MessageInput (send button)
- **Composes with:** Text (for labels with icons), Buttons

### Known Limitations

- Icon names must exist in `/assets/icons/` directory
- No fallback for missing icons (error if name doesn't exist)
- Icon size is fixed (no custom pixel sizes)
- No hover effects built-in (parent adds if needed)
- No badge/notification count overlay

---

## Component 4: Chip

**Story:** US-005 (Base Components - Tags/Labels)

### Purpose

Compact, dismissible labels for tags, user mentions, or status badges. Used in conversation lists, search results, and message metadata.

### Category

Base Component

### Input Props

| Prop | Type | Default | Required | Category | Description |
|------|------|---------|----------|----------|-------------|
| label | string | - | ✓ | Data | Text content of the chip |
| on_clicked | function() | ✗ | Behavior | Called when chip is clicked (optional) |
| on_dismissed | function() | ✗ | Behavior | Called when user clicks X button (optional; requires is_dismissible=true) |
| variant | string | "default" | ✗ | Style | One of: "default", "primary", "success", "warning", "error" |
| is_dismissible | bool | false | ✗ | Style | If true, show X button to dismiss |
| is_disabled | bool | false | ✗ | Style | If true, chip is grayed out and non-interactive |
| reduce_motion | bool | false | ✗ | Style | Respect motion preferences |

### Callbacks

| Callback | Args | When Triggered |
|----------|------|-----------------|
| on_clicked | none | User clicks chip (if on_clicked provided) |
| on_dismissed | none | User clicks X button (if is_dismissible=true and on_dismissed provided) |

### Visual Variants

#### variant prop

- **default**: Gray background (#E8E8E8), dark gray text, for neutral labels
- **primary**: Light blue background (#EFF6FC), blue text (#0078D4), for primary labels
- **success**: Light green background (#E1F0DD), green text (#107C10), for success/online status
- **warning**: Light orange background (#FCE8E1), orange text (#C9824B), for warnings
- **error**: Light red background (#FDE7E9), red text (#C50F1F), for errors/offline status

#### is_dismissible prop

- **false** (default): No X button, chip cannot be dismissed
- **true**: X button appears on the right side of chip, calls on_dismissed when clicked

#### size (implicit)

- **Height:** 28px (fixed, compact)
- **Padding:** 8px left/right, 4px top/bottom
- **Font:** 12px, regular weight
- **Border Radius:** 14px (full rounded)

### Accessibility

- **Role:** If on_clicked: "button", otherwise "status"
- **Label:** label prop text
- **Keyboard:** 
  - Tab to focus (if clickable)
  - Enter to activate click (if clickable)
  - If dismissible: Tab to X button, Enter to dismiss
- **Motion:** No motion involved
- **Contrast:** 4.5:1 for text/background

### States Over Time

1. **Display:** Chip renders with label and optional X button
2. **Hovered:** Background slightly darker (if clickable)
3. **Focused:** Focus ring appears (if clickable)
4. **Clicked:** on_clicked fires (if callback provided)
5. **Dismissed:** on_dismissed fires and chip is removed by parent

### Example Usage

```slint
// User mention chip
Chip {
    label: "@Alice";
    variant: "primary";
    on_clicked() => {
        app.dispatch(Action::ShowUserProfile { user_id: "alice" });
    }
}

// Status badge (dismissible)
Chip {
    label: "Unread";
    variant: "warning";
    is_dismissible: true;
    on_dismissed() => {
        app.dispatch(Action::MarkConversationRead);
    }
}

// Presence indicator (non-interactive)
Chip {
    label: "Online";
    variant: "success";
}

// Disabled tag
Chip {
    label: "Archived";
    variant: "default";
    is_disabled: true;
}
```

### Testing

- **Unit tests:**
  - Chip renders with correct label
  - Chip renders with correct variant color
  - on_clicked fires when clicked (if provided)
  - on_dismissed fires when X clicked (if is_dismissible=true)
  - is_disabled prevents interaction
  - X button only shows if is_dismissible=true
  - All variants render with correct colors

- **Integration tests:**
  - Chip in a list of chips
  - Dismissing chip updates parent state
  - Clicking chip navigates or performs action

- **Accessibility tests:**
  - Screen reader announces "Button: [label]" if clickable
  - Screen reader announces "Status: [label]" if non-interactive
  - Tab key navigates to chip if clickable
  - Tab key navigates to X button if dismissible
  - Enter activates click and dismiss

- **Visual tests:**
  - All variant colors
  - With and without X button
  - Disabled vs enabled
  - Different label lengths

### Related Components

- **Uses:** Text (for label), Icon (optional close button)
- **Used by:** MessageBubble (for @mentions), ConversationHeader (for tags), SearchResults
- **Composes with:** Lists of chips

### Known Limitations

- Fixed height (28px), no size variants
- No icon support (label-only)
- No badge/count overlay
- No drag-and-drop reordering

---

## Component 5: LoadingSpinner

**Story:** US-006 (Loading States & Feedback)

### Purpose

Animated spinner icon indicating an in-progress asynchronous operation. Used when loading messages, conversations, or waiting for server responses. Must respect motion preferences.

### Category

Feedback Component

### Input Props

| Prop | Type | Default | Required | Category | Description |
|------|------|---------|----------|----------|-------------|
| size | string | "medium" | ✗ | Style | One of: "small" (20px), "medium" (24px), "large" (32px) |
| color | string | "currentColor" | ✗ | Style | Spinner color (hex or color name) |
| reduce_motion | bool | false | ✗ | Style | If true, show static icon instead of animation |
| message | string | - | ✗ | Data | Optional loading message below spinner (e.g., "Loading messages...") |

### Callbacks

None. LoadingSpinner is purely presentational.

### Visual Variants

#### size prop

- **small**: 20px × 20px spinner, for inline/compact loading
- **medium** (default): 24px × 24px spinner, standard size
- **large**: 32px × 32px spinner, prominent loading screens

#### color prop

- **currentColor** (default): Inherits text color from parent
- **blue**: Fluent Blue (#0078D4)
- **gray**: Light gray (#A8A8A8) for secondary loaders
- Custom hex: Any valid color code

#### Animation

- **reduce_motion = false** (default):
  - Spinner rotates 360° continuously
  - Duration: 400ms
  - Easing: linear
  - Repeat: infinite

- **reduce_motion = true**:
  - Static spinner image (frozen at 0° rotation)
  - No animation

#### message prop

- **Not provided:** Spinner only (no text)
- **Provided:** Spinner above message text
  - Text: 12px, light gray (#A8A8A8), centered below spinner
  - Gap between spinner and text: 8px

### Accessibility

- **Role:** status
- **Label:** message prop (e.g., "Loading messages...")
- **Keyboard:** None (non-interactive)
- **Motion:** Respects reduce_motion
- **Aria-live:** "polite" (announces to screen readers)

### States Over Time

1. **Initial:** Spinner appears and begins rotation (if reduce_motion=false)
2. **Loading:** Continuous rotation (animated)
3. **Complete:** Parent removes spinner component

### Example Usage

```slint
// Loading spinner without message
LoadingSpinner {
    size: "medium";
    reduce_motion: app.reduce_motion;
}

// Loading spinner with message
LoadingSpinner {
    size: "large";
    message: "Loading messages...";
    color: "blue";
    reduce_motion: app.reduce_motion;
}

// Small inline spinner
LoadingSpinner {
    size: "small";
    reduce_motion: app.reduce_motion;
}
```

### Testing

- **Unit tests:**
  - Spinner renders with correct size (20/24/32px)
  - Spinner renders with correct color
  - Message displays below spinner if provided
  - Spinner rotates when reduce_motion=false
  - Spinner is static when reduce_motion=true
  - aria-live="polite" set on component

- **Integration tests:**
  - Spinner replaces content when loading state active
  - Spinner is removed when loading completes
  - Message text is readable

- **Accessibility tests:**
  - Screen reader announces "Loading messages..." (or provided message)
  - aria-live announcement works
  - Spinner is marked as status region

- **Visual tests:**
  - All sizes
  - All colors
  - Compare reduce_motion=true vs false (rotation)
  - Message below spinner is readable

### Related Components

- **Uses:** Icon component (for spinner SVG)
- **Used by:** MessageList (loading more messages), ConversationView (initial load), SearchResults
- **Composes with:** Message text, container layouts

### Known Limitations

- Only circular spinner style (no bar progress)
- No percentage/completion indication
- Cannot be dismissed by user (parent removes it)
- No custom duration/easing

---

## Implementation Timeline

### Pre-Week-1 (This Week)

1. ✅ Complete all 5 component definitions (this document)
2. ⏳ Amelia reviews definitions and asks questions
3. ⏳ Create Slint component shells with proper structure
4. ⏳ Set up component testing framework

### Week 1 (Dec 18-22)

**Stories US-002 through US-006 (5 components)**

- **Day 1-2:** Implement Button + Icon components
- **Day 2-3:** Implement TextField component
- **Day 3:** Implement Chip component
- **Day 4:** Implement LoadingSpinner component
- **Day 5:** Complete all unit tests + accessibility tests

**Definition of Done for each component:**
- ✅ Component renders correctly for all props/variants
- ✅ Accessibility properties set (role, label, keyboard)
- ✅ Respects reduce_motion if applicable
- ✅ Unit tests 100% passing
- ✅ Integration tests with parent components
- ✅ Accessibility testing (screen reader + keyboard)
- ✅ Code review + merged to main

### Week 2+

Components from Week 1 form the foundation for:
- MessageBubble (uses Icon, Button)
- ConversationItem (uses Chip, Icon)
- MessageList (uses LoadingSpinner)
- All other components build on this base

---

## Notes for Amelia

1. **Start with Button** - It's used everywhere, and once you have it working, other components are easier
2. **Icon is the wild card** - Make sure the icon library is locked down before starting; you'll reference it constantly
3. **TextField is tricky** - Text input events in Slint need careful handling; allocate extra time for testing
4. **Chip is simple** - Consider it a confidence builder after the others
5. **LoadingSpinner ties it together** - By the time you reach it, the pattern should be obvious

All definitions are based on:
- ✅ UX Specification Section 7 (Visual Design) + Section 8 (Component Strategy)
- ✅ WCAG 2.1 AA accessibility requirements
- ✅ Component API Standard (COMPONENT_API_STANDARD.md)

Ask questions early. Winston's architecture workshop happens before Week 1; leverage that to clarify state/event flow.

---

**Approved By:** Amelia (Developer), Sally (UX Designer), Winston (Architect)  
**Ready For:** Week 1 Implementation  
**Next Review:** Dec 23, 2025 (end of Week 1)
