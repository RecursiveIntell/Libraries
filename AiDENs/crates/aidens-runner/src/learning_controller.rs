//! Production composition for one bounded, real-sandbox learning run.
//! The controller owns no evidence: receipts and CEA remain canonical owners.

use crate::learning_effectful::{
    evaluate_effectful_persisted, EffectfulEvaluationReportV1, EffectfulEvaluationRequest,
};
use aidens_contracts::{LearningPreflightReceiptV1, PermitGrantV1, PermitUseReportV1};
use aidens_receipts::{CanonicalEventLog, CanonicalEventLogConfig};
use cea_sqlite::SqliteCeaStore;
use check_runner::{BackendConfig, ContainerBackend};
use serde::Serialize;
use std::path::PathBuf;
use typed_patch::{validate_patch, PatchPolicy, StructuredPatch};

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

pub async fn run_real_sandbox(
    config: RealSandboxLearningConfig,
) -> Result<EffectfulEvaluationReportV1, RealSandboxLearningError> {
    validate(&config)?;
    let patch_validation = validate_patch(&config.patch, &config.patch_policy);
    if !patch_validation.ok {
        return Err(RealSandboxLearningError::Invalid(
            "patch rejected before persistence".into(),
        ));
    }
    let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(
        config.receipt_root.clone(),
    ))
    .map_err(|e| RealSandboxLearningError::Receipt(e.to_string()))?;
    let material_id = digest_bytes(
        &serde_json::to_vec(&(
            fixture_digest(&config.fixture)?,
            &config.patch,
            format!("{:?}", &config.patch_policy),
            &config.permit_grant,
            &config.permit_use,
            &config.image,
            &config.run_id,
            &config.attempt_id,
            &config.trial_id,
            &config.trace_id,
        ))
        .map_err(|e| RealSandboxLearningError::Invalid(e.to_string()))?,
    );
    let preflight_id = format!("aidens-learning:preflight:{material_id}");
    let receipt = LearningPreflightReceiptV1 {
        schema: "AiDENsLearningPreflightReceiptV1".into(),
        material_id,
        fixture_tree_digest: fixture_digest(&config.fixture)?,
        patch_digest: digest_value(&config.patch)?,
        patch_policy_digest: digest_bytes(format!("{:?}", &config.patch_policy).as_bytes()),
        permit_grant_id: config.permit_grant.permit_id.as_str().into(),
        permit_use_id: config.permit_use.receipt_id.as_str().into(),
        permit_scope_digest: digest_value(&config.permit_grant)?,
        image: config.image.clone(),
        backend_limits_digest: digest_bytes(b"sealed_local:podman:900:2g:2"),
        run_id: config.run_id.clone(),
        attempt_id: config.attempt_id.clone(),
        trial_id: config.trial_id.clone(),
        trace_id: config.trace_id.clone(),
        cea_store_identity: config.cea_db.to_string_lossy().into(),
        receipt_root_owner: config.receipt_root.to_string_lossy().into(),
    };
    let preflight = log
        .append_json(
            "aidens-runner",
            "learning-preflight-v1",
            preflight_id,
            serde_json::to_value(&receipt)
                .map_err(|e| RealSandboxLearningError::Receipt(e.to_string()))?,
        )
        .map_err(|e| RealSandboxLearningError::Receipt(e.to_string()))?;
    if !preflight.verify_record_digest() {
        return Err(RealSandboxLearningError::Receipt(
            "preflight record verification failed".into(),
        ));
    }

    let backend = ContainerBackend::new(&BackendConfig {
        mode: "sealed_local".into(),
        execution_backend_preference: "container".into(),
        container_runtime_preference: "podman".into(),
        sealed_allow_host_backend: false,
        rust_image: config.image.clone(),
        command_timeout_secs: 900,
        memory_limit: "2g".into(),
        cpu_limit: "2".into(),
    })
    .map_err(|e| RealSandboxLearningError::Receipt(e.to_string()))?;
    let cea = SqliteCeaStore::open(&config.cea_db)
        .map_err(|e| RealSandboxLearningError::Receipt(e.to_string()))?;
    let request = EffectfulEvaluationRequest {
        fixture: config.fixture,
        patch: config.patch,
        patch_policy: config.patch_policy,
        permit_grant: config.permit_grant,
        permit_use: config.permit_use,
        preflight_persisted: preflight.verify_record_digest(),
        run_id: config.run_id,
        attempt_id: config.attempt_id,
        trial_id: config.trial_id.clone(),
        trace_id: config.trace_id,
        recorded_at: config.recorded_at,
    };
    let report = evaluate_effectful_persisted(
        request,
        &backend,
        || backend.capability_truth_receipt(),
        &cea,
    )
    .await?;
    log.append_json(
        "aidens-runner",
        "effectful-evaluation-report-v1",
        format!("aidens-learning:report:{}", config.trial_id),
        serde_json::to_value(&report)
            .map_err(|e| RealSandboxLearningError::Receipt(e.to_string()))?,
    )
    .map_err(|e| RealSandboxLearningError::Receipt(e.to_string()))?;
    Ok(report)
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
    ] {
        if value.trim().is_empty() {
            return Err(RealSandboxLearningError::Invalid(format!(
                "{name} is required"
            )));
        }
    }
    Ok(())
}

fn digest_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}
fn digest_value<T: Serialize>(value: &T) -> Result<String, RealSandboxLearningError> {
    serde_json::to_vec(value)
        .map(|v| digest_bytes(&v))
        .map_err(|e| RealSandboxLearningError::Invalid(e.to_string()))
}
fn fixture_digest(path: &std::path::Path) -> Result<String, RealSandboxLearningError> {
    let mut entries = Vec::new();
    for entry in walkdir::WalkDir::new(path).follow_links(false) {
        let entry = entry.map_err(|e| RealSandboxLearningError::Invalid(e.to_string()))?;
        if entry.file_type().is_file() {
            entries.push((
                entry
                    .path()
                    .strip_prefix(path)
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                std::fs::read(entry.path())
                    .map_err(|e| RealSandboxLearningError::Invalid(e.to_string()))?,
            ));
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    digest_value(&entries)
}
