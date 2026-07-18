//! Immutable Medusa learning-corpus validation and consumer isolation.
//!
//! This adapter owns no corpus truth: v1's manifest, task files, and oracle file
//! remain the canonical fixtures. Holdout oracle bytes are deliberately never
//! represented by [`PublicTask`].

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use thiserror::Error;
use walkdir::WalkDir;

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

/// Learner-safe projection of one validated executable learning-corpus v2 task.
///
/// This type deliberately has no oracle path, patch, or expected-source field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V2LearnerSafeTask {
    pub task_id: String,
    pub family: String,
    pub split: String,
    pub task_path: PathBuf,
    pub fixture_path: PathBuf,
    pub prompt: String,
    pub verifier: Vec<String>,
    pub baseline_expected: String,
    pub treatment_expected: String,
    pub fixture_tree_digest: String,
    pub oracle_digest: String,
}

/// A statically validated executable learning corpus v2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V2ValidatedCorpus {
    pub corpus_digest: String,
    pub tasks: Vec<V2LearnerSafeTask>,
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
    let mut family_splits = BTreeMap::<String, String>::new();
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
        if let Some(previous_split) = family_splits.get(&family) {
            if previous_split != &split {
                return Err(CorpusError::Invalid(format!(
                    "family leakage across splits: {family}"
                )));
            }
        } else {
            family_splits.insert(family.clone(), split.clone());
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
    let total = tasks.len();
    let expected = [total * 3 / 5, total / 5, total / 5];
    let observed = SPLITS.map(|split| split_counts.get(split).copied().unwrap_or(0));
    if observed != expected {
        return Err(CorpusError::Invalid(format!(
            "split counts must satisfy immutable 60/20/20 ratio: observed {split_counts:?}"
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

/// Validate the committed v2 corpus and return only learner-safe task metadata.
///
/// This validation is static. It neither executes fixtures nor reads evaluator
/// oracle files, and it makes no benchmark or promotion claim.
pub fn validate_and_consume_v2(root: impl AsRef<Path>) -> Result<V2ValidatedCorpus, CorpusError> {
    let root = std::fs::canonicalize(root.as_ref())
        .map_err(|e| CorpusError::Io(format!("{}: {e}", root.as_ref().display())))?;
    let manifest = read_json(&root.join("manifest.json"))?;

    if string(&manifest, "schema").as_deref() != Some("AiDENsExecutableLearningCorpusV2")
        || manifest.get("version").and_then(Value::as_u64) != Some(2)
    {
        return Err(CorpusError::Invalid(
            "v2 corpus schema or version mismatch".into(),
        ));
    }
    if string(&manifest, "claim_scope").as_deref() != Some("local-executable-corpus-only")
        || string(&manifest, "paired_evaluation_status").as_deref()
            != Some("not-run-owner-evidence-required")
    {
        return Err(CorpusError::Invalid(
            "v2 claim scope or paired-evaluation status mismatch".into(),
        ));
    }

    let partition = manifest
        .get("partition_policy")
        .and_then(Value::as_object)
        .ok_or_else(|| CorpusError::Invalid("missing v2 partition policy".into()))?;
    let declared_counts = partition
        .get("counts")
        .and_then(Value::as_object)
        .ok_or_else(|| CorpusError::Invalid("missing v2 partition counts".into()))?;
    if partition.get("family_clustered").and_then(Value::as_bool) != Some(true)
        || partition
            .get("oracle_excluded_from_public_task")
            .and_then(Value::as_bool)
            != Some(true)
        || declared_counts.get("development").and_then(Value::as_u64) != Some(9)
        || declared_counts.get("calibration").and_then(Value::as_u64) != Some(3)
        || declared_counts.get("holdout").and_then(Value::as_u64) != Some(3)
    {
        return Err(CorpusError::Invalid(
            "v2 partition policy declaration mismatch".into(),
        ));
    }

    let expected_corpus_digest = string(&manifest, "corpus_digest")
        .filter(|digest| !digest.is_empty())
        .ok_or_else(|| CorpusError::Invalid("missing v2 corpus digest".into()))?;
    let mut digest_material = manifest.clone();
    digest_material
        .as_object_mut()
        .ok_or_else(|| CorpusError::Invalid("v2 manifest must be an object".into()))?
        .remove("corpus_digest");
    let actual_corpus_digest = canonical_json_sha256(&digest_material)?;
    if actual_corpus_digest != expected_corpus_digest {
        return Err(CorpusError::Invalid("v2 corpus digest mismatch".into()));
    }

    let tasks = manifest
        .get("tasks")
        .and_then(Value::as_array)
        .ok_or_else(|| CorpusError::Invalid("missing v2 task inventory".into()))?;
    if tasks.len() != 15 {
        return Err(CorpusError::Invalid(format!(
            "v2 corpus must contain exactly 15 tasks, found {}",
            tasks.len()
        )));
    }

    let mut ids = BTreeSet::new();
    let mut task_paths = BTreeSet::new();
    let mut fixture_paths = BTreeSet::new();
    let mut task_digests = BTreeSet::new();
    let mut fixture_digests = BTreeSet::new();
    let mut oracle_digests = BTreeSet::new();
    let mut families = BTreeSet::new();
    let mut family_splits = BTreeMap::<String, String>::new();
    let mut split_counts = BTreeMap::<String, usize>::new();
    let mut public_tasks = Vec::with_capacity(tasks.len());

    for manifest_task in tasks {
        let task_id = required_string(manifest_task, "task_id")?;
        let family = required_string(manifest_task, "family")?;
        let split = required_string(manifest_task, "split")?;
        let task_path_text = required_string(manifest_task, "task_path")?;
        let fixture_path_text = required_string(manifest_task, "fixture_path")?;
        let expected_task_digest = required_string(manifest_task, "task_digest")?;
        let expected_fixture_digest = required_string(manifest_task, "fixture_tree_digest")?;
        let oracle_digest = required_string(manifest_task, "oracle_digest")?;
        let expected_task_path = format!("tasks/{family}/{task_id}/task.json");
        let expected_fixture_path = format!("tasks/{family}/{task_id}/fixture");

        if !ids.insert(task_id.clone()) {
            return Err(CorpusError::Invalid(format!(
                "duplicate v2 task id: {task_id}"
            )));
        }
        if task_path_text != expected_task_path || fixture_path_text != expected_fixture_path {
            return Err(CorpusError::Invalid(format!(
                "v2 task topology mismatch: {task_id}"
            )));
        }
        if !task_paths.insert(task_path_text.clone()) {
            return Err(CorpusError::Invalid(format!(
                "duplicate v2 task path: {task_path_text}"
            )));
        }
        if !fixture_paths.insert(fixture_path_text.clone()) {
            return Err(CorpusError::Invalid(format!(
                "duplicate v2 fixture path: {fixture_path_text}"
            )));
        }
        if !task_digests.insert(expected_task_digest.clone())
            || !fixture_digests.insert(expected_fixture_digest.clone())
            || !oracle_digests.insert(oracle_digest.clone())
        {
            return Err(CorpusError::Invalid(
                "duplicate v2 task, fixture, or oracle material".into(),
            ));
        }
        if !SPLITS.contains(&split.as_str()) {
            return Err(CorpusError::Invalid(format!("invalid v2 split: {split}")));
        }
        if let Some(previous) = family_splits.get(&family) {
            if previous != &split {
                return Err(CorpusError::Invalid(format!(
                    "v2 family leakage across splits: {family}"
                )));
            }
        } else {
            family_splits.insert(family.clone(), split.clone());
        }
        if oracle_digest.is_empty() {
            return Err(CorpusError::Invalid(format!(
                "empty v2 oracle digest: {task_id}"
            )));
        }
        families.insert(family.clone());
        *split_counts.entry(split.clone()).or_default() += 1;

        let task_path = resolve_manifest_path(&root, &task_path_text, "task", "tasks")?;
        if !task_path.is_file() {
            return Err(CorpusError::Invalid(format!(
                "v2 task JSON is not a file: {task_path_text}"
            )));
        }
        let fixture_path = resolve_manifest_path(&root, &fixture_path_text, "fixture", "tasks")?;
        if !fixture_path.is_dir() {
            return Err(CorpusError::Invalid(format!(
                "v2 fixture is not a directory: {fixture_path_text}"
            )));
        }

        let task = read_json(&task_path)?;
        reject_oracle_material_keys(&task)?;
        if required_string(&task, "schema")? != "AiDENsExecutableLearningTaskV2"
            || required_string(&task, "task_id")? != task_id
            || required_string(&task, "family")? != family
            || required_string(&task, "split")? != split
            || required_string(&task, "fixture_tree_digest")? != expected_fixture_digest
            || required_string(&task, "oracle_digest")? != oracle_digest
        {
            return Err(CorpusError::Invalid(format!(
                "v2 task identity or digest declaration mismatch: {task_id}"
            )));
        }

        let verifier = strings(&task, "verifier")
            .ok_or_else(|| CorpusError::Invalid(format!("invalid v2 verifier: {task_id}")))?;
        let expected_verifier = ["cargo", "test", "--offline", "--quiet"];
        if verifier.iter().map(String::as_str).ne(expected_verifier)
            || required_string(&task, "baseline_expected")? != "fail"
            || required_string(&task, "treatment_expected")? != "pass"
        {
            return Err(CorpusError::Invalid(format!(
                "v2 verifier or outcome declaration mismatch: {task_id}"
            )));
        }

        let actual_task_digest = canonical_json_sha256(&task)?;
        if actual_task_digest != expected_task_digest {
            return Err(CorpusError::Invalid(format!(
                "v2 task digest mismatch: {task_id}"
            )));
        }
        for required_fixture_file in ["Cargo.toml", "src/lib.rs"] {
            if !fixture_path.join(required_fixture_file).is_file() {
                return Err(CorpusError::Invalid(format!(
                    "missing v2 fixture file {required_fixture_file}: {task_id}"
                )));
            }
        }
        let actual_fixture_digest = fixture_tree_digest(&fixture_path)?;
        if actual_fixture_digest != expected_fixture_digest {
            return Err(CorpusError::Invalid(format!(
                "v2 fixture tree digest mismatch: {task_id}"
            )));
        }

        public_tasks.push(V2LearnerSafeTask {
            task_id,
            family,
            split,
            task_path,
            fixture_path,
            prompt: required_string(&task, "prompt")?,
            verifier,
            baseline_expected: "fail".into(),
            treatment_expected: "pass".into(),
            fixture_tree_digest: expected_fixture_digest,
            oracle_digest,
        });
    }

    if families.len() != 5 {
        return Err(CorpusError::Invalid(format!(
            "v2 corpus must contain exactly five family clusters, found {}",
            families.len()
        )));
    }
    let observed_counts = SPLITS.map(|split| split_counts.get(split).copied().unwrap_or(0));
    if observed_counts != [9, 3, 3] {
        return Err(CorpusError::Invalid(format!(
            "v2 split counts must be 9/3/3: observed {split_counts:?}"
        )));
    }

    Ok(V2ValidatedCorpus {
        corpus_digest: expected_corpus_digest,
        tasks: public_tasks,
    })
}

fn required_string(value: &Value, key: &str) -> Result<String, CorpusError> {
    string(value, key)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| CorpusError::Invalid(format!("missing or empty v2 field: {key}")))
}

fn resolve_manifest_path(
    root: &Path,
    relative: &str,
    kind: &str,
    role_root: &str,
) -> Result<PathBuf, CorpusError> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(CorpusError::Invalid(format!(
            "unsafe v2 {kind} path: {relative}"
        )));
    }
    let resolved = std::fs::canonicalize(root.join(relative_path)).map_err(|e| {
        CorpusError::Invalid(format!("unresolvable v2 {kind} path {relative}: {e}"))
    })?;
    let allowed = std::fs::canonicalize(root.join(role_root))
        .map_err(|e| CorpusError::Invalid(format!("unresolvable v2 {kind} role root: {e}")))?;
    if !resolved.starts_with(&allowed) {
        return Err(CorpusError::Invalid(format!(
            "v2 {kind} path escapes {role_root} role root: {relative}"
        )));
    }
    Ok(resolved)
}

fn reject_oracle_material_keys(value: &Value) -> Result<(), CorpusError> {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                if matches!(
                    key.as_str(),
                    "oracle" | "oracle_path" | "patch" | "expected_source"
                ) {
                    return Err(CorpusError::Invalid(format!(
                        "learner-visible v2 task contains forbidden key: {key}"
                    )));
                }
                reject_oracle_material_keys(child)?;
            }
        }
        Value::Array(array) => {
            for child in array {
                reject_oracle_material_keys(child)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn canonical_json_sha256(value: &Value) -> Result<String, CorpusError> {
    let mut bytes = Vec::new();
    write_canonical_json(value, &mut bytes)?;
    Ok(sha256_bytes(&bytes))
}

fn write_canonical_json(value: &Value, output: &mut Vec<u8>) -> Result<(), CorpusError> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => {
            serde_json::to_writer(output, value)
                .map_err(|e| CorpusError::Schema(format!("canonical JSON encoding: {e}")))?;
        }
        Value::String(string) => write_python_json_string(string, output),
        Value::Array(array) => {
            output.push(b'[');
            for (index, child) in array.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_canonical_json(child, output)?;
            }
            output.push(b']');
        }
        Value::Object(object) => {
            output.push(b'{');
            let mut keys: Vec<&String> = object.keys().collect();
            keys.sort_unstable();
            for (index, key) in keys.into_iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_python_json_string(key, output);
                output.push(b':');
                write_canonical_json(&object[key], output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn write_python_json_string(value: &str, output: &mut Vec<u8>) {
    output.push(b'"');
    for character in value.chars() {
        match character {
            '"' => output.extend_from_slice(br#"\""#),
            '\\' => output.extend_from_slice(br#"\\"#),
            '\u{0008}' => output.extend_from_slice(br#"\b"#),
            '\u{000c}' => output.extend_from_slice(br#"\f"#),
            '\n' => output.extend_from_slice(br#"\n"#),
            '\r' => output.extend_from_slice(br#"\r"#),
            '\t' => output.extend_from_slice(br#"\t"#),
            character if character.is_ascii_control() => {
                output.extend_from_slice(format!("\\u{:04x}", character as u32).as_bytes());
            }
            character if character.is_ascii() => output.push(character as u8),
            character if (character as u32) <= 0xffff => {
                output.extend_from_slice(format!("\\u{:04x}", character as u32).as_bytes());
            }
            character => {
                let scalar = character as u32 - 0x1_0000;
                let high = 0xd800 + (scalar >> 10);
                let low = 0xdc00 + (scalar & 0x3ff);
                output.extend_from_slice(format!("\\u{high:04x}\\u{low:04x}").as_bytes());
            }
        }
    }
    output.push(b'"');
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn fixture_tree_digest(root: &Path) -> Result<String, CorpusError> {
    let mut inventory = Vec::<Value>::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|e| CorpusError::Io(format!("fixture walk: {e}")))?;
        let relative = entry
            .path()
            .strip_prefix(root)
            .map_err(|e| CorpusError::Invalid(format!("fixture path inventory: {e}")))?;
        if relative
            .components()
            .any(|component| component.as_os_str() == OsStr::new("target"))
        {
            continue;
        }
        if entry.file_type().is_symlink() {
            return Err(CorpusError::Invalid(format!(
                "symlink is not allowed in v2 fixture: {}",
                relative.display()
            )));
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = relative
            .components()
            .map(|component| {
                component
                    .as_os_str()
                    .to_str()
                    .ok_or_else(|| CorpusError::Invalid("non-UTF-8 v2 fixture path".into()))
            })
            .collect::<Result<Vec<_>, _>>()?
            .join("/");
        let bytes = std::fs::read(entry.path())
            .map_err(|e| CorpusError::Io(format!("{}: {e}", entry.path().display())))?;
        inventory.push(serde_json::json!({
            "path": relative,
            "sha256": sha256_bytes(&bytes),
        }));
    }
    inventory.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    canonical_json_sha256(&Value::Array(inventory))
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
    use std::fs;

    fn v2_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/learning-coding-agent/v2")
    }

    fn copy_v2_corpus() -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let destination = temp.path().join("v2");
        for entry in WalkDir::new(v2_root()) {
            let entry = entry.unwrap();
            let relative = entry.path().strip_prefix(v2_root()).unwrap();
            let target = destination.join(relative);
            if entry.file_type().is_dir() {
                fs::create_dir_all(&target).unwrap();
            } else if entry.file_type().is_file() {
                fs::copy(entry.path(), target).unwrap();
            }
        }
        (temp, destination)
    }

    fn write_json(path: &Path, value: &Value) {
        fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    }

    fn refresh_manifest_digest(manifest: &mut Value) {
        manifest.as_object_mut().unwrap().remove("corpus_digest");
        let digest = canonical_json_sha256(manifest).unwrap();
        manifest["corpus_digest"] = Value::String(digest);
    }

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

    #[test]
    fn v2_committed_corpus_is_deterministic_and_learner_safe() {
        let first = validate_and_consume_v2(v2_root()).unwrap();
        let second = validate_and_consume_v2(v2_root()).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.tasks.len(), 15);
        assert_eq!(
            first
                .tasks
                .iter()
                .map(|task| task.family.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            5
        );
        let counts = SPLITS.map(|split| {
            first
                .tasks
                .iter()
                .filter(|task| task.split == split)
                .count()
        });
        assert_eq!(counts, [9, 3, 3]);
        assert!(first
            .tasks
            .iter()
            .filter(|task| task.split == "holdout")
            .all(|task| !task.task_path.to_string_lossy().contains("oracles")
                && !task.fixture_path.to_string_lossy().contains("oracles")));
    }

    #[test]
    fn v2_rejects_copied_fixture_drift() {
        let (_temp, root) = copy_v2_corpus();
        let fixture = root.join("tasks/borrow-check/borrow-check-01/fixture/src/lib.rs");
        let mut source = fs::read_to_string(&fixture).unwrap();
        source.push_str("\n// drift\n");
        fs::write(fixture, source).unwrap();

        let error = validate_and_consume_v2(root).unwrap_err();
        assert!(error.to_string().contains("fixture tree digest mismatch"));
    }

    #[test]
    fn v2_rejects_exposed_patch_even_with_refreshed_digests() {
        let (_temp, root) = copy_v2_corpus();
        let mut manifest = read_json(&root.join("manifest.json")).unwrap();
        let relative_task_path = manifest["tasks"][0]["task_path"]
            .as_str()
            .unwrap()
            .to_owned();
        let task_path = root.join(relative_task_path);
        let mut task = read_json(&task_path).unwrap();
        task["patch"] = serde_json::json!({"expected_source": "learner-visible leak"});
        let task_digest = canonical_json_sha256(&task).unwrap();
        write_json(&task_path, &task);
        manifest["tasks"][0]["task_digest"] = Value::String(task_digest);
        refresh_manifest_digest(&mut manifest);
        write_json(&root.join("manifest.json"), &manifest);

        let error = validate_and_consume_v2(root).unwrap_err();
        assert!(error.to_string().contains("forbidden key: patch"));
    }

    #[test]
    fn v2_rejects_duplicate_ids_and_family_leakage() {
        let (_duplicate_temp, duplicate_root) = copy_v2_corpus();
        let mut duplicate_manifest = read_json(&duplicate_root.join("manifest.json")).unwrap();
        duplicate_manifest["tasks"][1]["task_id"] =
            duplicate_manifest["tasks"][0]["task_id"].clone();
        refresh_manifest_digest(&mut duplicate_manifest);
        write_json(&duplicate_root.join("manifest.json"), &duplicate_manifest);
        let duplicate_error = validate_and_consume_v2(duplicate_root).unwrap_err();
        assert!(duplicate_error.to_string().contains("duplicate v2 task id"));

        let (_leak_temp, leak_root) = copy_v2_corpus();
        let mut leak_manifest = read_json(&leak_root.join("manifest.json")).unwrap();
        leak_manifest["tasks"][14]["split"] = Value::String("development".into());
        refresh_manifest_digest(&mut leak_manifest);
        write_json(&leak_root.join("manifest.json"), &leak_manifest);
        let leakage_error = validate_and_consume_v2(leak_root).unwrap_err();
        assert!(leakage_error.to_string().contains("family leakage"));
    }

    #[test]
    fn v2_rejects_role_confusion_and_duplicate_material_before_reading_tasks() {
        let (_role_temp, role_root) = copy_v2_corpus();
        let mut role_manifest = read_json(&role_root.join("manifest.json")).unwrap();
        role_manifest["tasks"][0]["task_path"] = Value::String(format!(
            "oracles/{}.patch.json",
            role_manifest["tasks"][0]["task_id"].as_str().unwrap()
        ));
        refresh_manifest_digest(&mut role_manifest);
        write_json(&role_root.join("manifest.json"), &role_manifest);
        let role_error = validate_and_consume_v2(role_root).unwrap_err();
        assert!(role_error.to_string().contains("task topology mismatch"));

        let (_duplicate_temp, duplicate_root) = copy_v2_corpus();
        let mut duplicate_manifest = read_json(&duplicate_root.join("manifest.json")).unwrap();
        for field in ["task_digest", "fixture_tree_digest", "oracle_digest"] {
            duplicate_manifest["tasks"][1][field] = duplicate_manifest["tasks"][0][field].clone();
        }
        refresh_manifest_digest(&mut duplicate_manifest);
        write_json(&duplicate_root.join("manifest.json"), &duplicate_manifest);
        let duplicate_error = validate_and_consume_v2(duplicate_root).unwrap_err();
        assert!(duplicate_error
            .to_string()
            .contains("duplicate v2 task, fixture, or oracle material"));
    }
}
