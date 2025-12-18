# Component: Chip

## Purpose
A compact, versatile element used for tags, user mentions, status badges, and dismissible labels.

## Category
Utility / Data Display

## Input Props

| Prop | Type | Default | Required | Category | Description |
|------|------|---------|----------|----------|-------------|
| label | string | "" | Yes | Data | The text displayed inside the chip. |
| variant | string | "default" | No | Style | "default" \| "primary" \| "success" \| "warning" \| "error" |
| is_dismissible | bool | false | No | Style | Shows a close icon that triggers `dismissed` callback. |
| is_disabled | bool | false | No | Style | Disables interaction and dims appearance. |
| is_clickable | bool | false | No | Style | Enables hover states and `clicked` callback. |
| reduce_motion | bool | Tokens.prefers_reduced_motion | No | Style | Disables transitions if true. |

## Callbacks

| Callback | Args | When Triggered |
|----------|------|-----------------|
| clicked | - | When the chip is clicked (if `is_clickable` is true). |
| dismissed | - | When the close icon is clicked or Delete/Backspace is pressed. |

## Visual Variants

### States
- **Normal:** Rounded ends (pill shape), fixed 28px height.
- **Hover:** Background darkens by 10% (only if `is_clickable`).
- **Focused:** 2px blue border.

### Colors (derived from tokens at 10% opacity)
- **default:** Neutral background and text.
- **primary:** Blue light background, blue text.
- **success:** Green light background, green text.
- **warning:** Orange light background, orange text.
- **error:** Red light background, red text.

## Accessibility
- **Role:** `button` if `is_clickable`, otherwise `none` (acts as status/text).
- **Label:** Uses `label` prop.
- **Keyboard:** Tab to focus, Enter/Space to click, Delete/Backspace to dismiss.

## Example Usage

```slint
import { Chip } from "components/chip.slint";
import { Tokens } from "design/tokens.slint";

App {
    Chip {
        label: "Feature Request";
        variant: "primary";
        is_clickable: true;
        clicked => { debug("Chip clicked"); }
    }
    
    Chip {
        label: "@alice";
        is_dismissible: true;
        dismissed => { debug("Mention removed"); }
    }
}
```
