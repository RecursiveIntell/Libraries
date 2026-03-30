use std::path::Path;

use crate::error::ForgeResult;
use crate::runtime::patch::types::*;

pub use typed_patch::LineAttributionMap;

/// Apply a StructuredPatch to a workspace directory.
///
/// Atomic: fails entirely on any op failure (no partial apply).
/// Returns line-mapping table for CEA position attribution.
pub fn apply_patch(
    patch: &StructuredPatch,
    workspace_dir: &Path,
) -> ForgeResult<LineAttributionMap> {
    let fs = sandbox_workspace::LocalPatchFs::new(workspace_dir);
    Ok(typed_patch::apply_patch(patch, &fs)?)
}
