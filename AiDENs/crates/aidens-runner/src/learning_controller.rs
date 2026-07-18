//! Production composition for one bounded, real-sandbox learning run.
//! The controller owns no evidence: receipts and CEA remain canonical owners.

use crate::learning_candidate::{extract_procedure_candidate, ProcedureCandidateBlueprintV1};
use crate::learning_effectful::{
    evaluate_effectful_persisted, validate_effect_permit, EffectfulEvaluationReportV1,
    EffectfulEvaluationRequest,
};
use crate::learning_lifecycle::ProcedureLifecycleAdapter;
use aidens_contracts::{
    generated_artifact_id_from_material, LearningPreflightReceiptV1, PermitGrantV1,
    PermitUseReportV1,
};
use aidens_receipts::{CanonicalEventLog, CanonicalEventLogConfig};
use cea_sqlite::SqliteCeaStore;
use check_runner::{BackendConfig, ContainerBackend};
use semantic_memory::{
    AllowedProcedureToolV1, ApplicabilityPredicateV1, AuthorityScopeV1, AuthorityScopesV1,
    ElevationRequirementV1, MemoryConfig, MemoryStore, NamespaceScopeV1, OriginAuthorityLabelV1,
    OriginClassV1, OriginRiskV1, ProceduralMemoryArtifactV1, ProcedureActionV1,
    ProcedureCapabilityV1, ProcedureEffectV1, ProcedureEvidenceTestEnvelopeV1, ProcedureFixtureV1,
    ProcedureLifecycleReceiptV1, ProcedurePreconditionV1, ProcedureRiskV1, ProcedureStepV1,
    RevocationStatusV1, SubjectPrincipalV1,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use typed_patch::{validate_patch, PatchPolicy, StructuredPatch};

const COMMAND_TIMEOUT_SECS: u64 = 900;
const MEMORY_LIMIT: &str = "2g";
const CPU_LIMIT: &str = "2";

#[derive(Debug, Clone)]
pub struct RealSandboxLearningConfig {
    pub fixture: PathBuf,
    pub patch: StructuredPatch,
    pub patch_policy: PatchPolicy,
    pub permit_grant: PermitGrantV1,
    pub permit_use: PermitUseReportV1,
    pub image: String,
    pub cea_db: PathBuf,
    pub receipt_root: PathBuf,
    pub memory_store: PathBuf,
    pub run_id: String,
    pub attempt_id: String,
    pub trial_id: String,
    pub trace_id: String,
    pub recorded_at: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RealSandboxLearningError {
    #[error("invalid real-sandbox input: {0}")]
    Invalid(String),
    #[error("receipt owner failed: {0}")]
    Receipt(String),
    #[error("learning evaluation failed: {0}")]
    Evaluation(#[from] crate::learning_effectful::EffectfulEvaluationError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TerminalPublicationStateV1 {
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealSandboxLearningOutcomeV1 {
    pub report: EffectfulEvaluationReportV1,
    pub terminal_event_receipt_id: String,
    pub terminal_event_log_verified: bool,
    pub terminal_publication: TerminalPublicationStateV1,
    pub terminal_lifecycle_receipt: Option<ProcedureLifecycleReceiptV1>,
    pub reason_codes: Vec<String>,
}

pub async fn run_real_sandbox(
    config: RealSandboxLearningConfig,
) -> Result<RealSandboxLearningOutcomeV1, RealSandboxLearningError> {
    validate(&config)?;
    let patch_validation = validate_patch(&config.patch, &config.patch_policy);
    if !patch_validation.ok {
        return Err(RealSandboxLearningError::Invalid(
            "patch rejected before persistence".into(),
        ));
    }
    let fixture_tree_digest = fixture_digest(&config.fixture)?;
    let mut request = effectful_request(&config, false);
    validate_effect_permit(&request)?;
    let (preflight_id, receipt) = preflight_receipt(&config, fixture_tree_digest)?;
    receipt
        .validate()
        .map_err(|reason| RealSandboxLearningError::Invalid(reason.into()))?;
    let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(
        config.receipt_root.clone(),
    ))
    .map_err(|e| RealSandboxLearningError::Receipt(e.to_string()))?;
    let preflight_body = serde_json::to_value(&receipt)
        .map_err(|e| RealSandboxLearningError::Receipt(e.to_string()))?;
    log.append_json(
        "aidens-runner",
        "learning-preflight-v1",
        preflight_id.clone(),
        preflight_body.clone(),
    )
    .map_err(|e| RealSandboxLearningError::Receipt(e.to_string()))?;
    verify_persisted(&log, &preflight_id, &preflight_body)?;
    request.preflight_persisted = true;

    let backend = ContainerBackend::new(&BackendConfig {
        mode: "sealed_local".into(),
        execution_backend_preference: "container".into(),
        container_runtime_preference: "podman".into(),
        sealed_allow_host_backend: false,
        rust_image: config.image.clone(),
        command_timeout_secs: COMMAND_TIMEOUT_SECS,
        memory_limit: MEMORY_LIMIT.into(),
        cpu_limit: CPU_LIMIT.into(),
    })
    .map_err(|e| RealSandboxLearningError::Receipt(e.to_string()))?;
    let cea = SqliteCeaStore::open(&config.cea_db)
        .map_err(|e| RealSandboxLearningError::Receipt(e.to_string()))?;
    let report = evaluate_effectful_persisted(
        request,
        &backend,
        || backend.capability_truth_receipt(),
        &cea,
    )
    .await?;
    let terminal_body = serde_json::to_value(&report)
        .map_err(|e| RealSandboxLearningError::Receipt(e.to_string()))?;
    let terminal_material = serde_json::to_string(&terminal_body)
        .map_err(|e| RealSandboxLearningError::Receipt(e.to_string()))?;
    let terminal_id =
        generated_artifact_id_from_material("aidens-learning-terminal-event", &terminal_material)
            .as_str()
            .to_string();
    log.append_json(
        "aidens-runner",
        "effectful-evaluation-report-v1",
        terminal_id.clone(),
        terminal_body.clone(),
    )
    .map_err(|e| RealSandboxLearningError::Receipt(e.to_string()))?;
    verify_persisted(&log, &terminal_id, &terminal_body)?;

    // The raw effectful report is durable evidence, but a single run must not create
    // semantic-memory's promotion prerequisite. Register only a Tested candidate.
    let lifecycle =
        register_single_run_candidate(&config, &report, &preflight_id, &terminal_id).await?;

    Ok(RealSandboxLearningOutcomeV1 {
        report,
        terminal_event_receipt_id: terminal_id,
        terminal_event_log_verified: true,
        terminal_publication: TerminalPublicationStateV1::Pending,
        terminal_lifecycle_receipt: lifecycle,
        reason_codes: vec![
            "terminal-v3-publication-pending".into(),
            "procedure-lifecycle-owner-permit-required".into(),
            "forge-export-owner-evidence-unavailable".into(),
            "replay-retention-policy-owner-unavailable".into(),
        ],
    })
}

fn validate(c: &RealSandboxLearningConfig) -> Result<(), RealSandboxLearningError> {
    if !c.fixture.is_dir() {
        return Err(RealSandboxLearningError::Invalid(
            "fixture must be a directory".into(),
        ));
    }
    if c.image.contains(":latest") || !c.image.contains("@sha256:") {
        return Err(RealSandboxLearningError::Invalid(
            "image must be digest-pinned".into(),
        ));
    }
    for (name, value) in [
        ("run_id", &c.run_id),
        ("attempt_id", &c.attempt_id),
        ("trial_id", &c.trial_id),
        ("trace_id", &c.trace_id),
        ("recorded_at", &c.recorded_at),
    ] {
        if value.trim().is_empty() || value.contains("local-process-seq") {
            return Err(RealSandboxLearningError::Invalid(format!(
                "{name} must contain durable material"
            )));
        }
    }
    if c.cea_db.as_os_str().is_empty()
        || c.receipt_root.as_os_str().is_empty()
        || c.memory_store.as_os_str().is_empty()
    {
        return Err(RealSandboxLearningError::Invalid(
            "CEA, receipt, and canonical memory-store paths are required".into(),
        ));
    }
    Ok(())
}

fn digest_value<T: Serialize>(value: &T) -> Result<String, RealSandboxLearningError> {
    serde_json::to_vec(value)
        .map(|value| blake3::hash(&value).to_hex().to_string())
        .map_err(|e| RealSandboxLearningError::Invalid(e.to_string()))
}

fn effectful_request(
    config: &RealSandboxLearningConfig,
    preflight_persisted: bool,
) -> EffectfulEvaluationRequest {
    EffectfulEvaluationRequest {
        fixture: config.fixture.clone(),
        patch: config.patch.clone(),
        patch_policy: config.patch_policy.clone(),
        permit_grant: config.permit_grant.clone(),
        permit_use: config.permit_use.clone(),
        preflight_persisted,
        run_id: config.run_id.clone(),
        attempt_id: config.attempt_id.clone(),
        trial_id: config.trial_id.clone(),
        trace_id: config.trace_id.clone(),
        recorded_at: config.recorded_at.clone(),
    }
}

fn patch_policy_material(policy: &PatchPolicy) -> (&[String], bool, usize, usize, usize) {
    (
        &policy.forbidden_paths,
        policy.allow_test_modifications,
        policy.max_files_changed,
        policy.max_total_lines_changed,
        policy.max_lines_changed_per_file,
    )
}

fn preflight_receipt(
    config: &RealSandboxLearningConfig,
    fixture_tree_digest: String,
) -> Result<(String, LearningPreflightReceiptV1), RealSandboxLearningError> {
    let patch_digest = digest_value(&config.patch)?;
    let patch_policy_digest = digest_value(&patch_policy_material(&config.patch_policy))?;
    let permit_scope_digest = digest_value(&config.permit_grant)?;
    let backend_limits_digest = digest_value(&(
        "sealed_local",
        "container",
        "podman",
        false,
        COMMAND_TIMEOUT_SECS,
        MEMORY_LIMIT,
        CPU_LIMIT,
    ))?;
    let material = serde_json::to_string(&(
        &fixture_tree_digest,
        &patch_digest,
        &patch_policy_digest,
        config.permit_grant.permit_id.as_str(),
        config.permit_use.receipt_id.as_str(),
        &permit_scope_digest,
        &config.image,
        &backend_limits_digest,
        &config.run_id,
        &config.attempt_id,
        &config.trial_id,
        &config.trace_id,
        &config.recorded_at,
        &config.cea_db,
        &config.receipt_root,
        &config.memory_store,
    ))
    .map_err(|error| RealSandboxLearningError::Invalid(error.to_string()))?;
    let material_id = generated_artifact_id_from_material("aidens-learning-preflight", &material)
        .as_str()
        .to_string();
    let receipt = LearningPreflightReceiptV1 {
        schema: LearningPreflightReceiptV1::SCHEMA.into(),
        material_id: material_id.clone(),
        fixture_tree_digest,
        patch_digest,
        patch_policy_digest,
        permit_grant_id: config.permit_grant.permit_id.as_str().into(),
        permit_use_id: config.permit_use.receipt_id.as_str().into(),
        permit_scope_digest,
        image: config.image.clone(),
        backend_limits_digest,
        run_id: config.run_id.clone(),
        attempt_id: config.attempt_id.clone(),
        trial_id: config.trial_id.clone(),
        trace_id: config.trace_id.clone(),
        requested_recorded_at: config.recorded_at.clone(),
        cea_store_identity: config.cea_db.to_string_lossy().into(),
        receipt_root_owner: config.receipt_root.to_string_lossy().into(),
        memory_store_owner: config.memory_store.to_string_lossy().into(),
    };
    Ok((material_id, receipt))
}

/// Build the canonical procedure blueprint from a publication-complete real evaluation.
///
/// The artifact is the durable, owner-validated description of *what* ran
/// (sealed cargo checks in a Rust fixture). Its lifecycle truth (compile, test,
/// promote) lives entirely in `semantic-memory`.
fn procedure_blueprint(
    config: &RealSandboxLearningConfig,
    report: &EffectfulEvaluationReportV1,
    preflight_receipt_id: &str,
    terminal_event_receipt_id: &str,
) -> Result<ProceduralMemoryArtifactV1, RealSandboxLearningError> {
    let patch_policy = patch_policy_json(&config.patch_policy);
    let evidence_binding = serde_json::json!({
        "schema": "AiDENsProcedureEvidenceBindingV1",
        "preflight_receipt_id": preflight_receipt_id,
        "terminal_event_receipt_id": terminal_event_receipt_id,
        "run_id": config.run_id,
        "attempt_id": config.attempt_id,
        "trial_id": config.trial_id,
        "trace_id": config.trace_id,
        "patch_digest": report.patch_digest,
        "before_tree_digest": report.before_tree_digest,
        "after_tree_digest": report.after_tree_digest,
        "rollback_tree_digest": report.rollback_tree_digest,
        "patch_policy": patch_policy,
        "sandbox_image": config.image,
        "sandbox_capability": report.sandbox_capability,
        "permit_use_receipt_id": report.permit_use_receipt_id,
        "check_output_digests": {
            "fmt": report.checks.fmt_output_digest,
            "clippy": report.checks.clippy_output_digest,
            "test": report.checks.test_output_digest,
        },
        "verification_decision_id": report.verification.promotion_decision.decision_id,
        "cea_run_hash": report.cea_run_hash,
    });
    let identity_material = serde_json::to_string(&evidence_binding)
        .map_err(|error| RealSandboxLearningError::Invalid(error.to_string()))?;
    let artifact_id =
        generated_artifact_id_from_material("aidens-learning-procedure", &identity_material)
            .as_str()
            .to_string();
    let (step, tool) = structured_patch_step(&artifact_id, &config.patch)?;
    let fixture_id =
        generated_artifact_id_from_material("aidens-learning-fixture", &report.before_tree_digest)
            .as_str()
            .to_string();
    let fixture = ProcedureFixtureV1::new(
        fixture_id,
        serde_json::json!({
            "language": "rust",
            "source_tree_digest": report.before_tree_digest,
            "patch_policy": patch_policy,
            "sandbox_image": config.image,
            "sandbox_capability_digest": report.sandbox_capability.content_digest,
            "evidence_binding": evidence_binding,
        }),
        vec!["typed-patch:structured-apply:1".into()],
        vec![ProcedureEffectV1::new(
            "fixture_source_edited",
            serde_json::json!(true),
        )],
        vec![ProcedureEffectV1::new(
            "network_access",
            serde_json::json!(false),
        )],
    );
    let envelope = ProcedureEvidenceTestEnvelopeV1::new("sandbox-v1", vec![fixture], vec![]);
    let principal = config.permit_grant.granted_by.clone();
    let origin = OriginAuthorityLabelV1::new(
        OriginClassV1::OperatorSystem,
        principal.clone(),
        "aidens-learning-controller",
        report.sandbox_capability.content_digest.clone(),
        OriginRiskV1::Medium,
        AuthorityScopesV1 {
            recall: AuthorityScopeV1::Audience,
            assertion: AuthorityScopeV1::Denied,
            action: AuthorityScopeV1::Audience,
        },
        ElevationRequirementV1::ExplicitOperatorApproval,
        None,
        RevocationStatusV1::Active,
        vec![principal.clone()],
    )
    .map_err(RealSandboxLearningError::Invalid)?
    .with_subject_principal(
        SubjectPrincipalV1::new(principal.clone())
            .map_err(|reason| RealSandboxLearningError::Invalid(reason.to_string()))?,
    )
    .with_resource_scope(NamespaceScopeV1::exact("aidens-learning"));
    ProceduralMemoryArtifactV1::new(
        artifact_id,
        ProcedureCapabilityV1::new("aidens-learning", "real-sandbox-evaluation"),
        ProcedureActionV1::new(
            "apply_typed_patch_and_run_sealed_checks",
            "Apply the source-bound typed patch and run sealed cargo fmt/clippy/test",
        ),
        vec![ApplicabilityPredicateV1::equals(
            "language",
            serde_json::json!("rust"),
        )],
        vec![ProcedurePreconditionV1::equals(
            "source_tree_digest",
            serde_json::json!(report.before_tree_digest),
        )],
        vec![step],
        vec![tool],
        vec![ProcedureEffectV1::new(
            "fixture_source_edited",
            serde_json::json!(true),
        )],
        vec![ProcedureEffectV1::new(
            "network_access",
            serde_json::json!(false),
        )],
        ProcedureRiskV1::Medium,
        origin,
        principal.clone(),
        vec![principal.clone()],
        NamespaceScopeV1::exact("aidens-learning"),
        1,
        None,
        envelope,
        None,
    )
    .map_err(RealSandboxLearningError::Invalid)
}

fn structured_patch_step(
    artifact_id: &str,
    patch: &StructuredPatch,
) -> Result<(ProcedureStepV1, AllowedProcedureToolV1), RealSandboxLearningError> {
    let arguments = serde_json::to_value(patch)
        .map_err(|error| RealSandboxLearningError::Invalid(error.to_string()))?;
    let argument_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "patch_id": {"type": "string"},
            "summary": {"type": "string"},
            "edits": {"type": "array", "items": {"type": "object"}},
            "notes": {"type": "array", "items": {"type": "string"}}
        },
        "required": ["patch_id", "summary", "edits", "notes"],
        "additionalProperties": false
    });
    let step = ProcedureStepV1::tool(
        format!("step:{artifact_id}"),
        "typed-patch:structured-apply:1",
        arguments,
        Some("restore_source_tree_from_pre_effect_snapshot".into()),
    );
    let tool = AllowedProcedureToolV1::new("typed-patch:structured-apply:1", argument_schema);
    Ok((step, tool))
}

fn patch_policy_json(policy: &PatchPolicy) -> serde_json::Value {
    serde_json::json!({
        "forbidden_paths": policy.forbidden_paths,
        "allow_test_modifications": policy.allow_test_modifications,
        "max_files_changed": policy.max_files_changed,
        "max_total_lines_changed": policy.max_total_lines_changed,
        "max_lines_changed_per_file": policy.max_lines_changed_per_file,
    })
}

/// Record one publication-complete evaluation without a controlled transition.
///
/// One successful run is effectful evidence, not paired promotion evidence. The
/// tested candidate therefore remains unavailable for action until a separately
/// governed multi-family evaluation and owner-issued permit authorize promotion.
async fn register_single_run_candidate(
    config: &RealSandboxLearningConfig,
    report: &EffectfulEvaluationReportV1,
    preflight_receipt_id: &str,
    terminal_event_id: &str,
) -> Result<Option<ProcedureLifecycleReceiptV1>, RealSandboxLearningError> {
    let blueprint =
        match procedure_blueprint(config, report, preflight_receipt_id, terminal_event_id) {
            Ok(blueprint) => blueprint,
            Err(error) => {
                return Err(RealSandboxLearningError::Receipt(format!(
                    "procedure blueprint construction failed: {error}"
                )))
            }
        };
    let store = MemoryStore::open(MemoryConfig {
        base_dir: config.memory_store.clone(),
        ..MemoryConfig::default()
    })
    .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    let adapter = ProcedureLifecycleAdapter::new(&store);
    let key = generated_artifact_id_from_material(
        "aidens-learning-lifecycle",
        &format!("{}-{}", blueprint.artifact_id, terminal_event_id),
    )
    .as_str()
    .to_string();
    let artifact = extract_procedure_candidate(
        Some(report),
        Some(&ProcedureCandidateBlueprintV1 {
            artifact: blueprint,
        }),
    )
    .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    let lifecycle_receipt = store
        .compile_procedure(artifact.clone(), format!("compile:{key}"))
        .await
        .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    let tested = adapter
        .test(&lifecycle_receipt.artifact_id, format!("test:{key}"))
        .await
        .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    Ok(Some(tested))
}

fn verify_persisted(
    log: &CanonicalEventLog,
    receipt_id: &str,
    expected_body: &serde_json::Value,
) -> Result<(), RealSandboxLearningError> {
    let reopened = CanonicalEventLog::open(log.config().clone())
        .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    let record = reopened
        .inspect(receipt_id)
        .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    let chain_verified = reopened
        .verify_chain()
        .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    if record.body != *expected_body
        || !record.verify_digest()
        || !record.verify_record_digest()
        || !chain_verified
    {
        return Err(RealSandboxLearningError::Receipt(format!(
            "persisted canonical record failed readback verification: {receipt_id}"
        )));
    }
    Ok(())
}

fn fixture_digest(path: &Path) -> Result<String, RealSandboxLearningError> {
    let mut entries = Vec::new();
    for entry in walkdir::WalkDir::new(path).follow_links(false) {
        let entry = entry.map_err(|e| RealSandboxLearningError::Invalid(e.to_string()))?;
        if entry.file_type().is_symlink() {
            return Err(RealSandboxLearningError::Invalid(format!(
                "fixture identity rejects symlink: {}",
                entry.path().display()
            )));
        }
        if entry.file_type().is_file() {
            let relative = entry.path().strip_prefix(path).map_err(|error| {
                RealSandboxLearningError::Invalid(format!(
                    "fixture path escaped declared root: {error}"
                ))
            })?;
            entries.push((
                relative.to_string_lossy().to_string(),
                std::fs::read(entry.path())
                    .map_err(|e| RealSandboxLearningError::Invalid(e.to_string()))?,
            ));
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    digest_value(&entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidens_contracts::{ArtifactId, CanonicalToolSideEffectClass};
    use semantic_memory::ProcedureLifecycleDispositionV1;

    fn config() -> (
        tempfile::TempDir,
        tempfile::TempDir,
        RealSandboxLearningConfig,
    ) {
        let fixture = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(fixture.path().join("src")).unwrap();
        std::fs::write(
            fixture.path().join("Cargo.toml"),
            "[package]\nname='controller-fixture'\nversion='0.1.0'\nedition='2021'\n",
        )
        .unwrap();
        std::fs::write(
            fixture.path().join("src/lib.rs"),
            "pub fn answer() -> u32 { 1 }\n",
        )
        .unwrap();
        let patch = serde_json::from_value(serde_json::json!({
            "patch_id": "00000000-0000-4000-8000-000000000030",
            "summary": "bounded controller patch",
            "edits": [{
                "path": "src/lib.rs",
                "ops": [{"Replace": {
                    "range": {"start": 1, "end_exclusive": 2},
                    "lines": ["pub fn answer() -> u32 {", "    42", "}"]
                }}],
                "mode": "Modify"
            }],
            "notes": []
        }))
        .unwrap();
        let root = fixture.path().to_string_lossy().to_string();
        let run_id = ArtifactId::new("run:controller-material");
        let attempt_id = ArtifactId::new("attempt:controller-material");
        let mut permit_grant = PermitGrantV1::scoped(
            CanonicalToolSideEffectClass::Write,
            "aidens:patch-apply:1",
            root.clone(),
            "operator:controller-test",
        );
        permit_grant.permit_id = ArtifactId::new("permit:controller-material");
        permit_grant.run_id = Some(run_id.clone());
        permit_grant.attempt_id = Some(attempt_id.clone());
        let mut permit_use = PermitUseReportV1::allowed(
            &permit_grant,
            "aidens:patch-apply:1",
            root,
            Some(run_id),
            Some(attempt_id),
        );
        permit_use.receipt_id = ArtifactId::new("permit-use:controller-material");
        let owner_root = tempfile::tempdir().unwrap();
        let config = RealSandboxLearningConfig {
            fixture: fixture.path().to_path_buf(),
            patch,
            patch_policy: PatchPolicy {
                forbidden_paths: vec![".git".into()],
                allow_test_modifications: false,
                max_files_changed: 2,
                max_total_lines_changed: 20,
                max_lines_changed_per_file: 20,
            },
            permit_grant,
            permit_use,
            image: format!("localhost/aidens-rust-checks@sha256:{}", "0".repeat(64)),
            cea_db: owner_root.path().join("cea.sqlite"),
            receipt_root: owner_root.path().join("receipts"),
            memory_store: owner_root.path().join("memory"),
            run_id: "run:controller-material".into(),
            attempt_id: "attempt:controller-material".into(),
            trial_id: "trial:controller-material".into(),
            trace_id: "0af7651916cd43dd8448eb211c80319c".into(),
            recorded_at: "2026-07-17T00:00:00Z".into(),
        };
        (fixture, owner_root, config)
    }

    #[test]
    fn preflight_identity_is_material_bound_and_readback_verified() {
        let (_fixture, _owner_root, mut config) = config();
        assert_eq!(config.permit_grant.granted_by, "operator:controller-test");
        let digest = fixture_digest(&config.fixture).unwrap();
        let (first_id, first) = preflight_receipt(&config, digest.clone()).unwrap();
        let (same_id, same) = preflight_receipt(&config, digest).unwrap();
        assert_eq!(first_id, same_id);
        assert_eq!(first, same);
        first.validate().unwrap();

        config.image = format!("localhost/other@sha256:{}", "1".repeat(64));
        let changed_digest = fixture_digest(&config.fixture).unwrap();
        let (changed_id, _) = preflight_receipt(&config, changed_digest).unwrap();
        assert_ne!(first_id, changed_id);

        config.image = same.image.clone();
        config.memory_store = config.memory_store.join("different-owner");
        let changed_digest = fixture_digest(&config.fixture).unwrap();
        let (destination_changed_id, destination_changed) =
            preflight_receipt(&config, changed_digest).unwrap();
        assert_ne!(first_id, destination_changed_id);
        assert_ne!(
            same.memory_store_owner,
            destination_changed.memory_store_owner
        );

        let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(
            config.receipt_root.clone(),
        ))
        .unwrap();
        let body = serde_json::to_value(&same).unwrap();
        log.append_json(
            "aidens-runner",
            "learning-preflight-v1",
            same_id.clone(),
            body.clone(),
        )
        .unwrap();
        verify_persisted(&log, &same_id, &body).unwrap();
    }

    #[test]
    fn procedure_step_is_the_exact_source_bound_structured_patch() {
        let (fixture, _owner_root, config) = config();
        let expected = serde_json::to_value(&config.patch).unwrap();
        let (step, tool) = structured_patch_step("artifact:one", &config.patch).unwrap();

        assert_eq!(step.arguments, expected);
        assert_eq!(step.tool, "typed-patch:structured-apply:1");
        assert_eq!(tool.tool, step.tool);
        assert!(!step
            .arguments
            .to_string()
            .contains(&fixture.path().to_string_lossy().to_string()));

        let mut changed_patch = config.patch.clone();
        changed_patch.summary = "materially different patch".into();
        let (changed_step, _) = structured_patch_step("artifact:two", &changed_patch).unwrap();
        assert_ne!(step.arguments, changed_step.arguments);
    }

    #[tokio::test]
    async fn invalid_permit_creates_no_preflight_record_or_backend_effect() {
        let (_fixture, _owner_root, mut config) = config();
        config.permit_use.allowed = false;
        let records_path = config.receipt_root.join("canonical-receipts.ndjson");
        let error = run_real_sandbox(config).await.unwrap_err();
        assert!(matches!(error, RealSandboxLearningError::Evaluation(_)));
        assert!(!records_path.exists());
    }

    #[tokio::test]
    #[ignore = "requires live rootless Podman and the digest-pinned AiDENs image"]
    async fn live_controller_registers_exact_effectful_candidate_without_promotion() {
        let (_fixture, _owner_root, mut config) = config();
        config.image = std::env::var("AIDENS_LIVE_RUST_IMAGE").unwrap_or_else(|_| {
            "localhost/aidens-rust-checks@sha256:96f6610f945d10b523a303848610bd6fbef241762c59c0e44d47af9089cb6d6b".into()
        });
        let outcome = run_real_sandbox(config).await.unwrap();
        assert!(
            outcome.report.verified,
            "live controller report remained nonverified: reasons={:?}, checks={:?}, adjudication={:?}",
            outcome.report.reason_codes,
            outcome.report.checks,
            outcome.report.verification
        );
        assert!(outcome.report.rollback_verified);
        assert!(outcome.report.cea_persisted);
        assert!(outcome.terminal_event_log_verified);
        let receipt = outcome
            .terminal_lifecycle_receipt
            .as_ref()
            .expect("procedure lifecycle receipt must be present after effectful evaluation");
        assert_eq!(receipt.disposition, ProcedureLifecycleDispositionV1::Tested);
        assert_eq!(
            outcome.terminal_publication,
            TerminalPublicationStateV1::Pending
        );
        assert_eq!(
            outcome.reason_codes,
            vec![
                String::from("terminal-v3-publication-pending"),
                String::from("procedure-lifecycle-owner-permit-required"),
                String::from("forge-export-owner-evidence-unavailable"),
                String::from("replay-retention-policy-owner-unavailable"),
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn fixture_identity_rejects_symlink_without_following_it() {
        use std::os::unix::fs::symlink;

        let (_fixture, _owner_root, config) = config();
        symlink("/etc/passwd", config.fixture.join("src/outside")).unwrap();
        assert!(matches!(
            fixture_digest(&config.fixture),
            Err(RealSandboxLearningError::Invalid(_))
        ));
    }
}
