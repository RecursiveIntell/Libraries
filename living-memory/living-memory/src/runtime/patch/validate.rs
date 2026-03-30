use crate::config::ForgeConfig;
use crate::runtime::patch::types::*;

pub use typed_patch::ValidationResult;

/// Validate a StructuredPatch against policy and invariant rules.
///
/// Returns all violations (not fail-fast).
pub fn validate_patch(patch: &StructuredPatch, config: &ForgeConfig) -> ValidationResult {
    typed_patch::validate_patch(
        patch,
        &typed_patch::PatchPolicy {
            forbidden_paths: config.forbidden_paths.clone(),
            allow_test_modifications: config.allow_test_modifications,
            max_files_changed: config.caps.max_files_changed,
            max_total_lines_changed: config.caps.max_total_lines_changed,
            max_lines_changed_per_file: config.caps.max_lines_changed_per_file,
        },
    )
}
