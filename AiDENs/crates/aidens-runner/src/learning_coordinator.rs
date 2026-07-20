//! Pure learning coordinator reducer and checkpoint projection persistence.

use aidens_contracts::{
    generated_artifact_id_from_material, LearningCoordinatorDispositionV1,
    LearningCoordinatorProjectionV1, LearningCoordinatorStageV1, NextLearningActionV1,
    OwnerBindingDigestsV1, OwnerReceiptPointerV1,
};
use aidens_receipts::{CanonicalEventLog, CanonicalEventLogEntry, CanonicalEventLogError};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningOwnerSnapshotV1 {
    pub run_id: String,
    pub preflight: Option<OwnerReceiptPointerV1>,
    pub preflight_run_id: Option<String>,
    pub executed_verified: Option<OwnerReceiptPointerV1>,
    pub executed_run_id: Option<String>,
    pub candidate_tested: Option<OwnerReceiptPointerV1>,
    pub candidate_tested_run_id: Option<String>,
    pub effectful_evidence: Option<OwnerReceiptPointerV1>,
    pub effectful_run_id: Option<String>,
    pub forge_published: Option<OwnerReceiptPointerV1>,
    pub forge_run_id: Option<String>,
    pub adjudicated: Option<OwnerReceiptPointerV1>,
    pub adjudicated_run_id: Option<String>,
    pub eligible_for_lifecycle: Option<OwnerReceiptPointerV1>,
    pub eligibility_run_id: Option<String>,
    pub quarantined: Option<OwnerReceiptPointerV1>,
    pub quarantine_run_id: Option<String>,
    pub promoted: Option<OwnerReceiptPointerV1>,
    pub promote_run_id: Option<String>,
    pub replay_admitted: Option<OwnerReceiptPointerV1>,
    pub replay_admission_run_id: Option<String>,
    pub replayed: Option<OwnerReceiptPointerV1>,
    pub replay_run_id: Option<String>,
    pub rolled_back: Option<OwnerReceiptPointerV1>,
    pub rollback_run_id: Option<String>,
    pub revoked: Option<OwnerReceiptPointerV1>,
    pub revoke_run_id: Option<String>,
    pub terminal_published: Option<OwnerReceiptPointerV1>,
    pub terminal_run_id: Option<String>,
    pub failed: bool,
    pub failed_reason_codes: Vec<String>,
    pub owner_binding_digests: OwnerBindingDigestsV1,
}

impl LearningOwnerSnapshotV1 {
    pub fn pointer_count(&self) -> usize {
        self.owner_receipts().len()
    }

    pub fn owner_receipts(&self) -> Vec<&OwnerReceiptPointerV1> {
        [
            self.preflight.as_ref(),
            self.executed_verified.as_ref(),
            self.candidate_tested.as_ref(),
            self.effectful_evidence.as_ref(),
            self.forge_published.as_ref(),
            self.adjudicated.as_ref(),
            self.eligible_for_lifecycle.as_ref(),
            self.quarantined.as_ref(),
            self.promoted.as_ref(),
            self.replay_admitted.as_ref(),
            self.replayed.as_ref(),
            self.rolled_back.as_ref(),
            self.revoked.as_ref(),
            self.terminal_published.as_ref(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    pub fn owner_receipts_sorted(&self) -> Vec<&OwnerReceiptPointerV1> {
        let mut pointers = self.owner_receipts();
        pointers.sort_by(|left, right| {
            let left_artifact_kind = format!("{:?}", left.artifact_kind);
            let right_artifact_kind = format!("{:?}", right.artifact_kind);
            left.owner_crate
                .cmp(&right.owner_crate)
                .then_with(|| left_artifact_kind.cmp(&right_artifact_kind))
                .then_with(|| left.receipt_id.cmp(&right.receipt_id))
                .then_with(|| left.receipt_digest.cmp(&right.receipt_digest))
        });
        pointers
    }

    pub fn to_projection_pointer_material(&self) -> Value {
        serde_json::json!({
            "run_id": self.run_id,
            "pointer_count": self.pointer_count(),
            "owner_receipts": self
                .owner_receipts_sorted()
                .into_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<Value>, _>>()
                .unwrap_or_default(),
            "failed": self.failed,
            "failed_reason_codes": self.failed_reason_codes,
            "owner_binding_digests": self.owner_binding_digests,
        })
    }

    pub fn to_projection_material(&self) -> String {
        serde_json::to_string(&self.to_projection_pointer_material())
            .unwrap_or_else(|_| "{}".to_string())
    }

    pub fn run_ids(&self) -> Vec<(String, String)> {
        let mut ids = Vec::new();
        let push_if_present =
            |ids: &mut Vec<(String, String)>, stage: &str, run_id: &Option<String>| {
                if let Some(run_id) = run_id.as_deref() {
                    ids.push((stage.to_string(), run_id.to_string()));
                }
            };
        push_if_present(&mut ids, "preflight", &self.preflight_run_id);
        push_if_present(&mut ids, "executed_verified", &self.executed_run_id);
        push_if_present(&mut ids, "candidate_tested", &self.candidate_tested_run_id);
        push_if_present(&mut ids, "effectful_evidence", &self.effectful_run_id);
        push_if_present(&mut ids, "forge_published", &self.forge_run_id);
        push_if_present(&mut ids, "adjudicated", &self.adjudicated_run_id);
        push_if_present(&mut ids, "eligible_for_lifecycle", &self.eligibility_run_id);
        push_if_present(&mut ids, "quarantined", &self.quarantine_run_id);
        push_if_present(&mut ids, "promoted", &self.promote_run_id);
        push_if_present(&mut ids, "replay_admitted", &self.replay_admission_run_id);
        push_if_present(&mut ids, "replayed", &self.replay_run_id);
        push_if_present(&mut ids, "rolled_back", &self.rollback_run_id);
        push_if_present(&mut ids, "revoked", &self.revoke_run_id);
        push_if_present(&mut ids, "terminal_published", &self.terminal_run_id);
        ids
    }

    pub fn mixed_run_ids(&self) -> bool {
        let run_ids = self.run_ids();
        let first_run_id = run_ids.iter().map(|(_, run_id)| run_id.as_str()).next();
        let Some(expected) = first_run_id else {
            return false;
        };
        run_ids
            .iter()
            .any(|(_, run_id)| run_id.as_str() != expected)
    }

    pub fn binding_digests_match(&self) -> bool {
        let mut ok = true;
        ok &= pointer_digest_matches(
            &self.preflight,
            self.owner_binding_digests.preflight.as_deref(),
        );
        ok &= pointer_digest_matches(
            &self.executed_verified,
            self.owner_binding_digests.executed.as_deref(),
        );
        ok &= pointer_digest_matches(
            &self.candidate_tested,
            self.owner_binding_digests.candidate_test.as_deref(),
        );
        ok &= pointer_digest_matches(
            &self.effectful_evidence,
            self.owner_binding_digests.effectful_evidence.as_deref(),
        );
        ok &= pointer_digest_matches(
            &self.forge_published,
            self.owner_binding_digests.forge.as_deref(),
        );
        ok &= pointer_digest_matches(
            &self.adjudicated,
            self.owner_binding_digests.adjudication.as_deref(),
        );
        ok &= pointer_digest_matches(
            &self.eligible_for_lifecycle,
            self.owner_binding_digests.lifecycle_eligibility.as_deref(),
        );
        ok &= pointer_digest_matches(
            &self.promoted,
            self.owner_binding_digests.lifecycle_promoted.as_deref(),
        );
        ok &= pointer_digest_matches(
            &self.replay_admitted,
            self.owner_binding_digests.replay_admission.as_deref(),
        );
        ok &= pointer_digest_matches(&self.replayed, self.owner_binding_digests.replay.as_deref());
        ok &= pointer_digest_matches(
            &self.rolled_back,
            self.owner_binding_digests.rollback.as_deref(),
        );
        ok &= pointer_digest_matches(&self.revoked, self.owner_binding_digests.revoke.as_deref());
        ok &= pointer_digest_matches(
            &self.terminal_published,
            self.owner_binding_digests.terminal.as_deref(),
        );
        ok
    }

    pub fn stage_diagnostics(&self) -> (Vec<String>, Vec<String>) {
        let mut missing = Vec::new();
        let mut blocked = Vec::new();

        if self.preflight.is_none() {
            missing.push("missing-preflight".into());
        } else {
            if self.executed_verified.is_none() && self.candidate_tested.is_some() {
                blocked.push("candidate-tested-without-executed".into());
            }
        }

        if self.executed_verified.is_none() && self.candidate_tested.is_some() {
            blocked.push("candidate-tested-without-executed".into());
        }
        if self.executed_verified.is_none() && self.effectful_evidence.is_some() {
            blocked.push("effectful-without-executed".into());
        }
        if self.candidate_tested.is_none() && self.effectful_evidence.is_some() {
            blocked.push("effectful-without-candidate".into());
        }
        if self.effectful_evidence.is_none() && self.forge_published.is_some() {
            blocked.push("forge-without-effectful".into());
        }
        if self.forge_published.is_none() && self.adjudicated.is_some() {
            blocked.push("adjudicated-without-forge".into());
        }
        if self.adjudicated.is_none() && self.eligible_for_lifecycle.is_some() {
            blocked.push("eligibility-without-adjudication".into());
        }
        if self.adjudicated.is_none() && self.promoted.is_some() {
            blocked.push("promoted-without-adjudication".into());
        }
        if self.promoted.is_none() && self.replay_admitted.is_some() {
            blocked.push("replay-admitted-without-promotion".into());
        }
        if self.replay_admitted.is_none() && self.replayed.is_some() {
            blocked.push("replayed-without-admission".into());
        }
        if self.replayed.is_none() && (self.revoked.is_some() || self.rolled_back.is_some()) {
            blocked.push("lifecycle-end-without-replay".into());
        }
        if self.replayed.is_none() && self.terminal_published.is_some() {
            blocked.push("terminal-without-replay".into());
        }

        (missing, blocked)
    }
}

fn pointer_digest_matches(pointer: &Option<OwnerReceiptPointerV1>, binding: Option<&str>) -> bool {
    match (pointer, binding) {
        (Some(pointer), Some(expected)) => pointer.receipt_digest == expected,
        (None, _) => true,
        (_, None) => true,
    }
}

pub fn reduce_learning_coordinator_projection(
    snapshot: &LearningOwnerSnapshotV1,
) -> LearningCoordinatorProjectionV1 {
    let mut reason_codes = Vec::new();

    if snapshot.run_id.trim().is_empty() {
        reason_codes.push("snapshot-missing-run-id".into());
    }
    if snapshot.failed {
        reason_codes.push("snapshot-declared-failed".into());
    }

    if snapshot.mixed_run_ids() {
        reason_codes.push("snapshot-has-mixed-run-ids".into());
    }

    if !snapshot.binding_digests_match() {
        reason_codes.push("snapshot-stale-binding-digest".into());
    }

    let (missing, blocked) = snapshot.stage_diagnostics();
    reason_codes.extend(missing.iter().cloned());
    reason_codes.extend(blocked.iter().cloned());

    let stage = if snapshot.terminal_published.is_some() {
        LearningCoordinatorStageV1::TerminalPublished
    } else if snapshot.replayed.is_some() {
        if snapshot.rolled_back.is_some() {
            LearningCoordinatorStageV1::RolledBack
        } else if snapshot.revoked.is_some() {
            LearningCoordinatorStageV1::Revoked
        } else {
            LearningCoordinatorStageV1::Replayed
        }
    } else if snapshot.replay_admitted.is_some() {
        LearningCoordinatorStageV1::ReplayAdmitted
    } else if snapshot.promoted.is_some() {
        LearningCoordinatorStageV1::Promoted
    } else if snapshot.quarantined.is_some() {
        LearningCoordinatorStageV1::Quarantined
    } else if snapshot.eligible_for_lifecycle.is_some() {
        LearningCoordinatorStageV1::EligibleForLifecycleConsideration
    } else if snapshot.adjudicated.is_some() {
        LearningCoordinatorStageV1::Adjudicated
    } else if snapshot.forge_published.is_some() {
        LearningCoordinatorStageV1::ForgePublished
    } else if snapshot.effectful_evidence.is_some() {
        LearningCoordinatorStageV1::EffectfulEvidencePersisted
    } else if snapshot.candidate_tested.is_some() {
        LearningCoordinatorStageV1::CandidateTested
    } else if snapshot.executed_verified.is_some() {
        LearningCoordinatorStageV1::ExecutedVerified
    } else {
        LearningCoordinatorStageV1::Preflighted
    };

    let next_action = match stage {
        LearningCoordinatorStageV1::Preflighted => {
            if snapshot.preflight.is_none() {
                NextLearningActionV1::AwaitPreflight
            } else {
                NextLearningActionV1::ExecuteAndVerify
            }
        }
        LearningCoordinatorStageV1::ExecutedVerified => NextLearningActionV1::TestCandidate,
        LearningCoordinatorStageV1::CandidateTested => {
            NextLearningActionV1::PersistEffectfulEvidence
        }
        LearningCoordinatorStageV1::EffectfulEvidencePersisted => {
            NextLearningActionV1::PublishForgeEvidence
        }
        LearningCoordinatorStageV1::ForgePublished => NextLearningActionV1::SeekAdjudication,
        LearningCoordinatorStageV1::Adjudicated => NextLearningActionV1::AwaitLifecycleDecision,
        LearningCoordinatorStageV1::EligibleForLifecycleConsideration => {
            if snapshot.quarantined.is_some() {
                NextLearningActionV1::WaitForManualResolution
            } else {
                NextLearningActionV1::PromoteProcedure
            }
        }
        LearningCoordinatorStageV1::Quarantined => NextLearningActionV1::WaitForManualResolution,
        LearningCoordinatorStageV1::Promoted => NextLearningActionV1::AdmitForReplay,
        LearningCoordinatorStageV1::ReplayAdmitted => NextLearningActionV1::ReplayProcedure,
        LearningCoordinatorStageV1::Replayed => NextLearningActionV1::PublishTerminalEvidence,
        LearningCoordinatorStageV1::RolledBack => NextLearningActionV1::Completed,
        LearningCoordinatorStageV1::Revoked => NextLearningActionV1::Completed,
        LearningCoordinatorStageV1::TerminalPublished => NextLearningActionV1::Completed,
    };

    let disposition = if snapshot.failed {
        LearningCoordinatorDispositionV1::Failed
    } else if reason_codes
        .iter()
        .any(|reason| reason.starts_with("snapshot-stale-binding"))
    {
        LearningCoordinatorDispositionV1::Indeterminate
    } else if snapshot.mixed_run_ids() || !blocked.is_empty() || !missing.is_empty() {
        LearningCoordinatorDispositionV1::Blocked
    } else {
        LearningCoordinatorDispositionV1::Pending
    };

    if reason_codes.is_empty() {
        reason_codes.push("ok".into());
    }

    LearningCoordinatorProjectionV1 {
        stage,
        disposition,
        owner_receipts: snapshot
            .owner_receipts_sorted()
            .into_iter()
            .cloned()
            .collect(),
        owner_binding_digests: snapshot.owner_binding_digests.clone(),
        next_action,
        reason_codes,
    }
}

pub fn learning_coordinator_checkpoint_id(
    snapshot: &LearningOwnerSnapshotV1,
) -> aidens_contracts::ArtifactId {
    generated_artifact_id_from_material(
        "learning-coordinator-checkpoint",
        &snapshot.to_projection_material(),
    )
}

pub fn project_or_rebuild_learning_coordinator_projection(
    log: &CanonicalEventLog,
    snapshot: &LearningOwnerSnapshotV1,
) -> Result<LearningCoordinatorProjectionV1, CanonicalEventLogError> {
    let checkpoint_id = learning_coordinator_checkpoint_id(snapshot).to_string();
    match log.inspect(&checkpoint_id) {
        Ok(record) => {
            let projection: LearningCoordinatorProjectionV1 = serde_json::from_value(record.body)
                .map_err(|error| {
                CanonicalEventLogError::Json {
                    path: log.config().records_path.clone(),
                    source: error,
                }
            })?;
            Ok(projection)
        }
        Err(CanonicalEventLogError::NotFound(_)) => {
            Ok(reduce_learning_coordinator_projection(snapshot))
        }
        Err(error) => Err(error),
    }
}

pub fn append_learning_coordinator_checkpoint(
    log: &CanonicalEventLog,
    snapshot: &LearningOwnerSnapshotV1,
) -> Result<CanonicalEventLogEntry, CanonicalEventLogError> {
    let checkpoint_id = learning_coordinator_checkpoint_id(snapshot).to_string();
    let projection = reduce_learning_coordinator_projection(snapshot);
    projection
        .validate()
        .map_err(|error| CanonicalEventLogError::Json {
            path: log.config().records_path.clone(),
            source: serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                error,
            )),
        })?;
    let body = serde_json::to_value(&projection).map_err(|error| CanonicalEventLogError::Json {
        path: log.config().records_path.clone(),
        source: error,
    })?;

    match log.inspect(&checkpoint_id) {
        Ok(existing) => {
            if existing.body == body {
                return Ok(existing);
            }
            Err(CanonicalEventLogError::DuplicateReceiptId(checkpoint_id))
        }
        Err(CanonicalEventLogError::NotFound(_)) => log.append_orchestration_report(
            "learning-coordinator-projection-v1",
            checkpoint_id,
            body,
        ),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidens_contracts::ArtifactKindV1;
    use aidens_receipts::CanonicalEventLogConfig;

    fn pointer(
        owner_crate: &str,
        kind: ArtifactKindV1,
        digest: &str,
        id: &str,
    ) -> OwnerReceiptPointerV1 {
        OwnerReceiptPointerV1 {
            owner_crate: owner_crate.into(),
            artifact_kind: kind,
            receipt_id: id.into(),
            receipt_digest: digest.into(),
        }
    }

    fn snapshot_base(run_id: &str) -> LearningOwnerSnapshotV1 {
        LearningOwnerSnapshotV1 {
            run_id: run_id.into(),
            preflight: None,
            preflight_run_id: Some(run_id.into()),
            executed_verified: None,
            executed_run_id: Some(run_id.into()),
            candidate_tested: None,
            candidate_tested_run_id: Some(run_id.into()),
            effectful_evidence: None,
            effectful_run_id: Some(run_id.into()),
            forge_published: None,
            forge_run_id: Some(run_id.into()),
            adjudicated: None,
            adjudicated_run_id: Some(run_id.into()),
            eligible_for_lifecycle: None,
            eligibility_run_id: Some(run_id.into()),
            quarantined: None,
            quarantine_run_id: Some(run_id.into()),
            promoted: None,
            promote_run_id: Some(run_id.into()),
            replay_admitted: None,
            replay_admission_run_id: Some(run_id.into()),
            replayed: None,
            replay_run_id: Some(run_id.into()),
            rolled_back: None,
            rollback_run_id: Some(run_id.into()),
            revoked: None,
            revoke_run_id: Some(run_id.into()),
            terminal_published: None,
            terminal_run_id: Some(run_id.into()),
            failed: false,
            failed_reason_codes: Vec::new(),
            owner_binding_digests: OwnerBindingDigestsV1::default(),
        }
    }

    fn snapshot_with_stage(
        stage: LearningCoordinatorStageV1,
        run_id: &str,
    ) -> LearningOwnerSnapshotV1 {
        match stage {
            LearningCoordinatorStageV1::Preflighted => {
                let mut snapshot = snapshot_base(run_id);
                snapshot.preflight = Some(pointer(
                    "semantic-memory",
                    ArtifactKindV1::Run,
                    "preflight-digest",
                    "preflight",
                ));
                snapshot.owner_binding_digests.preflight = Some("preflight-digest".into());
                snapshot
            }
            LearningCoordinatorStageV1::ExecutedVerified => {
                let mut snapshot =
                    snapshot_with_stage(LearningCoordinatorStageV1::Preflighted, run_id);
                snapshot.executed_verified = Some(pointer(
                    "semantic-memory",
                    ArtifactKindV1::Run,
                    "executed-digest",
                    "executed",
                ));
                snapshot.owner_binding_digests.executed = Some("executed-digest".into());
                snapshot
            }
            LearningCoordinatorStageV1::CandidateTested => {
                let mut snapshot =
                    snapshot_with_stage(LearningCoordinatorStageV1::ExecutedVerified, run_id);
                snapshot.candidate_tested = Some(pointer(
                    "semantic-memory",
                    ArtifactKindV1::Run,
                    "candidate-digest",
                    "candidate",
                ));
                snapshot.owner_binding_digests.candidate_test = Some("candidate-digest".into());
                snapshot
            }
            LearningCoordinatorStageV1::EffectfulEvidencePersisted => {
                let mut snapshot =
                    snapshot_with_stage(LearningCoordinatorStageV1::CandidateTested, run_id);
                snapshot.effectful_evidence = Some(pointer(
                    "semantic-memory",
                    ArtifactKindV1::Run,
                    "effectful-digest",
                    "effectful",
                ));
                snapshot.owner_binding_digests.effectful_evidence = Some("effectful-digest".into());
                snapshot
            }
            LearningCoordinatorStageV1::ForgePublished => {
                let mut snapshot = snapshot_with_stage(
                    LearningCoordinatorStageV1::EffectfulEvidencePersisted,
                    run_id,
                );
                snapshot.forge_published = Some(pointer(
                    "semantic-memory",
                    ArtifactKindV1::Run,
                    "forge-digest",
                    "forge",
                ));
                snapshot.owner_binding_digests.forge = Some("forge-digest".into());
                snapshot
            }
            LearningCoordinatorStageV1::Adjudicated => {
                let mut snapshot =
                    snapshot_with_stage(LearningCoordinatorStageV1::ForgePublished, run_id);
                snapshot.adjudicated = Some(pointer(
                    "semantic-memory",
                    ArtifactKindV1::Run,
                    "adjudication-digest",
                    "adjudication",
                ));
                snapshot.owner_binding_digests.adjudication = Some("adjudication-digest".into());
                snapshot
            }
            LearningCoordinatorStageV1::EligibleForLifecycleConsideration => {
                let mut snapshot =
                    snapshot_with_stage(LearningCoordinatorStageV1::Adjudicated, run_id);
                snapshot.eligible_for_lifecycle = Some(pointer(
                    "semantic-memory",
                    ArtifactKindV1::Run,
                    "eligibility-digest",
                    "eligibility",
                ));
                snapshot.owner_binding_digests.lifecycle_eligibility =
                    Some("eligibility-digest".into());
                snapshot
            }
            LearningCoordinatorStageV1::Quarantined => {
                let mut snapshot =
                    snapshot_with_stage(LearningCoordinatorStageV1::Adjudicated, run_id);
                snapshot.quarantined = Some(pointer(
                    "semantic-memory",
                    ArtifactKindV1::Run,
                    "quarantine-digest",
                    "quarantine",
                ));
                snapshot.owner_binding_digests.lifecycle_eligibility =
                    Some("eligibility-digest".into());
                snapshot.eligible_for_lifecycle = Some(pointer(
                    "semantic-memory",
                    ArtifactKindV1::Run,
                    "eligibility-digest",
                    "eligibility",
                ));
                snapshot
            }
            LearningCoordinatorStageV1::Promoted => {
                let mut snapshot = snapshot_with_stage(
                    LearningCoordinatorStageV1::EligibleForLifecycleConsideration,
                    run_id,
                );
                snapshot.promoted = Some(pointer(
                    "semantic-memory",
                    ArtifactKindV1::Run,
                    "promote-digest",
                    "promoted",
                ));
                snapshot.owner_binding_digests.lifecycle_promoted = Some("promote-digest".into());
                snapshot
            }
            LearningCoordinatorStageV1::ReplayAdmitted => {
                let mut snapshot =
                    snapshot_with_stage(LearningCoordinatorStageV1::Promoted, run_id);
                snapshot.replay_admitted = Some(pointer(
                    "semantic-memory",
                    ArtifactKindV1::Run,
                    "replay-admission-digest",
                    "replay-admitted",
                ));
                snapshot.owner_binding_digests.replay_admission =
                    Some("replay-admission-digest".into());
                snapshot
            }
            LearningCoordinatorStageV1::Replayed => {
                let mut snapshot =
                    snapshot_with_stage(LearningCoordinatorStageV1::ReplayAdmitted, run_id);
                snapshot.replayed = Some(pointer(
                    "semantic-memory",
                    ArtifactKindV1::Run,
                    "replay-digest",
                    "replayed",
                ));
                snapshot.owner_binding_digests.replay = Some("replay-digest".into());
                snapshot
            }
            LearningCoordinatorStageV1::RolledBack => {
                let mut snapshot =
                    snapshot_with_stage(LearningCoordinatorStageV1::Replayed, run_id);
                snapshot.rolled_back = Some(pointer(
                    "semantic-memory",
                    ArtifactKindV1::Run,
                    "rollback-digest",
                    "rollback",
                ));
                snapshot.owner_binding_digests.rollback = Some("rollback-digest".into());
                snapshot
            }
            LearningCoordinatorStageV1::Revoked => {
                let mut snapshot =
                    snapshot_with_stage(LearningCoordinatorStageV1::Replayed, run_id);
                snapshot.revoked = Some(pointer(
                    "semantic-memory",
                    ArtifactKindV1::Run,
                    "revoke-digest",
                    "revoked",
                ));
                snapshot.owner_binding_digests.revoke = Some("revoke-digest".into());
                snapshot
            }
            LearningCoordinatorStageV1::TerminalPublished => {
                let mut snapshot =
                    snapshot_with_stage(LearningCoordinatorStageV1::Replayed, run_id);
                snapshot.terminal_published = Some(pointer(
                    "semantic-memory",
                    ArtifactKindV1::Run,
                    "terminal-digest",
                    "terminal",
                ));
                snapshot.owner_binding_digests.terminal = Some("terminal-digest".into());
                snapshot
            }
        }
    }

    #[test]
    fn reduce_learning_coordinator_projection_is_transitionally_correct() {
        let preflight = snapshot_with_stage(
            LearningCoordinatorStageV1::Preflighted,
            "run:coordinator:one",
        );
        let executed = snapshot_with_stage(
            LearningCoordinatorStageV1::ExecutedVerified,
            "run:coordinator:one",
        );
        let candidate = snapshot_with_stage(
            LearningCoordinatorStageV1::CandidateTested,
            "run:coordinator:one",
        );
        let terminal = snapshot_with_stage(
            LearningCoordinatorStageV1::TerminalPublished,
            "run:coordinator:one",
        );

        let preflight_projection = reduce_learning_coordinator_projection(&preflight);
        assert_eq!(
            preflight_projection.stage,
            LearningCoordinatorStageV1::Preflighted
        );
        assert_eq!(
            preflight_projection.next_action,
            NextLearningActionV1::ExecuteAndVerify
        );

        let executed_projection = reduce_learning_coordinator_projection(&executed);
        assert_eq!(
            executed_projection.stage,
            LearningCoordinatorStageV1::ExecutedVerified
        );
        assert_eq!(
            executed_projection.next_action,
            NextLearningActionV1::TestCandidate
        );

        let candidate_projection = reduce_learning_coordinator_projection(&candidate);
        assert_eq!(
            candidate_projection.stage,
            LearningCoordinatorStageV1::CandidateTested
        );
        assert_eq!(
            candidate_projection.next_action,
            NextLearningActionV1::PersistEffectfulEvidence
        );

        let terminal_projection = reduce_learning_coordinator_projection(&terminal);
        assert_eq!(
            terminal_projection.stage,
            LearningCoordinatorStageV1::TerminalPublished
        );
        assert_eq!(
            terminal_projection.next_action,
            NextLearningActionV1::Completed
        );
        assert_eq!(
            terminal_projection.disposition,
            LearningCoordinatorDispositionV1::Pending
        );
    }

    #[test]
    fn mixed_run_ids_emit_blocked_projection() {
        let mut snapshot = snapshot_with_stage(
            LearningCoordinatorStageV1::CandidateTested,
            "run:coordinator:one",
        );
        snapshot.executed_run_id = Some("run:coordinator:two".into());

        let projection = reduce_learning_coordinator_projection(&snapshot);
        assert_eq!(
            projection.disposition,
            LearningCoordinatorDispositionV1::Blocked
        );
        assert!(projection
            .reason_codes
            .iter()
            .any(|reason| reason == "snapshot-has-mixed-run-ids"));
    }

    #[test]
    fn stale_binding_digests_force_indeterminate_projection() {
        let mut snapshot = snapshot_with_stage(
            LearningCoordinatorStageV1::CandidateTested,
            "run:coordinator:one",
        );
        snapshot.owner_binding_digests.executed = Some("mutated-executed-digest".into());

        let projection = reduce_learning_coordinator_projection(&snapshot);
        assert_eq!(
            projection.disposition,
            LearningCoordinatorDispositionV1::Indeterminate
        );
        assert!(projection
            .reason_codes
            .iter()
            .any(|reason| reason == "snapshot-stale-binding-digest"));
    }

    #[test]
    fn projection_validation_rejects_invalid_receipt_pointers() {
        let snapshot = snapshot_with_stage(
            LearningCoordinatorStageV1::Preflighted,
            "run:coordinator:one",
        );
        let projection = reduce_learning_coordinator_projection(&snapshot);
        assert!(projection.validate().is_ok());

        let mut bad_projection = projection.clone();
        bad_projection.owner_receipts[0].receipt_id = String::new();
        assert!(bad_projection.validate().is_err());
    }

    #[test]
    fn checkpoint_roundtrip_is_deterministic_and_duplicate_tolerant() {
        let tempdir = tempfile::tempdir().unwrap();
        let log =
            CanonicalEventLog::open(CanonicalEventLogConfig::for_root(tempdir.path())).unwrap();
        let snapshot = snapshot_with_stage(
            LearningCoordinatorStageV1::CandidateTested,
            "run:coordinator:cp",
        );

        let first = append_learning_coordinator_checkpoint(&log, &snapshot).unwrap();
        let second = append_learning_coordinator_checkpoint(&log, &snapshot).unwrap();

        assert_eq!(first.receipt_id, second.receipt_id);

        let reopened = CanonicalEventLog::open(log.config().clone()).unwrap();
        let body_projection =
            project_or_rebuild_learning_coordinator_projection(&reopened, &snapshot).unwrap();
        let fresh_projection = reduce_learning_coordinator_projection(&snapshot);
        assert_eq!(body_projection, fresh_projection);
        assert!(reopened
            .verify_chain()
            .unwrap_or_else(|_| panic!("checkpoint chain must verify")));
    }

    #[test]
    fn checkpoint_deletion_recovery_matches_owner_reconstruction() {
        let tempdir = tempfile::tempdir().unwrap();
        let root = tempdir.path().to_path_buf();
        let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&root)).unwrap();
        let snapshot = snapshot_with_stage(
            LearningCoordinatorStageV1::EligibleForLifecycleConsideration,
            "run:coordinator:deleted",
        );

        let original_projection = reduce_learning_coordinator_projection(&snapshot);
        append_learning_coordinator_checkpoint(&log, &snapshot).unwrap();

        std::fs::remove_file(log.config().records_path.clone()).unwrap();
        let reopened = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&root)).unwrap();
        let recovered_projection =
            project_or_rebuild_learning_coordinator_projection(&reopened, &snapshot).unwrap();
        assert_eq!(recovered_projection, original_projection);
        assert_eq!(
            recovered_projection.owner_receipts.len(),
            original_projection.owner_receipts.len()
        );
    }
}
