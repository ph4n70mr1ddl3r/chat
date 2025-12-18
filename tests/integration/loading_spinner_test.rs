// ============================================================================
// LoadingSpinner Component Integration Tests
// ============================================================================
// AC1: Animated Halo Style
// AC2: Respects reduce_motion
// AC3: Optional Loading Message
// AC4: Colorization Support
// AC5: Accessibility (ARIA Status)
// ============================================================================

use std::path::Path;

#[test]
fn test_loading_spinner_file_exists() {
    let path = Path::new("src/frontend/components/loading_spinner.slint");
    assert!(path.exists(), "LoadingSpinner component file missing");
}

#[test]
fn test_loading_spinner_props_definition() {
    let path = Path::new("src/frontend/components/loading_spinner.slint");
    let content = std::fs::read_to_string(path).unwrap();
    
    let required_props = vec![
        "message",
        "size",
        "color",
        "reduce_motion",
    ];
    
    for prop in required_props {
        assert!(content.contains(prop), "LoadingSpinner missing required prop: {}", prop);
    }
}
