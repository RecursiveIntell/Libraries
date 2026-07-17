//! Immutable Medusa learning-corpus validation and consumer isolation.
//!
//! This adapter owns no corpus truth: v1's manifest, task files, and oracle file
//! remain the canonical fixtures. Holdout oracle bytes are deliberately never
//! represented by [`PublicTask`].

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

const SPLITS: [&str; 3] = ["development", "calibration", "holdout"];
const REQUIRED_NEGATIVE: [&str; 15] = [
    "descendant_process",
    "duplicate_key_tool_call",
    "evaluator_tamper",
    "failing_baseline",
    "malformed_fixture",
    "malformed_tool_call",
    "network",
    "path_escape",
    "real_to_mock_fallback",
    "reward_hacking",
    "rollback_failure",
    "schema_invalid_tool_call",
    "secret_env",
    "stale_dependency_memory",
    "symlink_escape",
];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CorpusError {
    #[error("corpus I/O: {0}")]
    Io(String),
    #[error("schema-invalid corpus metadata: {0}")]
    Schema(String),
    #[error("corpus validation failed: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicTask {
    pub id: String,
    pub family: String,
    pub split: String,
    pub fixture: PathBuf,
    pub required_checks: Vec<String>,
    pub permitted_effects: Vec<String>,
    pub forbidden_effects: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedCorpus {
    pub digest: String,
    pub tasks: Vec<PublicTask>,
}

fn read_json(path: &Path) -> Result<Value, CorpusError> {
    let bytes =
        std::fs::read(path).map_err(|e| CorpusError::Io(format!("{}: {e}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|e| CorpusError::Schema(format!("{}: {e}", path.display())))
}

fn string(v: &Value, key: &str) -> Option<String> {
    v.get(key)?.as_str().map(ToOwned::to_owned)
}
fn strings(v: &Value, key: &str) -> Option<Vec<String>> {
    v.get(key)?
        .as_array()?
        .iter()
        .map(|x| x.as_str().map(ToOwned::to_owned))
        .collect()
}

/// Validate the committed v1 corpus and return only runner-safe task metadata.
pub fn validate_and_consume(root: impl AsRef<Path>) -> Result<ValidatedCorpus, CorpusError> {
    let root = root.as_ref();
    let manifest = read_json(&root.join("manifest.json"))?;
    let oracles = read_json(&root.join("oracles.json"))?;
    if string(&manifest, "corpus_version").as_deref() != Some("v1")
        || string(&manifest, "frozen_baseline_policy").as_deref()
            != Some("immutable-after-treatment")
    {
        return Err(CorpusError::Invalid(
            "unpinned corpus version or baseline policy".into(),
        ));
    }
    if string(&manifest, "toolchain").as_deref() != Some("rust-toolchain.toml@1.86.0")
        || string(&manifest, "verifier_manifest").as_deref()
            != Some("verifier-v1@sha256:medusa-local-v1")
    {
        return Err(CorpusError::Invalid("unpinned verifier/toolchain".into()));
    }
    let tasks = manifest
        .get("tasks")
        .and_then(Value::as_array)
        .ok_or_else(|| CorpusError::Invalid("too few tasks".into()))?;
    if tasks.len() < 5 {
        return Err(CorpusError::Invalid("too few tasks".into()));
    }
    let families: BTreeSet<String> = tasks.iter().filter_map(|t| string(t, "family")).collect();
    if families.len() < 5 {
        return Err(CorpusError::Invalid("too few families".into()));
    }
    let mut split_counts = BTreeMap::<String, usize>::new();
    let mut ids = BTreeSet::new();
    let mut fixtures = BTreeSet::new();
    let mut family_splits = BTreeSet::new();
    let canonical = manifest
        .get("canonical_fixture_digests")
        .and_then(Value::as_object)
        .ok_or_else(|| CorpusError::Invalid("missing canonical fixture digests".into()))?;
    let mut public = Vec::new();
    for task in tasks {
        let id = string(task, "id")
            .ok_or_else(|| CorpusError::Invalid("missing task field id".into()))?;
        let family = string(task, "family")
            .ok_or_else(|| CorpusError::Invalid("missing task field family".into()))?;
        let split = string(task, "split")
            .ok_or_else(|| CorpusError::Invalid("missing task field split".into()))?;
        let fixture = string(task, "fixture")
            .ok_or_else(|| CorpusError::Invalid("missing task field fixture".into()))?;
        let required_checks = strings(task, "required_checks")
            .ok_or_else(|| CorpusError::Invalid("missing task field required_checks".into()))?;
        let permitted_effects = strings(task, "permitted_effects")
            .ok_or_else(|| CorpusError::Invalid("missing task field permitted_effects".into()))?;
        let forbidden_effects = strings(task, "forbidden_effects")
            .ok_or_else(|| CorpusError::Invalid("missing task field forbidden_effects".into()))?;
        if !ids.insert(id.clone()) {
            return Err(CorpusError::Invalid(format!("duplicate task id: {id}")));
        }
        if !SPLITS.contains(&split.as_str()) {
            return Err(CorpusError::Invalid("invalid split".into()));
        }
        if !family_splits.insert((family.clone(), split.clone())) {
            return Err(CorpusError::Invalid(format!(
                "duplicate family split: ({family}, {split})"
            )));
        }
        if !fixtures.insert(fixture.clone()) {
            return Err(CorpusError::Invalid(format!(
                "duplicate fixture: {fixture}"
            )));
        }
        let rel = Path::new(&fixture);
        if rel.is_absolute() || rel.components().any(|c| c.as_os_str() == "..") {
            return Err(CorpusError::Invalid(format!(
                "fixture path escapes corpus: {fixture}"
            )));
        }
        let path = root.join(rel);
        if !path.is_file() {
            return Err(CorpusError::Invalid(format!("missing fixture {fixture}")));
        }
        let expected = canonical
            .get(&fixture)
            .and_then(Value::as_str)
            .ok_or_else(|| CorpusError::Invalid(format!("missing canonical digest: {fixture}")))?;
        let actual = sha256_file(&path)?;
        if actual != expected {
            return Err(CorpusError::Invalid(format!(
                "fixture digest mismatch: {fixture}"
            )));
        }
        if required_checks.is_empty()
            || permitted_effects.is_empty()
            || forbidden_effects.is_empty()
        {
            return Err(CorpusError::Invalid(
                "empty required checks/effects denominator".into(),
            ));
        }
        *split_counts.entry(split.clone()).or_default() += 1;
        public.push(PublicTask {
            id,
            family,
            split,
            fixture: path,
            required_checks,
            permitted_effects,
            forbidden_effects,
        });
    }
    if SPLITS
        .iter()
        .any(|s| split_counts.get(*s).copied().unwrap_or(0) == 0)
    {
        return Err(CorpusError::Invalid("empty split denominator".into()));
    }
    // v1 was frozen with one development, calibration, and holdout case per
    // family. Validate that immutable layout exactly; a 60/20/20 treatment
    // corpus must be introduced as a superseding version rather than rewriting
    // already-observed v1 bytes.
    if SPLITS
        .iter()
        .any(|split| split_counts.get(*split).copied() != Some(families.len()))
    {
        return Err(CorpusError::Invalid(format!(
            "v1 family/split coverage mismatch: observed {split_counts:?}"
        )));
    }
    if canonical
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        != fixtures.iter().map(String::as_str).collect()
    {
        return Err(CorpusError::Invalid(
            "canonical fixture digest inventory mismatch".into(),
        ));
    }
    let oracle_families: BTreeSet<String> = oracles
        .get("families")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|x| string(x, "family"))
        .collect();
    if oracle_families != families
        || oracles
            .get("families")
            .and_then(Value::as_array)
            .map_or(true, |a| {
                a.iter()
                    .any(|x| string(x, "oracle").map_or(true, |s| s.is_empty()))
            })
    {
        return Err(CorpusError::Invalid("invalid oracle schema".into()));
    }
    let required: BTreeSet<&str> = manifest
        .get("required_negative_categories")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let actual_negative: BTreeSet<String> = std::fs::read_dir(root.join("negative"))
        .map_err(|e| CorpusError::Io(e.to_string()))?
        .filter_map(Result::ok)
        .filter_map(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(ToOwned::to_owned)
        })
        .collect();
    if required != REQUIRED_NEGATIVE.into_iter().collect()
        || REQUIRED_NEGATIVE
            .iter()
            .any(|n| !actual_negative.contains(*n))
    {
        return Err(CorpusError::Invalid(
            "negative category inventory drift".into(),
        ));
    }
    if manifest.get("holdout_oracles").is_some() {
        return Err(CorpusError::Invalid("holdout oracle leak".into()));
    }
    Ok(ValidatedCorpus {
        digest: sha256_file(&root.join("manifest.json"))?,
        tasks: public,
    })
}

fn sha256_file(path: &Path) -> Result<String, CorpusError> {
    let out = Command::new("sha256sum")
        .arg(path)
        .output()
        .map_err(|e| CorpusError::Io(e.to_string()))?;
    if !out.status.success() {
        return Err(CorpusError::Io(format!(
            "sha256sum failed for {}",
            path.display()
        )));
    }
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .map(str::to_owned)
        .ok_or_else(|| CorpusError::Io("sha256sum returned no digest".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validator_is_deterministic_and_public_consumer_has_no_oracle() {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/learning-coding-agent/v1");
        let a = validate_and_consume(&root).unwrap();
        let b = validate_and_consume(&root).unwrap();
        assert_eq!(a.digest, b.digest);
        assert_eq!(a.tasks, b.tasks);
        assert!(a
            .tasks
            .iter()
            .all(|t| t.split != "holdout" || !t.fixture.to_string_lossy().contains("oracle")));
    }
}
