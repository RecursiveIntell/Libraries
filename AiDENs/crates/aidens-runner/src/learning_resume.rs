//! Reconstruct coordinator truth from owner-side artifacts.

use crate::learning_coordinator::{
    reduce_learning_coordinator_projection, LearningOwnerSnapshotV1,
};
use aidens_contracts::{
    AiDENsRunBundleV3, AiDENsRunChildReceiptV1, ArtifactKindV1, LearningCoordinatorDispositionV1,
    LearningCoordinatorProjectionV1, LearningCoordinatorStageV1, NextLearningActionV1,
    OwnerBindingDigestsV1, OwnerReceiptPointerV1,
};
use aidens_receipts::{
    RunBundleStore, RunBundleStoreConfig, RunBundleStoreError, RunBundleStoreInspection,
};
use forge_engine::ForgeStore;
use forge_memory_bridge::{
    AdjudicationBindingV1, BridgeError, ForgeAdjudicationStore, IdentityDigest,
};
use semantic_memory::{
    load_procedure_lifecycle_receipt, load_procedure_owner_snapshot,
    load_procedure_replay_snapshot, verify_procedure_lifecycle_receipt_v1, MemoryConfig,
    MemoryStore, ProcedureActionPermitV1, ProcedureLifecycleDispositionV1,
    ProcedureLifecyclePermitV1, ProcedureLifecycleReceiptV1, ProcedureOwnerSnapshotV1,
    ProcedureReplaySnapshotV1,
};
use serde_json::Value;
use std::path::PathBuf;
use thiserror::Error;
use verification_adjudication::CandidatePromotionAdjudicationV1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningOwnerLocatorV1 {
    pub memory_store_root: PathBuf,
    pub run_bundle_store_root: PathBuf,
    pub forge_store_root: PathBuf,
    pub run_id: String,
    pub candidate_artifact_id: String,
    pub tested_receipt_id: String,
    pub bundle_run_id: Option<String>,
    pub adjudication_id: Option<String>,
    pub replay_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LearningResumeReadbackV1 {
    pub locator: LearningOwnerLocatorV1,
    pub owner_snapshot: ProcedureOwnerSnapshotV1,
    pub tested_lifecycle_receipt: ProcedureLifecycleReceiptV1,
    pub replay_snapshot: Option<ProcedureReplaySnapshotV1>,
    pub adjudication: Option<CandidatePromotionAdjudicationV1>,
    pub run_bundle: Option<RunBundleStoreInspection>,
    pub owner_snapshot_rebuilt: LearningOwnerSnapshotV1,
    pub coordinator_projection: LearningCoordinatorProjectionV1,
}

#[derive(Debug, Error)]
pub enum LearningResumeError {
    #[error("learning resume locator is incomplete: {0}")]
    InvalidLocator(String),
    #[error("owner store failure: {0}")]
    OwnerRead(String),
    #[error("run bundle store failure: {0}")]
    RunBundleRead(String),
    #[error("forge adjudication store failure: {0}")]
    ForgeRead(String),
    #[error("bridge error: {0}")]
    Bridge(#[from] BridgeError),
    #[error("tested lifecycle receipt does not match requested candidate")]
    LifecycleMismatch,
    #[error("replay witness conflicts with run-bundle child record")]
    ReplayConflict,
}

pub async fn rebuild_learning_owner_snapshot(
    locator: &LearningOwnerLocatorV1,
) -> Result<LearningResumeReadbackV1, LearningResumeError> {
    validate_locator(locator)?;

    let memory = MemoryStore::open(MemoryConfig {
        base_dir: locator.memory_store_root.clone(),
        ..Default::default()
    })
    .map_err(|error| LearningResumeError::OwnerRead(error.to_string()))?;

    let owner_snapshot = load_procedure_owner_snapshot(&memory, &locator.candidate_artifact_id)
        .await
        .map_err(|error| LearningResumeError::OwnerRead(error.to_string()))?;

    let tested_lifecycle_receipt =
        load_procedure_lifecycle_receipt(&memory, &locator.tested_receipt_id)
            .await
            .map_err(|error| LearningResumeError::OwnerRead(error.to_string()))?;

    if tested_lifecycle_receipt.artifact_id != owner_snapshot.artifact.artifact_id
        || tested_lifecycle_receipt.artifact_digest != owner_snapshot.artifact.artifact_digest
        || !verify_procedure_lifecycle_receipt_v1(&tested_lifecycle_receipt)
    {
        return Err(LearningResumeError::LifecycleMismatch);
    }

    let mut reconstructed = empty_learning_snapshot(&locator.run_id);

    apply_candidate_test(
        &mut reconstructed,
        &tested_lifecycle_receipt,
        locator.run_id.clone(),
    );

    if let Some(effectful) = owner_snapshot.effectful_receipt.as_ref() {
        apply_effectful_evidence(
            &mut reconstructed,
            &effectful.receipt_id,
            &effectful.receipt_digest,
            locator.run_id.clone(),
        );
    }

    let mut replay_snapshot = None;
    if let Some(replay_id) = locator.replay_id.as_deref() {
        let replay = load_procedure_replay_snapshot(&memory, replay_id)
            .await
            .map_err(|error| LearningResumeError::OwnerRead(error.to_string()))?;
        if replay.inputs.original_artifact_id != locator.candidate_artifact_id {
            return Err(LearningResumeError::OwnerRead(
                "replay snapshot does not match candidate artifact".into(),
            ));
        }
        apply_replay_from_snapshot(&mut reconstructed, &replay, locator.run_id.clone());
        replay_snapshot = Some(replay);
    }

    let mut run_bundle = None;
    let mut bundle_children: Vec<AiDENsRunChildReceiptV1> = Vec::new();

    if let Some(inspection) = load_run_bundle_inspection(locator).await? {
        let children = decode_bundle_children(&inspection)?;
        run_bundle = Some(inspection);
        bundle_children = children;

        for child in &bundle_children {
            apply_bundle_child(
                &mut reconstructed,
                child,
                locator.run_id.clone(),
                &mut replay_snapshot,
            )?;
        }
    }

    let mut adjudication = None;
    let evidence_bundle_binding = bundle_children.iter().find_map(|child| {
        if child.owner_id != "owner:forge-evidence-bundle" {
            return None;
        }
        let bundle_id = child
            .receipt
            .pointer("/bundle_id")
            .and_then(Value::as_str)?;
        Some((bundle_id.to_string(), child.digest.clone()))
    });

    if let Some(adjudication_id) = locator.adjudication_id.as_deref() {
        let (evidence_bundle_id, evidence_bundle_digest) =
            evidence_bundle_binding.ok_or(LearningResumeError::ForgeRead(
                "missing forge evidence bundle linkage for adjudication binding".into(),
            ))?;

        let forge = ForgeStore::open(&locator.forge_store_root)
            .map_err(|error| LearningResumeError::ForgeRead(error.to_string()))?;
        let loaded = forge
            .read_verified_adjudication(adjudication_id)
            .map_err(|error| LearningResumeError::ForgeRead(error.to_string()))?;
        forge
            .verify_adjudication_binding(
                adjudication_id,
                &AdjudicationBindingV1 {
                    candidate_id: owner_snapshot.artifact.artifact_id.clone(),
                    candidate_digest: IdentityDigest::new(
                        owner_snapshot.artifact.artifact_digest.clone(),
                    )
                    .map_err(|error| LearningResumeError::ForgeRead(error.to_string()))?,
                    evidence_bundle_id,
                    evidence_bundle_digest: IdentityDigest::new(evidence_bundle_digest)
                        .map_err(|error| LearningResumeError::ForgeRead(error.to_string()))?,
                },
            )
            .map_err(LearningResumeError::Bridge)?;

        apply_adjudication(
            &mut reconstructed,
            &loaded,
            locator.run_id.clone(),
            &tested_lifecycle_receipt,
        )?;
        adjudication = Some(loaded);
    }

    let coordinator_projection = reduce_learning_coordinator_projection(&reconstructed);

    Ok(LearningResumeReadbackV1 {
        locator: locator.clone(),
        owner_snapshot,
        tested_lifecycle_receipt,
        replay_snapshot,
        adjudication,
        run_bundle,
        owner_snapshot_rebuilt: reconstructed,
        coordinator_projection,
    })
}

/// Authority supplied for a resume transition. Only populated fields are used;
/// missing entries produce `Pending`, not a widened request.
#[derive(Debug, Clone, Default)]
pub struct LearningResumeAuthorityV1 {
    pub lifecycle_permit: Option<ProcedureLifecyclePermitV1>,
    pub action_permit: Option<ProcedureActionPermitV1>,
    pub idempotency_key: Option<String>,
}

/// Report from a single resume invocation.
#[derive(Debug, Clone)]
pub struct LearningResumeReportV1 {
    pub before: LearningCoordinatorProjectionV1,
    pub after: LearningCoordinatorProjectionV1,
    pub transition_performed: Option<String>,
    pub owner_receipt_id: Option<String>,
    pub owner_receipt_digest: Option<String>,
}

/// Rebuild from owner stores, perform at most one authorized transition, rebuild again.
/// Missing authority returns pending without mutation.
pub async fn resume_learning_once(
    locator: &LearningOwnerLocatorV1,
    authority: &LearningResumeAuthorityV1,
) -> Result<LearningResumeReportV1, LearningResumeError> {
    let before_readback = rebuild_learning_owner_snapshot(locator).await?;
    let before = before_readback.coordinator_projection.clone();

    if before.disposition != LearningCoordinatorDispositionV1::Pending {
        return Ok(LearningResumeReportV1 {
            before: before.clone(),
            after: before,
            transition_performed: None,
            owner_receipt_id: None,
            owner_receipt_digest: None,
        });
    }

    let result = match before.next_action {
        NextLearningActionV1::PromoteProcedure => {
            let permit = authority.lifecycle_permit.as_ref().ok_or_else(|| {
                LearningResumeError::OwnerRead("lifecycle permit required for promotion".into())
            })?;
            let adjudication = before_readback.adjudication.as_ref().ok_or_else(|| {
                LearningResumeError::OwnerRead(
                    "verified adjudication required for promotion".into(),
                )
            })?;
            let memory = MemoryStore::open(MemoryConfig {
                base_dir: locator.memory_store_root.clone(),
                ..Default::default()
            })
            .map_err(|e| LearningResumeError::OwnerRead(e.to_string()))?;
            let forge = ForgeStore::open(&locator.forge_store_root)
                .map_err(|e| LearningResumeError::ForgeRead(e.to_string()))?;
            let adapter = crate::learning_lifecycle::ProcedureLifecycleAdapter::new(&memory);
            let key = authority
                .idempotency_key
                .clone()
                .unwrap_or_else(|| format!("resume:promote:{}", locator.run_id));
            let receipt = adapter
                .promote_adjudicated(&forge, permit.clone(), &adjudication.adjudication_id, key)
                .await
                .map_err(|e| LearningResumeError::OwnerRead(e.to_string()))?;
            Some((
                "promote".to_string(),
                receipt.receipt_id,
                receipt.receipt_digest,
            ))
        }
        NextLearningActionV1::PublishTerminalEvidence => {
            return Err(LearningResumeError::OwnerRead(
                "terminal publication requires learn close with the original run config".into(),
            ));
        }
        _ => None,
    };

    let after_readback = rebuild_learning_owner_snapshot(locator).await?;
    Ok(LearningResumeReportV1 {
        before,
        after: after_readback.coordinator_projection,
        transition_performed: result.as_ref().map(|(t, _, _)| t.clone()),
        owner_receipt_id: result.as_ref().map(|(_, id, _)| id.clone()),
        owner_receipt_digest: result.as_ref().map(|(_, _, digest)| digest.clone()),
    })
}

/// Validate that all owner receipts needed for terminal closure exist.
/// Returns the coordinator projection showing whether closure is ready.
/// Actual closure requires the original runtime config and outcome objects
/// from the initial `learn run` — this function does not synthesize them.
pub async fn close_learning_from_owners(
    locator: &LearningOwnerLocatorV1,
) -> Result<LearningCoordinatorProjectionV1, LearningResumeError> {
    let readback = rebuild_learning_owner_snapshot(locator).await?;
    let projection = readback.coordinator_projection;

    if projection.stage != LearningCoordinatorStageV1::TerminalPublished
        && projection.next_action != NextLearningActionV1::PublishTerminalEvidence
        && projection.next_action != NextLearningActionV1::Completed
    {
        return Err(LearningResumeError::OwnerRead(format!(
            "not ready for closure: stage={:?} next_action={:?}",
            projection.stage, projection.next_action
        )));
    }

    Ok(projection)
}

async fn load_run_bundle_inspection(
    locator: &LearningOwnerLocatorV1,
) -> Result<Option<RunBundleStoreInspection>, LearningResumeError> {
    let run_id = locator
        .bundle_run_id
        .as_deref()
        .unwrap_or(locator.run_id.as_str());
    let store = RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(
        locator.run_bundle_store_root.clone(),
    ))
    .map_err(|error| LearningResumeError::RunBundleRead(error.to_string()))?;

    match store.inspect(run_id) {
        Ok(inspection) => Ok(Some(inspection)),
        Err(RunBundleStoreError::NotFound(_)) => Ok(None),
        Err(error) => Err(LearningResumeError::RunBundleRead(error.to_string())),
    }
}

fn decode_bundle_children(
    inspection: &RunBundleStoreInspection,
) -> Result<Vec<AiDENsRunChildReceiptV1>, LearningResumeError> {
    let bundle: AiDENsRunBundleV3 = serde_json::from_value(inspection.bundle.clone())
        .map_err(|error| LearningResumeError::RunBundleRead(error.to_string()))?;
    Ok(bundle.child_receipts)
}

fn validate_locator(locator: &LearningOwnerLocatorV1) -> Result<(), LearningResumeError> {
    if locator.memory_store_root.as_os_str().is_empty() {
        return Err(LearningResumeError::InvalidLocator(
            "memory_store_root".into(),
        ));
    }
    if locator.run_bundle_store_root.as_os_str().is_empty() {
        return Err(LearningResumeError::InvalidLocator(
            "run_bundle_store_root".into(),
        ));
    }
    if locator.forge_store_root.as_os_str().is_empty() {
        return Err(LearningResumeError::InvalidLocator(
            "forge_store_root".into(),
        ));
    }
    if locator.run_id.trim().is_empty() {
        return Err(LearningResumeError::InvalidLocator("run_id".into()));
    }
    if locator.candidate_artifact_id.trim().is_empty() {
        return Err(LearningResumeError::InvalidLocator(
            "candidate_artifact_id".into(),
        ));
    }
    if locator.tested_receipt_id.trim().is_empty() {
        return Err(LearningResumeError::InvalidLocator(
            "tested_receipt_id".into(),
        ));
    }
    Ok(())
}

fn empty_learning_snapshot(run_id: &str) -> LearningOwnerSnapshotV1 {
    LearningOwnerSnapshotV1 {
        run_id: run_id.to_string(),
        preflight: None,
        preflight_run_id: Some(run_id.to_string()),
        executed_verified: None,
        executed_run_id: Some(run_id.to_string()),
        candidate_tested: None,
        candidate_tested_run_id: Some(run_id.to_string()),
        effectful_evidence: None,
        effectful_run_id: Some(run_id.to_string()),
        forge_published: None,
        forge_run_id: Some(run_id.to_string()),
        adjudicated: None,
        adjudicated_run_id: Some(run_id.to_string()),
        eligible_for_lifecycle: None,
        eligibility_run_id: Some(run_id.to_string()),
        quarantined: None,
        quarantine_run_id: Some(run_id.to_string()),
        promoted: None,
        promote_run_id: Some(run_id.to_string()),
        replay_admitted: None,
        replay_admission_run_id: Some(run_id.to_string()),
        replayed: None,
        replay_run_id: Some(run_id.to_string()),
        rolled_back: None,
        rollback_run_id: Some(run_id.to_string()),
        revoked: None,
        revoke_run_id: Some(run_id.to_string()),
        terminal_published: None,
        terminal_run_id: Some(run_id.to_string()),
        failed: false,
        failed_reason_codes: Vec::new(),
        owner_binding_digests: OwnerBindingDigestsV1::default(),
    }
}

fn apply_candidate_test(
    snapshot: &mut LearningOwnerSnapshotV1,
    receipt: &ProcedureLifecycleReceiptV1,
    run_id: String,
) {
    if matches!(
        receipt.disposition,
        ProcedureLifecycleDispositionV1::Tested
            | ProcedureLifecycleDispositionV1::Promoted
            | ProcedureLifecycleDispositionV1::Quarantined
            | ProcedureLifecycleDispositionV1::Revoked
            | ProcedureLifecycleDispositionV1::RolledBack
    ) {
        snapshot.candidate_tested = Some(pointer(
            "semantic-memory",
            &receipt.receipt_id,
            &receipt.receipt_digest,
        ));
        snapshot.owner_binding_digests.candidate_test = Some(receipt.receipt_digest.clone());
        snapshot.candidate_tested_run_id = Some(run_id);
    }
}

fn apply_effectful_evidence(
    snapshot: &mut LearningOwnerSnapshotV1,
    receipt_id: &str,
    receipt_digest: &str,
    run_id: String,
) {
    snapshot.effectful_evidence = Some(pointer("semantic-memory", receipt_id, receipt_digest));
    snapshot.owner_binding_digests.effectful_evidence = Some(receipt_digest.to_string());
    snapshot.effectful_run_id = Some(run_id);
}

fn apply_replay_from_snapshot(
    snapshot: &mut LearningOwnerSnapshotV1,
    replay: &ProcedureReplaySnapshotV1,
    run_id: String,
) {
    snapshot.replay_admitted = Some(pointer(
        "semantic-memory",
        &replay.admission.replay_id,
        &replay.admission.admission_digest,
    ));
    snapshot.owner_binding_digests.replay_admission =
        Some(replay.admission.admission_digest.clone());
    snapshot.replay_admission_run_id = Some(run_id.clone());

    if let Some(result) = &replay.result {
        snapshot.replayed = Some(pointer(
            "semantic-memory",
            &result.replay_id,
            &result.result_digest,
        ));
        snapshot.owner_binding_digests.replay = Some(result.result_digest.clone());
        snapshot.replay_run_id = Some(run_id);
    }
}

fn apply_bundle_child(
    snapshot: &mut LearningOwnerSnapshotV1,
    child: &AiDENsRunChildReceiptV1,
    run_id: String,
    replay_snapshot: &mut Option<ProcedureReplaySnapshotV1>,
) -> Result<(), LearningResumeError> {
    match child.owner_id.as_str() {
        "owner:procedure-lifecycle-tested" => {
            if let (Some(receipt_id), Some(receipt_digest)) = (
                child.receipt.pointer("/receipt_id").and_then(Value::as_str),
                child
                    .receipt
                    .pointer("/receipt_digest")
                    .and_then(Value::as_str),
            ) {
                snapshot.candidate_tested =
                    Some(pointer("semantic-memory", receipt_id, receipt_digest));
                snapshot.owner_binding_digests.candidate_test = Some(receipt_digest.to_string());
                snapshot.candidate_tested_run_id = Some(run_id);
            }
        }
        "owner:procedure-effectful-prerequisite" => {
            if let (Some(receipt_id), Some(receipt_digest)) = (
                child.receipt.pointer("/receipt_id").and_then(Value::as_str),
                child
                    .receipt
                    .pointer("/receipt_digest")
                    .and_then(Value::as_str),
            ) {
                snapshot.effectful_evidence =
                    Some(pointer("semantic-memory", receipt_id, receipt_digest));
                snapshot.owner_binding_digests.effectful_evidence =
                    Some(receipt_digest.to_string());
                snapshot.effectful_run_id = Some(run_id);
            }
        }
        "owner:forge-evidence-bundle" => {
            if let Some(bundle_id) = child.receipt.pointer("/bundle_id").and_then(Value::as_str) {
                snapshot.forge_published =
                    Some(pointer("semantic-memory", bundle_id, &child.digest));
                snapshot.owner_binding_digests.forge = Some(child.digest.clone());
                snapshot.forge_run_id = Some(run_id);
            }
        }
        "owner:procedure-lifecycle-promoted" => {
            if let (Some(receipt_id), Some(receipt_digest)) = (
                child.receipt.pointer("/receipt_id").and_then(Value::as_str),
                child
                    .receipt
                    .pointer("/receipt_digest")
                    .and_then(Value::as_str),
            ) {
                snapshot.promoted = Some(pointer("semantic-memory", receipt_id, receipt_digest));
                snapshot.owner_binding_digests.lifecycle_promoted =
                    Some(receipt_digest.to_string());
                snapshot.promote_run_id = Some(run_id);
            }
        }
        "owner:promoted-procedure-replay" => {
            if let Some(replay_id) = child.receipt.pointer("/replay_id").and_then(Value::as_str) {
                if let Some(snapshot_replay) = replay_snapshot {
                    if snapshot_replay.inputs.replay_id != replay_id {
                        return Err(LearningResumeError::ReplayConflict);
                    }
                } else {
                    snapshot.replay_admitted =
                        Some(pointer("aidens-runner", replay_id, &child.digest));
                    snapshot.owner_binding_digests.replay_admission = Some(child.digest.clone());
                    snapshot.replay_admission_run_id = Some(run_id);
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn apply_adjudication(
    snapshot: &mut LearningOwnerSnapshotV1,
    adjudication: &CandidatePromotionAdjudicationV1,
    run_id: String,
    tested_lifecycle_receipt: &ProcedureLifecycleReceiptV1,
) -> Result<(), LearningResumeError> {
    if tested_lifecycle_receipt.disposition != ProcedureLifecycleDispositionV1::Promoted
        && tested_lifecycle_receipt.disposition != ProcedureLifecycleDispositionV1::Quarantined
        && tested_lifecycle_receipt.disposition != ProcedureLifecycleDispositionV1::RolledBack
        && tested_lifecycle_receipt.disposition != ProcedureLifecycleDispositionV1::Revoked
    {
        return Err(LearningResumeError::OwnerRead(
            "lifecycle stage not eligible for adjudication binding".into(),
        ));
    }

    snapshot.adjudicated = Some(pointer(
        "verification-adjudication",
        &adjudication.adjudication_id,
        adjudication.adjudication_digest.as_str(),
    ));
    snapshot.owner_binding_digests.adjudication =
        Some(adjudication.adjudication_digest.as_str().to_string());
    snapshot.adjudicated_run_id = Some(run_id.clone());

    if adjudication.decision
        == verification_adjudication::AdjudicationDecisionV1::EligibleForLifecycleConsideration
    {
        snapshot.eligible_for_lifecycle = Some(pointer(
            "verification-adjudication",
            &adjudication.adjudication_id,
            adjudication.adjudication_digest.as_str(),
        ));
        snapshot.owner_binding_digests.lifecycle_eligibility =
            Some(adjudication.adjudication_digest.as_str().to_string());
        snapshot.eligibility_run_id = Some(run_id);
    }

    Ok(())
}

fn pointer(owner_crate: &str, receipt_id: &str, receipt_digest: &str) -> OwnerReceiptPointerV1 {
    OwnerReceiptPointerV1 {
        owner_crate: owner_crate.to_string(),
        artifact_kind: ArtifactKindV1::Run,
        receipt_id: receipt_id.to_string(),
        receipt_digest: receipt_digest.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rebuild_learning_owner_snapshot_rejects_empty_locator_fields() {
        let locator = LearningOwnerLocatorV1 {
            memory_store_root: std::env::temp_dir(),
            run_bundle_store_root: std::env::temp_dir(),
            forge_store_root: std::env::temp_dir(),
            run_id: String::new(),
            candidate_artifact_id: String::new(),
            tested_receipt_id: String::new(),
            bundle_run_id: None,
            adjudication_id: None,
            replay_id: None,
        };

        let result = rebuild_learning_owner_snapshot(&locator).await;
        assert!(matches!(
            result,
            Err(LearningResumeError::InvalidLocator(_))
        ));
    }
}
