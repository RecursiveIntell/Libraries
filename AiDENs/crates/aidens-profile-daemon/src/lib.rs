//! Daemon profile status.
//!
//! This crate is a partial safe-mode status surface. Broad autonomous daemon
//! operation remains unsupported.

pub const PROFILE_ID: &str = "autonomous-daemon";
pub const SUPPORT_TIER: &str = "partial/safe-mode";
pub const NON_GOAL: &str = "not broad autonomous operation";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileDaemonStatus {
    pub enabled: bool,
    pub note: String,
}

impl Default for ProfileDaemonStatus {
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
    fn daemon_profile_discloses_safe_mode_only() {
        let status = ProfileDaemonStatus::default();
        assert!(!status.enabled);
        assert!(status.note.contains(SUPPORT_TIER));
        assert!(status.note.contains("not broad autonomous operation"));
    }
}
