//! Bounded, restartable phase chaining for autonomous learning.
//!
//! This module is deliberately a controller, not another owner of learning
//! truth.  Every invocation reconstructs the state from the owner stores,
//! records a projection checkpoint, performs at most one owner-native effect,
//! and reconstructs the state again.

use crate::learning_controller::{run_real_sandbox, RealSandboxLearningConfig};
use crate::learning_coordinator::{
    append_learning_coordinator_checkpoint, LearningOwnerSnapshotV1,
};
use crate::learning_resume::{
    rebuild_learning_owner_snapshot, resume_learning_once, LearningOwnerLocatorV1,
    LearningResumeAuthorityV1, LearningResumeError,
};
use crate::learning_sealed_replay::{
    execute_owner_admitted_sealed_replay, prepare_owner_admitted_sealed_replay_material,
    OwnerAdmittedSealedReplayMaterialV1, OwnerAdmittedSealedReplayRequestV1,
};
use aidens_contracts::{
    LearningCoordinatorDispositionV1, LearningCoordinatorProjectionV1, NextLearningActionV1,
};
use aidens_receipts::{CanonicalEventLog, CanonicalEventLogConfig};
use authority_delegation::{LearningAuthorizedActionV1, LearningAutonomyLeaseV1};
use semantic_memory::{ProcedureActionPermitV1, ProcedureLifecyclePermitV1};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Inputs needed by the controller.  The optional owner-native inputs are
/// intentionally supplied by the caller; the controller never invents them.
#[derive(Debug, Clone)]
pub struct LearningAutonomyContextV1 {
    pub locator: LearningOwnerLocatorV1,
    pub lease: LearningAutonomyLeaseV1,
    pub controller_id: String,
    pub now: String,
    pub transitions_used: u64,
    pub executions_used: u64,
    pub lifecycle_permit: Option<ProcedureLifecyclePermitV1>,
    pub action_permit: Option<ProcedureActionPermitV1>,
    pub real_sandbox: Option<RealSandboxLearningConfig>,
    pub replay_material: Option<OwnerAdmittedSealedReplayMaterialV1>,
}

#[derive(Debug, Error)]
pub enum LearningAutonomyError {
    #[error("owner reconstruction failed: {0}")]
    Rebuild(#[from] LearningResumeError),
    #[error("coordinator checkpoint failed: {0}")]
    Checkpoint(String),
    #[error("learning action failed: {0}")]
    Action(String),
    #[error("invalid autonomy context: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LearningAutonomyBoundaryV1 {
    InProgress,
    Completed,
    Quarantined,
    Failed,
    AwaitingAuthority,
    AwaitingCapability,
    BudgetExhausted,
}

/// The complete readback for one controller turn.  `transition_performed`
/// is `None` whenever an authority/capability/budget boundary was reached.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningTransitionReportV1 {
    pub schema: String,
    pub before: LearningCoordinatorProjectionV1,
    pub after: LearningCoordinatorProjectionV1,
    pub before_checkpoint_id: String,
    pub after_checkpoint_id: String,
    pub transition_performed: Option<String>,
    pub boundary: LearningAutonomyBoundaryV1,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningDriveReportV1 {
    pub schema: String,
    pub transitions: Vec<LearningTransitionReportV1>,
    pub final_projection: LearningCoordinatorProjectionV1,
    pub boundary: LearningAutonomyBoundaryV1,
    pub transitions_used: u64,
    pub executions_used: u64,
}

/// Admit replay through the semantic-memory owner and return the exact
/// material that the sealed replay executor is allowed to consume.  This is
/// an adapter only: admission, lifecycle state, and permit binding remain
/// owned by semantic-memory.
pub async fn admit_replay_from_adjudication(
    request: OwnerAdmittedSealedReplayRequestV1,
) -> Result<OwnerAdmittedSealedReplayMaterialV1, LearningAutonomyError> {
    prepare_owner_admitted_sealed_replay_material(request)
        .await
        .map_err(|error| LearningAutonomyError::Action(error.to_string()))
}

/// Execute previously admitted replay material through the sealed owner path.
/// Material cannot be widened or converted to a host/fixture execution here.
pub async fn replay_admitted_candidate(
    material: OwnerAdmittedSealedReplayMaterialV1,
) -> Result<crate::learning_sealed_replay::OwnerAdmittedSealedReplayReportV1, LearningAutonomyError>
{
    execute_owner_admitted_sealed_replay(material)
        .await
        .map_err(|error| LearningAutonomyError::Action(error.to_string()))
}

/// Rebuild, reduce, checkpoint, verify authority/budgets, and dispatch at most
/// one owner-native transition.  A second rebuild and checkpoint follows every
/// dispatch, so a process crash can safely resume from the last owner receipt.
pub async fn execute_learning_transition_once(
    context: &LearningAutonomyContextV1,
) -> Result<LearningTransitionReportV1, LearningAutonomyError> {
    validate_context(context)?;
    let before_readback = rebuild_learning_owner_snapshot(&context.locator).await?;
    let before = before_readback.coordinator_projection.clone();
    let before_checkpoint_id =
        checkpoint(&context.locator, &before_readback.owner_snapshot_rebuilt)?;

    let initial_boundary = boundary_for_projection(&before);
    if initial_boundary != LearningAutonomyBoundaryV1::InProgress {
        return Ok(report(
            before.clone(),
            before,
            before_checkpoint_id.clone(),
            before_checkpoint_id,
            None,
            initial_boundary,
            vec!["terminal-or-blocked-owner-state".into()],
        ));
    }

    if context.transitions_used >= context.lease.maximum_transitions {
        return Ok(report(
            before.clone(),
            before,
            before_checkpoint_id.clone(),
            before_checkpoint_id,
            None,
            LearningAutonomyBoundaryV1::BudgetExhausted,
            vec!["maximum-transitions-exhausted".into()],
        ));
    }

    let action = before.next_action.clone();
    let required_authority = authority_action(&action);
    if let Some(required) = required_authority {
        if !context.lease.allowed_actions.contains(&required) {
            return Ok(report(
                before.clone(),
                before,
                before_checkpoint_id.clone(),
                before_checkpoint_id,
                None,
                LearningAutonomyBoundaryV1::AwaitingAuthority,
                vec![format!("lease-does-not-delegate:{required:?}")],
            ));
        }
    }

    if action_consumes_execution(&action)
        && context.executions_used >= context.lease.maximum_executions
    {
        return Ok(report(
            before.clone(),
            before,
            before_checkpoint_id.clone(),
            before_checkpoint_id,
            None,
            LearningAutonomyBoundaryV1::BudgetExhausted,
            vec!["maximum-executions-exhausted".into()],
        ));
    }

    let transition = match action {
        NextLearningActionV1::ExecuteAndVerify => {
            let config = context.real_sandbox.clone().ok_or_else(|| {
                LearningAutonomyError::Invalid("real-sandbox configuration is required".into())
            });
            match config {
                Ok(config) => run_real_sandbox(config)
                    .await
                    .map(|_| "execute-and-verify".to_string())
                    .map_err(|error| LearningAutonomyError::Action(error.to_string())),
                Err(error) => Err(error),
            }
        }
        NextLearningActionV1::PromoteProcedure => {
            let authority = LearningResumeAuthorityV1 {
                lifecycle_permit: context.lifecycle_permit.clone(),
                action_permit: context.action_permit.clone(),
                idempotency_key: Some(format!("autonomy:promote:{}", context.locator.run_id)),
            };
            resume_learning_once(&context.locator, &authority)
                .await
                .map(|result| {
                    result
                        .transition_performed
                        .unwrap_or_else(|| "promote".into())
                })
                .map_err(|error| LearningAutonomyError::Action(error.to_string()))
        }
        NextLearningActionV1::ReplayProcedure => {
            let material = context.replay_material.clone().ok_or_else(|| {
                LearningAutonomyError::Invalid("sealed replay material is required".into())
            });
            match material {
                Ok(material) => replay_admitted_candidate(material)
                    .await
                    .map(|_| "replay-procedure".to_string())
                    .map_err(|error| LearningAutonomyError::Action(error.to_string())),
                Err(error) => Err(error),
            }
        }
        NextLearningActionV1::PublishTerminalEvidence => {
            let after_checkpoint_id =
                checkpoint(&context.locator, &before_readback.owner_snapshot_rebuilt)?;
            return Ok(report(
                before.clone(),
                before,
                before_checkpoint_id,
                after_checkpoint_id,
                None,
                LearningAutonomyBoundaryV1::AwaitingAuthority,
                vec!["terminal-publication-authority-required".into()],
            ));
        }
        _ => {
            let after_checkpoint_id =
                checkpoint(&context.locator, &before_readback.owner_snapshot_rebuilt)?;
            return Ok(report(
                before.clone(),
                before,
                before_checkpoint_id,
                after_checkpoint_id,
                None,
                LearningAutonomyBoundaryV1::AwaitingCapability,
                vec![format!("owner-transition-not-yet-adapted:{action:?}")],
            ));
        }
    }?;

    let after_readback = rebuild_learning_owner_snapshot(&context.locator).await?;
    let after = after_readback.coordinator_projection.clone();
    let after_checkpoint_id = checkpoint(&context.locator, &after_readback.owner_snapshot_rebuilt)?;
    let boundary = boundary_for_projection(&after);
    Ok(report(
        before,
        after,
        before_checkpoint_id,
        after_checkpoint_id,
        Some(transition),
        boundary,
        Vec::new(),
    ))
}

/// Continue one-transition turns until a terminal, authority, capability, or
/// budget boundary is reached.  Counters are local controller inputs; durable
/// evidence remains exclusively in the owner stores.
pub async fn drive_learning_until_boundary(
    context: LearningAutonomyContextV1,
) -> Result<LearningDriveReportV1, LearningAutonomyError> {
    let mut context = context;
    let mut transitions = Vec::new();
    loop {
        let report = execute_learning_transition_once(&context).await?;
        let boundary = report.boundary.clone();
        if report.transition_performed.is_some() {
            context.transitions_used += 1;
            if report
                .transition_performed
                .as_deref()
                .is_some_and(|name| name == "execute-and-verify" || name == "replay-procedure")
            {
                context.executions_used += 1;
            }
        }
        transitions.push(report);
        if boundary != LearningAutonomyBoundaryV1::InProgress {
            let final_projection = transitions
                .last()
                .expect("transition report was just pushed")
                .after
                .clone();
            return Ok(LearningDriveReportV1 {
                schema: "AiDENsLearningDriveReportV1".into(),
                transitions,
                final_projection,
                boundary,
                transitions_used: context.transitions_used,
                executions_used: context.executions_used,
            });
        }
    }
}

fn validate_context(context: &LearningAutonomyContextV1) -> Result<(), LearningAutonomyError> {
    context
        .lease
        .validate()
        .map_err(|error| LearningAutonomyError::Invalid(error.to_string()))?;
    if context.controller_id != context.lease.controller_id {
        return Err(LearningAutonomyError::Invalid(
            "controller does not match lease".into(),
        ));
    }
    if context.now >= context.lease.expires_at {
        return Err(LearningAutonomyError::Invalid(
            "autonomy lease expired".into(),
        ));
    }
    Ok(())
}

fn checkpoint(
    locator: &LearningOwnerLocatorV1,
    snapshot: &LearningOwnerSnapshotV1,
) -> Result<String, LearningAutonomyError> {
    let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(
        locator.run_bundle_store_root.clone(),
    ))
    .map_err(|error| LearningAutonomyError::Checkpoint(error.to_string()))?;
    append_learning_coordinator_checkpoint(&log, snapshot)
        .map(|entry| entry.receipt_id)
        .map_err(|error| LearningAutonomyError::Checkpoint(error.to_string()))
}

fn authority_action(action: &NextLearningActionV1) -> Option<LearningAuthorizedActionV1> {
    match action {
        NextLearningActionV1::ExecuteAndVerify => {
            Some(LearningAuthorizedActionV1::ExecuteAndVerify)
        }
        NextLearningActionV1::PromoteProcedure => {
            Some(LearningAuthorizedActionV1::PromoteCandidate)
        }
        NextLearningActionV1::ReplayProcedure => Some(LearningAuthorizedActionV1::ReplayProcedure),
        NextLearningActionV1::PublishTerminalEvidence => {
            Some(LearningAuthorizedActionV1::PublishTerminalEvidence)
        }
        _ => None,
    }
}

fn action_consumes_execution(action: &NextLearningActionV1) -> bool {
    matches!(
        action,
        NextLearningActionV1::ExecuteAndVerify | NextLearningActionV1::ReplayProcedure
    )
}

fn boundary_for_projection(
    projection: &LearningCoordinatorProjectionV1,
) -> LearningAutonomyBoundaryV1 {
    if projection.disposition == LearningCoordinatorDispositionV1::Failed {
        return LearningAutonomyBoundaryV1::Failed;
    }
    if projection
        .reason_codes
        .iter()
        .any(|reason| reason.contains("mixed-run") || reason.contains("stale-binding"))
    {
        return LearningAutonomyBoundaryV1::Quarantined;
    }
    if projection.stage == aidens_contracts::LearningCoordinatorStageV1::Quarantined {
        return LearningAutonomyBoundaryV1::Quarantined;
    }
    if matches!(
        projection.stage,
        aidens_contracts::LearningCoordinatorStageV1::TerminalPublished
            | aidens_contracts::LearningCoordinatorStageV1::RolledBack
            | aidens_contracts::LearningCoordinatorStageV1::Revoked
    ) {
        return LearningAutonomyBoundaryV1::Completed;
    }
    if projection.next_action == NextLearningActionV1::PublishTerminalEvidence {
        return LearningAutonomyBoundaryV1::InProgress;
    }
    if projection.disposition != LearningCoordinatorDispositionV1::Pending {
        return LearningAutonomyBoundaryV1::AwaitingCapability;
    }
    LearningAutonomyBoundaryV1::InProgress
}

fn report(
    before: LearningCoordinatorProjectionV1,
    after: LearningCoordinatorProjectionV1,
    before_checkpoint_id: String,
    after_checkpoint_id: String,
    transition_performed: Option<String>,
    boundary: LearningAutonomyBoundaryV1,
    reason_codes: Vec<String>,
) -> LearningTransitionReportV1 {
    LearningTransitionReportV1 {
        schema: "AiDENsLearningTransitionReportV1".into(),
        before,
        after,
        before_checkpoint_id,
        after_checkpoint_id,
        transition_performed,
        boundary,
        reason_codes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_owner_actions_to_authority_actions() {
        assert_eq!(
            authority_action(&NextLearningActionV1::PromoteProcedure),
            Some(LearningAuthorizedActionV1::PromoteCandidate)
        );
        assert_eq!(
            authority_action(&NextLearningActionV1::ReplayProcedure),
            Some(LearningAuthorizedActionV1::ReplayProcedure)
        );
    }

    #[test]
    fn mixed_owner_evidence_is_a_quarantine_boundary() {
        let projection = LearningCoordinatorProjectionV1 {
            stage: aidens_contracts::LearningCoordinatorStageV1::Preflighted,
            disposition: LearningCoordinatorDispositionV1::Blocked,
            owner_receipts: Vec::new(),
            owner_binding_digests: Default::default(),
            next_action: NextLearningActionV1::AwaitPreflight,
            reason_codes: vec!["snapshot-has-mixed-run-ids".into()],
        };
        assert_eq!(
            boundary_for_projection(&projection),
            LearningAutonomyBoundaryV1::Quarantined
        );
    }
}
