// ============================================================================
// MessageBubble Component Integration Tests
// ============================================================================

use std::path::Path;

#[test]
fn test_message_bubble_exists() {
    let path = Path::new("src/frontend/components/message_bubble.slint");
    assert!(path.exists());
}

#[test]
fn test_message_bubble_props() {
    let path = Path::new("src/frontend/components/message_bubble.slint");
    let content = std::fs::read_to_string(path).unwrap();
    
    assert!(content.contains("content"));
    assert!(content.contains("sender_name"));
    assert!(content.contains("timestamp"));
    assert!(content.contains("is_own"));
    assert!(content.contains("status"));
}
