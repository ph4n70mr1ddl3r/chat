// ============================================================================
// ConversationHeader Rust Handler
// ============================================================================
// Handles integration between ConversationHeader Slint component and backend
// services for real-time presence updates, status formatting, and last-seen
// time calculations.

use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a conversation participant for the header
#[derive(Clone, Debug)]
pub struct ConversationParticipant {
    pub id: String,
    pub name: String,
}

/// Presence data for displaying status in the header
#[derive(Clone, Debug, PartialEq)]
pub enum PresenceStatus {
    Online,
    Away,
    Offline,
}

#[derive(Clone, Debug)]
pub struct PresenceData {
    pub status: PresenceStatus,
    pub last_seen_seconds: i64,  // Unix timestamp in seconds, only used if status == Offline
}

/// Formats a Unix timestamp into a human-readable "last seen" string
/// Examples: "5m ago", "2h ago", "1d ago"
pub fn format_last_seen(last_seen_seconds: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    
    let seconds_ago = now - last_seen_seconds;
    
    if seconds_ago < 60 {
        format!("{}s ago", seconds_ago)
    } else if seconds_ago < 3600 {
        let minutes = seconds_ago / 60;
        format!("{}m ago", minutes)
    } else if seconds_ago < 86400 {
        let hours = seconds_ago / 3600;
        format!("{}h ago", hours)
    } else {
        let days = seconds_ago / 86400;
        format!("{}d ago", days)
    }
}

/// Formats presence status for display in the header
pub fn format_status_label(status: &PresenceStatus) -> String {
    match status {
        PresenceStatus::Online => "Online".to_string(),
        PresenceStatus::Away => "Away".to_string(),
        PresenceStatus::Offline => "Offline".to_string(),
    }
}

/// Generates accessibility label for screen readers
pub fn generate_accessible_label(
    participant: &ConversationParticipant,
    presence: &PresenceData,
) -> String {
    let status_text = format_status_label(&presence.status);
    
    let additional_info = if presence.status == PresenceStatus::Offline && presence.last_seen_seconds > 0 {
        format!(". Last seen {}", format_last_seen(presence.last_seen_seconds))
    } else {
        String::new()
    };
    
    format!(
        "Conversation with {}. Status: {}{}.",
        participant.name, status_text, additional_info
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_status_label_online() {
        let label = format_status_label(&PresenceStatus::Online);
        assert_eq!(label, "Online");
    }

    #[test]
    fn test_format_status_label_away() {
        let label = format_status_label(&PresenceStatus::Away);
        assert_eq!(label, "Away");
    }

    #[test]
    fn test_format_status_label_offline() {
        let label = format_status_label(&PresenceStatus::Offline);
        assert_eq!(label, "Offline");
    }

    #[test]
    fn test_format_last_seen_seconds() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let thirty_seconds_ago = now - 30;
        let formatted = format_last_seen(thirty_seconds_ago);
        assert!(formatted.contains("30s ago"));
    }

    #[test]
    fn test_format_last_seen_minutes() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let five_minutes_ago = now - (5 * 60);
        let formatted = format_last_seen(five_minutes_ago);
        assert!(formatted.contains("5m ago"));
    }

    #[test]
    fn test_format_last_seen_hours() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let two_hours_ago = now - (2 * 3600);
        let formatted = format_last_seen(two_hours_ago);
        assert!(formatted.contains("2h ago"));
    }

    #[test]
    fn test_format_last_seen_days() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let one_day_ago = now - 86400;
        let formatted = format_last_seen(one_day_ago);
        assert!(formatted.contains("1d ago"));
    }

    #[test]
    fn test_generate_accessible_label_online() {
        let participant = ConversationParticipant {
            id: "user-123".to_string(),
            name: "Alice".to_string(),
        };
        
        let presence = PresenceData {
            status: PresenceStatus::Online,
            last_seen_seconds: 0,
        };
        
        let label = generate_accessible_label(&participant, &presence);
        assert!(label.contains("Alice"));
        assert!(label.contains("Online"));
    }

    #[test]
    fn test_generate_accessible_label_offline_with_last_seen() {
        let participant = ConversationParticipant {
            id: "user-123".to_string(),
            name: "Bob".to_string(),
        };
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let one_hour_ago = now - 3600;
        
        let presence = PresenceData {
            status: PresenceStatus::Offline,
            last_seen_seconds: one_hour_ago,
        };
        
        let label = generate_accessible_label(&participant, &presence);
        assert!(label.contains("Bob"));
        assert!(label.contains("Offline"));
        assert!(label.contains("Last seen"));
    }

    #[test]
    fn test_generate_accessible_label_away() {
        let participant = ConversationParticipant {
            id: "user-456".to_string(),
            name: "Charlie".to_string(),
        };
        
        let presence = PresenceData {
            status: PresenceStatus::Away,
            last_seen_seconds: 0,
        };
        
        let label = generate_accessible_label(&participant, &presence);
        assert!(label.contains("Charlie"));
        assert!(label.contains("Away"));
    }
}
