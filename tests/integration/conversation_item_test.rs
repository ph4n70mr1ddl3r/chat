// ============================================================================
// ConversationItem Component Integration Tests
// ============================================================================
// AC1: Displays participant/display name and presence indicator
// AC2: Shows most recent message snippet (truncated if too long)
// AC3: Shows relative timestamp of the last message
// AC4: Displays a prominent unread indicator when unread messages exist
// AC5: on_clicked callback fires when the item is selected
// AC6: Visual highlight state for selected vs. unselected conversations
// AC7: Hover state feedback for interactive feel
// AC8: Keyboard accessible (Tab navigation, focus indicator, Enter to select)
// ============================================================================

use std::path::Path;

#[test]
fn test_conversation_item_file_exists() {
    let path = Path::new("src/frontend/components/discovery/ConversationItem.slint");
    assert!(path.exists(), "ConversationItem component file missing");
}

#[test]
fn test_conversation_item_props_definition() {
    let path = Path::new("src/frontend/components/discovery/ConversationItem.slint");
    let content = std::fs::read_to_string(path).unwrap();
    
    let required_props = vec![
        "name",
        "last_message",
        "timestamp",
        "unread_count",
        "is_selected",
        "presence_online",
    ];
    
    for prop in required_props {
        assert!(content.contains(prop), "ConversationItem missing required prop: {}", prop);
    }
}

#[test]
fn test_conversation_item_callbacks_definition() {
    let path = Path::new("src/frontend/components/discovery/ConversationItem.slint");
    let content = std::fs::read_to_string(path).unwrap();
    
    assert!(content.contains("clicked"), "ConversationItem missing clicked callback");
}

#[test]
fn test_conversation_item_uses_tokens() {
    let path = Path::new("src/frontend/components/discovery/ConversationItem.slint");
    let content = std::fs::read_to_string(path).unwrap();
    
    assert!(content.contains("Tokens"), "ConversationItem should import and use Tokens");
    assert!(content.contains("import { Tokens }"), "ConversationItem should import Tokens");
}

#[test]
fn test_conversation_item_uses_online_indicator() {
    let path = Path::new("src/frontend/components/discovery/ConversationItem.slint");
    let content = std::fs::read_to_string(path).unwrap();
    
    assert!(content.contains("OnlineIndicator"), "ConversationItem should use OnlineIndicator for presence");
}

#[test]
fn test_conversation_item_registered_in_module() {
    let path = Path::new("src/frontend/components/mod.rs");
    let content = std::fs::read_to_string(path).unwrap();
    
    assert!(content.contains("CONVERSATION_ITEM_PATH"), "ConversationItem path should be exported in mod.rs");
    assert!(content.contains("components/discovery/ConversationItem.slint"), "ConversationItem path should match actual file location");
}

