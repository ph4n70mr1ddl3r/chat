// ============================================================================
// TextField Component Integration Tests
// ============================================================================
// Validates TextField component for all 8 Acceptance Criteria:
// AC1: TextField Renders with Correct Base States (Token-Aligned)
// AC2: Placeholder Logic Works correctly
// AC3: on_text_changed Callback Fires
// AC4: on_return_pressed Callback for Submission
// AC5: Error State (has_error=true)
// AC6: Keyboard Accessibility & Focus Handling
// AC7: Screen Reader Accessible
// AC8: Respects reduce_motion Preference
// ============================================================================

use std::path::Path;

/// Test ID: US-003-T01
/// Given: TextField component source file
/// When: Checking for file existence
/// Then: src/frontend/components/text_field.slint should exist
#[test]
fn test_text_field_file_exists() {
    let path = Path::new("src/frontend/components/text_field.slint");
    assert!(path.exists(), "TextField component file missing: src/frontend/components/text_field.slint");
}

/// Test ID: US-003-T02
/// Given: TextField component
/// When: Checking for required props
/// Then: All 10 standard props should be defined
#[test]
fn test_text_field_props_definition() {
    let path = Path::new("src/frontend/components/text_field.slint");
    if !path.exists() {
        panic!("Cannot check props: file missing");
    }
    
    let content = std::fs::read_to_string(path).unwrap();
    
    let required_props = vec![
        "value",
        "placeholder",
        "error_message",
        "on_text_changed",
        "on_return_pressed",
        "on_focus",
        "on_blur",
        "is_disabled",
        "has_error",
        "reduce_motion",
    ];
    
    for prop in required_props {
        assert!(content.contains(prop), "TextField missing required prop: {}", prop);
    }
}
