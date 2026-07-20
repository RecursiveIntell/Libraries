//! Production composition for one bounded, real-sandbox learning run.
//! The controller owns no evidence: receipts and CEA remain canonical owners.

use crate::learning_candidate::{extract_procedure_candidate, ProcedureCandidateBlueprintV1};
use crate::learning_effectful::{
    evaluate_effectful_persisted, validate_effect_permit, EffectfulEvaluationReportV1,
    EffectfulEvaluationRequest,
};
use crate::learning_lifecycle::ProcedureLifecycleAdapter;
use aidens_contracts::{
    generated_artifact_id_from_material, LearningPreflightReceiptV1, LearningPreflightReceiptV2,
    PermitGrantV1, PermitUseReportV1,
};
use aidens_receipts::{CanonicalEventLog, CanonicalEventLogConfig};
use cea_sqlite::SqliteCeaStore;
use check_runner::{BackendConfig, ContainerBackend};
use forge_engine::lab::evaluate::ScoreVector;
use forge_engine::lab::evidence::{
    BundleScope, Covariates, ReceiptKind, ReceiptRef, ReceiptStorage, Treatment,
};
use forge_engine::{ClaimStrength, ExperimentEvidenceBundle, ForgeStore};
use semantic_memory::{
    verify_procedure_lifecycle_receipt_v1, AllowedProcedureToolV1, ApplicabilityPredicateV1,
    AuthorityScopeV1, AuthorityScopesV1, CallerPrincipalV1, ElevationRequirementV1,
    GovernedAccessPurposeV1, MemoryConfig, MemoryStore, NamespaceScopeV1, OriginAuthorityLabelV1,
    OriginClassV1, OriginRiskV1, ProceduralMemoryArtifactV1, ProcedureAccessPathV1,
    ProcedureActionPermitV1, ProcedureActionV1, ProcedureCapabilityV1, ProcedureEffectV1,
    ProcedureEffectfulEvaluationReceiptV1, ProcedureEvidenceTestEnvelopeV1, ProcedureFixtureV1,
    ProcedureLifecycleDispositionV1, ProcedureLifecycleReceiptV1, ProcedurePreconditionV1,
    ProcedureRetrievalRequestV1, ProcedureRiskV1, ProcedureStepV1, RevocationStatusV1,
    SubjectPrincipalV1,
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
    pub forge_store: PathBuf,
    pub publication_namespace: String,
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
    pub preflight_receipt_id: String,
    pub preflight_receipt: LearningPreflightReceiptV2,
    pub terminal_event_receipt_id: String,
    pub terminal_event_log_verified: bool,
    pub terminal_publication: TerminalPublicationStateV1,
    pub terminal_lifecycle_receipt: Option<ProcedureLifecycleReceiptV1>,
    pub terminal_effectful_receipt: Option<ProcedureEffectfulEvaluationReceiptV1>,
    pub terminal_evidence_bundle: Option<ExperimentEvidenceBundle>,
    pub terminal_evidence_readback_verified: bool,
    pub reason_codes: Vec<String>,
}

struct RegisteredCandidateV1 {
    artifact: ProceduralMemoryArtifactV1,
    lifecycle: ProcedureLifecycleReceiptV1,
    effectful: ProcedureEffectfulEvaluationReceiptV1,
}

pub(crate) struct EffectfulRunCoreV1 {
    pub report: EffectfulEvaluationReportV1,
    pub preflight_id: String,
    preflight_receipt: LearningPreflightReceiptV2,
    pub terminal_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotedProcedureReplayOutcomeV1 {
    pub schema: String,
    pub artifact_id: String,
    pub artifact_digest: String,
    pub report: EffectfulEvaluationReportV1,
    pub terminal_event_receipt_id: String,
    pub terminal_event_log_verified: bool,
    pub action_allowed: bool,
    pub retained_patch_exact: bool,
    pub governed_retrieval_receipt_digest: String,
    pub promotion_receipt_id: String,
    pub promotion_receipt_digest: String,
}

pub async fn run_real_sandbox(
    config: RealSandboxLearningConfig,
) -> Result<RealSandboxLearningOutcomeV1, RealSandboxLearningError> {
    let core = execute_effectful_run(&config).await?;
    let report = core.report;
    let preflight_id = core.preflight_id;
    let preflight_receipt = core.preflight_receipt;
    let terminal_id = core.terminal_id;

    // The exact candidate's publication-complete execution is registered as its
    // owner-native promotion prerequisite, but promotion remains separately permitted.
    let registered =
        register_single_run_candidate(&config, &report, &preflight_id, &terminal_id).await?;
    let evidence_bundle =
        build_terminal_evidence_bundle(&config, &report, &preflight_id, &terminal_id, &registered)?;
    persist_and_verify_terminal_bundle(&config, &evidence_bundle)?;

    Ok(RealSandboxLearningOutcomeV1 {
        report,
        preflight_receipt_id: preflight_id,
        preflight_receipt,
        terminal_event_receipt_id: terminal_id,
        terminal_event_log_verified: true,
        terminal_publication: TerminalPublicationStateV1::Pending,
        terminal_lifecycle_receipt: Some(registered.lifecycle),
        terminal_effectful_receipt: Some(registered.effectful),
        terminal_evidence_bundle: Some(evidence_bundle),
        terminal_evidence_readback_verified: true,
        reason_codes: vec![
            "terminal-v3-publication-pending".into(),
            "procedure-lifecycle-owner-permit-required".into(),
            "replay-retention-policy-owner-unavailable".into(),
        ],
    })
}

pub(crate) async fn execute_effectful_run(
    config: &RealSandboxLearningConfig,
) -> Result<EffectfulRunCoreV1, RealSandboxLearningError> {
    validate(config)?;
    let patch_validation = validate_patch(&config.patch, &config.patch_policy);
    if !patch_validation.ok {
        return Err(RealSandboxLearningError::Invalid(
            "patch rejected before persistence".into(),
        ));
    }
    let fixture_tree_digest = fixture_digest(&config.fixture)?;
    let mut request = effectful_request(config, false);
    validate_effect_permit(&request)?;
    let (preflight_id, receipt) = preflight_receipt(config, fixture_tree_digest)?;
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

    Ok(EffectfulRunCoreV1 {
        report,
        preflight_id,
        preflight_receipt: receipt,
        terminal_id,
    })
}

/// Retrieve and execute one promoted, exact source-bound procedure through the
/// same preflighted sealed owner path without registering a new candidate.
pub async fn replay_promoted_procedure(
    mut config: RealSandboxLearningConfig,
    artifact_id: &str,
    expected_source_tree_digest: &str,
    action_permit: ProcedureActionPermitV1,
) -> Result<PromotedProcedureReplayOutcomeV1, RealSandboxLearningError> {
    validate(&config)?;
    let principal = config.permit_grant.granted_by.clone();
    let memory = MemoryStore::open(MemoryConfig {
        base_dir: config.memory_store.clone(),
        ..MemoryConfig::default()
    })
    .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    let mut request = ProcedureRetrievalRequestV1::new(
        ProcedureCapabilityV1::new("aidens-learning", "real-sandbox-evaluation"),
        ProcedureActionV1::new(
            "apply_typed_patch_and_run_sealed_checks",
            "Apply the source-bound typed patch and run sealed cargo fmt/clippy/test",
        ),
        serde_json::json!({
            "language": "rust",
            "source_tree_digest": expected_source_tree_digest,
        }),
        CallerPrincipalV1::new(principal.clone())
            .map_err(|reason| RealSandboxLearningError::Invalid(reason.to_string()))?,
        SubjectPrincipalV1::new(principal.clone())
            .map_err(|reason| RealSandboxLearningError::Invalid(reason.to_string()))?,
        vec![principal],
        NamespaceScopeV1::exact("aidens-learning"),
        GovernedAccessPurposeV1::Action,
        ProcedureAccessPathV1::DirectId,
    );
    request.artifact_id = Some(artifact_id.into());
    request.action_permit = Some(action_permit);
    let retrieved = memory
        .retrieve_procedure(request)
        .await
        .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    if !retrieved.decision.action_allowed {
        return Err(RealSandboxLearningError::Invalid(format!(
            "governed promoted-procedure retrieval denied action: {:?}",
            retrieved.decision
        )));
    }
    let lifecycle = retrieved.lifecycle_receipt.clone().ok_or_else(|| {
        RealSandboxLearningError::Invalid(
            "governed retrieval omitted the current lifecycle receipt".into(),
        )
    })?;
    if lifecycle.disposition != ProcedureLifecycleDispositionV1::Promoted
        || lifecycle.artifact_id != artifact_id
        || !verify_procedure_lifecycle_receipt_v1(&lifecycle)
    {
        return Err(RealSandboxLearningError::Invalid(
            "governed retrieval lifecycle receipt is not a valid current promotion".into(),
        ));
    }
    let governed_retrieval_receipt_digest = retrieved.receipt_digest.clone();
    let artifact = retrieved.candidate.ok_or_else(|| {
        RealSandboxLearningError::Invalid("promoted procedure candidate not found".into())
    })?;
    if lifecycle.artifact_digest != artifact.artifact_digest {
        return Err(RealSandboxLearningError::Invalid(
            "promotion receipt artifact digest differs from the governed candidate".into(),
        ));
    }
    let step = artifact.steps.first().ok_or_else(|| {
        RealSandboxLearningError::Invalid("promoted procedure has no executable step".into())
    })?;
    if step.tool != "typed-patch:structured-apply:1" {
        return Err(RealSandboxLearningError::Invalid(
            "promoted procedure tool is not the canonical typed-patch owner".into(),
        ));
    }
    let retained_patch: StructuredPatch = serde_json::from_value(step.arguments.clone())
        .map_err(|error| RealSandboxLearningError::Invalid(error.to_string()))?;
    if digest_value(&retained_patch)? != digest_value(&config.patch)? {
        return Err(RealSandboxLearningError::Invalid(
            "caller expected patch differs from retained promoted procedure".into(),
        ));
    }
    config.patch = retained_patch;
    let core = execute_effectful_run(&config).await?;
    if core.report.before_tree_digest != expected_source_tree_digest {
        return Err(RealSandboxLearningError::Invalid(
            "replay source tree differs from the promoted procedure binding".into(),
        ));
    }
    Ok(PromotedProcedureReplayOutcomeV1 {
        schema: "AiDENsPromotedProcedureReplayOutcomeV1".into(),
        artifact_id: artifact.artifact_id,
        artifact_digest: artifact.artifact_digest,
        report: core.report,
        terminal_event_receipt_id: core.terminal_id,
        terminal_event_log_verified: true,
        action_allowed: true,
        retained_patch_exact: true,
        governed_retrieval_receipt_digest,
        promotion_receipt_id: lifecycle.receipt_id,
        promotion_receipt_digest: lifecycle.receipt_digest,
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
        || c.forge_store.as_os_str().is_empty()
        || c.publication_namespace.trim().is_empty()
    {
        return Err(RealSandboxLearningError::Invalid(
            "CEA, receipt, canonical memory/Forge stores, and publication namespace are required"
                .into(),
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

pub(crate) fn patch_policy_material(
    policy: &PatchPolicy,
) -> (&[String], bool, usize, usize, usize) {
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
) -> Result<(String, LearningPreflightReceiptV2), RealSandboxLearningError> {
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
    let cea_store_identity = owner_path_digest(&config.cea_db)?;
    let receipt_root_owner = owner_path_digest(&config.receipt_root)?;
    let memory_store_owner = owner_path_digest(&config.memory_store)?;
    let forge_store_owner = owner_path_digest(&config.forge_store)?;
    let material = serde_json::to_string(&serde_json::json!({
        "fixture_tree_digest": fixture_tree_digest,
        "patch_digest": patch_digest,
        "patch_policy_digest": patch_policy_digest,
        "permit_grant_id": config.permit_grant.permit_id.as_str(),
        "permit_use_id": config.permit_use.receipt_id.as_str(),
        "permit_scope_digest": permit_scope_digest,
        "image": config.image,
        "backend_limits_digest": backend_limits_digest,
        "run_id": config.run_id,
        "attempt_id": config.attempt_id,
        "trial_id": config.trial_id,
        "trace_id": config.trace_id,
        "recorded_at": config.recorded_at,
        "cea_store_identity": cea_store_identity,
        "receipt_root_owner": receipt_root_owner,
        "memory_store_owner": memory_store_owner,
        "forge_store_owner": forge_store_owner,
        "publication_namespace": config.publication_namespace,
    }))
    .map_err(|error| RealSandboxLearningError::Invalid(error.to_string()))?;
    let material_id = generated_artifact_id_from_material("aidens-learning-preflight", &material)
        .as_str()
        .to_string();
    let receipt = LearningPreflightReceiptV2 {
        schema: LearningPreflightReceiptV2::SCHEMA.into(),
        execution: LearningPreflightReceiptV1 {
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
            cea_store_identity,
            receipt_root_owner,
            memory_store_owner,
        },
        forge_store_owner,
        publication_namespace: config.publication_namespace.clone(),
    };
    Ok((material_id, receipt))
}

fn owner_path_digest(path: &Path) -> Result<String, RealSandboxLearningError> {
    digest_value(&path.to_string_lossy()).map(|digest| format!("blake3:{digest}"))
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
) -> Result<RegisteredCandidateV1, RealSandboxLearningError> {
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
    let effectful = adapter
        .record_effectful_evaluation(
            &artifact.artifact_id,
            &artifact.artifact_digest,
            report,
            format!("effectful:{key}"),
        )
        .await
        .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    Ok(RegisteredCandidateV1 {
        artifact,
        lifecycle: tested,
        effectful,
    })
}

fn owner_receipt_ref(
    receipt_id: &str,
    kind: ReceiptKind,
    table: &str,
    content_hash: &str,
    trace_id: &str,
) -> ReceiptRef {
    ReceiptRef {
        receipt_id: receipt_id.into(),
        kind,
        storage: ReceiptStorage::StoreRow {
            table: table.into(),
            key: receipt_id.into(),
        },
        content_hash: content_hash.trim_start_matches("blake3:").into(),
        trace_id: Some(trace_id.into()),
        replay_handle: None,
    }
}

fn build_terminal_evidence_bundle(
    config: &RealSandboxLearningConfig,
    report: &EffectfulEvaluationReportV1,
    preflight_receipt_id: &str,
    terminal_event_id: &str,
    registered: &RegisteredCandidateV1,
) -> Result<ExperimentEvidenceBundle, RealSandboxLearningError> {
    let bundle_material = serde_json::to_string(&(
        "aidens-exact-source-execution-evidence-v1",
        &registered.artifact.artifact_id,
        &registered.artifact.artifact_digest,
        &registered.effectful.receipt_id,
        preflight_receipt_id,
        terminal_event_id,
        &config.publication_namespace,
    ))
    .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    let bundle_id = generated_artifact_id_from_material(
        "aidens-exact-source-execution-evidence",
        &bundle_material,
    )
    .as_str()
    .to_string();
    let terminal_hash = digest_value(report)?;
    let mut config_flags = report.sandbox_capability.mechanisms.clone();
    config_flags.push(format!("image:{}", config.image));
    config_flags.push(format!(
        "publication_namespace:{}",
        config.publication_namespace
    ));
    config_flags.push(format!(
        "memory_store_owner_digest:{}",
        digest_value(&config.memory_store.to_string_lossy().to_string())?
    ));
    config_flags.push(format!(
        "forge_store_owner_digest:{}",
        digest_value(&config.forge_store.to_string_lossy().to_string())?
    ));
    config_flags.sort();
    let mut bundle = ExperimentEvidenceBundle {
        bundle_id,
        candidate_id: registered.artifact.artifact_id.clone(),
        eval_id: registered.effectful.receipt_id.clone(),
        version_id: "aidens-exact-source-execution-evidence-v1".into(),
        supersedes_claim_version_id: None,
        relation_lineage_hints: Default::default(),
        scores: ScoreVector {
            correctness: 1.0,
            novelty: 0.0,
            stability: 0.0,
            weighted_total: 1.0,
            cea_confidence: Some(1.0),
            cea_predicted_correctness: None,
        },
        hypotheses: Vec::new(),
        verification: None,
        trace_id: Some(config.trace_id.clone()),
        experiment_diff: None,
        attribution_json: Some(
            serde_json::to_string(&report.cea_backpointer)
                .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?,
        ),
        assessment: None,
        warnings: vec![
            "single-arm execution; no comparative effect claim".into(),
            "exact source-tree scope only".into(),
            "v2 oracle corpus is qualification-only and not candidate evidence".into(),
        ],
        created_at: config.recorded_at.clone(),
        run_id: Some(config.run_id.clone()),
        attempt_id: Some(config.attempt_id.clone()),
        causal_question: Some(
            "Did the exact source-bound patch complete its declared sealed checks?".into(),
        ),
        unit_definition: Some("one exact patch execution on one frozen source tree".into()),
        bundle_scope: Some(BundleScope {
            workload_id: report.before_tree_digest.clone(),
            backend_family: "sealed-container".into(),
            selected_checks: vec!["fmt".into(), "clippy".into(), "test".into()],
            timeout_class: format!("{COMMAND_TIMEOUT_SECS}s"),
            config_flags: config_flags.clone(),
        }),
        pair_comparability: None,
        claim_strength: ClaimStrength::ExecutionVerifiedNoComparison,
        identification_rationale: Some(
            "observed sealed execution only; no baseline or generalization estimator".into(),
        ),
        known_threats: vec![
            "single source tree".into(),
            "single effectful execution".into(),
            "no cross-task learner treatment".into(),
        ],
        patch_hash: Some(report.patch_digest.clone()),
        treatment: Some(Treatment {
            kind: "exact_source_bound_patch".into(),
            patch_hash: report.patch_digest.clone(),
            patch_summary: config.patch.summary.clone(),
        }),
        outcome: Some("fmt, clippy, and tests passed in sealed execution".into()),
        covariates: Some(Covariates {
            env_fingerprint: report.sandbox_capability.content_digest.clone(),
            dependency_fingerprint: Some(report.before_tree_digest.clone()),
            config_flags,
            workload_id: report.before_tree_digest.clone(),
            selected_checks: vec!["fmt".into(), "clippy".into(), "test".into()],
            adjacent_edits: false,
            adjacent_edit_signatures: Vec::new(),
        }),
        promotion_state: Some(semantic_memory_forge::PromotionState::NotPromoted),
        primary_effect: None,
        all_effects: Vec::new(),
        hypothesis_edges: Vec::new(),
        receipts: vec![
            owner_receipt_ref(
                terminal_event_id,
                ReceiptKind::TrialLog,
                "aidens_canonical_event_log",
                &terminal_hash,
                &config.trace_id,
            ),
            owner_receipt_ref(
                &registered.lifecycle.receipt_id,
                ReceiptKind::PatchApplicationRecord,
                "procedural_memory_receipts",
                &registered.lifecycle.receipt_digest,
                &config.trace_id,
            ),
            owner_receipt_ref(
                &registered.effectful.receipt_id,
                ReceiptKind::CheckResult,
                "procedural_effectful_evaluations",
                &registered.effectful.receipt_digest,
                &config.trace_id,
            ),
        ],
        verification_trials: Vec::new(),
        refutation_artifacts: Vec::new(),
        sealed: false,
    };
    bundle
        .seal()
        .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    Ok(bundle)
}

fn persist_and_verify_terminal_bundle(
    config: &RealSandboxLearningConfig,
    bundle: &ExperimentEvidenceBundle,
) -> Result<(), RealSandboxLearningError> {
    let store = ForgeStore::open(&config.forge_store)
        .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    store
        .insert_canonical_evidence_bundle(bundle)
        .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    drop(store);
    let reopened = ForgeStore::open(&config.forge_store)
        .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    let loaded = reopened
        .get_canonical_evidence_bundle(&bundle.bundle_id)
        .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?
        .ok_or_else(|| {
            RealSandboxLearningError::Receipt(
                "canonical Forge evidence bundle missing after reopen".into(),
            )
        })?;
    let expected = serde_json::to_value(bundle)
        .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    let observed = serde_json::to_value(loaded)
        .map_err(|error| RealSandboxLearningError::Receipt(error.to_string()))?;
    if expected != observed {
        return Err(RealSandboxLearningError::Receipt(
            "canonical Forge evidence bundle changed across reopen".into(),
        ));
    }
    Ok(())
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
            forge_store: owner_root.path().join("forge.sqlite"),
            publication_namespace: "aidens-learning".into(),
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
        let durable = serde_json::to_string(&(&first_id, &first)).unwrap();
        for physical_path in [
            &config.cea_db,
            &config.receipt_root,
            &config.memory_store,
            &config.forge_store,
        ] {
            assert!(!durable.contains(&physical_path.to_string_lossy().to_string()));
        }
        assert!(first.execution.cea_store_identity.starts_with("blake3:"));
        assert!(first.execution.receipt_root_owner.starts_with("blake3:"));
        assert!(first.execution.memory_store_owner.starts_with("blake3:"));
        assert!(first.forge_store_owner.starts_with("blake3:"));

        config.image = format!("localhost/other@sha256:{}", "1".repeat(64));
        let changed_digest = fixture_digest(&config.fixture).unwrap();
        let (changed_id, _) = preflight_receipt(&config, changed_digest).unwrap();
        assert_ne!(first_id, changed_id);

        config.image = same.execution.image.clone();
        config.memory_store = config.memory_store.join("different-owner");
        config.forge_store = config.forge_store.with_file_name("different-forge.sqlite");
        config.publication_namespace = "aidens-learning-different".into();
        let changed_digest = fixture_digest(&config.fixture).unwrap();
        let (destination_changed_id, destination_changed) =
            preflight_receipt(&config, changed_digest).unwrap();
        assert_ne!(first_id, destination_changed_id);
        assert_ne!(
            same.execution.memory_store_owner,
            destination_changed.execution.memory_store_owner
        );
        assert_ne!(
            same.forge_store_owner,
            destination_changed.forge_store_owner
        );
        assert_ne!(
            same.publication_namespace,
            destination_changed.publication_namespace
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
    async fn live_controller_closes_terminal_v3_replay_and_lifecycle_drill() {
        let (_fixture, _owner_root, mut config) = config();
        config.image = std::env::var("AIDENS_LIVE_RUST_IMAGE").unwrap_or_else(|_| {
            "localhost/aidens-rust-checks@sha256:96f6610f945d10b523a303848610bd6fbef241762c59c0e44d47af9089cb6d6b".into()
        });
        let outcome = run_real_sandbox(config.clone()).await.unwrap();
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
        let effectful = outcome
            .terminal_effectful_receipt
            .as_ref()
            .expect("exact candidate effectful receipt must be registered");
        assert_eq!(effectful.artifact_id, receipt.artifact_id);
        assert!(semantic_memory::verify_procedure_effectful_evaluation_receipt_v1(effectful));
        let bundle = outcome
            .terminal_evidence_bundle
            .as_ref()
            .expect("typed Forge evidence bundle must be persisted");
        assert_eq!(bundle.candidate_id, receipt.artifact_id);
        assert_eq!(
            bundle.claim_strength,
            forge_engine::ClaimStrength::ExecutionVerifiedNoComparison
        );
        assert!(outcome.terminal_evidence_readback_verified);
        assert_eq!(
            outcome.terminal_publication,
            TerminalPublicationStateV1::Pending
        );
        assert_eq!(
            outcome.reason_codes,
            vec![
                String::from("terminal-v3-publication-pending"),
                String::from("procedure-lifecycle-owner-permit-required"),
                String::from("replay-retention-policy-owner-unavailable"),
            ]
        );

        let memory = MemoryStore::open(MemoryConfig {
            base_dir: config.memory_store.clone(),
            ..MemoryConfig::default()
        })
        .unwrap();
        let artifact_id = receipt.artifact_id.clone();
        let principal = config.permit_grant.granted_by.clone();
        let promote_permit = semantic_memory::ProcedureLifecyclePermitV1::elevated_for(
            principal.clone(),
            "operator:live-promote",
            "promote",
            artifact_id.clone(),
            "2999-01-01T00:00:00Z",
        );
        let promoted = memory
            .promote_procedure(
                promote_permit.clone(),
                &artifact_id,
                "live:promote:exact-source",
            )
            .await
            .unwrap();
        assert_eq!(
            promoted.disposition,
            ProcedureLifecycleDispositionV1::Promoted
        );
        assert!(memory
            .promote_procedure(promote_permit, &artifact_id, "live:promote:permit-reuse",)
            .await
            .is_err());

        let retrieval_request = |source_tree_digest: String| {
            let mut request = semantic_memory::ProcedureRetrievalRequestV1::new(
                ProcedureCapabilityV1::new("aidens-learning", "real-sandbox-evaluation"),
                ProcedureActionV1::new(
                    "apply_typed_patch_and_run_sealed_checks",
                    "Apply the source-bound typed patch and run sealed cargo fmt/clippy/test",
                ),
                serde_json::json!({
                    "language": "rust",
                    "source_tree_digest": source_tree_digest,
                }),
                semantic_memory::CallerPrincipalV1::new(principal.clone()).unwrap(),
                SubjectPrincipalV1::new(principal.clone()).unwrap(),
                vec![principal.clone()],
                NamespaceScopeV1::exact("aidens-learning"),
                semantic_memory::GovernedAccessPurposeV1::Action,
                semantic_memory::ProcedureAccessPathV1::DirectId,
            );
            request.artifact_id = Some(artifact_id.clone());
            request.action_permit = Some(semantic_memory::ProcedureActionPermitV1::elevated(
                principal.clone(),
                "operator:live-action",
                NamespaceScopeV1::exact("aidens-learning"),
            ));
            request
        };
        let retrieved = memory
            .retrieve_procedure(retrieval_request(outcome.report.before_tree_digest.clone()))
            .await
            .unwrap();
        let selected = retrieved
            .candidate
            .expect("promoted exact-source candidate must be retrievable");
        assert!(retrieved.decision.action_allowed);
        assert_eq!(selected.artifact_id, artifact_id);
        assert_eq!(
            selected.steps[0].arguments,
            serde_json::to_value(&config.patch).unwrap()
        );

        let replay_run_id = aidens_contracts::ArtifactId::new("run:live-replay-material");
        let replay_attempt_id = aidens_contracts::ArtifactId::new("attempt:live-replay-material");
        let replay_root = config.fixture.to_string_lossy().to_string();
        let mut replay_grant = PermitGrantV1::scoped(
            CanonicalToolSideEffectClass::Write,
            "aidens:patch-apply:1",
            replay_root.clone(),
            principal.clone(),
        );
        replay_grant.permit_id = aidens_contracts::ArtifactId::new("permit:live-replay-material");
        replay_grant.run_id = Some(replay_run_id.clone());
        replay_grant.attempt_id = Some(replay_attempt_id.clone());
        let mut replay_permit_use = PermitUseReportV1::allowed(
            &replay_grant,
            "aidens:patch-apply:1",
            replay_root,
            Some(replay_run_id),
            Some(replay_attempt_id),
        );
        replay_permit_use.receipt_id =
            aidens_contracts::ArtifactId::new("permit-use:live-replay-material");
        let mut replay_config = config.clone();
        replay_config.permit_grant = replay_grant;
        replay_config.permit_use = replay_permit_use;
        replay_config.run_id = "run:live-replay-material".into();
        replay_config.attempt_id = "attempt:live-replay-material".into();
        replay_config.trial_id = "trial:live-replay-material".into();
        replay_config.receipt_root = config.receipt_root.join("replay");
        replay_config.cea_db = config.cea_db.with_file_name("replay-cea.sqlite");
        replay_config.forge_store = config.forge_store.with_file_name("replay-forge.sqlite");
        replay_config.recorded_at = "2026-07-18T01:00:00Z".into();
        let replayed = replay_promoted_procedure(
            replay_config.clone(),
            &artifact_id,
            &outcome.report.before_tree_digest,
            semantic_memory::ProcedureActionPermitV1::elevated(
                principal.clone(),
                "operator:live-replay-action",
                NamespaceScopeV1::exact("aidens-learning"),
            ),
        )
        .await
        .unwrap();
        assert!(replayed.report.verified);
        assert!(replayed.action_allowed);
        assert!(replayed.retained_patch_exact);
        assert_eq!(replayed.artifact_id, artifact_id);
        assert_eq!(replayed.report.patch_digest, outcome.report.patch_digest);
        assert_eq!(
            replayed.report.before_tree_digest,
            outcome.report.before_tree_digest
        );
        assert_eq!(
            replayed.report.after_tree_digest,
            outcome.report.after_tree_digest
        );
        let replay_check_state = [
            replayed.report.checks.fmt_executed,
            replayed.report.checks.fmt_passed,
            replayed.report.checks.clippy_executed,
            replayed.report.checks.clippy_passed,
            replayed.report.checks.test_executed,
            replayed.report.checks.test_passed,
        ];
        let original_check_state = [
            outcome.report.checks.fmt_executed,
            outcome.report.checks.fmt_passed,
            outcome.report.checks.clippy_executed,
            outcome.report.checks.clippy_passed,
            outcome.report.checks.test_executed,
            outcome.report.checks.test_passed,
        ];
        assert_eq!(replay_check_state, original_check_state);
        assert!(replay_check_state.into_iter().all(|state| state));

        let mut altered_replay = replay_config;
        altered_replay.patch.summary = "caller attempted different retained patch".into();
        assert!(replay_promoted_procedure(
            altered_replay,
            &artifact_id,
            &outcome.report.before_tree_digest,
            semantic_memory::ProcedureActionPermitV1::elevated(
                principal.clone(),
                "operator:live-replay-mismatch",
                NamespaceScopeV1::exact("aidens-learning"),
            ),
        )
        .await
        .is_err());

        let publication_request = crate::learning_publication::TerminalPublicationRequestV1 {
            forge_store: config.forge_store.clone(),
            memory_store: config.memory_store.clone(),
            bundle_id: bundle.bundle_id.clone(),
            publication_namespace: config.publication_namespace.clone(),
        };
        let publication =
            crate::learning_publication::publish_terminal_evidence(publication_request.clone())
                .await
                .unwrap();
        assert_eq!(
            publication.disposition,
            crate::learning_publication::TerminalPublicationDispositionV1::Published
        );
        let mut mismatched_owner_outcome = outcome.clone();
        mismatched_owner_outcome.preflight_receipt.execution.run_id = "forged-run-id".into();
        assert!(matches!(
            crate::learning_terminal::close_real_sandbox_terminal(
                &config,
                &mismatched_owner_outcome,
                &publication,
                &promoted,
                &replayed,
                crate::learning_terminal::TerminalOwnerVerificationSealV1::verified(),
            ),
            Err(crate::learning_terminal::TerminalBundlePublicationError::Integration(_))
        ));
        let bundle_store = aidens_receipts::RunBundleStore::open(
            aidens_receipts::RunBundleStoreConfig::for_receipt_root(&config.receipt_root),
        )
        .unwrap();
        assert!(matches!(
            bundle_store.inspect(&config.run_id),
            Err(aidens_receipts::RunBundleStoreError::NotFound(_))
        ));
        let terminal = crate::learning_terminal::close_real_sandbox_terminal(
            &config,
            &outcome,
            &publication,
            &promoted,
            &replayed,
            crate::learning_terminal::TerminalOwnerVerificationSealV1::verified(),
        )
        .unwrap();
        assert_eq!(
            terminal.terminal_projection.state,
            aidens_contracts::CodingLearningTerminalStateV1::SucceededVerified
        );
        assert!(terminal.terminal_projection_readback_verified);
        assert!(terminal.terminal_bundle.digest_verified);
        assert!(terminal.terminal_bundle.index_verified);
        assert!(terminal.terminal_bundle.recovery_verified);
        assert_eq!(terminal.terminal_bundle.bundle.child_receipts.len(), 8);
        assert_eq!(
            terminal.terminal_bundle.bundle.event_log.event_log_path,
            "canonical-receipts.ndjson"
        );
        assert!(terminal
            .terminal_projection
            .canonical_backpointers
            .iter()
            .any(|pointer| {
                pointer.role == "published-run-bundle"
                    && pointer.external_id.as_deref()
                        == Some(terminal.terminal_bundle.bundle.bundle_id.as_str())
            }));

        let recovered_publication =
            crate::learning_publication::publish_terminal_evidence(publication_request)
                .await
                .unwrap();
        assert_eq!(
            recovered_publication.disposition,
            crate::learning_publication::TerminalPublicationDispositionV1::RecoveredIdempotently
        );
        assert_eq!(
            aidens_contracts::StackContentDigest::compute_json(&publication.export_receipt)
                .unwrap(),
            aidens_contracts::StackContentDigest::compute_json(
                &recovered_publication.export_receipt,
            )
            .unwrap(),
            "persisted Forge export receipt changed on idempotent export"
        );
        assert_eq!(
            aidens_contracts::StackContentDigest::compute_json(&publication.import_readback)
                .unwrap(),
            aidens_contracts::StackContentDigest::compute_json(
                &recovered_publication.import_readback,
            )
            .unwrap(),
            "semantic-memory import readback changed on idempotent import"
        );
        let recovered_terminal = crate::learning_terminal::close_real_sandbox_terminal(
            &config,
            &outcome,
            &recovered_publication,
            &promoted,
            &replayed,
            crate::learning_terminal::TerminalOwnerVerificationSealV1::verified(),
        )
        .unwrap();
        assert_eq!(
            recovered_terminal.terminal_bundle.disposition,
            crate::learning_terminal::TerminalBundlePublicationDispositionV1::RecoveredIdempotently
        );
        assert_eq!(
            recovered_terminal.terminal_bundle.bundle.bundle_id,
            terminal.terminal_bundle.bundle.bundle_id
        );
        assert_eq!(
            recovered_terminal.terminal_projection_receipt_id,
            terminal.terminal_projection_receipt_id
        );
        let wrong_source = memory
            .retrieve_procedure(retrieval_request("blake3:wrong-source-tree".into()))
            .await
            .unwrap();
        assert!(wrong_source.candidate.is_none());

        let rolled_back = memory
            .rollback_procedure(
                semantic_memory::ProcedureLifecyclePermitV1::elevated_for(
                    principal.clone(),
                    "operator:live-rollback",
                    "rollback",
                    artifact_id.clone(),
                    "2999-01-01T00:00:00Z",
                ),
                &artifact_id,
                "live:rollback:exact-source",
                "bounded rollback drill",
            )
            .await
            .unwrap();
        assert_eq!(
            rolled_back.disposition,
            ProcedureLifecycleDispositionV1::RolledBack
        );
        assert!(memory
            .retrieve_procedure(retrieval_request(outcome.report.before_tree_digest.clone()))
            .await
            .unwrap()
            .candidate
            .is_none());

        let revoked = memory
            .revoke_procedure(
                semantic_memory::ProcedureLifecyclePermitV1::elevated_for(
                    principal,
                    "operator:live-revoke",
                    "revoke",
                    artifact_id.clone(),
                    "2999-01-01T00:00:00Z",
                ),
                &artifact_id,
                "live:revoke:exact-source",
                "bounded revoke drill",
            )
            .await
            .unwrap();
        assert_eq!(
            revoked.disposition,
            ProcedureLifecycleDispositionV1::Revoked
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
