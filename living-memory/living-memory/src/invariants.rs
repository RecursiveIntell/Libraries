use std::path::Path;

use crate::error::{ForgeError, ForgeResult};
use crate::store::schema;

/// Refuses to open a database that is not a verified Forge DB.
///
/// Checks (all must pass):
/// 1. File must be a valid SQLite database (magic bytes check).
/// 2. `PRAGMA user_version` must be within valid range.
/// 3. Table `forge_meta` must exist.
/// 4. `forge_meta` row `key = 'schema_hash'` must exist.
/// 5. `forge_meta.value` for `schema_hash` must equal the compiled-in `FORGE_SCHEMA_HASH`.
pub fn refuse_to_open_db(path: &Path) -> ForgeResult<()> {
    let spec = forge_policy::DbIdentitySpec {
        min_user_version: schema::FORGE_MIN_USER_VERSION,
        max_user_version: schema::FORGE_MAX_USER_VERSION,
        required_tables: schema::REQUIRED_TABLES,
        schema_table: "forge_meta",
        schema_key: "schema_hash",
        expected_schema_hash: schema::forge_schema_hash(),
    };

    forge_policy::verify_sqlite_db_identity(path, &spec).map_err(|error| match error {
        forge_policy::PolicyError::RefuseToOpenDb { reason } => {
            ForgeError::RefuseToOpenDb { reason }
        }
        other => ForgeError::RefuseToOpenDb {
            reason: other.to_string(),
        },
    })
}

/// Validates that patches do not touch forbidden paths.
pub fn validate_forbidden_paths(
    paths: &[&str],
    forbidden_patterns: &[String],
    allow_test_modifications: bool,
) -> Vec<crate::error::Violation> {
    let owned_paths: Vec<String> = paths.iter().map(|path| (*path).to_string()).collect();
    forge_policy::validate_forbidden_paths(
        &owned_paths,
        forbidden_patterns,
        allow_test_modifications,
    )
}

/// Validates patch caps (max files, max total lines, max lines per file).
pub fn validate_patch_caps(
    files_changed: usize,
    total_lines_changed: usize,
    per_file_lines: &[usize],
    max_files: usize,
    max_total_lines: usize,
    max_per_file: usize,
) -> Vec<crate::error::Violation> {
    forge_policy::validate_patch_caps(
        files_changed,
        total_lines_changed,
        per_file_lines,
        max_files,
        max_total_lines,
        max_per_file,
    )
}

/// Validates that a CEA node contains no raw source code.
/// Checks recursively through all nested objects and arrays.
pub fn validate_cea_no_raw_source(sig_json: &str) -> ForgeResult<()> {
    let value: serde_json::Value =
        serde_json::from_str(sig_json).map_err(|_| ForgeError::CeaRawSourceDetected)?;
    check_value_no_raw_source(&value)
}

/// Recursively check a JSON value for forbidden raw-source fields.
fn check_value_no_raw_source(value: &serde_json::Value) -> ForgeResult<()> {
    match value {
        serde_json::Value::Object(obj) => {
            for key in &["context_before", "context_after", "raw_source", "raw_lines"] {
                if obj.contains_key(*key) {
                    return Err(ForgeError::CeaRawSourceDetected);
                }
            }

            // Validate context_hash is a hex string (blake3 = 64 hex chars)
            if let Some(hash) = obj.get("context_hash") {
                if let Some(s) = hash.as_str() {
                    if s.len() != 64 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
                        return Err(ForgeError::CeaRawSourceDetected);
                    }
                }
            }

            // Check file_extension is just an extension, not a full path
            if let Some(ext) = obj.get("file_extension") {
                if let Some(s) = ext.as_str() {
                    if s.contains('/') || s.contains('\\') || s.len() > 10 {
                        return Err(ForgeError::CeaRawSourceDetected);
                    }
                }
            }

            // Recurse into all values
            for v in obj.values() {
                check_value_no_raw_source(v)?;
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                check_value_no_raw_source(v)?;
            }
        }
        _ => {}
    }
    Ok(())
}
