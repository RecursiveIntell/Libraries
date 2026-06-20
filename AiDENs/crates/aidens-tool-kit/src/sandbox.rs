use aidens_security_kit::{validate_sandbox_path, PathSafetyError};
use anyhow::{anyhow, bail, Context};
use serde_json::Value;
use std::path::{Component, Path, PathBuf};

pub(crate) fn canonical_sandbox_root(path: &Path) -> anyhow::Result<PathBuf> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("sandbox root does not exist: {}", path.display()))?;
    if !canonical.is_dir() {
        bail!("sandbox root is not a directory: {}", canonical.display())
    }
    Ok(canonical)
}

pub(crate) fn resolve_existing_sandboxed_path(
    sandbox_root: &Path,
    requested: &str,
) -> anyhow::Result<PathBuf> {
    let requested_path = Path::new(requested);
    if requested_path.is_absolute() {
        bail!("absolute path escape rejected by sandbox")
    }
    if requested_path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        bail!("path traversal rejected by sandbox: {requested}")
    }

    let joined = sandbox_root.join(requested_path);
    let canonical = joined.canonicalize().with_context(|| {
        format!("repo-read path cannot be resolved inside sandbox: {requested}")
    })?;
    validate_sandbox_path(&canonical, sandbox_root)
        .map_err(|error| anyhow!(path_safety_message(error, requested)))?;
    Ok(canonical)
}

pub(crate) fn resolve_target_sandboxed_path(
    sandbox_root: &Path,
    requested: &str,
) -> anyhow::Result<PathBuf> {
    let requested_path = Path::new(requested);
    if requested_path.is_absolute() {
        bail!("absolute path escape rejected by sandbox")
    }
    if requested_path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        bail!("path traversal rejected by sandbox: {requested}")
    }
    let joined = sandbox_root.join(requested_path);
    let parent = joined.parent().unwrap_or(sandbox_root);
    let canonical_parent = parent.canonicalize().with_context(|| {
        format!("patch target parent cannot be resolved inside sandbox: {requested}")
    })?;
    validate_sandbox_path(&canonical_parent, sandbox_root)
        .map_err(|error| anyhow!(path_safety_message(error, requested)))?;
    let file_name = joined
        .file_name()
        .ok_or_else(|| anyhow!("patch target must name a file: {requested}"))?;
    if let Ok(metadata) = std::fs::symlink_metadata(&joined) {
        if metadata.file_type().is_symlink() {
            bail!("symlink write target rejected by sandbox: {requested}");
        }
        reject_hardlinked_file(&joined, &metadata)?;
        let canonical_target = joined.canonicalize().with_context(|| {
            format!("patch target cannot be resolved inside sandbox: {requested}")
        })?;
        validate_sandbox_path(&canonical_target, sandbox_root)
            .map_err(|error| anyhow!(path_safety_message(error, requested)))?;
    }
    Ok(canonical_parent.join(file_name))
}

pub(crate) fn path_safety_message(error: PathSafetyError, requested: &str) -> String {
    match error {
        PathSafetyError::TraversalNotAllowed => {
            format!("path traversal rejected by sandbox: {requested}")
        }
        PathSafetyError::OutsideSandbox { root } => {
            let _ = root;
            format!("path escape rejected by sandbox: {requested}; outside declared sandbox root")
        }
        PathSafetyError::SensitivePrefix { prefix } => {
            format!("sensitive prefix rejected by sandbox: {requested}; prefix {prefix}")
        }
        PathSafetyError::HiddenOrSensitiveComponent { component } => {
            format!("hidden or sensitive component rejected by sandbox: {requested}; component {component}")
        }
    }
}

#[cfg(unix)]
pub(crate) fn reject_hardlinked_file(
    path: &Path,
    metadata: &std::fs::Metadata,
) -> anyhow::Result<()> {
    use std::os::unix::fs::MetadataExt;

    if metadata.is_file() && metadata.nlink() > 1 {
        bail!(
            "hardlink read target rejected by sandbox: {}",
            display_redacted_path(path)
        );
    }
    Ok(())
}

#[cfg(not(unix))]
pub(crate) fn reject_hardlinked_file(
    _path: &Path,
    _metadata: &std::fs::Metadata,
) -> anyhow::Result<()> {
    Ok(())
}

pub(crate) fn display_redacted_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("<sandbox>/{name}"))
        .unwrap_or_else(|| "<sandbox>/<unknown>".into())
}

pub(crate) fn path_is_denied_by_prefix(sandbox_root: &Path, path: &Path) -> bool {
    validate_sandbox_path(path, sandbox_root).is_err()
}

pub(crate) fn display_sandbox_path(sandbox_root: &Path, path: &Path) -> String {
    path.strip_prefix(sandbox_root)
        .map(|relative| relative.display().to_string().replace('\\', "/"))
        .unwrap_or_else(|_| display_redacted_path(path))
}

pub(crate) fn collect_search_matches(
    sandbox_root: &Path,
    current: &Path,
    query: &str,
    max_matches: usize,
    matches: &mut Vec<Value>,
) -> anyhow::Result<()> {
    if matches.len() >= max_matches || path_is_denied_by_prefix(sandbox_root, current) {
        return Ok(());
    }
    let metadata = std::fs::symlink_metadata(current)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in std::fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            let display = display_sandbox_path(sandbox_root, &path);
            if path_has_denied_component(Path::new(&display)) {
                continue;
            }
            collect_search_matches(sandbox_root, &path, query, max_matches, matches)?;
            if matches.len() >= max_matches {
                break;
            }
        }
    } else if metadata.is_file() && metadata.len() <= 1_048_576 {
        let Ok(content) = std::fs::read_to_string(current) else {
            return Ok(());
        };
        for (line_index, line) in content.lines().enumerate() {
            if line.contains(query) {
                matches.push(serde_json::json!({
                    "path": display_sandbox_path(sandbox_root, current),
                    "line": line_index + 1,
                    "text": line,
                }));
                if matches.len() >= max_matches {
                    break;
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn path_has_denied_component(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            Component::Normal(name) if name == ".git" || name == "target"
        )
    })
}
