//! Production composition for one bounded, real-sandbox learning run.
//! The controller owns no evidence: receipts and CEA remain canonical owners.

use crate::learning_effectful::{
    evaluate_effectful_persisted, EffectfulEvaluationReportV1, EffectfulEvaluationRequest,
};
use aidens_contracts::{PermitGrantV1, PermitUseReportV1};
use aidens_receipts::{CanonicalEventLog, CanonicalEventLogConfig};
use cea_sqlite::SqliteCeaStore;
use check_runner::{BackendConfig, ContainerBackend};
use std::path::PathBuf;
use typed_patch::{PatchPolicy, StructuredPatch};

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
    let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(
        config.receipt_root.clone(),
    ))
    .map_err(|e| RealSandboxLearningError::Receipt(e.to_string()))?;
    let preflight_id = format!("aidens-learning:preflight:{}", config.trial_id);
    let preflight = log
        .append_json(
            "aidens-runner",
            "learning-preflight-v1",
            preflight_id,
            serde_json::json!({
                "schema": "AiDENsLearningPreflightV1", "run_id": config.run_id,
                "attempt_id": config.attempt_id, "trial_id": config.trial_id,
                "fixture": config.fixture, "patch_id": config.patch.patch_id.to_string(),
                "image": config.image, "receipt_root": config.receipt_root,
            }),
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
