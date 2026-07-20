//! Prepare owner-admitted sealed replay material for replay execution.

use semantic_memory::{
    load_procedure_owner_snapshot, load_procedure_replay_snapshot,
    verify_procedure_lifecycle_receipt_v1, MemoryConfig, MemoryError, MemoryStore,
    ProcedureActionPermitV1, ProcedureLifecycleDispositionV1, ProcedureOwnerSnapshotV1,
    ProcedureReplayInputsV1, ProcedureReplaySnapshotV1,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use typed_patch::{validate_patch, PatchPolicy, StructuredPatch};
use walkdir::WalkDir;

const PROCEDURE_REPLAY_ADMISSION_SCHEMA: &str = "procedure_replay_v1";
const PATCH_TOOL: &str = "typed-patch:structured-apply:1";

#[derive(Debug, Clone)]
pub struct OwnerAdmittedSealedReplayRequestV1 {
    pub replay_id: String,
    pub operational_store: PathBuf,
    pub fixture: PathBuf,
    pub execution_permits: Vec<ProcedureActionPermitV1>,
    pub run_id: String,
    pub attempt_id: String,
    pub trial_id: String,
    pub trace_id: String,
}

#[derive(Debug, Clone)]
pub struct OwnerAdmittedSealedReplayMaterialV1 {
    pub replay_id: String,
    pub fixture_path: PathBuf,
    pub operational_store: PathBuf,
    pub patch: StructuredPatch,
    pub patch_policy: PatchPolicy,
    pub image: String,
    pub fixture_digest: String,
    pub execution_permits: Vec<ProcedureActionPermitV1>,
    pub run_id: String,
    pub attempt_id: String,
    pub trial_id: String,
    pub trace_id: String,
    pub promotion_receipt_id: String,
    pub promotion_receipt_digest: String,
    pub admission_digest: String,
}

#[derive(Debug, thiserror::Error)]
pub enum OwnerAdmittedSealedReplayError {
    #[error("invalid owner-admitted replay request: {0}")]
    Invalid(String),
    #[error("store operation failed: {0}")]
    Store(#[from] MemoryError),
}

pub async fn prepare_owner_admitted_sealed_replay_material(
    request: OwnerAdmittedSealedReplayRequestV1,
) -> Result<OwnerAdmittedSealedReplayMaterialV1, OwnerAdmittedSealedReplayError> {
    validate_request(&request)?;
    validate_permits(&request.execution_permits)?;

    let memory = MemoryStore::open(MemoryConfig {
        base_dir: request.operational_store.clone(),
        ..MemoryConfig::default()
    })?;
    let replay = load_procedure_replay_snapshot(&memory, request.replay_id.clone()).await?;
    validate_admission(&replay)?;

    let owner =
        load_procedure_owner_snapshot(&memory, replay.inputs.original_artifact_id.as_str()).await?;
    validate_owner_artifact(&replay, &owner)?;

    let lifecycle = owner.lifecycle_receipt.as_ref().ok_or_else(|| {
        OwnerAdmittedSealedReplayError::Invalid(
            "owner snapshot has no lifecycle receipt for requested artifact".into(),
        )
    })?;
    if lifecycle.disposition != ProcedureLifecycleDispositionV1::Promoted {
        return Err(OwnerAdmittedSealedReplayError::Invalid(
            "owner snapshot lifecycle state is not current promoted".into(),
        ));
    }
    if lifecycle.receipt_id != replay.inputs.promotion_receipt_ref {
        return Err(OwnerAdmittedSealedReplayError::Invalid(
            "replay admission references a non-current promotion receipt".into(),
        ));
    }
    if !verify_procedure_lifecycle_receipt_v1(lifecycle) {
        return Err(OwnerAdmittedSealedReplayError::Invalid(
            "owner lifecycle receipt failed verification".into(),
        ));
    }

    let fixture_digest = fixture_digest(&request.fixture)?;
    if normalize_digest(&fixture_digest) != normalize_digest(&replay.inputs.source_tree_digest) {
        return Err(OwnerAdmittedSealedReplayError::Invalid(
            "fixture digest does not match replay snapshot source_tree_digest".into(),
        ));
    }

    let (patch, patch_policy, image) = extract_owner_material(&owner)?;
    validate_patch_signature(&patch, &patch_policy, &replay.inputs)?;

    Ok(OwnerAdmittedSealedReplayMaterialV1 {
        replay_id: request.replay_id,
        fixture_path: request.fixture,
        operational_store: request.operational_store,
        patch,
        patch_policy,
        image,
        fixture_digest,
        execution_permits: request.execution_permits,
        run_id: request.run_id,
        attempt_id: request.attempt_id,
        trial_id: request.trial_id,
        trace_id: request.trace_id,
        promotion_receipt_id: lifecycle.receipt_id.clone(),
        promotion_receipt_digest: lifecycle.receipt_digest.clone(),
        admission_digest: replay.admission.admission_digest,
    })
}

fn validate_request(
    request: &OwnerAdmittedSealedReplayRequestV1,
) -> Result<(), OwnerAdmittedSealedReplayError> {
    if request.replay_id.trim().is_empty() {
        return Err(OwnerAdmittedSealedReplayError::Invalid(
            "replay_id is required".into(),
        ));
    }
    for (name, value) in [
        ("run_id", &request.run_id),
        ("attempt_id", &request.attempt_id),
        ("trial_id", &request.trial_id),
        ("trace_id", &request.trace_id),
    ] {
        if value.trim().is_empty() {
            return Err(OwnerAdmittedSealedReplayError::Invalid(format!(
                "{name} must be durable material",
            )));
        }
    }
    if !request.operational_store.exists() {
        return Err(OwnerAdmittedSealedReplayError::Invalid(
            "operational_store must point to an existing path".into(),
        ));
    }
    if !request.fixture.exists() || !request.fixture.is_dir() {
        return Err(OwnerAdmittedSealedReplayError::Invalid(
            "fixture must be an existing directory".into(),
        ));
    }
    if request.execution_permits.is_empty() {
        return Err(OwnerAdmittedSealedReplayError::Invalid(
            "at least one execution permit is required".into(),
        ));
    }
    Ok(())
}

fn validate_permits(
    permits: &[ProcedureActionPermitV1],
) -> Result<(), OwnerAdmittedSealedReplayError> {
    let mut unique = BTreeSet::new();
    for permit in permits {
        if permit.principal.trim().is_empty()
            || permit.caller_id.trim().is_empty()
            || !permit.scope.is_bound()
        {
            return Err(OwnerAdmittedSealedReplayError::Invalid(
                "execution permit must include principal, caller_id and bounded scope".into(),
            ));
        }
        if permit.capability != ProcedureActionPermitV1::CAPABILITY {
            return Err(OwnerAdmittedSealedReplayError::Invalid(
                "execution permit has an invalid capability".into(),
            ));
        }
        let tuple = (
            permit.principal.as_str(),
            permit.caller_id.as_str(),
            permit.expires_at.as_str(),
        );
        if !unique.insert(tuple) {
            return Err(OwnerAdmittedSealedReplayError::Invalid(
                "execution permits must not contain duplicate principal/caller/expires_at".into(),
            ));
        }
    }
    Ok(())
}

fn validate_admission(
    snapshot: &ProcedureReplaySnapshotV1,
) -> Result<(), OwnerAdmittedSealedReplayError> {
    let request_digest = digest_request(&snapshot.inputs)?;
    let expected_admission = replay_admission_digest(&snapshot.inputs.replay_id, &request_digest)?;
    if normalize_digest(&snapshot.admission.admission_digest)
        != normalize_digest(&expected_admission)
    {
        return Err(OwnerAdmittedSealedReplayError::Invalid(
            "replay admission digest mismatch".into(),
        ));
    }
    if snapshot.admission.schema_version != PROCEDURE_REPLAY_ADMISSION_SCHEMA {
        return Err(OwnerAdmittedSealedReplayError::Invalid(
            "unexpected replay admission schema".into(),
        ));
    }
    Ok(())
}

fn validate_owner_artifact(
    replay: &ProcedureReplaySnapshotV1,
    owner: &ProcedureOwnerSnapshotV1,
) -> Result<(), OwnerAdmittedSealedReplayError> {
    if owner.artifact.artifact_id != replay.inputs.original_artifact_id {
        return Err(OwnerAdmittedSealedReplayError::Invalid(
            "replay admission references non-matching artifact id".into(),
        ));
    }
    if normalize_digest(&owner.artifact.artifact_digest)
        != normalize_digest(&replay.inputs.original_artifact_digest)
    {
        return Err(OwnerAdmittedSealedReplayError::Invalid(
            "replay admission references non-matching artifact digest".into(),
        ));
    }
    Ok(())
}

fn extract_owner_material(
    owner: &ProcedureOwnerSnapshotV1,
) -> Result<(StructuredPatch, PatchPolicy, String), OwnerAdmittedSealedReplayError> {
    let step = owner.artifact.steps.first().ok_or_else(|| {
        OwnerAdmittedSealedReplayError::Invalid(
            "owner artifact contains no executable steps".into(),
        )
    })?;
    if step.tool != PATCH_TOOL {
        return Err(OwnerAdmittedSealedReplayError::Invalid(
            "owner artifact executable step is not canonical typed-patch tool".into(),
        ));
    }

    let patch: StructuredPatch =
        serde_json::from_value(step.arguments.clone()).map_err(|error| {
            OwnerAdmittedSealedReplayError::Invalid(format!(
                "owner step arguments are not a typed patch: {error}"
            ))
        })?;

    let fixture = owner
        .artifact
        .evidence_test_envelope
        .fixtures
        .first()
        .ok_or_else(|| {
            OwnerAdmittedSealedReplayError::Invalid(
                "owner artifact contains no fixture evidence".into(),
            )
        })?;
    let image = fixture
        .context
        .get("sandbox_image")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            OwnerAdmittedSealedReplayError::Invalid("owner fixture lacks sandbox_image".into())
        })?
        .to_string();
    if image.contains(":latest") || !image.contains("@sha256:") {
        return Err(OwnerAdmittedSealedReplayError::Invalid(
            "owner sandbox image must be digest pinned".into(),
        ));
    }

    let patch_policy =
        parse_patch_policy(fixture.context.get("patch_policy").ok_or_else(|| {
            OwnerAdmittedSealedReplayError::Invalid("owner fixture lacks patch_policy".into())
        })?)?;

    Ok((patch, patch_policy, image))
}

fn parse_patch_policy(value: &Value) -> Result<PatchPolicy, OwnerAdmittedSealedReplayError> {
    let forbidden_paths = value
        .get("forbidden_paths")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            OwnerAdmittedSealedReplayError::Invalid("patch_policy.forbidden_paths missing".into())
        })?
        .iter()
        .map(|entry| {
            entry.as_str().map(ToString::to_string).ok_or_else(|| {
                OwnerAdmittedSealedReplayError::Invalid(
                    "patch_policy.forbidden_paths contains non-string value".into(),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let allow_test_modifications = value
        .get("allow_test_modifications")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            OwnerAdmittedSealedReplayError::Invalid(
                "patch_policy.allow_test_modifications missing".into(),
            )
        })?;
    let max_files_changed = value
        .get("max_files_changed")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            OwnerAdmittedSealedReplayError::Invalid("patch_policy.max_files_changed missing".into())
        })? as usize;
    let max_total_lines_changed = value
        .get("max_total_lines_changed")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            OwnerAdmittedSealedReplayError::Invalid(
                "patch_policy.max_total_lines_changed missing".into(),
            )
        })? as usize;
    let max_lines_changed_per_file = value
        .get("max_lines_changed_per_file")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            OwnerAdmittedSealedReplayError::Invalid(
                "patch_policy.max_lines_changed_per_file missing".into(),
            )
        })? as usize;

    Ok(PatchPolicy {
        forbidden_paths,
        allow_test_modifications,
        max_files_changed,
        max_total_lines_changed,
        max_lines_changed_per_file,
    })
}

fn validate_patch_signature(
    patch: &StructuredPatch,
    patch_policy: &PatchPolicy,
    inputs: &ProcedureReplayInputsV1,
) -> Result<(), OwnerAdmittedSealedReplayError> {
    let patch_validation = validate_patch(patch, patch_policy);
    if !patch_validation.ok {
        return Err(OwnerAdmittedSealedReplayError::Invalid(
            "replayed patch is invalid against owner patch policy".into(),
        ));
    }

    let derived = digest_value(patch)?;
    if normalize_digest(&derived) != normalize_digest(&inputs.patch_digest) {
        return Err(OwnerAdmittedSealedReplayError::Invalid(
            "patch identity does not match owner-admitted replay inputs".into(),
        ));
    }

    Ok(())
}

fn digest_request(
    inputs: &ProcedureReplayInputsV1,
) -> Result<String, OwnerAdmittedSealedReplayError> {
    let bytes = serde_json::to_vec(inputs).map_err(|error| {
        OwnerAdmittedSealedReplayError::Invalid(format!(
            "replay inputs are not serializable: {error}"
        ))
    })?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn replay_admission_digest(
    replay_id: &str,
    request_digest: &str,
) -> Result<String, OwnerAdmittedSealedReplayError> {
    let bytes = serde_json::to_vec(&(PROCEDURE_REPLAY_ADMISSION_SCHEMA, replay_id, request_digest))
        .map_err(|error| {
            OwnerAdmittedSealedReplayError::Invalid(format!(
                "admission material is not serializable: {error}"
            ))
        })?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn fixture_digest(path: &Path) -> Result<String, OwnerAdmittedSealedReplayError> {
    let mut entries = Vec::new();
    for entry in WalkDir::new(path).follow_links(false) {
        let entry = entry.map_err(|error| {
            OwnerAdmittedSealedReplayError::Invalid(format!("fixture walk failed: {error}"))
        })?;
        if entry.file_type().is_symlink() {
            return Err(OwnerAdmittedSealedReplayError::Invalid(format!(
                "fixture identity rejects symlink: {}",
                entry.path().display()
            )));
        }
        if entry.file_type().is_file() {
            let relative = entry
                .path()
                .strip_prefix(path)
                .map_err(|error| OwnerAdmittedSealedReplayError::Invalid(error.to_string()))?;
            entries.push((
                relative.to_string_lossy().to_string(),
                std::fs::read(entry.path())
                    .map_err(|error| OwnerAdmittedSealedReplayError::Invalid(error.to_string()))?,
            ));
        }
    }
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    digest_value(&entries)
}

fn digest_value<T: Serialize>(value: &T) -> Result<String, OwnerAdmittedSealedReplayError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| OwnerAdmittedSealedReplayError::Invalid(error.to_string()))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn normalize_digest(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("blake3:")
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_request_requires_run_identifiers() {
        let request = OwnerAdmittedSealedReplayRequestV1 {
            replay_id: "replay:sealed".into(),
            operational_store: std::env::temp_dir(),
            fixture: std::env::temp_dir(),
            execution_permits: vec![],
            run_id: String::new(),
            attempt_id: "attempt".into(),
            trial_id: "trial".into(),
            trace_id: "trace".into(),
        };
        assert!(matches!(
            validate_request(&request),
            Err(OwnerAdmittedSealedReplayError::Invalid(_))
        ));
    }
}
