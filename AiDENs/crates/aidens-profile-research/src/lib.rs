//! Research-workbench profile status.
//!
//! This crate is a deferred/example-only surface. It must not be presented as a
//! completed research product profile without executable receipts and profile tests.

pub const PROFILE_ID: &str = "research-workbench";
pub const SUPPORT_TIER: &str = "deferred/example-only";
pub const NON_GOAL: &str = "not a completed research product surface";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileResearchStatus {
    pub enabled: bool,
    pub note: String,
}

impl Default for ProfileResearchStatus {
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
    fn research_profile_is_not_marked_product_ready() {
        let status = ProfileResearchStatus::default();
        assert!(!status.enabled);
        assert!(status.note.contains(SUPPORT_TIER));
        assert!(status
            .note
            .contains("not a completed research product surface"));
    }
}
