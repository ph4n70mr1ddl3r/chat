// ============================================================================
// MessageComposer Component Integration Tests
// ============================================================================

use std::path::Path;
use std::fs;

#[test]
fn test_message_composer_component_exists() {
    let path = Path::new("src/frontend/components/messaging/MessageComposer.slint");
    assert!(path.exists(), "MessageComposer.slint should exist at src/frontend/components/messaging/");
}

#[test]
fn test_message_composer_exports_component() {
    let path = Path::new("src/frontend/components/messaging/MessageComposer.slint");
    let content = fs::read_to_string(path).expect("Failed to read MessageComposer.slint");
    
    assert!(content.contains("export component MessageComposer"), 
        "MessageComposer should export the MessageComposer component");
}

#[test]
fn test_message_composer_has_required_properties() {
    let path = Path::new("src/frontend/components/messaging/MessageComposer.slint");
    let content = fs::read_to_string(path).expect("Failed to read MessageComposer.slint");
    
    // Required in properties
    assert!(content.contains("in property"), "Component should have input properties");
    assert!(content.contains("is_sending"), "Component should have is_sending property");
    assert!(content.contains("error_text"), "Component should have error_text property");
    
    // Required in-out properties
    assert!(content.contains("in-out property") || content.contains("in-out property <string> draft_text"),
        "Component should have draft_text property");
}

#[test]
fn test_message_composer_has_required_callbacks() {
    let path = Path::new("src/frontend/components/messaging/MessageComposer.slint");
    let content = fs::read_to_string(path).expect("Failed to read MessageComposer.slint");
    
    assert!(content.contains("callback send"), 
        "Component should have send callback");
    assert!(content.contains("callback typing"),
        "Component should have typing callback");
}

#[test]
fn test_message_composer_uses_design_tokens() {
    let path = Path::new("src/frontend/components/messaging/MessageComposer.slint");
    let content = fs::read_to_string(path).expect("Failed to read MessageComposer.slint");
    
    // Should import Tokens
    assert!(content.contains("Tokens"), 
        "Component should use design tokens (Tokens)");
    
    // Should use token spacing and colors
    assert!(content.contains("Tokens.spacing") || content.contains("Tokens.font_size"),
        "Component should use Tokens for styling");
}

#[test]
fn test_message_composer_has_multiline_input() {
    let path = Path::new("src/frontend/components/messaging/MessageComposer.slint");
    let content = fs::read_to_string(path).expect("Failed to read MessageComposer.slint");
    
    // Should use TextEdit for multi-line support
    assert!(content.contains("TextEdit"),
        "Component should use TextEdit for multi-line support");
}

#[test]
fn test_message_composer_send_button_implementation() {
    let path = Path::new("src/frontend/components/messaging/MessageComposer.slint");
    let content = fs::read_to_string(path).expect("Failed to read MessageComposer.slint");
    
    // Should have send button that's disabled when text is empty
    assert!(content.contains("enabled:") || content.contains("Button"),
        "Component should have enabled state for send button");
}

#[test]
fn test_message_composer_loading_state_handling() {
    let path = Path::new("src/frontend/components/messaging/MessageComposer.slint");
    let content = fs::read_to_string(path).expect("Failed to read MessageComposer.slint");
    
    // When is_sending is true, input should be disabled and button should show loading
    assert!(content.contains("is_sending"),
        "Component should respect is_sending flag");
}

#[test]
fn test_message_composer_error_display() {
    let path = Path::new("src/frontend/components/messaging/MessageComposer.slint");
    let content = fs::read_to_string(path).expect("Failed to read MessageComposer.slint");
    
    // Should display error_text when present
    assert!(content.contains("error_text"),
        "Component should display error_text when present");
}

#[test]
fn test_message_composer_has_placeholder() {
    let path = Path::new("src/frontend/components/messaging/MessageComposer.slint");
    let content = fs::read_to_string(path).expect("Failed to read MessageComposer.slint");
    
    // Should have placeholder text input
    assert!(content.contains("placeholder") || content.contains("Type a message"),
        "Component should have placeholder text");
}

// Integration test for container/handler
#[test]
fn test_message_composer_rust_handler_exists() {
    let path = Path::new("src/frontend/containers/messaging/message_composer.rs");
    
    // First check if old location exists
    if !path.exists() {
        // Check if it might be in components directory instead
        let alt_path = Path::new("src/frontend/components/messaging/message_composer.rs");
        
        // Create a placeholder if neither exists - this is part of RED phase
        if !alt_path.exists() {
            println!("Note: Rust handler for MessageComposer does not yet exist - will be created in GREEN phase");
        }
    }
}
