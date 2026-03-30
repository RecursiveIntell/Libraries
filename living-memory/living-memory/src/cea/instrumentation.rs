use crate::error::ForgeResult;
use crate::exec::backend::{CheckResult, EffectSignature};
use crate::invariants;
use crate::runtime::patch::apply::LineAttributionMap;
use crate::runtime::patch::types::*;

pub use cea_core::{AttributedRunResult, AttributionTriple, EditOpSignature};

/// Build EditOpSignature from an EditOp, enforcing invariant I9 (no raw source).
pub fn build_edit_op_signature(
    op: &EditOp,
    op_idx: usize,
    total_ops: usize,
    file_idx: usize,
    total_files: usize,
    file_extension: &str,
) -> ForgeResult<EditOpSignature> {
    let sig = cea_core::build_edit_op_signature(
        op,
        op_idx,
        total_ops,
        file_idx,
        total_files,
        file_extension,
    )?;
    let sig_json = serde_json::to_string(&sig)?;
    invariants::validate_cea_no_raw_source(&sig_json)?;
    Ok(sig)
}

/// Compute the node ID for an EditOpSignature.
pub fn edit_op_node_id(sig: &EditOpSignature) -> String {
    cea_core::edit_op_node_id(sig)
}

/// Compute the node ID for an EffectSignature.
pub fn effect_node_id(sig: &EffectSignature) -> String {
    cea_core::effect_node_id(sig)
}

/// Attribute effects to edit ops based on line distance in patched-file space.
///
/// Uses the `LineAttributionMap` to translate original line numbers into
/// patched-space positions, so that distance-to-error calculations are
/// accurate after insertions/deletions have shifted lines.
pub fn attribute_effects(
    patch: &StructuredPatch,
    check_result: &CheckResult,
    line_map: &LineAttributionMap,
    max_line_distance: u32,
) -> ForgeResult<Vec<AttributionTriple>> {
    Ok(cea_core::attribute_effects(
        patch,
        check_result,
        line_map,
        max_line_distance,
    )?)
}

/// Compute the run hash for idempotency.
pub fn compute_run_hash(result: &AttributedRunResult) -> String {
    cea_core::compute_run_hash(result)
}
