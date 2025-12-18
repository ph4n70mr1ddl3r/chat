# Component: TextField

## Purpose
A reusable text input component based on Slint's LineEdit, supporting placeholders, error states, and accessibility feature.

## Category
Input

## Input Props

| Prop | Type | Default | Required | Category | Description |
|------|------|---------|----------|----------|-------------|
| value | string | "" | No | Data | The current text value of the input |
| placeholder | string | "" | No | Data | Grayed-out text visible when value is empty |
| error_message | string | "" | No | Data | Message displayed below input when has_error is true |
| label | string | "" | No | Data | Accessible label for screen readers |
| is_disabled | bool | false | No | Style | Disables input and styling |
| has_error | bool | false | No | Style | Applies error styling (red border) |
| reduce_motion | bool | false | No | Style | Disables transitions if true |

## Callbacks

| Callback | Args | When Triggered |
|----------|------|-----------------|
| on_text_changed | string | Every time the user edits the text |
| on_return_pressed | - | When the user presses the Enter key |
| on_focus | - | When the component receives focus |
| on_blur | - | When the component loses focus |

## Visual Variants

### States
- **Normal:** 1px neutral border, neutral light background.
- **Focused:** 2px fluent blue border, white background.
- **Error:** 2px red border, error message displayed below.
- **Disabled:** 50% opacity border and text, neutral light background.

## Accessibility

- **Role:** `text-input`
- **Label:** Uses `label` prop or falls back to `placeholder`.
- **Keyboard:** Tab to focus, Enter to submit.
- **Motion:** Fade-in on error message respects `reduce_motion`.

## Example Usage

```slint
import { TextField } from "components/text_field.slint";

App {
    TextField {
        placeholder: "Search messages...";
        on_text_changed(text) => {
            debug("Searching: " + text);
        }
    }
}
```
