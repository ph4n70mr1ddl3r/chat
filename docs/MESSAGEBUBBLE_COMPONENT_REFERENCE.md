# Component: MessageBubble

## Purpose
Displays an individual message in a conversation thread with metadata and actions.

## Category
Core / Messaging

## Input Props

| Prop | Type | Default | Required | Description |
|------|------|---------|----------|-------------|
| content | string | "" | Yes | The text content of the message. |
| sender_name | string | "" | Yes | Name of the sender. |
| timestamp | string | "" | Yes | When the message was sent (e.g. "10:45 AM"). |
| is_own | bool | false | No | If true, aligns to right and uses primary theme. |
| status | string | "sent" | No | "pending"\|"sent"\|"delivered"\|"failed". |

## Callbacks
- `clicked()`: Fired when the bubble is tapped.
- `long_pressed()`: Fired on long press.
- `reply()`: Fired when Reply action is selected.
- `delete()`: Fired when Delete action is selected.

## Visuals
- **Sent (Own):** Aligned right, Fluent Blue background, white text.
- **Received:** Aligned left, Neutral Light background, dark text.
- **Actions:** Reply and Delete buttons appear on hover.

## Accessibility
- **Role:** `list-item`.
- **Label:** Combines sender, content, and timestamp for screen readers.
- **Keyboard:** Tab navigation to focus message.
