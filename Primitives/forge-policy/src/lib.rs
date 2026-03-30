//! # forge-policy
//!
//! Filesystem, environment, and SQLite guardrails for internal forge runtimes.
//!
//! This Tier 3 crate centralizes low-level path safety, patch-cap checks,
//! allowlisting, and database identity validation. It is intentionally policy
//! infrastructure rather than an execution or storage layer.

use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("refuse to open database: {reason}")]
    RefuseToOpenDb { reason: String },

    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Violation {
    pub kind: ViolationKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ViolationKind {
    ForbiddenPath,
    CapExceeded,
    EmptyPatch,
    UselessEdit,
    DegenerateRange,
    InvalidOccurrence,
    DuplicatePath,
}

#[derive(Debug, Clone)]
pub struct DbIdentitySpec<'a> {
    pub min_user_version: u32,
    pub max_user_version: u32,
    pub required_tables: &'a [&'a str],
    pub schema_table: &'a str,
    pub schema_key: &'a str,
    pub expected_schema_hash: &'a str,
}

const ALLOWED_ENV_PREFIXES: &[&str] = &[
    "PATH",
    "HOME",
    "USER",
    "LOGNAME",
    "LANG",
    "LC_",
    "LANGUAGE",
    "TERM",
    "COLORTERM",
    "SHELL",
    "CARGO",
    "RUSTUP",
    "RUST",
    "XDG_",
    "TMPDIR",
    "TMP",
    "TEMP",
    "CC",
    "CXX",
    "LD_",
    "PKG_CONFIG",
    "SCCACHE",
    "CCACHE",
];

pub fn verify_sqlite_db_identity(
    path: &Path,
    spec: &DbIdentitySpec<'_>,
) -> Result<(), PolicyError> {
    if !path.exists() {
        return Ok(());
    }

    use std::io::Read;

    let mut file = std::fs::File::open(path).map_err(|error| PolicyError::RefuseToOpenDb {
        reason: format!("cannot read file: {error}"),
    })?;
    let mut magic = [0_u8; 16];
    let count = file
        .read(&mut magic)
        .map_err(|error| PolicyError::RefuseToOpenDb {
            reason: format!("cannot read magic bytes: {error}"),
        })?;
    if count < 16 || magic != *b"SQLite format 3\0" {
        return Err(PolicyError::RefuseToOpenDb {
            reason: "file is not a valid SQLite database".to_string(),
        });
    }

    let connection = rusqlite::Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|error| PolicyError::RefuseToOpenDb {
        reason: format!("cannot open database: {error}"),
    })?;

    let user_version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|error| PolicyError::RefuseToOpenDb {
            reason: format!("cannot read user_version: {error}"),
        })?;
    if !(spec.min_user_version..=spec.max_user_version).contains(&user_version) {
        return Err(PolicyError::RefuseToOpenDb {
            reason: format!(
                "user_version {user_version} is outside valid range [{}, {}]",
                spec.min_user_version, spec.max_user_version
            ),
        });
    }

    let has_schema_table: bool = connection
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name = ?1",
            [spec.schema_table],
            |row| row.get(0),
        )
        .map_err(|error| PolicyError::RefuseToOpenDb {
            reason: format!("cannot query sqlite_master: {error}"),
        })?;
    if !has_schema_table {
        return Err(PolicyError::RefuseToOpenDb {
            reason: format!("table '{}' does not exist", spec.schema_table),
        });
    }

    let query = format!("SELECT value FROM {} WHERE key = ?1", spec.schema_table);
    let stored_hash: String = connection
        .query_row(query.as_str(), [spec.schema_key], |row| row.get(0))
        .map_err(|_| PolicyError::RefuseToOpenDb {
            reason: format!("{} row '{}' not found", spec.schema_table, spec.schema_key),
        })?;
    if stored_hash != spec.expected_schema_hash {
        return Err(PolicyError::RefuseToOpenDb {
            reason: format!(
                "schema_hash mismatch: stored={stored_hash}, expected={}",
                spec.expected_schema_hash
            ),
        });
    }

    let existing_tables: Vec<String> = connection
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")
        .map_err(|error| PolicyError::RefuseToOpenDb {
            reason: format!("cannot query sqlite_master for tables: {error}"),
        })?
        .query_map([], |row| row.get(0))
        .map_err(|error| PolicyError::RefuseToOpenDb {
            reason: format!("cannot iterate tables: {error}"),
        })?
        .filter_map(|row| row.ok())
        .collect();

    for table in spec.required_tables {
        if !existing_tables.iter().any(|existing| existing == table) {
            return Err(PolicyError::RefuseToOpenDb {
                reason: format!("required table '{}' missing from database", table),
            });
        }
    }

    Ok(())
}

pub fn ensure_relative_path(path: &Path) -> Result<(), PolicyError> {
    if path.is_absolute() {
        return Err(PolicyError::InvalidPath(format!(
            "path '{}' is absolute",
            path.display()
        )));
    }

    for component in path.components() {
        if component == std::path::Component::ParentDir {
            return Err(PolicyError::InvalidPath(format!(
                "path '{}' contains '..'",
                path.display()
            )));
        }
    }

    Ok(())
}

pub fn reject_symlinks(root: &Path, relative_path: &Path) -> Result<(), PolicyError> {
    let mut current = root.to_path_buf();
    for component in relative_path.components() {
        current.push(component);
        if current.exists() {
            let metadata = std::fs::symlink_metadata(&current)?;
            if metadata.file_type().is_symlink() {
                return Err(PolicyError::InvalidPath(format!(
                    "symlink detected at {}",
                    current.display()
                )));
            }
        }
    }
    Ok(())
}

pub fn resolve_workspace_path(root: &Path, relative_path: &Path) -> Result<PathBuf, PolicyError> {
    ensure_relative_path(relative_path)?;
    reject_symlinks(root, relative_path)?;

    let full_path = root.join(relative_path);
    if let Ok(root_canonical) = root.canonicalize() {
        if let Some(parent) = full_path.parent() {
            if parent.exists() {
                let parent_canonical = parent.canonicalize()?;
                if !parent_canonical.starts_with(&root_canonical) {
                    return Err(PolicyError::InvalidPath(format!(
                        "resolved path '{}' escapes workspace '{}'",
                        parent_canonical.display(),
                        root_canonical.display()
                    )));
                }
            }
        }
    }

    Ok(full_path)
}

pub fn validate_forbidden_paths(
    paths: &[String],
    forbidden_patterns: &[String],
    allow_test_modifications: bool,
) -> Vec<Violation> {
    let hardcoded_denies: &[&str] = &[
        "tests/**",
        "**/*_test.rs",
        "**/fixtures/**",
        "**/*.snap",
        "Cargo.lock",
    ];

    let mut violations = Vec::new();
    for path in paths {
        if !allow_test_modifications {
            for pattern in hardcoded_denies {
                if glob_matches(pattern, path) {
                    violations.push(Violation {
                        kind: ViolationKind::ForbiddenPath,
                        message: format!(
                            "path '{}' matches hardcoded forbidden pattern '{}'",
                            path, pattern
                        ),
                    });
                    break;
                }
            }
        }

        for pattern in forbidden_patterns {
            if !allow_test_modifications && hardcoded_denies.contains(&pattern.as_str()) {
                continue;
            }
            if glob_matches(pattern, path) {
                violations.push(Violation {
                    kind: ViolationKind::ForbiddenPath,
                    message: format!("path '{}' matches forbidden pattern '{}'", path, pattern),
                });
                break;
            }
        }
    }

    violations
}

pub fn validate_patch_caps(
    files_changed: usize,
    total_lines_changed: usize,
    per_file_lines: &[usize],
    max_files: usize,
    max_total_lines: usize,
    max_per_file: usize,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    if files_changed > max_files {
        violations.push(Violation {
            kind: ViolationKind::CapExceeded,
            message: format!("files changed ({files_changed}) exceeds cap ({max_files})"),
        });
    }

    if total_lines_changed > max_total_lines {
        violations.push(Violation {
            kind: ViolationKind::CapExceeded,
            message: format!(
                "total lines changed ({total_lines_changed}) exceeds cap ({max_total_lines})"
            ),
        });
    }

    for (index, lines) in per_file_lines.iter().enumerate() {
        if *lines > max_per_file {
            violations.push(Violation {
                kind: ViolationKind::CapExceeded,
                message: format!(
                    "file {index}: lines changed ({lines}) exceeds per-file cap ({max_per_file})"
                ),
            });
        }
    }

    violations
}

pub fn is_env_allowed(key: &str) -> bool {
    ALLOWED_ENV_PREFIXES
        .iter()
        .any(|prefix| key.starts_with(prefix))
}

fn glob_matches(pattern: &str, path: &str) -> bool {
    match glob::Pattern::new(pattern) {
        Ok(glob) => glob.matches(path),
        Err(error) => {
            tracing::warn!("invalid glob pattern '{}': {}", pattern, error);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::path::Path;

    #[test]
    fn path_guard_rejects_absolute_and_parent_components() {
        assert!(ensure_relative_path(Path::new("src/lib.rs")).is_ok());
        assert!(ensure_relative_path(Path::new("/tmp/nope")).is_err());
        assert!(ensure_relative_path(Path::new("../nope")).is_err());
    }

    #[test]
    fn path_guard_rejects_multiple_escape_shapes() {
        for candidate in [
            "/tmp/escape",
            "../escape",
            "nested/../../escape",
            "./../escape",
        ] {
            assert!(
                ensure_relative_path(Path::new(candidate)).is_err(),
                "expected '{candidate}' to be rejected"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn reject_symlinks_blocks_existing_symlink_segments() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("linked")).unwrap();

        let error = reject_symlinks(root.path(), Path::new("linked/escape.txt")).unwrap_err();
        assert!(matches!(error, PolicyError::InvalidPath(_)));
        assert!(error.to_string().contains("symlink detected"));
    }

    #[cfg(unix)]
    #[test]
    fn resolve_workspace_path_rejects_symlinked_parent_escape() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("dir")).unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("dir/link")).unwrap();

        let error =
            resolve_workspace_path(root.path(), Path::new("dir/link/outside.txt")).unwrap_err();
        assert!(matches!(error, PolicyError::InvalidPath(_)));
    }

    #[test]
    fn env_allowlist_matches_expected_prefixes() {
        assert!(is_env_allowed("CARGO_HOME"));
        assert!(is_env_allowed("PATH"));
        assert!(!is_env_allowed("AWS_SECRET_ACCESS_KEY"));
    }

    proptest! {
        #[test]
        fn relative_path_guard_rejects_parent_escape_shapes(path_tail in "[A-Za-z0-9_./-]{1,24}") {
            for candidate in [
                format!("../{path_tail}"),
                format!("nested/../../{path_tail}"),
                format!("./../{path_tail}"),
            ] {
                prop_assert!(ensure_relative_path(Path::new(&candidate)).is_err());
            }
        }

        #[test]
        fn relative_path_guard_accepts_safe_relative_shapes(path_tail in "[A-Za-z0-9_./-]{1,24}") {
            prop_assume!(!path_tail.starts_with('/'));
            prop_assume!(!path_tail.contains(".."));
            let path = format!("safe/{path_tail}");
            prop_assert!(ensure_relative_path(Path::new(&path)).is_ok());
        }
    }
}
