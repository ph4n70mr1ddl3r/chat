# ConversationItem Component Reference

## Overview
Interactive conversation list item component displaying participant information, latest message snippet, timestamp, unread count, and presence status.

## File Location
- **Component:** `src/frontend/components/discovery/ConversationItem.slint`
- **Module Export:** ` `CONVERSATION_ITEM_PATH` in `src/frontend/components/mod.rs`

## Properties

### `name: string`
Display name of the conversation participant.

### `last_message: string`
Most recent message snippet. Automatically truncated with ellipsis if too long.

### `timestamp: string`
Relative time of the last message (e.g., "2m ago", "10:30 AM").

### `unread_count: int`
Number of unread messages. Badge is only displayed when `> 0`.

### `is_selected: bool`
Whether this conversation is currently selected. Applies visual highlighting.

### `presence_online: bool`
Online/offline presence status for the participant. Currently mapped to boolean; will integrate with `PresenceIndicator` component when available.

## Callbacks

### `clicked()`
Fired when the conversation item is clicked/tapped.

## Visual States

### Default
- Transparent background
- Standard text colors from Tokens

### Hover
- Light gray background (`Tokens.neutral_light`)

### Selected
- Blue tinted background (`Tokens.fluent_blue` with 10% alpha)

## Layout
- **Height:** 64px fixed
- **Padding:** `Tokens.spacing_md` (12px)
- **Structure:** Horizontal layout with avatar/presence indicator, text content (name + snippet), and optional unread badge

## Accessibility
- Interactive via TouchArea
- Keyboard navigable (Tab focus, Enter to select)
- Focus indicator via visual states

## Design Tokens Used
- `Tokens.spacing_md`: padding and spacing
- `Tokens.font_size_body`: name text
- `Tokens.font_size_caption`: message snippet and timestamp
- `Tokens.neutral_dark`: primary text color
- `Tokens.neutral_medium`: secondary text color
- `Tokens.neutral_light`: hover background
- `Tokens.fluent_blue`: selected background and unread badge

## Integration Example
```slint
import { ConversationItem } from "components/discovery/ConversationItem.slint";

ConversationItem {
    name: "Alice";
    last_message: "Hey, how are you?";
    timestamp: "2m ago";
    unread_count: 3;
    is_selected: false;
    presence_online: true;
    
    clicked => {
        // Handle conversation selection
        select_conversation(id);
    }
}
```

## Testing
Integration tests available in `tests/integration/conversation_item_test.rs`:
- File existence validation
- Property definitions
- Callback definitions
- Design token usage
- Module registration

## Dependencies
- `Tokens` (design system)
- `OnlineIndicator` (presence status)

## Future Enhancements
- Integration with dedicated `PresenceIndicator` component (US-009)
- Support for group conversations with multi-avatar display
- Last message type indicators (image, file, etc.)
- Message preview for rich content
