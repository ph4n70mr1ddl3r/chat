# Component: LoadingSpinner

## Purpose
A visual indicator used to signal that the application is busy processing an action or loading data.

## Category
Feedback / Visual

## Input Props

| Prop | Type | Default | Required | Category | Description |
|------|------|---------|----------|----------|-------------|
| message | string | "" | No | Data | Optional text displayed below the spinner. |
| size | string | "medium" | No | Style | "small" (20px) \| "medium" (24px) \| "large" (32px) |
| color | brush | Tokens.neutral_medium | No | Style | The color of the spinner halo. |
| reduce_motion | bool | Tokens.prefers_reduced_motion | No | Style | If true, the spinner remains static (no rotation). |

## Visual Variants

### Sizes
- **small:** 20px diameter, 2px border.
- **medium:** 24px diameter, 2px border.
- **large:** 32px diameter, 2px border.

### Animation
- **Normal:** 360-degree rotation every 800ms (linear).
- **Reduced Motion:** Static quarter-circle halo.

## Accessibility
- **Role:** `status`.
- **Label:** Defaults to "Loading..." or the provided `message`.
- **Live Region:** Acts as a "polite" update notification.

## Example Usage

```slint
import { LoadingSpinner } from "components/loading_spinner.slint";
import { Tokens } from "design/tokens.slint";

App {
    // Basic spinner
    LoadingSpinner {}
    
    // Large spinner with message
    LoadingSpinner {
        size: "large";
        color: Tokens.fluent_blue;
        message: "Fetching messages...";
    }
}
```
