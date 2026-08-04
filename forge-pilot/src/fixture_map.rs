//! Fixture path resolution for kernel-generated syndromes.
//!
//! Bridges the gap between abstract kernel inference nodes (node_id, syndrome
//! signature) and concrete forge-engine fixture paths (crate root directories
//! with Cargo.toml). Built once per observation cycle from the workspace's
//! Cargo.toml layout.
//!
//! Fail-open contract: if a fixture cannot be resolved, returns `None` —
//! the caller falls back to existing oracle/advisory plan selection.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Maps crate names to concrete fixture paths.
///
/// Built by scanning the resolved workspace for Cargo.toml files during
/// observation. Each Cargo.toml directory becomes a fixture entry.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FixtureMap {
    /// crate_name (from Cargo.toml [package].name) → fixture path
    crate_map: BTreeMap<String, PathBuf>,
    /// All known fixture paths (for fallback resolution)
    fixture_paths: Vec<PathBuf>,
    /// The resolved workspace root
    workspace_path: PathBuf,
}

impl FixtureMap {
    /// Build from a workspace root by scanning for Cargo.toml files.
    ///
    /// Walks up to `max_depth` levels deep. Each directory containing a
    /// Cargo.toml becomes a fixture entry keyed by its package name.
    pub fn from_workspace(workspace_path: &Path) -> Self {
        let mut crate_map = BTreeMap::new();
        let mut fixture_paths = Vec::new();

        if let Ok(entries) = fs::read_dir(workspace_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let cargo_toml = path.join("Cargo.toml");
                    if cargo_toml.exists() {
                        if let Ok(name) = Self::extract_package_name(&cargo_toml) {
                            crate_map.insert(name, path.clone());
                        }
                        fixture_paths.push(path);
                    } else {
                        // Recurse one level for nested crates
                        Self::scan_nested(&path, &mut crate_map, &mut fixture_paths);
                    }
                }
            }
        }

        FixtureMap {
            crate_map,
            fixture_paths,
            workspace_path: workspace_path.to_path_buf(),
        }
    }

    fn scan_nested(
        dir: &Path,
        crate_map: &mut BTreeMap<String, PathBuf>,
        fixture_paths: &mut Vec<PathBuf>,
    ) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && !Self::is_hidden_or_target(&path) {
                    let cargo_toml = path.join("Cargo.toml");
                    if cargo_toml.exists() {
                        if let Ok(name) = Self::extract_package_name(&cargo_toml) {
                            crate_map.entry(name).or_insert_with(|| path.clone());
                        }
                        fixture_paths.push(path);
                    }
                }
            }
        }
    }

    fn is_hidden_or_target(path: &Path) -> bool {
        path.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with('.') || n == "target")
    }

    fn extract_package_name(cargo_toml: &Path) -> Result<String, std::io::Error> {
        let content = fs::read_to_string(cargo_toml)?;
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("name = \"") {
                if let Some(name) = rest.strip_suffix('"') {
                    return Ok(name.to_string());
                }
            }
        }
        // Fall back to directory name
        cargo_toml
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(String::from)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no package name"))
    }

    /// Resolve a syndrome to its most likely fixture path.
    ///
    /// Tries to extract a crate name from the syndrome signature (which often
    /// contains identifiers like "cea-core", "forge-engine", etc.) and looks
    /// it up in the crate map. Falls back to the first fixture path if no
    /// match is found.
    ///
    /// Returns `None` only if no fixture paths exist in the workspace.
    pub fn resolve_fixture_from_signature(&self, signature: &str) -> Option<PathBuf> {
        // Try exact crate name match
        for (name, path) in &self.crate_map {
            if signature.contains(name.as_str()) {
                return Some(path.clone());
            }
        }

        // Try word-by-word match
        for word in signature.split(|c: char| !c.is_alphanumeric() && c != '-') {
            if let Some(path) = self.crate_map.get(word) {
                return Some(path.clone());
            }
        }

        // No match found — fail open, return None so caller falls back
        // to advisory/oracle plan selection. Never guess a fixture.
        None
    }

    /// Returns the resolved workspace root.
    pub fn workspace_path(&self) -> &Path {
        &self.workspace_path
    }

    /// Returns the number of crates discovered.
    pub fn crate_count(&self) -> usize {
        self.fixture_paths.len()
    }

    /// Returns true if any fixtures were discovered.
    pub fn has_fixtures(&self) -> bool {
        !self.fixture_paths.is_empty()
    }

    /// Returns all discovered fixture paths.
    pub fn fixture_paths(&self) -> &[PathBuf] {
        &self.fixture_paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn empty_workspace_returns_no_fixtures() {
        let tmp = std::env::temp_dir().join("fixture_map_test_empty");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let map = FixtureMap::from_workspace(&tmp);
        assert!(!map.has_fixtures());
        assert_eq!(map.crate_count(), 0);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn extract_package_name_parses_toml() {
        let tmp = std::env::temp_dir().join("fixture_map_test_pkg");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let cargo_toml = tmp.join("Cargo.toml");
        fs::write(
            &cargo_toml,
            "[package]\nname = \"test-crate\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let name = FixtureMap::extract_package_name(&cargo_toml).unwrap();
        assert_eq!(name, "test-crate");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn resolve_signature_finds_crate() {
        let tmp = std::env::temp_dir().join("fixture_map_test_resolve");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let cea_dir = tmp.join("cea-core");
        fs::create_dir_all(&cea_dir).unwrap();
        fs::write(
            cea_dir.join("Cargo.toml"),
            "[package]\nname = \"cea-core\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        let map = FixtureMap::from_workspace(&tmp);
        assert!(map.has_fixtures());

        let resolved = map
            .resolve_fixture_from_signature("missing claim family for cea-core entity X")
            .unwrap();
        assert!(resolved.ends_with("cea-core"));

        let _ = fs::remove_dir_all(&tmp);
    }
}
