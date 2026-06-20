use anyhow::bail;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileReplacement {
    pub(crate) path: String,
    pub(crate) removed: Vec<String>,
    pub(crate) added: Vec<String>,
}

pub(crate) fn touched_paths_from_diff(diff: &str) -> anyhow::Result<Vec<String>> {
    let paths = parse_simple_unified_diff(diff)?
        .into_iter()
        .map(|replacement| replacement.path)
        .collect::<Vec<_>>();
    if paths.is_empty() {
        bail!("patch proposal contains no file paths")
    }
    Ok(paths)
}

pub(crate) fn parse_simple_unified_diff(diff: &str) -> anyhow::Result<Vec<FileReplacement>> {
    let mut replacements = Vec::new();
    let mut current: Option<FileReplacement> = None;
    let mut seen_paths = BTreeSet::new();
    let mut saw_old_path = false;
    for line in diff.lines() {
        if line.starts_with("--- ") {
            saw_old_path = true;
            continue;
        }
        if let Some(path) = line.strip_prefix("+++ ") {
            if !saw_old_path {
                bail!("unsupported unified diff: missing old-path header before new-path header")
            }
            if let Some(replacement) = current.take() {
                replacements.push(replacement);
            }
            let path = normalize_diff_path(path)?;
            if !seen_paths.insert(path.clone()) {
                bail!("ambiguous patch targets the same file more than once: {path}")
            }
            current = Some(FileReplacement {
                path,
                removed: Vec::new(),
                added: Vec::new(),
            });
            saw_old_path = false;
            continue;
        }
        if line.starts_with("@@") {
            continue;
        }
        if let Some(replacement) = current.as_mut() {
            if let Some(removed) = line.strip_prefix('-') {
                replacement.removed.push(removed.to_string());
            } else if let Some(added) = line.strip_prefix('+') {
                replacement.added.push(added.to_string());
            }
        }
    }
    if let Some(replacement) = current.take() {
        replacements.push(replacement);
    }
    replacements.retain(|replacement| {
        !replacement.path.is_empty()
            && (!replacement.removed.is_empty() || !replacement.added.is_empty())
    });
    if replacements.is_empty() {
        bail!("unsupported or empty unified diff")
    }
    if replacements
        .iter()
        .any(|replacement| replacement.removed.is_empty())
    {
        bail!("ambiguous patch lacks removal context")
    }
    Ok(replacements)
}

fn normalize_diff_path(path: &str) -> anyhow::Result<String> {
    let path = path.trim();
    let path = path.strip_prefix("b/").unwrap_or(path);
    if path == "/dev/null" {
        bail!("delete-only patch targets are not supported in P10")
    }
    if path.trim().is_empty() {
        bail!("diff path must not be empty")
    }
    Ok(path.to_string())
}

pub(crate) fn apply_single_replacement(
    before: &str,
    replacement: &FileReplacement,
) -> anyhow::Result<String> {
    let removed = replacement.removed.join("\n");
    let added = replacement.added.join("\n");
    if removed.is_empty() {
        let mut out = before.to_string();
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&added);
        out.push('\n');
        return Ok(out);
    }
    let with_newline = format!("{removed}\n");
    let added_with_newline = format!("{added}\n");
    let newline_matches = before.matches(&with_newline).count();
    if newline_matches == 1 {
        return Ok(before.replacen(&with_newline, &added_with_newline, 1));
    }
    if newline_matches > 1 {
        bail!(
            "ambiguous patch context appears multiple times in {}",
            replacement.path
        )
    }
    let raw_matches = before.matches(&removed).count();
    if raw_matches == 1 {
        return Ok(before.replacen(&removed, &added, 1));
    }
    if raw_matches > 1 {
        bail!(
            "ambiguous patch context appears multiple times in {}",
            replacement.path
        )
    }
    bail!(
        "patch context not found in {}; refused to guess",
        replacement.path
    )
}
