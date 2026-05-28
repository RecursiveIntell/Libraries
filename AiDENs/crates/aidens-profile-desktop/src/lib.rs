//! Desktop profile status.
//!
//! This crate is deferred until desktop product wiring exists with receipt
//! evidence. It exposes status only.

pub const PROFILE_ID: &str = "desktop";
pub const SUPPORT_TIER: &str = "deferred/profile-status-only";
pub const NON_GOAL: &str = "not a desktop product runtime";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileDesktopStatus {
    pub enabled: bool,
    pub note: String,
}

impl Default for ProfileDesktopStatus {
    fn default() -> Self {
        Self {
            enabled: false,
            note: format!("{PROFILE_ID}: {SUPPORT_TIER}; {NON_GOAL}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_profile_is_status_only() {
        let status = ProfileDesktopStatus::default();
        assert!(!status.enabled);
        assert!(status.note.contains(SUPPORT_TIER));
        assert!(status.note.contains("not a desktop product runtime"));
    }
}
