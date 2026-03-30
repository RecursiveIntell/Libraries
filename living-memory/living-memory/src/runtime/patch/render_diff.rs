use std::path::Path;

use crate::error::ForgeResult;

/// Generate a unified diff from (original_dir, patched_dir).
///
/// Prefers `git diff --no-index` if git is on PATH.
/// Falls back to internal line-diff producing valid unified format.
///
/// This is async because it may spawn a `git` subprocess.
pub async fn render_diff(original_dir: &Path, patched_dir: &Path) -> ForgeResult<String> {
    Ok(typed_patch::render_diff(original_dir, patched_dir).await?)
}
