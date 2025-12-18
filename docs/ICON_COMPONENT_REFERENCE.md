# Component: Icon

## Purpose
A reusable icon component that renders SVG assets with consistent sizing and dynamic colorization via design tokens.

## Category
Utility / Visual

## Input Props

| Prop | Type | Default | Required | Category | Description |
|------|------|---------|----------|----------|-------------|
| name | string | - | Yes | Data | The filename of the icon in `assets/icons/` (without .svg) |
| alt_text | string | "" | No | Data | Description for screen readers. Sets `accessible-role: img` if provided. |
| size | string | "medium" | No | Style | "small" (16px) \| "medium" (24px) \| "large" (32px) \| "xlarge" (48px) |
| color | brush | Tokens.neutral_medium | No | Style | The color of the icon using Slint's `colorize` |
| reduce_motion | bool | Tokens.prefers_reduced_motion | No | Style | Disables rotation for animated icons (e.g., spinner) |

## Available Icons (Initial Library)
- `checkmark`
- `checkmark-double`
- `spinner` (Animated)
- `close`
- `send`
- `settings`
- `search`
- `user-profile`
- `online-dot`
- `offline-dot`

## Accessibility
- **Role:** `img` if `alt_text` is provided, otherwise `presentation`.
- **Label:** Uses `alt_text`.

## Example Usage

```slint
import { Icon } from "components/icon.slint";
import { Tokens } from "design/tokens.slint";

App {
    Icon {
        name: "checkmark";
        size: "large";
        color: Tokens.success;
        alt_text: "Message sent";
    }
}
```
