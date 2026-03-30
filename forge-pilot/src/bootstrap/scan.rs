use crate::bootstrap::classify::{classify_language, is_supported_text_file};
use crate::bootstrap::ignore::should_skip_workspace_dir;
use crate::bootstrap::types::{
    BootstrapSourceSkippedFile, SourceFileRecord, MAX_SOURCE_FILE_BYTES,
};
use crate::error::PilotError;
use stack_ids::ContentDigest;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceScan {
    pub files: Vec<SourceFileRecord>,
    pub skipped_files: Vec<BootstrapSourceSkippedFile>,
}

pub(crate) fn scan_workspace_source_files(
    workspace_path: &Path,
) -> Result<WorkspaceScan, PilotError> {
    let mut files = Vec::new();
    let mut skipped_files = Vec::new();
    let mut stack = vec![workspace_path.to_path_buf()];

    while let Some(path) = stack.pop() {
        let entries = fs::read_dir(&path).map_err(|error| {
            PilotError::Other(format!(
                "failed to read workspace directory {}: {}",
                path.display(),
                error
            ))
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                PilotError::Other(format!(
                    "failed to read workspace entry under {}: {}",
                    path.display(),
                    error
                ))
            })?;
            let entry_path = entry.path();
            let file_type = entry.file_type().map_err(|error| {
                PilotError::Other(format!(
                    "failed to inspect workspace entry {}: {}",
                    entry_path.display(),
                    error
                ))
            })?;
            if file_type.is_dir() {
                if should_skip_workspace_dir(&entry_path, None) {
                    continue;
                }
                stack.push(entry_path);
                continue;
            }
            if !file_type.is_file() {
                continue;
            }

            let relative_path = normalize_relative_path(workspace_path, &entry_path)?;
            if !is_supported_text_file(&relative_path) {
                continue;
            }

            let bytes = fs::read(&entry_path).map_err(|error| {
                PilotError::Other(format!(
                    "failed to read source file {}: {}",
                    entry_path.display(),
                    error
                ))
            })?;
            if bytes.len() > MAX_SOURCE_FILE_BYTES {
                skipped_files.push(BootstrapSourceSkippedFile {
                    path: relative_path,
                    reason: format!(
                        "file exceeds {} byte bootstrap limit",
                        MAX_SOURCE_FILE_BYTES
                    ),
                });
                continue;
            }

            let content = match String::from_utf8(bytes) {
                Ok(content) => content,
                Err(_) => {
                    skipped_files.push(BootstrapSourceSkippedFile {
                        path: relative_path,
                        reason: "file is not valid UTF-8 text".into(),
                    });
                    continue;
                }
            };

            let content_digest = ContentDigest::compute(content.as_bytes());
            let line_count = content.lines().count();
            let byte_count = content.len();
            let file_extension = Path::new(&relative_path)
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_string());
            let language = classify_language(&relative_path)
                .unwrap_or("generic")
                .to_string();

            files.push(SourceFileRecord {
                relative_path,
                content,
                content_digest,
                line_count,
                byte_count,
                file_extension,
                language,
            });
        }
    }

    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    skipped_files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(WorkspaceScan {
        files,
        skipped_files,
    })
}

pub(crate) fn normalize_relative_path(root: &Path, path: &Path) -> Result<String, PilotError> {
    let relative = path.strip_prefix(root).map_err(|error| {
        PilotError::Other(format!(
            "failed to normalize source path {} relative to {}: {}",
            path.display(),
            root.display(),
            error
        ))
    })?;
    Ok(relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/"))
}
