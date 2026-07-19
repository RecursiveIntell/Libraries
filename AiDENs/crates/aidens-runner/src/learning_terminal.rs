//! Strict publication of a child-closed coding-learning run bundle.
//!
//! This module composes canonical contract and receipt-store owners. It does
//! not mint task, verification, lifecycle, export, replay, or terminal truth.

use crate::learning_controller::{
    patch_policy_material, PromotedProcedureReplayOutcomeV1, RealSandboxLearningConfig,
    RealSandboxLearningOutcomeV1,
};
use crate::learning_publication::TerminalPublicationOutcomeV1;
use aidens_contracts::{
    canonical_stack::{ForgeDispatchOutcomeV1, ForgeExecutionContextV1},
    generated_artifact_id_from_material, project_terminal_state, AiDENsRunBudgetDeadlineV1,
    AiDENsRunBundleV3, AiDENsRunChildReceiptV1, AiDENsRunEventLogDigestV1, AiDENsRunFailureClassV1,
    AiDENsRunFailureTaxonomyV1, AiDENsRunReplayNormalizationV1, AiDENsRunSupportTierEvidenceV1,
    ArtifactId, CanonicalBackpointerV1, CodingLearningEvidenceV1,
    CodingLearningTerminalProjectionV1, CodingLearningTerminalStateV1, DisplayDigestV1,
    StackAttemptId, StackContentDigest, StackTraceCtx, StackTrialId,
};
use aidens_receipts::{
    CanonicalEventLog, CanonicalEventLogConfig, CanonicalEventLogError, RunBundleRecoveryState,
    RunBundleStore, RunBundleStoreConfig, RunBundleStoreError, RunBundleStoreRecord,
};
use semantic_memory::{ProcedureLifecycleDispositionV1, ProcedureLifecycleReceiptV1};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalBundlePublicationRequestV1 {
    pub schema: String,
    pub store_root: PathBuf,
    pub identity_material: String,
    pub run_id: String,
    pub profile: String,
    pub canonical_execution_context: ForgeExecutionContextV1,
    pub event_log: AiDENsRunEventLogDigestV1,
    pub budget: AiDENsRunBudgetDeadlineV1,
    pub support: AiDENsRunSupportTierEvidenceV1,
    pub support_labels: Vec<String>,
    pub replay: AiDENsRunReplayNormalizationV1,
    pub failure: AiDENsRunFailureTaxonomyV1,
    pub attempt_family_id: ArtifactId,
    pub attempt_id: StackAttemptId,
    pub trial_id: StackTrialId,
    pub agent_spec_digest: DisplayDigestV1,
    pub owner_backpointers: Vec<CanonicalBackpointerV1>,
    pub child_receipts: Vec<AiDENsRunChildReceiptV1>,
}

impl TerminalBundlePublicationRequestV1 {
    pub const SCHEMA: &'static str = "AiDENsTerminalBundlePublicationRequestV1";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TerminalBundlePublicationDispositionV1 {
    Published,
    RecoveredIdempotently,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalBundlePublicationOutcomeV1 {
    pub schema: String,
    pub disposition: TerminalBundlePublicationDispositionV1,
    pub bundle: AiDENsRunBundleV3,
    pub store_record: RunBundleStoreRecord,
    pub digest_verified: bool,
    pub index_verified: bool,
    pub recovery_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealSandboxTerminalClosureOutcomeV1 {
    pub schema: String,
    pub terminal_bundle: TerminalBundlePublicationOutcomeV1,
    pub terminal_projection: CodingLearningTerminalProjectionV1,
    pub terminal_projection_receipt_id: String,
    pub terminal_projection_readback_verified: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum TerminalBundlePublicationError {
    #[error("terminal bundle publication request schema is invalid")]
    InvalidSchema,
    #[error("terminal bundle contract rejected owner material: {0:?}")]
    Contract(Vec<String>),
    #[error("terminal bundle store rejected publication: {0}")]
    Store(#[from] RunBundleStoreError),
    #[error("existing terminal artifact conflicts with requested canonical material: {0}")]
    ExistingConflict(String),
    #[error("terminal bundle readback failed: {0}")]
    Readback(String),
    #[error("real-sandbox terminal closure rejected owner evidence: {0}")]
    Integration(String),
}

pub fn publish_terminal_bundle(
    request: TerminalBundlePublicationRequestV1,
) -> Result<TerminalBundlePublicationOutcomeV1, TerminalBundlePublicationError> {
    if request.schema != TerminalBundlePublicationRequestV1::SCHEMA {
        return Err(TerminalBundlePublicationError::InvalidSchema);
    }
    let requested_bundle = AiDENsRunBundleV3::new_material_bound(
        &request.identity_material,
        request.run_id.clone(),
        request.profile,
        request.canonical_execution_context,
        request.event_log,
        request.budget,
        request.support,
        request.support_labels,
        request.replay,
        request.failure,
        request.attempt_family_id,
        request.attempt_id,
        request.trial_id,
        request.agent_spec_digest,
        request.owner_backpointers,
    )
    .and_then(|bundle| bundle.with_coding_learning_child_closure(request.child_receipts))
    .map_err(TerminalBundlePublicationError::Contract)?;

    let config = RunBundleStoreConfig::for_receipt_root(request.store_root);
    let store = RunBundleStore::open(config.clone())?;
    let (disposition, bundle) = match store.inspect(&request.run_id) {
        Ok(inspection) => {
            let existing: AiDENsRunBundleV3 = serde_json::from_value(inspection.bundle)
                .map_err(|error| TerminalBundlePublicationError::Readback(error.to_string()))?;
            if existing.bundle_id != requested_bundle.bundle_id
                || existing.validate_child_closure().is_err()
                || !inspection.digest_verified
            {
                return Err(TerminalBundlePublicationError::ExistingConflict(format!(
                    "run-bundle-identity-or-child-closure:existing={}:requested={}",
                    existing.bundle_id, requested_bundle.bundle_id
                )));
            }
            (
                TerminalBundlePublicationDispositionV1::RecoveredIdempotently,
                existing,
            )
        }
        Err(RunBundleStoreError::NotFound(_)) => {
            store.write_bundle(&requested_bundle)?;
            (
                TerminalBundlePublicationDispositionV1::Published,
                requested_bundle,
            )
        }
        Err(error) => return Err(error.into()),
    };

    // Reopen to ensure no in-memory state is carrying the verification result.
    let reopened = RunBundleStore::open(config)?;
    let inspection = reopened.inspect(&request.run_id)?;
    let readback: AiDENsRunBundleV3 = serde_json::from_value(inspection.bundle.clone())
        .map_err(|error| TerminalBundlePublicationError::Readback(error.to_string()))?;
    if readback != bundle || !inspection.digest_verified {
        return Err(TerminalBundlePublicationError::Readback(
            "typed bundle or content digest differs after reopen".into(),
        ));
    }
    let recovery_verified = reopened.reconcile()?.entries.into_iter().any(|entry| {
        entry.run_id == request.run_id && entry.state == RunBundleRecoveryState::Published
    });
    if !recovery_verified {
        return Err(TerminalBundlePublicationError::Readback(
            "reconciliation did not report the bundle as published".into(),
        ));
    }
    Ok(TerminalBundlePublicationOutcomeV1 {
        schema: "AiDENsTerminalBundlePublicationOutcomeV1".into(),
        disposition,
        bundle,
        store_record: inspection.record,
        digest_verified: inspection.digest_verified,
        index_verified: true,
        recovery_verified,
    })
}

pub fn close_real_sandbox_terminal(
    config: &RealSandboxLearningConfig,
    initial: &RealSandboxLearningOutcomeV1,
    publication: &TerminalPublicationOutcomeV1,
    promotion: &ProcedureLifecycleReceiptV1,
    replay: &PromotedProcedureReplayOutcomeV1,
) -> Result<RealSandboxTerminalClosureOutcomeV1, TerminalBundlePublicationError> {
    let tested = initial.terminal_lifecycle_receipt.as_ref().ok_or_else(|| {
        TerminalBundlePublicationError::Integration("tested lifecycle receipt missing".into())
    })?;
    let effectful = initial.terminal_effectful_receipt.as_ref().ok_or_else(|| {
        TerminalBundlePublicationError::Integration(
            "effectful lifecycle prerequisite receipt missing".into(),
        )
    })?;
    let evidence_bundle = initial.terminal_evidence_bundle.as_ref().ok_or_else(|| {
        TerminalBundlePublicationError::Integration("Forge evidence bundle missing".into())
    })?;
    if !initial.report.verified
        || !initial.terminal_event_log_verified
        || !initial.terminal_evidence_readback_verified
        || !publication.readback_verified
        || !replay.report.verified
        || !replay.terminal_event_log_verified
        || !replay.action_allowed
        || !replay.retained_patch_exact
    {
        return Err(TerminalBundlePublicationError::Integration(
            "one or more required owner verification gates are false".into(),
        ));
    }
    if tested.disposition != ProcedureLifecycleDispositionV1::Tested
        || promotion.disposition != ProcedureLifecycleDispositionV1::Promoted
        || tested.artifact_id != promotion.artifact_id
        || effectful.artifact_id != promotion.artifact_id
        || replay.artifact_id != promotion.artifact_id
        || evidence_bundle.candidate_id != promotion.artifact_id
    {
        return Err(TerminalBundlePublicationError::Integration(
            "procedure lifecycle, evidence, and replay identities do not close".into(),
        ));
    }
    if publication.bundle_id != evidence_bundle.bundle_id
        || publication.envelope_id != publication.export_envelope.envelope_id.as_str()
        || publication.envelope_id != publication.export_receipt.export_key
        || publication.bundle_id != publication.export_receipt.bundle_id
        || publication.content_digest != publication.export_envelope.content_digest.hex()
        || publication.import_readback.source_envelope_id != publication.envelope_id
        || publication.import_readback.evidence_bundle_id.as_deref()
            != Some(evidence_bundle.bundle_id.as_str())
    {
        return Err(TerminalBundlePublicationError::Integration(
            "Forge export and semantic-memory import identities do not close".into(),
        ));
    }

    let cea_id = initial
        .report
        .cea_backpointer
        .external_id
        .clone()
        .or_else(|| {
            initial
                .report
                .cea_backpointer
                .artifact_id
                .as_ref()
                .map(ToString::to_string)
        })
        .ok_or_else(|| {
            TerminalBundlePublicationError::Integration(
                "CEA owner backpointer has no durable identity".into(),
            )
        })?;
    let policy_digest =
        StackContentDigest::compute_json(&patch_policy_material(&config.patch_policy))
            .map_err(|error| TerminalBundlePublicationError::Integration(error.to_string()))?
            .hex()
            .to_string();
    let owner_backpointers = vec![
        owner_pointer(
            "aidens-runner",
            "RealSandboxLearningConfig",
            "task",
            &config.run_id,
        ),
        owner_pointer(
            "typed-patch",
            "SourceTreeDigest",
            "source-tree",
            &initial.report.before_tree_digest,
        ),
        owner_pointer("typed-patch", "PatchPolicy", "policy", &policy_digest),
        owner_pointer(
            "check-runner",
            "SandboxCapabilityTruthReceiptV1",
            "sandbox",
            &initial.report.sandbox_capability.content_digest,
        ),
        owner_pointer(
            "typed-patch",
            "StructuredPatch",
            "patch",
            &initial.report.patch_digest,
        ),
        owner_pointer(
            "check-runner",
            "CheckEvidenceV1",
            "checks",
            &initial.report.checks.output_digest,
        ),
        owner_pointer(
            "verification-adjudication",
            "PromotionDecisionV1",
            "verification",
            initial
                .report
                .verification
                .promotion_decision
                .decision_id
                .as_ref(),
        ),
        owner_pointer("cea-store", "CeaRun", "cea", &cea_id),
        owner_pointer(
            "semantic-memory",
            "ProcedureLifecycleReceiptV1",
            "procedure-lifecycle",
            &promotion.receipt_id,
        ),
        owner_pointer(
            "semantic-memory-forge",
            "ExportEnvelopeV3",
            "forge-export-envelope-v3",
            &publication.envelope_id,
        ),
        owner_pointer(
            "aidens-runner",
            "PromotedProcedureReplayOutcomeV1",
            "replay",
            &replay.terminal_event_receipt_id,
        ),
    ];
    let child_receipts = vec![
        closed_child("owner:effectful-evaluation", &initial.report)?,
        closed_child("owner:procedure-lifecycle-tested", tested)?,
        closed_child("owner:procedure-effectful-prerequisite", effectful)?,
        closed_child("owner:forge-evidence-bundle", evidence_bundle)?,
        closed_child("owner:forge-export-receipt", &publication.export_receipt)?,
        closed_child(
            "owner:semantic-memory-projection-import",
            &publication.import_readback,
        )?,
        closed_child("owner:procedure-lifecycle-promoted", promotion)?,
        closed_child("owner:promoted-procedure-replay", replay)?,
    ];

    let event_log_path = config.receipt_root.join("canonical-receipts.ndjson");
    let event_log_text = std::fs::read_to_string(&event_log_path)
        .map_err(|error| TerminalBundlePublicationError::Integration(error.to_string()))?;
    let mut prepublication_events = Vec::new();
    for line in event_log_text
        .lines()
        .filter(|line| !line.trim().is_empty())
    {
        let value: serde_json::Value = serde_json::from_str(line)
            .map_err(|error| TerminalBundlePublicationError::Integration(error.to_string()))?;
        if value.get("schema_name").and_then(serde_json::Value::as_str)
            != Some("coding-learning-terminal-projection-v1")
        {
            prepublication_events.push(value);
        }
    }
    let event_log_snapshot = serde_json::to_string(&prepublication_events)
        .map_err(|error| TerminalBundlePublicationError::Integration(error.to_string()))?;
    let event_count = prepublication_events.len();
    let replay_material = serde_json::json!({
        "artifact_id": replay.artifact_id,
        "artifact_digest": replay.artifact_digest,
        "patch_digest": replay.report.patch_digest,
        "before_tree_digest": replay.report.before_tree_digest,
        "after_tree_digest": replay.report.after_tree_digest,
        "rollback_tree_digest": replay.report.rollback_tree_digest,
        "checks": {
            "fmt_executed": replay.report.checks.fmt_executed,
            "fmt_passed": replay.report.checks.fmt_passed,
            "clippy_executed": replay.report.checks.clippy_executed,
            "clippy_passed": replay.report.checks.clippy_passed,
            "test_executed": replay.report.checks.test_executed,
            "test_passed": replay.report.checks.test_passed,
        },
    });
    let replay_digest = StackContentDigest::compute_json(&replay_material)
        .map_err(|error| TerminalBundlePublicationError::Integration(error.to_string()))?;
    let event_log = AiDENsRunEventLogDigestV1 {
        event_log_path: "canonical-receipts.ndjson".into(),
        digest: StackContentDigest::compute_str(&event_log_snapshot),
        replay_normalized_digest: replay_digest.clone(),
        canonical_record_count: event_count,
        event_count,
        reason_codes: vec![
            "event-log-digested-by-stack-ids".into(),
            "post-publication-terminal-projection-excluded-from-prepublication-snapshot".into(),
            "replay-normalization-uses-stable-owner-facts".into(),
        ],
    };
    let attempt_id = StackAttemptId::new(config.attempt_id.clone());
    let trial_id = StackTrialId::new(config.trial_id.clone());
    let mut execution_context =
        ForgeExecutionContextV1::new(StackTraceCtx::from_trace_id(config.trace_id.clone()));
    execution_context.attempt_id = Some(attempt_id.clone());
    execution_context.trial_id = Some(trial_id.clone());
    execution_context.replay_link =
        Some("aidens-runner::learning_controller::replay_promoted_procedure".into());
    execution_context.workload_class = Some("bounded-exact-source-learning".into());
    execution_context.deadline = Some("900000ms".into());
    execution_context.cost_budget_units = Some(3);
    execution_context.degradation_markers = vec!["elapsed-time-unavailable".into()];
    execution_context.dispatch_outcome = ForgeDispatchOutcomeV1::Succeeded;
    execution_context.environment_fingerprint =
        Some(initial.report.sandbox_capability.content_digest.clone());

    let identity_material = serde_json::to_string(&serde_json::json!({
        "run_id": config.run_id,
        "terminal_event_receipt_id": initial.terminal_event_receipt_id,
        "candidate_id": promotion.artifact_id,
        "evidence_bundle_id": evidence_bundle.bundle_id,
        "export_envelope_id": publication.envelope_id,
        "promotion_receipt_id": promotion.receipt_id,
        "replay_receipt_id": replay.terminal_event_receipt_id,
    }))
    .map_err(|error| TerminalBundlePublicationError::Integration(error.to_string()))?;
    let terminal_bundle = publish_terminal_bundle(TerminalBundlePublicationRequestV1 {
        schema: TerminalBundlePublicationRequestV1::SCHEMA.into(),
        store_root: config.receipt_root.clone(),
        identity_material,
        run_id: config.run_id.clone(),
        profile: "coding".into(),
        canonical_execution_context: execution_context,
        event_log,
        budget: AiDENsRunBudgetDeadlineV1 {
            max_steps: 1,
            max_tool_calls: 3,
            max_retries: 0,
            max_turn_millis: 900_000,
            elapsed_ms: 0,
            deadline: Some("900000ms".into()),
            cost_budget_units: Some(3),
            degradation_markers: vec!["elapsed-time-unavailable".into()],
        },
        support: AiDENsRunSupportTierEvidenceV1 {
            support_tier: "supported-local".into(),
            supported: vec![
                "exact-source-effectful-evaluation".into(),
                "governed-procedure-replay".into(),
                "canonical-v3-publication".into(),
            ],
            partial: vec!["elapsed-time-accounting".into()],
            deferred: vec!["cross-task-generalization".into()],
            reason_codes: vec!["bounded-exact-source-claim-only".into()],
        },
        support_labels: vec!["supported-local".into(), "bounded-proof".into()],
        replay: AiDENsRunReplayNormalizationV1 {
            replay_command: "aidens-runner::learning_controller::replay_promoted_procedure".into(),
            fixture_path: None,
            normalized_fields: vec![
                "run_id".into(),
                "attempt_id".into(),
                "trial_id".into(),
                "recorded_at".into(),
                "receipt_id".into(),
            ],
            deterministic_compare: true,
            normalized_digest: replay_digest,
            reason_codes: vec!["stable-owner-fact-replay-comparison".into()],
        },
        failure: AiDENsRunFailureTaxonomyV1 {
            class: AiDENsRunFailureClassV1::None,
            reason_codes: vec!["bounded-exact-source-loop-verified".into()],
            degraded: false,
            blocked: false,
        },
        attempt_family_id: generated_artifact_id_from_material(
            "aidens-learning-attempt-family",
            &promotion.artifact_id,
        ),
        attempt_id,
        trial_id,
        agent_spec_digest: DisplayDigestV1::for_json_value(&serde_json::json!({
            "schema": "AiDENsExactSourceLearningAgentSpecV1",
            "execution_mode": "real_sandbox",
            "network": "sealed",
            "claim_scope": "exact-source-execution-only",
        })),
        owner_backpointers: owner_backpointers.clone(),
        child_receipts,
    })?;

    let preflight_record = prepublication_events.iter().find(|entry| {
        entry
            .pointer("/owner_crate")
            .and_then(serde_json::Value::as_str)
            == Some("aidens-runner")
            && entry
                .pointer("/schema_name")
                .and_then(serde_json::Value::as_str)
                == Some("learning-preflight-v1")
            && entry
                .pointer("/body/execution/run_id")
                .and_then(serde_json::Value::as_str)
                == Some(config.run_id.as_str())
    });
    let permits_valid = preflight_record.is_some_and(|entry| {
        entry
            .pointer("/body/execution/permit_grant_id")
            .and_then(serde_json::Value::as_str)
            == Some(config.permit_grant.permit_id.as_str())
            && entry
                .pointer("/body/execution/permit_use_id")
                .and_then(serde_json::Value::as_str)
                == Some(config.permit_use.receipt_id.as_str())
    });
    let effectful_record_persisted = prepublication_events.iter().any(|entry| {
        entry
            .pointer("/receipt_id")
            .and_then(serde_json::Value::as_str)
            == Some(initial.terminal_event_receipt_id.as_str())
            && entry
                .pointer("/schema_name")
                .and_then(serde_json::Value::as_str)
                == Some("effectful-evaluation-report-v1")
    });
    let required_checks_executed = initial.report.checks.fmt_executed
        && initial.report.checks.fmt_passed
        && initial.report.checks.clippy_executed
        && initial.report.checks.clippy_passed
        && initial.report.checks.test_executed
        && initial.report.checks.test_passed;
    let required_digests_present = [
        initial.report.before_tree_digest.as_str(),
        initial.report.after_tree_digest.as_str(),
        initial.report.rollback_tree_digest.as_str(),
        initial.report.patch_digest.as_str(),
        initial.report.checks.output_digest.as_str(),
        publication.content_digest.as_str(),
    ]
    .iter()
    .all(|digest| !digest.trim().is_empty());
    let mut projection_backpointers = owner_backpointers;
    projection_backpointers.push(CanonicalBackpointerV1::external(
        "aidens-receipts",
        "AiDENsRunBundleV3",
        "published-run-bundle",
        terminal_bundle.bundle.bundle_id.to_string(),
    ));
    let evidence = CodingLearningEvidenceV1 {
        execution_mode: initial.report.execution_mode.clone(),
        preflight_persisted: preflight_record.is_some(),
        permits_valid,
        typed_patch_applied: initial.report.verified,
        required_checks_executed,
        verification_positive: initial.report.verified,
        verification_degraded: false,
        required_digests_present,
        terminal_receipts_durable: initial.terminal_event_log_verified
            && effectful_record_persisted
            && terminal_bundle.digest_verified,
        receipts_healthy: terminal_bundle.digest_verified
            && terminal_bundle.index_verified
            && terminal_bundle.recovery_verified,
        publication_complete: publication.readback_verified,
        index_complete: terminal_bundle.index_verified,
        blocked: terminal_bundle.bundle.failure.blocked,
        revoked: promotion.disposition != ProcedureLifecycleDispositionV1::Promoted,
        stale: replay.report.before_tree_digest != initial.report.before_tree_digest,
        mock_only: false,
        fixture_only: false,
        indeterminate: !terminal_bundle.recovery_verified,
        canonical_backpointers: projection_backpointers,
        reason_codes: Vec::new(),
    };
    let terminal_projection = project_terminal_state(&evidence);
    if terminal_projection.state != CodingLearningTerminalStateV1::SucceededVerified {
        return Err(TerminalBundlePublicationError::Integration(format!(
            "terminal projection remained non-success: {:?}",
            terminal_projection.reason_codes
        )));
    }

    let projection_body = serde_json::to_value(&terminal_projection)
        .map_err(|error| TerminalBundlePublicationError::Integration(error.to_string()))?;
    let projection_material = serde_json::to_string(&serde_json::json!({
        "bundle_id": terminal_bundle.bundle.bundle_id,
        "projection": projection_body,
    }))
    .map_err(|error| TerminalBundlePublicationError::Integration(error.to_string()))?;
    let projection_receipt_id = generated_artifact_id_from_material(
        "aidens-learning-terminal-projection",
        &projection_material,
    )
    .to_string();
    let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&config.receipt_root))
        .map_err(|error| TerminalBundlePublicationError::Integration(error.to_string()))?;
    match log.inspect(&projection_receipt_id) {
        Ok(existing) => {
            if existing.body != projection_body
                || !existing.verify_digest()
                || !existing.verify_record_digest()
            {
                return Err(TerminalBundlePublicationError::ExistingConflict(
                    "terminal-projection-receipt".into(),
                ));
            }
        }
        Err(CanonicalEventLogError::NotFound(_)) => {
            log.append_json(
                "aidens-contracts",
                "coding-learning-terminal-projection-v1",
                projection_receipt_id.clone(),
                projection_body.clone(),
            )
            .map_err(|error| TerminalBundlePublicationError::Integration(error.to_string()))?;
        }
        Err(error) => {
            return Err(TerminalBundlePublicationError::Integration(
                error.to_string(),
            ));
        }
    }
    let reopened = CanonicalEventLog::open(log.config().clone())
        .map_err(|error| TerminalBundlePublicationError::Integration(error.to_string()))?;
    let projection_record = reopened
        .inspect(&projection_receipt_id)
        .map_err(|error| TerminalBundlePublicationError::Integration(error.to_string()))?;
    if projection_record.body != projection_body
        || !projection_record.verify_digest()
        || !projection_record.verify_record_digest()
        || !reopened
            .verify_chain()
            .map_err(|error| TerminalBundlePublicationError::Integration(error.to_string()))?
    {
        return Err(TerminalBundlePublicationError::Readback(
            "terminal projection failed fresh-log readback".into(),
        ));
    }
    Ok(RealSandboxTerminalClosureOutcomeV1 {
        schema: "AiDENsRealSandboxTerminalClosureOutcomeV1".into(),
        terminal_bundle,
        terminal_projection,
        terminal_projection_receipt_id: projection_receipt_id,
        terminal_projection_readback_verified: true,
    })
}

fn owner_pointer(
    owner_crate: &str,
    owner_type: &str,
    role: &str,
    external_id: &str,
) -> CanonicalBackpointerV1 {
    CanonicalBackpointerV1::external(owner_crate, owner_type, role, external_id)
}

fn closed_child<T: Serialize>(
    owner_id: &str,
    receipt: &T,
) -> Result<AiDENsRunChildReceiptV1, TerminalBundlePublicationError> {
    let value = serde_json::to_value(receipt)
        .map_err(|error| TerminalBundlePublicationError::Integration(error.to_string()))?;
    AiDENsRunChildReceiptV1::closed(owner_id, value)
        .map_err(|reasons| TerminalBundlePublicationError::Contract(vec![reasons]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidens_contracts::required_coding_learning_owner_roles;

    fn request(root: PathBuf, identity: &str) -> TerminalBundlePublicationRequestV1 {
        let fixture = include_str!("../../../tests/fixtures/p26/aidens_run_bundle_v3.json");
        let fixture: AiDENsRunBundleV3 = serde_json::from_str(fixture).unwrap();
        let owner_backpointers = required_coding_learning_owner_roles()
            .iter()
            .map(|role| {
                CanonicalBackpointerV1::external(
                    format!("owner:{role}"),
                    "OwnerNativeReceiptV1",
                    *role,
                    format!("{role}:blake3:0123456789abcdef"),
                )
            })
            .collect();
        let child_receipts = vec![
            AiDENsRunChildReceiptV1::closed(
                "owner:effectful-evaluation",
                serde_json::json!({"schema": "AiDENsEffectfulEvaluationReportV1", "verified": true}),
            )
            .unwrap(),
            AiDENsRunChildReceiptV1::closed(
                "owner:procedure-lifecycle-tested",
                serde_json::json!({"schema_version": "procedure_lifecycle_receipt_v1", "receipt_id": "tested", "receipt_digest": "digest-tested", "operation": "test", "disposition": "tested"}),
            )
            .unwrap(),
            AiDENsRunChildReceiptV1::closed(
                "owner:procedure-effectful-prerequisite",
                serde_json::json!({"schema_version": "procedure_effectful_evaluation_receipt_v1", "receipt_id": "effectful", "receipt_digest": "digest-effectful", "verified": true}),
            )
            .unwrap(),
            AiDENsRunChildReceiptV1::closed(
                "owner:forge-evidence-bundle",
                serde_json::json!({"version_id": "aidens-exact-source-execution-evidence-v1", "bundle_id": "evidence", "candidate_id": "candidate"}),
            )
            .unwrap(),
            AiDENsRunChildReceiptV1::closed(
                "owner:forge-export-receipt",
                serde_json::json!({"rendering_version": 3, "export_key": "export", "bundle_id": "evidence", "namespace": "aidens-learning"}),
            )
            .unwrap(),
            AiDENsRunChildReceiptV1::closed(
                "owner:semantic-memory-projection-import",
                serde_json::json!({"status": "complete", "direct_write": false, "source_envelope_id": "export", "content_digest": "digest-export"}),
            )
            .unwrap(),
            AiDENsRunChildReceiptV1::closed(
                "owner:procedure-lifecycle-promoted",
                serde_json::json!({"schema_version": "procedure_lifecycle_receipt_v1", "receipt_id": "promoted", "receipt_digest": "digest-promoted", "operation": "promote", "disposition": "promoted"}),
            )
            .unwrap(),
            AiDENsRunChildReceiptV1::closed(
                "owner:promoted-procedure-replay",
                serde_json::json!({"schema": "AiDENsPromotedProcedureReplayOutcomeV1", "action_allowed": true, "retained_patch_exact": true, "report": {"verified": true}}),
            )
            .unwrap(),
        ];
        TerminalBundlePublicationRequestV1 {
            schema: TerminalBundlePublicationRequestV1::SCHEMA.into(),
            store_root: root,
            identity_material: identity.into(),
            run_id: fixture.run_id,
            profile: fixture.profile,
            canonical_execution_context: fixture.canonical_execution_context,
            event_log: fixture.event_log,
            budget: fixture.budget,
            support: fixture.support,
            support_labels: fixture.support_labels,
            replay: fixture.replay,
            failure: fixture.failure,
            attempt_family_id: fixture.attempt_family_id,
            attempt_id: fixture.attempt_id,
            trial_id: fixture.trial_id,
            agent_spec_digest: fixture.agent_spec_digest,
            owner_backpointers,
            child_receipts,
        }
    }

    #[test]
    fn publishes_reopens_and_recovers_exact_child_closed_bundle() {
        let temp = tempfile::tempdir().unwrap();
        let first =
            publish_terminal_bundle(request(temp.path().into(), "terminal-material")).unwrap();
        assert_eq!(
            first.disposition,
            TerminalBundlePublicationDispositionV1::Published
        );
        assert!(first.digest_verified && first.index_verified && first.recovery_verified);

        let recovered =
            publish_terminal_bundle(request(temp.path().into(), "terminal-material")).unwrap();
        assert_eq!(
            recovered.disposition,
            TerminalBundlePublicationDispositionV1::RecoveredIdempotently
        );
        assert_eq!(recovered.bundle.bundle_id, first.bundle.bundle_id);
        assert_eq!(recovered.store_record, first.store_record);
    }

    #[test]
    fn missing_required_owner_role_fails_before_store_write() {
        let temp = tempfile::tempdir().unwrap();
        let mut request = request(temp.path().into(), "missing-owner");
        request
            .owner_backpointers
            .retain(|backpointer| backpointer.role != "replay");
        let error = publish_terminal_bundle(request).unwrap_err();
        assert!(matches!(
            error,
            TerminalBundlePublicationError::Contract(ref reasons)
                if reasons.iter().any(|reason| reason == "required-owner-reference-missing:replay")
        ));
        let store =
            RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(temp.path())).unwrap();
        assert!(store.list_records().unwrap().is_empty());
    }

    #[test]
    fn conflicting_retry_is_rejected_without_overwriting_published_bundle() {
        let temp = tempfile::tempdir().unwrap();
        let first = publish_terminal_bundle(request(temp.path().into(), "first-material")).unwrap();
        let error =
            publish_terminal_bundle(request(temp.path().into(), "changed-material")).unwrap_err();
        assert!(matches!(
            error,
            TerminalBundlePublicationError::ExistingConflict(_)
        ));
        let store =
            RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(temp.path())).unwrap();
        let inspection = store.inspect(&first.bundle.run_id).unwrap();
        assert_eq!(inspection.record, first.store_record);
    }
}
