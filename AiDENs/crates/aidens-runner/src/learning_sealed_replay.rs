//! Prepare owner-admitted sealed replay material for replay execution.

use crate::learning_controller::{execute_effectful_run, RealSandboxLearningConfig};
use aidens_contracts::{
    ArtifactId, CanonicalToolSideEffectClass, PermitGrantV1, PermitUseReportV1,
};
use aidens_receipts::{CanonicalEventLog, CanonicalEventLogConfig};
use semantic_memory::{
    compare_replay_observation, load_procedure_owner_snapshot, load_procedure_replay_snapshot,
    procedure_replay_permit_ref, record_replay_result, verify_procedure_lifecycle_receipt_v1,
    MemoryConfig, MemoryError, MemoryStore, ProcedureActionPermitV1,
    ProcedureLifecycleDispositionV1, ProcedureOwnerSnapshotV1, ProcedureReplayInputsV1,
    ProcedureReplayOutcomeV1, ProcedureReplayResultV1, ProcedureReplaySnapshotV1,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::future::Future;
use std::path::{Path, PathBuf};
use typed_patch::{validate_patch, PatchPolicy, StructuredPatch};
use walkdir::WalkDir;

const PROCEDURE_REPLAY_ADMISSION_SCHEMA: &str = "procedure_replay_v1";
const PATCH_TOOL: &str = "typed-patch:structured-apply:1";
const OWNER_ADMITTED_SEALED_REPLAY_EXECUTION_SCHEMA: &str =
    "owner-admitted-sealed-replay-execution-v1";

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

#[derive(Debug, Clone)]
pub struct OwnerAdmittedSealedReplayReportV1 {
    pub replay_id: String,
    pub execution_receipt_id: String,
    pub execution_preflight_receipt_id: String,
    pub execution_terminal_receipt_id: String,
    pub execution_report: crate::learning_effectful::EffectfulEvaluationReportV1,
    pub replay_result: ProcedureReplayResultV1,
    pub replay_snapshot: ProcedureReplaySnapshotV1,
}

#[derive(Debug, Clone)]
pub(crate) struct OwnerAdmittedSealedReplayExecutionV1 {
    pub report: crate::learning_effectful::EffectfulEvaluationReportV1,
    pub preflight_receipt_id: String,
    pub terminal_receipt_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OwnerAdmittedSealedReplayExecutionReceiptV1 {
    pub schema: String,
    pub replay_id: String,
    pub preflight_receipt_id: String,
    pub terminal_receipt_id: String,
    pub execution_report: crate::learning_effectful::EffectfulEvaluationReportV1,
}

#[derive(Debug, thiserror::Error)]
pub enum OwnerAdmittedSealedReplayError {
    #[error("invalid owner-admitted replay request: {0}")]
    Invalid(String),
    #[error("store operation failed: {0}")]
    Store(#[from] MemoryError),
    #[error("execution failed: {0}")]
    Execution(String),
}

pub async fn execute_owner_admitted_sealed_replay(
    material: OwnerAdmittedSealedReplayMaterialV1,
) -> Result<OwnerAdmittedSealedReplayReportV1, OwnerAdmittedSealedReplayError> {
    execute_owner_admitted_sealed_replay_with_executor(
        material,
        |config: RealSandboxLearningConfig| async move {
            let core = execute_effectful_run(&config)
                .await
                .map_err(|error| OwnerAdmittedSealedReplayError::Execution(error.to_string()))?;
            Ok(OwnerAdmittedSealedReplayExecutionV1 {
                report: core.report,
                preflight_receipt_id: core.preflight_id,
                terminal_receipt_id: core.terminal_id,
            })
        },
    )
    .await
}

pub(crate) async fn execute_owner_admitted_sealed_replay_with_executor<F, Fut>(
    material: OwnerAdmittedSealedReplayMaterialV1,
    mut executor: F,
) -> Result<OwnerAdmittedSealedReplayReportV1, OwnerAdmittedSealedReplayError>
where
    F: FnMut(RealSandboxLearningConfig) -> Fut,
    Fut: Future<
        Output = Result<OwnerAdmittedSealedReplayExecutionV1, OwnerAdmittedSealedReplayError>,
    >,
{
    let memory = MemoryStore::open(MemoryConfig {
        base_dir: material.operational_store.clone(),
        ..MemoryConfig::default()
    })?;
    let snapshot = load_procedure_replay_snapshot(&memory, material.replay_id.clone()).await?;
    validate_admission(&snapshot)?;

    let permit = locate_matching_permit(&snapshot, &material.execution_permits)?;
    let config = build_execution_config(&material, permit)?;

    let execution_receipt_id = execution_receipt_id(&material);
    let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(
        config.receipt_root.clone(),
    ))
    .map_err(|error| OwnerAdmittedSealedReplayError::Execution(error.to_string()))?;

    let execution = match log.inspect(&execution_receipt_id) {
        Ok(record) => {
            let stored = verify_log_record(&log, &execution_receipt_id, &record.body)?;
            let receipt: OwnerAdmittedSealedReplayExecutionReceiptV1 =
                serde_json::from_value(stored).map_err(|error| {
                    OwnerAdmittedSealedReplayError::Execution(format!(
                        "existing execution receipt is not schema-compatible: {error}"
                    ))
                })?;
            if receipt.replay_id != material.replay_id {
                return Err(OwnerAdmittedSealedReplayError::Execution(
                    "existing execution receipt belongs to a different replay id".into(),
                ));
            }
            Ok::<OwnerAdmittedSealedReplayExecutionV1, OwnerAdmittedSealedReplayError>(
                OwnerAdmittedSealedReplayExecutionV1 {
                    report: receipt.execution_report,
                    preflight_receipt_id: receipt.preflight_receipt_id,
                    terminal_receipt_id: receipt.terminal_receipt_id,
                },
            )
        }
        Err(canonical_error) => {
            if !matches!(
                canonical_error,
                aidens_receipts::CanonicalEventLogError::NotFound(_)
            ) {
                return Err(OwnerAdmittedSealedReplayError::Execution(
                    canonical_error.to_string(),
                ));
            }

            let execution = executor(config.clone()).await?;
            let receipt = OwnerAdmittedSealedReplayExecutionReceiptV1 {
                schema: OWNER_ADMITTED_SEALED_REPLAY_EXECUTION_SCHEMA.into(),
                replay_id: material.replay_id.clone(),
                preflight_receipt_id: execution.preflight_receipt_id.clone(),
                terminal_receipt_id: execution.terminal_receipt_id.clone(),
                execution_report: execution.report.clone(),
            };

            let body = serde_json::to_value(&receipt)
                .map_err(|error| OwnerAdmittedSealedReplayError::Execution(error.to_string()))?;
            log.append_json(
                "aidens-runner",
                OWNER_ADMITTED_SEALED_REPLAY_EXECUTION_SCHEMA,
                execution_receipt_id.clone(),
                body.clone(),
            )
            .map_err(|error| OwnerAdmittedSealedReplayError::Execution(error.to_string()))?;
            let stored = verify_log_record(
                &log,
                &execution_receipt_id,
                &log.inspect(&execution_receipt_id)
                    .map_err(|error| OwnerAdmittedSealedReplayError::Execution(error.to_string()))?
                    .body,
            )?;
            let _: Value = serde_json::from_value(stored)
                .map_err(|error| OwnerAdmittedSealedReplayError::Execution(error.to_string()))?;
            Ok::<OwnerAdmittedSealedReplayExecutionV1, OwnerAdmittedSealedReplayError>(execution)
        }
    }?;

    let observed_inputs = observed_inputs(&snapshot.inputs, &execution, &material.patch)?;
    let mut comparison = compare_replay_observation(&snapshot.inputs, &observed_inputs);
    if !execution.report.verified {
        comparison.outcome = ProcedureReplayOutcomeV1::Failed;
        comparison
            .reason_codes
            .push("execution-not-verified".into());
    }
    let result = ProcedureReplayResultV1 {
        replay_id: material.replay_id.clone(),
        result_digest: replay_result_digest(&snapshot.inputs, &observed_inputs, &comparison)?,
        outcome: comparison.outcome,
        reason_codes: comparison.reason_codes,
    };
    record_replay_result(&memory, result.clone()).await?;

    let snapshot = load_procedure_replay_snapshot(&memory, material.replay_id).await?;

    Ok(OwnerAdmittedSealedReplayReportV1 {
        replay_id: snapshot.inputs.replay_id.clone(),
        execution_receipt_id,
        execution_preflight_receipt_id: execution.preflight_receipt_id,
        execution_terminal_receipt_id: execution.terminal_receipt_id,
        execution_report: execution.report,
        replay_result: result,
        replay_snapshot: snapshot,
    })
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

fn locate_matching_permit(
    snapshot: &ProcedureReplaySnapshotV1,
    permits: &[ProcedureActionPermitV1],
) -> Result<ProcedureActionPermitV1, OwnerAdmittedSealedReplayError> {
    let expected = normalize_digest(&snapshot.inputs.action_permit_ref);
    for permit in permits {
        let binding =
            procedure_replay_permit_ref(permit, &snapshot.inputs.replay_id).map_err(|error| {
                OwnerAdmittedSealedReplayError::Invalid(format!(
                    "execution permit binding is not verifiable: {error}",
                ))
            })?;
        if normalize_digest(&binding) == expected {
            return Ok(permit.clone());
        }
    }
    Err(OwnerAdmittedSealedReplayError::Invalid(
        "no execution permit matched replay action permit binding".into(),
    ))
}

fn build_execution_config(
    material: &OwnerAdmittedSealedReplayMaterialV1,
    permit: ProcedureActionPermitV1,
) -> Result<RealSandboxLearningConfig, OwnerAdmittedSealedReplayError> {
    let replay_root = material.operational_store.join("sealed-replay");
    std::fs::create_dir_all(&replay_root)
        .map_err(|error| OwnerAdmittedSealedReplayError::Execution(error.to_string()))?;

    let receipt_root = replay_root.join("receipts");
    std::fs::create_dir_all(&receipt_root)
        .map_err(|error| OwnerAdmittedSealedReplayError::Execution(error.to_string()))?;

    let run_id = ArtifactId::new(material.run_id.clone());
    let attempt_id = ArtifactId::new(material.attempt_id.clone());
    let fixture_root = material.fixture_path.to_string_lossy().to_string();
    let mut permit_grant = PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Write,
        "aidens:patch-apply:1",
        fixture_root.as_str(),
        permit.principal,
    );
    permit_grant.run_id = Some(run_id.clone());
    permit_grant.attempt_id = Some(attempt_id.clone());
    let permit_use = PermitUseReportV1::allowed(
        &permit_grant,
        "aidens:patch-apply:1",
        fixture_root,
        Some(run_id),
        Some(attempt_id),
    );

    Ok(RealSandboxLearningConfig {
        fixture: material.fixture_path.clone(),
        patch: material.patch.clone(),
        patch_policy: material.patch_policy.clone(),
        permit_grant,
        permit_use,
        image: material.image.clone(),
        cea_db: replay_root.join("cea.sqlite"),
        receipt_root,
        memory_store: material.operational_store.clone(),
        forge_store: replay_root.join("forge.sqlite"),
        publication_namespace: "aidens-learning-sealed-replay".into(),
        run_id: material.run_id.clone(),
        attempt_id: material.attempt_id.clone(),
        trial_id: material.trial_id.clone(),
        trace_id: material.trace_id.clone(),
        recorded_at: "2026-07-19T00:00:00Z".into(),
    })
}

fn execution_receipt_id(material: &OwnerAdmittedSealedReplayMaterialV1) -> String {
    format!(
        "owner-admitted-sealed-replay-execution:{}:{}:{}",
        normalize_digest(&material.replay_id),
        normalize_digest(&material.run_id),
        normalize_digest(&material.attempt_id)
    )
}

fn verify_log_record(
    log: &CanonicalEventLog,
    receipt_id: &str,
    stored: &Value,
) -> Result<Value, OwnerAdmittedSealedReplayError> {
    if stored == &Value::Null {
        return Err(OwnerAdmittedSealedReplayError::Execution(
            "execution receipt body is empty".into(),
        ));
    }
    let reopened = CanonicalEventLog::open(log.config().clone())
        .map_err(|error| OwnerAdmittedSealedReplayError::Execution(error.to_string()))?;
    let record = reopened
        .inspect(receipt_id)
        .map_err(|error| OwnerAdmittedSealedReplayError::Execution(error.to_string()))?;
    if record.body != *stored
        || !record.verify_digest()
        || !record.verify_record_digest()
        || !reopened
            .verify_chain()
            .map_err(|error| OwnerAdmittedSealedReplayError::Execution(error.to_string()))?
    {
        return Err(OwnerAdmittedSealedReplayError::Execution(
            "execution receipt failed canonical verification".into(),
        ));
    }
    Ok(record.body)
}

fn observed_inputs(
    snapshot_inputs: &ProcedureReplayInputsV1,
    execution: &OwnerAdmittedSealedReplayExecutionV1,
    patch: &StructuredPatch,
) -> Result<ProcedureReplayInputsV1, OwnerAdmittedSealedReplayError> {
    Ok(ProcedureReplayInputsV1 {
        replay_id: snapshot_inputs.replay_id.clone(),
        original_artifact_id: snapshot_inputs.original_artifact_id.clone(),
        original_artifact_digest: snapshot_inputs.original_artifact_digest.clone(),
        patch_digest: digest_value(patch)?,
        source_tree_digest: execution.report.before_tree_digest.clone(),
        verifier_digest: snapshot_inputs.verifier_digest.clone(),
        check_policy_digest: snapshot_inputs.check_policy_digest.clone(),
        environment_digest: snapshot_inputs.environment_digest.clone(),
        image_digest: snapshot_inputs.image_digest.clone(),
        store_identity_digest: snapshot_inputs.store_identity_digest.clone(),
        retained_input_digest: snapshot_inputs.retained_input_digest.clone(),
        promotion_receipt_ref: snapshot_inputs.promotion_receipt_ref.clone(),
        action_permit_ref: snapshot_inputs.action_permit_ref.clone(),
    })
}

fn replay_result_digest(
    snapshot: &ProcedureReplayInputsV1,
    observed: &ProcedureReplayInputsV1,
    comparison: &semantic_memory::ProcedureReplayComparisonV1,
) -> Result<String, OwnerAdmittedSealedReplayError> {
    digest_value(&(snapshot.replay_id.clone(), observed.clone(), comparison))
        .map_err(|error| OwnerAdmittedSealedReplayError::Execution(error.to_string()))
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
