//! Canonical procedure-candidate boundary. This module is an adapter only: semantic-memory owns
//! artifact validation, lifecycle state, and receipts.
use crate::learning_effectful::EffectfulEvaluationReportV1;
use semantic_memory::{
    validate_procedure_artifact_v1, MemoryError, MemoryStore, ProceduralMemoryArtifactV1,
    ProcedureEffectfulEvaluationReceiptV1, ProcedureLifecycleReceiptV1,
};
use verification_adjudication::VerificationDisposition;

#[derive(Debug, Clone)]
pub struct ProcedureCandidateBlueprintV1 {
    pub artifact: ProceduralMemoryArtifactV1,
}

#[derive(Debug, thiserror::Error)]
pub enum ProcedureCandidateError {
    #[error("effectful evaluation is missing")]
    MissingEvaluation,
    #[error("effectful evaluation is not publication-complete: {0}")]
    Incomplete(String),
    #[error("canonical procedure artifact rejected: {0}")]
    InvalidArtifact(String),
    #[error("canonical owner rejected candidate: {0}")]
    Owner(#[from] MemoryError),
}

fn publication_complete(report: &EffectfulEvaluationReportV1) -> Result<(), String> {
    let checks = &report.checks;
    let complete_checks = checks.fmt_executed
        && checks.fmt_passed
        && checks.clippy_executed
        && checks.clippy_passed
        && checks.test_executed
        && checks.test_passed
        && !checks.fmt_output_digest.is_empty()
        && !checks.clippy_output_digest.is_empty()
        && !checks.test_output_digest.is_empty();
    let lineage = !report.permit_use_receipt_id.trim().is_empty()
        && !report.cea_run_hash.trim().is_empty()
        && report.cea_persisted
        && report
            .cea_backpointer
            .external_id
            .as_deref()
            .is_some_and(|id| !id.trim().is_empty());
    if report.execution_mode != "real_sandbox"
        || !report.verified
        || report.verification.disposition != VerificationDisposition::EligibleForPromotion
        || !report.sandbox_capability.verify()
        || !complete_checks
        || !lineage
        || report.before_tree_digest.is_empty()
        || report.after_tree_digest.is_empty()
        || !report.rollback_verified
        || report.rollback_tree_digest != report.before_tree_digest
    {
        return Err("real evidence, independent verification, CEA, lineage, complete checks, and rollback are required".into());
    }
    Ok(())
}

pub fn extract_procedure_candidate(
    report: Option<&EffectfulEvaluationReportV1>,
    blueprint: Option<&ProcedureCandidateBlueprintV1>,
) -> Result<ProceduralMemoryArtifactV1, ProcedureCandidateError> {
    let report = report.ok_or(ProcedureCandidateError::MissingEvaluation)?;
    publication_complete(report).map_err(ProcedureCandidateError::Incomplete)?;
    let blueprint = blueprint
        .ok_or_else(|| ProcedureCandidateError::Incomplete("typed blueprint is missing".into()))?;
    let validation = validate_procedure_artifact_v1(&blueprint.artifact);
    if !validation.valid {
        return Err(ProcedureCandidateError::InvalidArtifact(
            validation.reason_codes.join(","),
        ));
    }
    Ok(blueprint.artifact.clone())
}

pub async fn compile_and_register_effectful(
    store: &MemoryStore,
    report: Option<&EffectfulEvaluationReportV1>,
    blueprint: &ProcedureCandidateBlueprintV1,
    compile_key: impl Into<String>,
    effectful_key: impl Into<String>,
) -> Result<
    (
        ProcedureLifecycleReceiptV1,
        ProcedureEffectfulEvaluationReceiptV1,
    ),
    ProcedureCandidateError,
> {
    let artifact = extract_procedure_candidate(report, Some(blueprint))?;
    let report = report.ok_or(ProcedureCandidateError::MissingEvaluation)?;
    let lifecycle = store
        .compile_procedure(artifact.clone(), compile_key)
        .await?;
    let effectful = ProcedureEffectfulEvaluationReceiptV1::verified(
        &artifact.artifact_id,
        &artifact.artifact_digest,
        report.sandbox_capability.content_digest.clone(),
        vec![
            report.checks.fmt_output_digest.clone(),
            report.checks.clippy_output_digest.clone(),
            report.checks.test_output_digest.clone(),
        ],
        report
            .verification
            .promotion_decision
            .decision_id
            .to_string(),
        report.cea_run_hash.clone(),
        report.before_tree_digest.clone(),
        report.after_tree_digest.clone(),
    )?;
    let registered = store
        .record_effectful_procedure_evaluation(effectful, effectful_key)
        .await?;
    Ok((lifecycle, registered))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_effectful_report_is_rejected_before_canonical_promotion() {
        assert!(matches!(
            extract_procedure_candidate(None, None),
            Err(ProcedureCandidateError::MissingEvaluation)
        ));
    }
}
