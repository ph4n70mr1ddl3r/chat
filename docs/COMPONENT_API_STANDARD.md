# Component API Standard

**Version:** 1.0  
**Date:** December 16, 2025  
**Audience:** All developers building Slint components  
**Purpose:** Establish consistent component contracts across the codebase to prevent prop-drilling nightmares, state sync bugs, and maintainability issues.

---

## Overview

Every component in this codebase follows the same five architectural rules. This ensures:

- **Consistency** - Components look and feel the same from a developer's perspective
- **Composability** - Components fit together like LEGO blocks
- **Testability** - Every component tests the same way
- **Maintainability** - New developers understand patterns immediately
- **Performance** - Predictable re-rendering and optimization opportunities

---

## Rule 1: Input Props Structure (Maximum 12 Props)

Every component has exactly three categories of input properties:

### Category 1: Data Props (Structural)

These are the core data the component displays or operates on. They should be:
- Flattened (no nested structs passed directly; pass IDs instead)
- Immutable from the component's perspective
- Primitives or simple enums (strings, numbers, booleans)

**Example:**
```slint
export component MessageBubble {
    in property message_id: string;
    in property content: string;
    in property sender_id: string;
    in property sender_name: string;
    in property timestamp: string;
    in property delivery_status: string; // "pending" | "sent" | "delivered" | "failed"
}
```

**Rules:**
- Data props come first in the component definition
- No nested objects (use IDs to reference app state)
- Prop names: `snake_case`, max 20 characters
- If you need > 8 data props, refactor into sub-components

---

### Category 2: Behavior Props (Callbacks)

These are functions the component calls when user interactions happen. They should:
- Be named `on_<subject>_<verb>`
- Take only primitives (IDs, strings, numbers - NOT complex structs)
- Have clear, single responsibility
- Always be optional (component doesn't crash if not provided)

**Example:**
```slint
export component MessageBubble {
    in property on_message_selected: function(string); // takes message_id
    in property on_message_long_pressed: function(string);
    in property on_action_menu_opened: function(string);
    in property on_message_deleted: function(string);
}
```

**Rules:**
- Callback naming: `on_<subject>_<verb>` (NOT `handleMessageSelected` or `onSelect`)
- Callbacks never modify component state directly
- Callbacks pass arguments that identify WHAT happened, not HOW to respond
- Maximum 4 callbacks per component (if you need more, you're designing wrong)

---

### Category 3: Style & Context Props (Optional)

These are feature flags and environment context. They should:
- Be boolean flags or enum-like strings
- Default to a sensible value
- Not require the component to know about global application state

**Example:**
```slint
export component MessageBubble {
    in property reduce_motion: bool;
    in property is_compact_mode: bool;
    in property high_contrast: bool;
}
```

**Rules:**
- Style props come last in the component definition
- All should have sensible defaults
- Maximum 3 style props per component (if more, use sub-components)
- These props control variants, not core functionality

---

### Total Props Limit: 12 Maximum

If you're defining more than 12 props, you're violating Rule 1. Refactor by:
- Breaking component into smaller sub-components
- Moving some props to parent component management
- Creating a "container" component that composes smaller pieces

---

## Rule 2: Internal State Management

Every component has exactly three types of internal state:

### Type 1: Local UI State

State that lives entirely within the component and never needs to be seen by the app or other components.

**Example:**
```slint
export component MessageBubble {
    private property is_hovered: bool;
    private property animation_progress: float;
    private property is_expanded: bool;
}
```

**Rules:**
- Only the component owns and modifies this state
- Never expose UI state as props
- Use for hover, animation, temporary expand/collapse, etc.
- Should NOT drive business logic

---

### Type 2: Application State (Read-Only References)

References to application-level state that the component reads but never modifies directly.

**Example:**
```slint
export component MessageBubble {
    in property current_user_id: string; // from app state, read-only
    in property is_user_admin: bool; // from app state, read-only
}
```

**Rules:**
- Component receives these as props
- Component NEVER mutates app state directly
- Component NEVER calls setState or app.dispatch from within prop change handlers
- Component only reads these values to make rendering decisions

---

### Type 3: Application State (Delegated Updates)

When component state needs to update app state, it does so by calling a callback. The app is responsible for updating state and passing new props back.

**Pattern (One-Way Data Flow):**

```
1. User clicks component
   ↓
2. Component calls on_something(id) callback
   ↓
3. App receives callback, dispatches action to state reducer
   ↓
4. App state changes
   ↓
5. New props flow back to component
   ↓
6. Component re-renders with new props
```

**Example:**

```slint
export component MessageBubble {
    in property is_selected: bool; // from app state
    in property on_message_selected: function(string);

    area := TouchArea {
        clicked => {
            if !is_selected {
                on_message_selected(message_id);
            }
        }
    }
}
```

**Rules:**
- NO two-way binding (prop → callback → prop change → callback cycle)
- Component never directly modifies is_selected
- Component calls on_message_selected()
- App updates app.state.selected_message_id
- App passes new is_selected prop
- Component re-renders

---

## Rule 3: Event Flow & Communication

Every user interaction follows this single unidirectional pattern:

```
User Action (tap/click/type/long-press)
    ↓
Component detects (TouchArea, TextInput, etc.)
    ↓
Component fires callback with minimal data (usually just IDs)
    ↓
App receives callback, interprets it, dispatches action to reducer
    ↓
Reducer updates app state
    ↓
App passes new props to component
    ↓
Component re-renders with new props
```

### No Bidirectional Communication

❌ **WRONG:**
```slint
export component MessageBubble {
    in-out property is_selected: bool; // NO! This enables two-way binding
}
```

✅ **RIGHT:**
```slint
export component MessageBubble {
    in property is_selected: bool; // Read from app
    in property on_message_selected: function(string); // Write to app via callback
}
```

### No Complex Structs in Callbacks

❌ **WRONG:**
```slint
on_message_updated(message: Message) => { ... } // Passing entire struct
```

✅ **RIGHT:**
```slint
on_message_selected(message_id: string) => { ... } // Passing ID only
```

Why? Because:
- Component doesn't need to know the full Message structure
- App handles the interpretation
- Easy to refactor Message schema later
- Clear contract: "When this happens, tell the app the ID"

---

## Rule 4: Variants & Visual States

Every component documents its visual states as prop variants:

### Enum-Like Props Create Variants

If a prop has enum-like values (a few fixed options), document every visual variant:

**Example: delivery_status prop**

```slint
export component MessageBubble {
    in property delivery_status: string; // "pending" | "sent" | "delivered" | "failed"
}

// Variants:
// - pending: gray text, spinner icon, 400ms rotation (or static if reduce_motion=true)
// - sent: green checkmark icon
// - delivered: green double-checkmark icon  
// - failed: red X icon, red text, error icon
```

### Boolean Props Create Variants

If a boolean prop changes appearance, document both states:

**Example: reduce_motion prop**

```slint
// reduce_motion = false: animations enabled
//   - Spinner rotates (400ms, linear)
//   - Messages fade in (200ms, ease-out)
//   - Presence indicator pulses (1s, ease-in-out)

// reduce_motion = true: animations disabled
//   - Static spinner icon (no rotation)
//   - Messages appear instantly
//   - Presence indicator static
```

### Orthogonal Variants

Variants should be independent. If two props interact, document the combination:

**Example:**
```slint
// is_compact_mode = false: full layout
//   - Avatar + name + timestamp all visible
//   - May have multiple lines

// is_compact_mode = true: compact layout  
//   - No avatar, minimal name, no timestamp
//   - Single line when possible
//   - Works with all delivery_status values
```

---

## Rule 5: Accessibility is Built-In

Every component must implement these accessibility features:

### 5.1: Semantic Role

Every component must declare its semantic role for screen readers:

```slint
export component MessageBubble {
    accessible-role: "listitem";
}
```

**Common roles:**
- `button` - clickable interactive element
- `textinput` - text input field
- `listitem` - item in a list
- `heading` - section heading
- `image` - image with alt text
- `region` - generic container with purpose

---

### 5.2: Accessible Label

Every component must have an accessible-label that describes it for screen readers:

```slint
export component MessageBubble {
    accessible-label: "\{sender_name}: \{content}";
}
```

**Rules:**
- Label should be human-readable
- Should include the core information
- Should be unique or clearly associated with the component
- Should be updated dynamically if content changes

---

### 5.3: Keyboard Navigation

Every interactive component must be keyboard navigable:

```slint
export component MessageBubble {
    focus-able: true;

    key-pressed(event) => {
        if event.text == Key.Return {
            on_message_selected(message_id);
            accept;
        }
        if event.text == Key.Space {
            on_message_selected(message_id);
            accept;
        }
    }
}
```

**Rules:**
- Interactive components must be `focus-able: true`
- Must respond to Enter (primary action)
- Must respond to Space (secondary interaction)
- Must respond to Arrow keys for navigation (in lists)
- Use `accept` to consume the event

---

### 5.4: Motion Preferences (WCAG 2.3.3 & 2.3.4)

Every animated component must respect `reduce_motion`:

```slint
export component LoadingSpinner {
    in property reduce_motion: bool;

    @keyframes spin-animation {
        0% { rotation: 0deg; }
        100% { rotation: 360deg; }
    }

    Image {
        source: "spinner.svg";
        
        if !reduce_motion {
            animate rotation {
                duration: 400ms;
                easing: linear;
                repeat: infinite;
            }
        }
    }
}
```

**Rules:**
- Respect the `reduce_motion` prop
- No animations when `reduce_motion = true`
- No `@keyframes` / animation shortcuts when reduce_motion=true
- Test with `prefers-reduced-motion` media query equivalent
- Document what animates and what doesn't

---

### 5.5: High Contrast Support

Every visual component should support high contrast mode:

```slint
export component Button {
    in property high_contrast: bool;

    Rectangle {
        background: high_contrast ? black : white;
        border-color: high_contrast ? white : gray;
        border-width: high_contrast ? 2px : 1px;
    }
}
```

**Rules:**
- Support high contrast when available
- Use sufficient color contrast (WCAG AA: 4.5:1 for text)
- Don't rely solely on color to convey information
- Test with high contrast mode enabled

---

## Component Definition Template

Every component ships with a **Component Definition Document** following this structure:

```markdown
# Component: [ComponentName]

## Purpose
[One sentence: what is this component for?]

## Category
[Base | Layout | Input | Feedback | Navigation | Message]

## Input Props

| Prop | Type | Default | Required | Category | Description |
|------|------|---------|----------|----------|-------------|
| ... | ... | ... | ... | ... | ... |

## Callbacks

| Callback | Args | When Triggered |
|----------|------|-----------------|
| ... | ... | ... |

## Visual Variants

### [Prop Name]
- **value1**: [Description and visual appearance]
- **value2**: [Description and visual appearance]

## Accessibility

- **Role:** [semantic role]
- **Label:** [how it's labeled for screen readers]
- **Keyboard:** [which keys work and what they do]
- **Motion:** [what animates, what doesn't when reduce_motion=true]
- **Contrast:** [WCAG level supported]

## States Over Time

[Describe the component lifecycle and state transitions]

## Example Usage

\`\`\`slint
ComponentName {
    prop1: value1;
    prop2: value2;
    
    on_something(arg) => {
        app.dispatch(Action::DoSomething { arg: arg });
    }
}
\`\`\`

## Testing

- Unit test: [specific test scenarios]
- Integration test: [how it works with parent]
- Accessibility test: [screen reader, keyboard]
- Visual test: [compare variants]

## Related Components

- [Component1] (composes with this)
- [Component2] (parent of this)

## Known Limitations

[Any constraints or trade-offs]
```

---

## Common Violations & How to Fix Them

### Violation 1: Too Many Props

❌ **WRONG:**
```slint
export component ConversationItem {
    in property conversation_id: string;
    in property title: string;
    in property last_message: string;
    in property timestamp: string;
    in property sender_name: string;
    in property avatar_url: string;
    in property unread_count: int;
    in property is_pinned: bool;
    in property is_muted: bool;
    in property is_archived: bool;
    in property last_activity_time: string;
    in property has_reactions: bool;
    in property background_color: string;
}
```

**Problem:** 12 props is too many. This component is trying to do everything.

✅ **RIGHT:**
```slint
export component ConversationItem {
    // Data Props (5)
    in property conversation_id: string;
    in property title: string;
    in property last_message: string;
    in property unread_count: int;
    in property avatar_url: string;
    
    // Behavior Props (2)
    in property on_conversation_selected: function(string);
    in property on_long_pressed: function(string);
    
    // Style Props (1)
    in property is_compact_mode: bool;
}
```

**Then create sub-components for details:**
- `ConversationMeta` (handles timestamp, is_pinned, is_muted)
- `ConversationStatus` (handles unread_count, last_activity_time)

---

### Violation 2: Two-Way Binding

❌ **WRONG:**
```slint
export component TextField {
    in-out property value: string; // Two-way binding!
}

// Usage:
TextField {
    value: app.current_message;
}
// Now if user types, the component directly modifies app.current_message
// And if app changes app.current_message, component updates
// This creates hard-to-debug sync issues!
```

✅ **RIGHT:**
```slint
export component TextField {
    in property value: string;
    in property on_text_changed: function(string);
}

// Usage:
TextField {
    value: app.current_message;
    
    on_text_changed(new_text) => {
        app.dispatch(Action::UpdateMessage { text: new_text });
    }
}
```

---

### Violation 3: Complex Structs in Callbacks

❌ **WRONG:**
```slint
on_message_action(message: Message) => {
    app.update_message(message);
}
```

**Problem:** Component needs to know about Message struct. What if Message changes? Component breaks.

✅ **RIGHT:**
```slint
on_message_deleted(message_id: string) => {
    app.dispatch(Action::DeleteMessage { message_id: message_id });
}
```

---

### Violation 4: Component Modifying App State Directly

❌ **WRONG:**
```slint
export component MessageBubble {
    in property app_state: AppState;
    
    area := TouchArea {
        clicked => {
            app_state.selected_message_id = message_id; // NO! Direct mutation
        }
    }
}
```

✅ **RIGHT:**
```slint
export component MessageBubble {
    in property on_message_selected: function(string);
    
    area := TouchArea {
        clicked => {
            on_message_selected(message_id); // Delegate to app
        }
    }
}
```

---

### Violation 5: No Accessibility

❌ **WRONG:**
```slint
export component Button {
    Rectangle { }
    Text { text: "Click me"; }
}
```

**Problem:** Screen readers can't tell this is a button. Keyboard users can't interact.

✅ **RIGHT:**
```slint
export component Button {
    in property on_clicked: function;
    
    accessible-role: "button";
    accessible-label: "Click me";
    focus-able: true;

    key-pressed(event) => {
        if event.text == Key.Return || event.text == Key.Space {
            on_clicked();
            accept;
        }
    }

    Rectangle { }
    Text { text: "Click me"; }
}
```

---

## Enforcement & Code Review

### During Code Review, Check:

1. ✅ Does this component have ≤12 props?
2. ✅ Are props organized into Data / Behavior / Style categories?
3. ✅ Does it have a Component Definition Document?
4. ✅ Are callbacks named `on_<subject>_<verb>`?
5. ✅ Does it declare `accessible-role` and `accessible-label`?
6. ✅ Is it keyboard navigable (focus-able + key-pressed)?
7. ✅ Does it respect `reduce_motion`?
8. ✅ Are there unit tests for the component in isolation?
9. ✅ Are there integration tests with parent?
10. ✅ Are there accessibility tests?

### If Any Check Fails:

- Request changes before merging
- Point to this document
- Help developer refactor if needed

---

## FAQ

**Q: Can I use `in-out` properties?**  
A: No. Never. Always use one-way data flow with callbacks.

**Q: What if I need to pass a complex struct?**  
A: Pass the ID instead. Let the app look up the full struct in state.

**Q: What if component needs multiple callbacks?**  
A: Maximum 4. If you need more, break it into sub-components.

**Q: What about prop drilling?**  
A: Acceptable up to 2-3 levels. If deeper, consider a container component or state management solution.

**Q: Can components call other components' functions?**  
A: No. Always go through app state and callbacks. Keep components isolated.

**Q: What if I need to animate something and reduce_motion is true?**  
A: Use instant transitions instead. Position changes happen immediately, no duration.

**Q: Should every component have high_contrast prop?**  
A: Only if the component has visual styling. Base components (Button, Text) yes. Layout components probably not.

---

## Version History

- **1.0** (Dec 16, 2025) - Initial standard established for the private-chat project

---

**Approved By:** Sally (UX Designer), Amelia (Developer), Winston (Architect)  
**Effective Date:** Pre-Week-1 (before code starts)  
**Next Review:** After Week 1 (Dec 23, 2025)
