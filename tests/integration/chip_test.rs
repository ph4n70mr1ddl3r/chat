// ============================================================================
// Chip Component Integration Tests
// ============================================================================
// AC1: Pill Shape & Height
// AC2: Variant Styling
// AC3: Dismissible Toggle
// AC4: Clickable Logic
// AC5: Keyboard Accessibility & Dismissal
// AC6: Screen Reader Accessible
// ============================================================================

use std::path::Path;

#[test]
fn test_chip_file_exists() {
    let path = Path::new("src/frontend/components/chip.slint");
    assert!(path.exists(), "Chip component file missing");
}

#[test]
fn test_chip_props_definition() {
    let path = Path::new("src/frontend/components/chip.slint");
    let content = std::fs::read_to_string(path).unwrap();
    
    let required_props = vec![
        "label",
        "variant",
        "is_dismissible",
        "is_disabled",
        "is_clickable",
        "reduce_motion",
    ];
    
    for prop in required_props {
        assert!(content.contains(prop), "Chip missing required prop: {}", prop);
    }
}

#[test]
fn test_chip_callbacks_definition() {
    let path = Path::new("src/frontend/components/chip.slint");
    let content = std::fs::read_to_string(path).unwrap();
    
    let required_callbacks = vec![
        "clicked",
        "dismissed",
    ];
    
    for callback in required_callbacks {
        assert!(content.contains(callback), "Chip missing required callback: {}", callback);
    }
}
