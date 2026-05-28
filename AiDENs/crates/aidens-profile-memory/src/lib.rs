//! Memory-agent profile status.
//!
//! This crate is a partial/proof-only surface. Canonical memory truth remains
//! owned by the memory/runtime crates, not by this profile wrapper.

pub const PROFILE_ID: &str = "memory-agent";
pub const SUPPORT_TIER: &str = "partial/proof-only";
pub const NON_GOAL: &str = "not a canonical memory truth owner";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileMemoryStatus {
    pub enabled: bool,
    pub note: String,
}

impl Default for ProfileMemoryStatus {
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
    fn memory_profile_discloses_partial_non_owner_status() {
        let status = ProfileMemoryStatus::default();
        assert!(!status.enabled);
        assert!(status.note.contains(SUPPORT_TIER));
        assert!(status.note.contains("not a canonical memory truth owner"));
    }
}
