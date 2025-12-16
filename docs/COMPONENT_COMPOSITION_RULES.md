# Component Composition Rules

**Version:** 1.0  
**Date:** December 16, 2025  
**Purpose:** Guidelines for composing components together to build larger, more complex UIs while maintaining the API Standard  
**Related Documents:** `COMPONENT_API_STANDARD.md`, `WEEK1_COMPONENT_DEFINITIONS.md`

---

## Overview

Components don't exist in isolation. By Week 2+, we'll compose base components from Week 1 into larger UI structures:

```
Button + Icon           → MessageActionButton
TextField + Chip        → TagInput
Icon + Text             → StatusBadge
Button + Spinner        → AsyncButton
MessageBubble + List    → MessageList
ConversationItem + List → ConversationList
```

These composition patterns must follow clear rules to avoid spaghetti architecture.

---

## Composition Principle: Props Drilling vs State Lifting

### The Problem

When you nest components, props flow downward:

```
App
  └─ ConversationView
      ├─ MessageList (needs messages, on_message_clicked)
      │   └─ MessageBubble (needs message, on_clicked)
      └─ MessageInput (needs on_send)
```

At each level, you pass props down. But after 3 levels, prop-drilling gets unwieldy:

```slint
// Bad: Drilling props 3+ levels deep
ConversationView {
    messages: app.messages;  // Level 1
    on_message_clicked: (id) => {
        MessageList {
            messages: messages;  // Level 2 (redundant)
            on_message_clicked: on_message_clicked;
            
            for message in messages: MessageBubble {
                message_content: message.content;  // Level 3
                on_clicked: (id) => {
                    on_message_clicked(id);  // Back up 2 levels
                }
            }
        }
    }
}
```

**Result:** Hard to track where props come from, hard to test, hard to refactor.

### The Solution: State Lifting Rule

**Rule:** Props can be drilled through a maximum of **2-3 levels**. After that, lift state into a container component.

### Rule 1: Container Components

When you have 3+ levels of nesting, create a **Container Component** that:
- Manages state for a whole region
- Passes down only the props needed by children
- Receives callbacks from children and updates app state
- Doesn't render directly (pure composition)

**Example:**

```slint
// ConversationViewContainer - manages the entire conversation region
export component ConversationViewContainer {
    in property conversation_id: string;
    in property app_state: ApplicationState;
    in property on_action: function(AppAction);

    Rectangle {
        VerticalLayout {
            ConversationHeader {
                title: app_state.get_conversation(conversation_id).title;
                on_settings_clicked() => {
                    on_action(AppAction::ShowSettings);
                }
            }

            MessageListContainer {
                conversation_id: conversation_id;
                messages: app_state.get_conversation(conversation_id).messages;
                on_message_clicked: (id) => {
                    on_action(AppAction::SelectMessage { message_id: id });
                }
                on_load_more: () => {
                    on_action(AppAction::LoadMoreMessages);
                }
            }

            MessageInputContainer {
                on_send: (text) => {
                    on_action(AppAction::SendMessage { 
                        conversation_id: conversation_id,
                        text: text 
                    });
                }
            }
        }
    }
}
```

**Benefits:**
- Each container manages one region's state
- Components inside are focused and testable
- Props only drill through 1-2 levels
- Easy to see dependencies and data flow

---

## Rule 2: When to Create a Sub-Component vs Prop

### When to Create a Sub-Component

Create a **new component** when:

1. **It has distinct visual boundaries** - A clear, recognizable UI unit (Button, Card, Dialog)
2. **It's reusable in multiple places** - Used by 2+ parent components
3. **It has internal state** - Has hover/animation/focus that's local to it
4. **It has its own interactions** - User can click/type/focus on it independently

**Example: MessageBubble is a component because:**
- ✅ Visual boundary: clearly a discrete message in the list
- ✅ Reusable: used in MessageList and SearchResults
- ✅ Internal state: has hover, selection state
- ✅ Interactions: user can click it, long-press it

### When to Use Props Instead

Use **props** (don't create new component) when:

1. **It's purely presentational** - No interaction, no internal state
2. **It's not reused** - Only used by one parent
3. **It's a variant of existing component** - Pass prop to change appearance

**Example: MessageBubbleHeader (sender name + timestamp) is NOT a component:**
- ❌ Purely presentational: just layout + text
- ❌ Only used by MessageBubble
- ❌ Could be a prop controlling visibility/layout

Instead, MessageBubble has props: `show_timestamp`, `show_sender_name`

---

## Rule 3: Component Hierarchy Patterns

### Pattern 1: Leaf Components (No Children)

Simplest components. Render themselves, no child components.

**Examples:**
- Button
- Icon
- Chip
- TextField

**Props Limit:** 12 max (enforced by API Standard)

**Example:**

```slint
export component Button {
    in property label: string;
    in property on_clicked: function;
    in property variant: string;
    // ... other props
}
```

---

### Pattern 2: Container Components (Manage Children)

Components that compose multiple child components and manage their interactions.

**Examples:**
- MessageBubble (composes Icon + Text + Buttons)
- ConversationHeader (composes Icon + Text + Button)
- MessageInput (composes TextField + Button)

**Props Limit:** 8 max (tighter than leaf, because they're managing children)

**Pattern:**

```slint
export component MessageBubble {
    // Data Props (3)
    in property message_id: string;
    in property content: string;
    in property sender_name: string;
    
    // Behavior Props (2)
    in property on_clicked: function(string);
    in property on_deleted: function(string);
    
    // Style Props (1)
    in property reduce_motion: bool;

    Rectangle {
        VerticalLayout {
            Text {
                text: sender_name;
                font-size: 12px;
            }
            
            Text {
                text: content;
            }
            
            Icon {
                name: "checkmark";
                // Icon inherits reduce_motion from parent
            }
        }
    }
}
```

---

### Pattern 3: Region/Screen Components (Compose Containers)

Large components representing a screen or major region. Compose multiple container components.

**Examples:**
- ConversationView (entire conversation screen)
- SearchResultsView (search results region)
- SettingsView (settings screen)

**Props Limit:** 4-6 max (very focused, just high-level inputs)

**Pattern:**

```slint
export component ConversationView {
    in property conversation_id: string;
    in property on_action: function(AppAction);
    in property reduce_motion: bool;

    Rectangle {
        VerticalLayout {
            ConversationHeaderContainer {
                conversation_id: conversation_id;
                on_action: on_action;
            }
            
            MessageListContainer {
                conversation_id: conversation_id;
                on_action: on_action;
                reduce_motion: reduce_motion;
            }
            
            MessageInputContainer {
                on_action: on_action;
                reduce_motion: reduce_motion;
            }
        }
    }
}
```

---

## Rule 4: Data Flow Through Composition

### Principle: Single Direction (Downward Props + Upward Callbacks)

Data flows DOWN via props, events flow UP via callbacks:

```
App State (source of truth at top)
    ↓ props
Container Components (manage regions)
    ↓ props
Components (present data)
    ↑ callbacks
    ↓ (interpreted by container)
App State (reducer updates)
    ↓ new props
```

### Example: Message Sending Flow

1. **User types in MessageInput** → TextField fires on_text_changed
2. **MessageInputContainer receives callback** → dispatches Action::UpdateMessageText
3. **App state updates** → message_text field changes
4. **New props flow down** → MessageInput re-renders with new value
5. **User clicks Send** → Button fires on_clicked
6. **MessageInputContainer receives callback** → dispatches Action::SendMessage
7. **App state updates** → message moves from pending to sent
8. **MessageList receives new props** → MessageBubble shows checkmark

Each step is unidirectional. No back-channels, no shortcuts.

---

## Rule 5: Avoiding Prop Drilling (The 3-Level Rule)

### Rule: No More Than 3 Levels of Props

**Bad (4 levels):**

```
App
  props→ ConversationView
    props→ MessageListContainer
      props→ MessageList
        props→ MessageBubble  // Level 4 - TOO DEEP
```

At MessageBubble level, you've lost track of where props came from and why.

**Good (Refactor using container):**

```
App
  props→ ConversationView (Level 1)
    props→ MessageListContainer (Level 2)
    props→ MessageList (uses state from MessageListContainer, level 2)
      props→ MessageBubble (Level 3)
```

**How to refactor:**

1. Identify that we're drilling too deep
2. Create MessageListContainer to manage MessageList + MessageBubble interaction
3. MessageListContainer handles:
   - Receives messages + on_action from above
   - Passes only needed data to MessageList
   - Passes only needed data to MessageBubble
   - Receives clicks from MessageBubble and dispatches actions

---

## Rule 6: Sub-Component Composition (Advanced)

When components get complex, break them into sub-components for organization.

### Example: MessageBubble with Sub-Components

```
MessageBubble
├── MessageBubbleHeader (sender + timestamp)
├── MessageBubbleContent (text + reactions)
└── MessageBubbleActions (delete, reply buttons)
```

But these are **internal sub-components**, not exposed to parent:

```slint
export component MessageBubble {
    // Public API (props + callbacks)
    in property message_id: string;
    in property on_clicked: function;
    
    // Internal structure (not exposed)
    private component MessageBubbleHeader {
        in property sender_name: string;
        // ...
    }
    
    private component MessageBubbleContent {
        in property content: string;
        // ...
    }
    
    Rectangle {
        VerticalLayout {
            MessageBubbleHeader {
                sender_name: sender_name;
            }
            
            MessageBubbleContent {
                content: content;
            }
        }
    }
}
```

**Why use private sub-components?**

1. **Organizational** - Breaks large components into logical pieces
2. **Reusability within component** - Share code between parts
3. **Hidden implementation** - Parent doesn't know or care about sub-structure

**When NOT to use private sub-components:**

If a piece is reused by OTHER parent components, make it public instead:

```slint
// Good: Icon is public, used by many components
export component Icon { ... }

// Use it everywhere:
Button { /* composes Icon */ }
MessageBubble { /* composes Icon */ }
ConversationHeader { /* composes Icon */ }

// Bad: MessageBubbleSpecialHeader only used by MessageBubble
// → keep it private inside MessageBubble, don't expose publicly
```

---

## Rule 7: Testing Compositions

### Unit Test (Component in Isolation)

Test each component alone, with mock props:

```rust
#[test]
fn test_message_bubble_renders_with_props() {
    let component = MessageBubble {
        message_id: "msg-123".to_string(),
        content: "Hello world".to_string(),
        sender_name: "Alice".to_string(),
        // ...
    };
    
    assert!(component.renders_correctly());
    assert_eq!(component.get_label(), "Alice: Hello world");
}
```

---

### Integration Test (Component with Parent)

Test component + parent together:

```rust
#[test]
fn test_message_bubble_click_triggers_action() {
    let mut app_state = AppState::new();
    let component = ConversationView {
        conversation_id: "conv-1".to_string(),
        on_action: |action| {
            app_state.dispatch(action);
        },
    };
    
    // Simulate click on MessageBubble
    component.message_bubble.click();
    
    // Verify action was dispatched
    assert_eq!(app_state.selected_message_id, "msg-123");
}
```

---

### Composition Test (Multiple Levels)

Test the entire composition flow:

```rust
#[test]
fn test_message_input_to_list_flow() {
    let mut app_state = AppState::new();
    let component = ConversationView { /* ... */ };
    
    // User types in MessageInput
    component.message_input.set_value("Hello");
    assert_eq!(app_state.current_message_text, "Hello");
    
    // User clicks Send
    component.message_input.send_button.click();
    
    // Message appears in MessageList
    assert!(app_state.messages.contains("Hello"));
    
    // Message has pending status
    let message = app_state.messages.last();
    assert_eq!(message.status, "pending");
}
```

---

## Rule 8: Performance - Preventing Unnecessary Re-Renders

### Problem: Props Changes Trigger Re-Renders

When a parent component updates, all children re-render:

```slint
ConversationView {
    // If ANY of these props change, all children re-render
    messages: app.messages;           // Changes frequently
    active_conversation_id: app.active_conversation_id;  // Changes rarely
    reduce_motion: app.reduce_motion; // Changes rarely
}

// All 3 children re-render even if only 1 prop changed
MessageList { /* re-renders */ }
MessageInput { /* re-renders */ }
ConversationHeader { /* re-renders */ }
```

### Solution: Use Container Components as Memoization Boundaries

Container components receive top-level props but only pass what children need:

```slint
export component ConversationView {
    in property messages: [Message];
    in property active_conversation_id: string;
    in property reduce_motion: bool;

    // Instead of passing all props to children:
    MessageListContainer {
        // Only pass the props MessageList cares about
        messages: messages;
        reduce_motion: reduce_motion;
        
        // Don't pass: active_conversation_id (irrelevant to message list)
    }
    
    ConversationHeaderContainer {
        conversation_id: active_conversation_id;
        
        // Don't pass: messages, reduce_motion (irrelevant to header)
    }
}
```

**Result:** When messages change, MessageListContainer re-renders (needed). But ConversationHeaderContainer doesn't (unnecessary props didn't change).

---

## Composition Architecture Summary

### Component Pyramid

```
┌─────────────────────────────────┐
│   Application / Screen Level    │ (ConversationView, SettingsView)
│   High-level prop aggregation   │
├─────────────────────────────────┤
│   Region / Container Level      │ (MessageListContainer, etc.)
│   Manage interactions + state    │
├─────────────────────────────────┤
│   Component Level               │ (MessageBubble, ConversationHeader)
│   Compose + present data        │
├─────────────────────────────────┤
│   Leaf Level                    │ (Button, Icon, TextField, etc.)
│   Presentational, no children   │
└─────────────────────────────────┘
```

### Props Flow by Level

| Level | Props In | Props Down | Props Out (Callbacks) | State? |
|-------|----------|-----------|----------------------|--------|
| App | - | High-level feature state | - | App state |
| Screen | Feature state | Region-specific state | App actions | None (receives from App) |
| Container | Region state | Component-specific state | Region actions | None (receives from Screen) |
| Component | Component state | Child component state | Component interactions | Internal UI state |
| Leaf | Leaf state | None | Leaf interactions | Internal UI state only |

---

## Practical Examples

### Example 1: Simple Button

```slint
// Leaf component - no children
export component Button {
    in property label: string;
    in property on_clicked: function;
    
    Rectangle { /* ... */ }
    Text { text: label; }
}

// Used directly by parent
ConversationHeader {
    Rectangle {
        Button {
            label: "Settings";
            on_clicked() => {
                on_action(Action::ShowSettings);
            }
        }
    }
}
```

**Props levels:** 1 (no nesting)

---

### Example 2: MessageBubble with Interactions

```slint
// Component - composes leaves + has internal state
export component MessageBubble {
    in property message_id: string;
    in property content: string;
    in property on_clicked: function;
    
    private property is_hovered: bool;
    
    Rectangle {
        VerticalLayout {
            Text { text: content; }
            
            if is_hovered {
                HorizontalLayout {
                    Button { label: "Reply"; /* ... */ }
                    Button { label: "Delete"; /* ... */ }
                }
            }
        }
        
        TouchArea {
            mouse-entered => { is_hovered = true; }
            mouse-exited => { is_hovered = false; }
            clicked => { on_clicked(message_id); }
        }
    }
}

// Used by container
MessageListContainer {
    for message in messages: {
        MessageBubble {
            message_id: message.id;
            content: message.content;
            on_clicked: (id) => {
                on_action(Action::SelectMessage { message_id: id });
            }
        }
    }
}
```

**Props levels:** 2 (MessageListContainer → MessageBubble)

---

### Example 3: Full Screen with Deep Composition

```slint
// Screen - coordinates regions
export component ConversationView {
    in property conversation_id: string;
    in property on_action: function;
    
    Rectangle {
        VerticalLayout {
            ConversationHeaderContainer { /* manages header */ }
            MessageListContainer { /* manages message list */ }
            MessageInputContainer { /* manages input */ }
        }
    }
}

// Container - manages region
export component MessageListContainer {
    in property messages: [Message];
    in property on_action: function;
    
    Rectangle {
        VerticalLayout {
            if messages.is_empty {
                Text { text: "No messages"; }
            } else {
                for message in messages: {
                    MessageBubble {
                        message_id: message.id;
                        on_clicked: (id) => {
                            on_action(Action::SelectMessage { message_id: id });
                        }
                    }
                }
            }
            
            if is_loading {
                LoadingSpinner { }
            }
        }
    }
}

// Component - presents data
export component MessageBubble {
    in property message_id: string;
    in property on_clicked: function;
    
    Rectangle { /* ... */ }
}
```

**Props levels:**
- App → ConversationView (Level 1)
- ConversationView → MessageListContainer (Level 2)
- MessageListContainer → MessageBubble (Level 3) ✅ Allowed
- MessageBubble → (no children) ✅ Leaf

---

## Checklist: Composition Review

Before implementing a new component hierarchy, ask:

- [ ] **Is my component a Leaf?** (no children)
  - If yes → stick to 12 props max
  - If no → proceed

- [ ] **Does my component compose children?**
  - If yes → it's a Container (8 props max)
  - If no → it's a Leaf

- [ ] **How deep is my nesting?**
  - Count levels: App → Container → Component → Leaf
  - If > 3 levels to Leaf → refactor with container components

- [ ] **Am I prop-drilling?**
  - Are the same props passed through 2+ levels unchanged?
  - If yes → consolidate at container level

- [ ] **Does each level know what it needs?**
  - Does Component know why it got those props?
  - Can you explain the data flow?
  - If confused → simplify

- [ ] **Are my props organized?**
  - Data props first
  - Behavior props second
  - Style props last
  - See COMPONENT_API_STANDARD.md

- [ ] **Can I test this in isolation?**
  - Can I unit test the component with mock props?
  - Can I integration test with a parent?
  - If hard to test → poor composition

- [ ] **Will this perform?**
  - Do unnecessary props cause re-renders?
  - Should I use a container component as a memoization boundary?

---

## What's Next?

Week 1: Implement Leaf components (Button, Icon, TextField, Chip, LoadingSpinner)

Week 2: Compose first Container components (MessageBubble, ConversationItem)

Week 3+: Build Screens using containers + components

---

**Approved By:** Amelia (Developer), Winston (Architect), Sally (UX Designer)  
**Effective Date:** Pre-Week-1  
**Reviewed Every:** Week (adjust based on real experience)
