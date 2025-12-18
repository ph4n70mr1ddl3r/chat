// ============================================================================
// Icon Component Integration Tests
// ============================================================================
// AC1: Icon Renders SVG from Assets Folder
// AC2: Sizing Scale Support
// AC3: Colorization via colorize Property
// AC4: Accessibility (Alt-Text & Roles)
// AC5: Respects reduce_motion
// AC6: Initial Asset Library Creation
// ============================================================================

use std::path::Path;

#[test]
fn test_icon_file_exists() {
    let path = Path::new("src/frontend/components/icon.slint");
    assert!(path.exists(), "Icon component file missing");
}

#[test]
fn test_icon_assets_exist() {
    let icons = vec![
        "checkmark", "checkmark-double", "spinner", "close", "send",
        "settings", "search", "user-profile", "online-dot", "offline-dot"
    ];
    
    for icon in icons {
        let path = format!("assets/icons/{}.svg", icon);
        assert!(Path::new(&path).exists(), "Icon asset missing: {}", path);
    }
}

#[test]
fn test_icon_props_definition() {
    let path = Path::new("src/frontend/components/icon.slint");
    let content = std::fs::read_to_string(path).unwrap();
    
    let required_props = vec![
        "name",
        "alt_text",
        "size",
        "color",
        "reduce_motion",
    ];
    
    for prop in required_props {
        assert!(content.contains(prop), "Icon missing required prop: {}", prop);
    }
}
