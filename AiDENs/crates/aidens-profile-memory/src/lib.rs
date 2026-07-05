//! Memory-agent profile status.
//!
//! This crate is a partial/proof-only surface. Canonical memory truth remains
//! owned by the memory/runtime crates, not by this profile wrapper.

pub use aidens_memory_kit::SemanticMemoryBackendProfileV1;

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
    fn semantic_memory_backend_profile_locks_provekv_candidate_boundary() {
        let profile = SemanticMemoryBackendProfileV1::provekv_derived_candidate();
        assert!(profile.validate().is_ok());
        assert_eq!(profile.owner, "semantic-memory");
        assert_eq!(
            profile.derived_candidate_backend,
            "provekv_pool_candidate_then_exact_f32"
        );
        assert!(profile.exact_f32_rerank_required);
        assert!(!profile.direct_provekv_dependency_allowed);
        assert!(!profile.recall_integration_allowed);
    }

    #[test]
    fn memory_profile_discloses_partial_non_owner_status() {
        let status = ProfileMemoryStatus::default();
        assert!(!status.enabled);
        assert!(status.note.contains(SUPPORT_TIER));
        assert!(status.note.contains("not a canonical memory truth owner"));
    }
}
