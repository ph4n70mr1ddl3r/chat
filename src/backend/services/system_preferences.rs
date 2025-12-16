// ============================================================================
// System Preferences Module
// ============================================================================
// Handles detection and management of system accessibility preferences
// Supports Windows Settings: Ease of Access > Display > Show animations
// Provides callbacks for preference changes
// ============================================================================

use crate::models::AppState;
use std::sync::{Arc, Mutex};

/// System preference for motion animation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotionPreference {
    /// Show animations (default)
    AnimationsEnabled,
    /// Respect system "Reduce motion" preference
    ReducedMotion,
    /// User manually disabled animations
    ManuallyDisabled,
}

impl Default for MotionPreference {
    fn default() -> Self {
        MotionPreference::AnimationsEnabled
    }
}

/// System preferences manager
pub struct SystemPreferences {
    motion_preference: Arc<Mutex<MotionPreference>>,
}

impl SystemPreferences {
    /// Create new system preferences manager
    pub fn new() -> Self {
        let preference = Self::detect_system_preference();
        SystemPreferences {
            motion_preference: Arc::new(Mutex::new(preference)),
        }
    }

    /// Detect Windows system preference for reduced motion
    /// Reads from Windows Registry: HKEY_CURRENT_USER\Control Panel\Accessibility
    /// Value: "DisableAnimations" (0 = animations enabled, 1 = animations disabled)
    #[cfg(target_os = "windows")]
    fn detect_system_preference() -> MotionPreference {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;

        match RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey("Control Panel\\Accessibility")
        {
            Ok(key) => {
                match key.get_value::<u32, _>("DisableAnimations") {
                    Ok(1) => MotionPreference::ReducedMotion,
                    Ok(0) => MotionPreference::AnimationsEnabled,
                    _ => MotionPreference::AnimationsEnabled,
                }
            }
            Err(_) => MotionPreference::AnimationsEnabled,
        }
    }

    /// Detect system preference for non-Windows platforms (fallback)
    #[cfg(not(target_os = "windows"))]
    fn detect_system_preference() -> MotionPreference {
        // On non-Windows platforms, default to animations enabled
        // Future: Could check environment variables like PREFERS_REDUCED_MOTION
        MotionPreference::AnimationsEnabled
    }

    /// Get current motion preference
    pub fn motion_preference(&self) -> MotionPreference {
        *self.motion_preference.lock().unwrap()
    }

    /// Set motion preference (used for user overrides)
    pub fn set_motion_preference(&self, preference: MotionPreference) {
        *self.motion_preference.lock().unwrap() = preference;
    }

    /// Check if animations should be shown
    pub fn should_show_animations(&self) -> bool {
        matches!(
            self.motion_preference(),
            MotionPreference::AnimationsEnabled
        )
    }

    /// Refresh system preference detection
    pub fn refresh(&self) {
        let new_preference = Self::detect_system_preference();
        self.set_motion_preference(new_preference);
    }
}

impl Default for SystemPreferences {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_motion_preference() {
        let prefs = SystemPreferences::new();
        // Default should allow animations
        assert!(prefs.should_show_animations());
    }

    #[test]
    fn test_set_motion_preference() {
        let prefs = SystemPreferences::new();
        prefs.set_motion_preference(MotionPreference::ReducedMotion);
        assert!(!prefs.should_show_animations());
    }

    #[test]
    fn test_motion_preference_toggle() {
        let prefs = SystemPreferences::new();
        
        // Start with animations
        assert_eq!(prefs.motion_preference(), MotionPreference::AnimationsEnabled);
        
        // Disable
        prefs.set_motion_preference(MotionPreference::ReducedMotion);
        assert_eq!(prefs.motion_preference(), MotionPreference::ReducedMotion);
        
        // Re-enable
        prefs.set_motion_preference(MotionPreference::AnimationsEnabled);
        assert_eq!(prefs.motion_preference(), MotionPreference::AnimationsEnabled);
    }
}
