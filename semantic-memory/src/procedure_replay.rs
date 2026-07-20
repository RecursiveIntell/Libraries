//! Canonical owner for exact coding-procedure replay retention (V38).
use crate::procedural_memory::{
    verify_procedure_effectful_evaluation_receipt_v1, verify_procedure_lifecycle_receipt_v1,
    ProceduralMemoryArtifactV1, ProcedureEffectfulEvaluationReceiptV1,
    ProcedureLifecycleDispositionV1, ProcedureLifecycleReceiptV1,
};
use crate::{MemoryError, MemoryStore, ProcedureActionPermitV1};
use blake3::Hasher;
use chrono::{DateTime, Utc};
use forge_memory_bridge::{AdjudicationBindingV1, ForgeAdjudicationStore};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use verification_adjudication::{AdjudicationDecisionV1, CandidatePromotionAdjudicationV1};

const SCHEMA: &str = "procedure_replay_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcedureReplayInputsV1 {
    pub replay_id: String,
    pub original_artifact_id: String,
    pub original_artifact_digest: String,
    pub patch_digest: String,
    pub source_tree_digest: String,
    pub verifier_digest: String,
    pub check_policy_digest: String,
    pub environment_digest: String,
    pub image_digest: String,
    pub store_identity_digest: String,
    pub retained_input_digest: String,
    pub promotion_receipt_ref: String,
    pub action_permit_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcedureReplayObservedIdentityV1 {
    pub patch_digest: String,
    pub source_tree_digest: String,
    pub verifier_digest: String,
    pub check_policy_digest: String,
    pub environment_digest: String,
    pub image_digest: String,
    pub store_identity_digest: String,
    pub retained_input_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcedureReplayAdmissionV1 {
    pub schema_version: String,
    pub replay_id: String,
    pub admission_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcedureReplayOutcomeV1 {
    ExactMatch,
    Mismatch,
    Drift,
    Inconclusive,
    NotAvailable,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcedureReplayResultV1 {
    pub replay_id: String,
    pub result_digest: String,
    pub outcome: ProcedureReplayOutcomeV1,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcedureReplayComparisonV1 {
    pub outcome: ProcedureReplayOutcomeV1,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcedureOwnerSnapshotV1 {
    pub artifact: ProceduralMemoryArtifactV1,
    pub lifecycle_receipt: Option<ProcedureLifecycleReceiptV1>,
    pub effectful_receipt: Option<ProcedureEffectfulEvaluationReceiptV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcedureReplaySnapshotV1 {
    pub inputs: ProcedureReplayInputsV1,
    pub admission: ProcedureReplayAdmissionV1,
    pub result: Option<ProcedureReplayResultV1>,
}

fn digest<T: Serialize>(v: &T) -> Result<String, MemoryError> {
    let bytes = serde_json::to_vec(v).map_err(|error| MemoryError::Other(error.to_string()))?;
    let mut h = Hasher::new();
    h.update(&bytes);
    Ok(h.finalize().to_hex().to_string())
}
fn nonempty(inputs: &ProcedureReplayInputsV1) -> Result<(), MemoryError> {
    let fields = [
        &inputs.replay_id,
        &inputs.original_artifact_id,
        &inputs.original_artifact_digest,
        &inputs.patch_digest,
        &inputs.source_tree_digest,
        &inputs.verifier_digest,
        &inputs.check_policy_digest,
        &inputs.environment_digest,
        &inputs.image_digest,
        &inputs.store_identity_digest,
        &inputs.retained_input_digest,
        &inputs.promotion_receipt_ref,
        &inputs.action_permit_ref,
    ];
    if fields.iter().any(|v| v.trim().is_empty()) {
        return Err(MemoryError::ProceduralMemoryRejected {
            reason: "replay identity contains an empty field".into(),
        });
    }
    Ok(())
}
fn permit_ref(p: &ProcedureActionPermitV1, replay_id: &str) -> Result<String, MemoryError> {
    digest(&(SCHEMA, replay_id, p))
}

/// Compute the permit binding required by [`ProcedureReplayInputsV1::action_permit_ref`].
pub fn procedure_replay_permit_ref(
    permit: &ProcedureActionPermitV1,
    replay_id: &str,
) -> Result<String, MemoryError> {
    permit_ref(permit, replay_id)
}

impl MemoryStore {
    pub async fn admit_adjudicated_procedure_replay(
        &self,
        forge: &dyn ForgeAdjudicationStore,
        replay_id: impl Into<String>,
        adjudication_id: &str,
        permit: ProcedureActionPermitV1,
    ) -> Result<ProcedureReplayAdmissionV1, MemoryError> {
        let replay_id = replay_id.into();
        let adjudication: CandidatePromotionAdjudicationV1 = forge
            .read_verified_adjudication(adjudication_id)
            .map_err(|error| rejected(error.to_string()))?;
        adjudication
            .validate()
            .map_err(|error| rejected(error.to_string()))?;
        if adjudication.decision != AdjudicationDecisionV1::EligibleForLifecycleConsideration {
            return Err(rejected("adjudication is not lifecycle-eligible"));
        }
        forge
            .verify_adjudication_binding(
                adjudication_id,
                &AdjudicationBindingV1 {
                    candidate_id: adjudication.candidate_id.clone(),
                    candidate_digest: adjudication.candidate_digest.clone(),
                    evidence_bundle_id: adjudication.evidence_bundle_id.clone(),
                    evidence_bundle_digest: adjudication.evidence_bundle_digest.clone(),
                },
            )
            .map_err(|error| rejected(error.to_string()))?;
        let snapshot = self
            .load_procedure_owner_snapshot(&adjudication.candidate_id)
            .await?;
        if snapshot.artifact.artifact_id != adjudication.candidate_id {
            return Err(rejected(
                "adjudication candidate id does not match owner snapshot",
            ));
        }
        if strip_blake3_prefix(&snapshot.artifact.artifact_digest)
            != adjudication.candidate_digest.as_str()
        {
            return Err(rejected(
                "adjudication candidate digest does not match owner artifact",
            ));
        }
        let lifecycle_receipt = snapshot
            .lifecycle_receipt
            .as_ref()
            .ok_or_else(|| rejected("owner candidate has no lifecycle receipt"))?;
        let action_permit_ref = permit_ref(&permit, &replay_id)?;
        let store_identity_digest = derive_store_identity(
            &snapshot.artifact,
            snapshot.lifecycle_receipt.as_ref(),
            snapshot.effectful_receipt.as_ref(),
        )?;
        let retained_input_digest = digest(&(
            &replay_id,
            &snapshot.artifact.artifact_id,
            &snapshot.artifact.artifact_digest,
            adjudication.patch_digest.as_str(),
            adjudication.source_tree_digest.as_str(),
            adjudication.verifier_digest.as_str(),
            adjudication.check_policy_digest.as_str(),
            adjudication.environment_digest.as_str(),
            adjudication.image_digest.as_str(),
            &store_identity_digest,
        ))?;
        let inputs = ProcedureReplayInputsV1 {
            replay_id,
            original_artifact_id: snapshot.artifact.artifact_id.clone(),
            original_artifact_digest: snapshot.artifact.artifact_digest.clone(),
            patch_digest: adjudication.patch_digest.as_str().to_owned(),
            source_tree_digest: adjudication.source_tree_digest.as_str().to_owned(),
            verifier_digest: adjudication.verifier_digest.as_str().to_owned(),
            check_policy_digest: adjudication.check_policy_digest.as_str().to_owned(),
            environment_digest: adjudication.environment_digest.as_str().to_owned(),
            image_digest: adjudication.image_digest.as_str().to_owned(),
            store_identity_digest,
            retained_input_digest,
            promotion_receipt_ref: lifecycle_receipt.receipt_id.clone(),
            action_permit_ref,
        };
        self.admit_procedure_replay(inputs, permit).await
    }

    pub async fn load_procedure_owner_snapshot(
        &self,
        artifact_id: impl Into<String>,
    ) -> Result<ProcedureOwnerSnapshotV1, MemoryError> {
        let artifact_id = artifact_id.into();
        self.with_read_conn(move |conn| {
            let artifact_json: String = conn
                .query_row(
                    "SELECT artifact_json FROM procedural_memory_artifacts WHERE artifact_id=?1",
                    params![artifact_id],
                    |row| row.get(0),
                )
                .optional()?
                .ok_or_else(|| MemoryError::ProceduralMemoryNotFound {
                    artifact_id: artifact_id.clone(),
                })?;
            let artifact: ProceduralMemoryArtifactV1 =
                serde_json::from_str(&artifact_json).map_err(|error| MemoryError::CorruptData {
                    table: "procedural_memory_artifacts",
                    row_id: artifact_id.clone(),
                    detail: error.to_string(),
                })?;
            if artifact.artifact_id != artifact_id || artifact.artifact_digest != artifact.compute_digest() {
                return Err(MemoryError::CorruptData {
                    table: "procedural_memory_artifacts",
                    row_id: artifact_id.clone(),
                    detail: "artifact identity mismatch".into(),
                });
            }
            let lifecycle_receipt = conn
                .query_row(
                    "SELECT receipt_json FROM procedural_memory_receipts WHERE artifact_id = ?1 ORDER BY rowid DESC LIMIT 1",
                    params![artifact.artifact_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?
                .map(|raw| {
                    serde_json::from_str::<ProcedureLifecycleReceiptV1>(&raw).map_err(|error| {
                        MemoryError::CorruptData {
                            table: "procedural_memory_receipts",
                            row_id: artifact.artifact_id.clone(),
                            detail: error.to_string(),
                        }
                    })
                })
                .transpose()?
                .map(|receipt| {
                    if !verify_procedure_lifecycle_receipt_v1(&receipt)
                        || receipt.artifact_id != artifact.artifact_id
                        || receipt.artifact_digest != artifact.artifact_digest
                    {
                        Err(MemoryError::CorruptData {
                            table: "procedural_memory_receipts",
                            row_id: artifact.artifact_id.clone(),
                            detail:
                                "stored lifecycle receipt failed verification against owner".into(),
                        })
                    } else {
                        Ok(receipt)
                    }
                })
                .transpose()?;
            let effectful_receipt = conn
                .query_row(
                    "SELECT receipt_json FROM procedural_effectful_evaluations WHERE artifact_id = ?1 AND artifact_digest = ?2 ORDER BY rowid DESC LIMIT 1",
                    params![artifact.artifact_id, artifact.artifact_digest],
                    |row| row.get::<_, String>(0),
                )
                .optional()?
                .map(|raw| {
                    serde_json::from_str::<ProcedureEffectfulEvaluationReceiptV1>(&raw).map_err(
                        |error| MemoryError::CorruptData {
                            table: "procedural_effectful_evaluations",
                            row_id: artifact.artifact_id.clone(),
                            detail: error.to_string(),
                        },
                    )
                })
                .transpose()?
                .map(|receipt| {
                    if !verify_procedure_effectful_evaluation_receipt_v1(&receipt) {
                        Err(MemoryError::CorruptData {
                            table: "procedural_effectful_evaluations",
                            row_id: artifact.artifact_id.clone(),
                            detail:
                                "stored effectful evaluation receipt failed verification".into(),
                        })
                    } else {
                        Ok(receipt)
                    }
                })
                .transpose()?;
            Ok(ProcedureOwnerSnapshotV1 {
                artifact,
                lifecycle_receipt,
                effectful_receipt,
            })
        })
        .await
    }

    pub async fn load_procedure_replay_snapshot(
        &self,
        replay_id: impl Into<String>,
    ) -> Result<ProcedureReplaySnapshotV1, MemoryError> {
        let replay_id = replay_id.into();
        self.with_read_conn(move |conn| {
            let parsed = conn
                .query_row(
                    "SELECT payload_json FROM procedure_replay_inputs WHERE replay_id=?1",
                    params![replay_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?
                .ok_or_else(|| MemoryError::ProceduralMemoryNotFound {
                    artifact_id: replay_id.clone(),
                })?;
            let inputs: ProcedureReplayInputsV1 =
                serde_json::from_str(&parsed).map_err(|error| MemoryError::CorruptData {
                    table: "procedure_replay_inputs",
                    row_id: replay_id.clone(),
                    detail: error.to_string(),
                })?;
            let request_digest: String = conn
                .query_row(
                    "SELECT request_digest FROM procedure_replay_inputs WHERE replay_id=?1",
                    params![inputs.replay_id],
                    |row| row.get(0),
                )?;
            let expected = digest(&inputs)?;
            if expected != request_digest {
                return Err(MemoryError::CorruptData {
                    table: "procedure_replay_inputs",
                    row_id: inputs.replay_id.clone(),
                    detail: "request digest mismatch".into(),
                });
            }
            let (admission_digest, admission_raw) = conn
                .query_row(
                    "SELECT admission_digest, receipt_json FROM procedure_replay_admissions WHERE replay_id=?1",
                    params![inputs.replay_id],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )
                .optional()?
                .ok_or_else(|| MemoryError::ProceduralMemoryNotFound {
                    artifact_id: inputs.replay_id.clone(),
                })?;
            let admission: ProcedureReplayAdmissionV1 =
                serde_json::from_str(&admission_raw).map_err(|error| MemoryError::CorruptData {
                    table: "procedure_replay_admissions",
                    row_id: inputs.replay_id.clone(),
                    detail: error.to_string(),
                })?;
            let expected_admission = digest(&(SCHEMA, &inputs.replay_id, &request_digest))?;
            if admission.admission_digest != admission_digest
                || admission.admission_digest != expected_admission
            {
                return Err(MemoryError::CorruptData {
                    table: "procedure_replay_admissions",
                    row_id: inputs.replay_id.clone(),
                    detail: "admission digest mismatch".into(),
                });
            }
            let result: Option<ProcedureReplayResultV1> = conn
                .query_row(
                    "SELECT receipt_json FROM procedure_replay_results WHERE replay_id=?1",
                    params![inputs.replay_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?
                .map(|raw| {
                    serde_json::from_str::<ProcedureReplayResultV1>(&raw).map_err(|error| {
                        MemoryError::CorruptData {
                            table: "procedure_replay_results",
                            row_id: inputs.replay_id.clone(),
                            detail: error.to_string(),
                        }
                    })
                })
                .transpose()?;
            Ok(ProcedureReplaySnapshotV1 {
                inputs,
                admission,
                result,
            })
        })
        .await
    }

    pub async fn admit_procedure_replay(
        &self,
        mut inputs: ProcedureReplayInputsV1,
        permit: ProcedureActionPermitV1,
    ) -> Result<ProcedureReplayAdmissionV1, MemoryError> {
        nonempty(&inputs)?;
        if inputs.action_permit_ref != permit_ref(&permit, &inputs.replay_id)?
            || permit.capability != ProcedureActionPermitV1::CAPABILITY
            || !permit.scope.is_bound()
        {
            return Err(MemoryError::ProceduralMemoryUnauthorized {
                principal: permit.principal.clone(),
            });
        }
        let expiry = DateTime::parse_from_rfc3339(&permit.expires_at).map_err(|_| {
            MemoryError::ProceduralMemoryUnauthorized {
                principal: permit.principal.clone(),
            }
        })?;
        if expiry.with_timezone(&Utc) <= Utc::now() {
            return Err(MemoryError::ProceduralMemoryUnauthorized {
                principal: permit.principal.clone(),
            });
        }
        inputs.retained_input_digest = digest(&(
            &inputs.replay_id,
            &inputs.original_artifact_id,
            &inputs.original_artifact_digest,
            &inputs.patch_digest,
            &inputs.source_tree_digest,
            &inputs.verifier_digest,
            &inputs.check_policy_digest,
            &inputs.environment_digest,
            &inputs.image_digest,
            &inputs.store_identity_digest,
        ))?;
        let request_digest = digest(&inputs)?;
        let json = serde_json::to_string(&inputs).map_err(|e| MemoryError::Other(e.to_string()))?;
        let replay_id = inputs.replay_id.clone();
        let permit_id = inputs.action_permit_ref.clone();
        self.with_write_conn(move |conn| {
            crate::db::with_transaction(conn, |tx| {
                let artifact_json: String = tx.query_row("SELECT artifact_json FROM procedural_memory_artifacts WHERE artifact_id=?1", params![inputs.original_artifact_id], |r| r.get(0))
                    .map_err(|_| MemoryError::ProceduralMemoryNotFound { artifact_id: inputs.original_artifact_id.clone() })?;
                let artifact: crate::ProceduralMemoryArtifactV1 = serde_json::from_str(&artifact_json)
                    .map_err(|e| MemoryError::CorruptData { table: "procedural_memory_artifacts", row_id: inputs.original_artifact_id.clone(), detail: e.to_string() })?;
                if artifact.artifact_digest != inputs.original_artifact_digest || artifact.scope != permit.scope {
                    return Err(MemoryError::ProceduralMemoryRejected { reason: "candidate identity or permit scope drift".into() });
                }
                let state: Option<String> = tx.query_row("SELECT disposition FROM procedural_memory_events WHERE artifact_id=?1 ORDER BY rowid DESC LIMIT 1", params![inputs.original_artifact_id], |r| r.get(0)).optional()?;
                if state.as_deref() != Some(ProcedureLifecycleDispositionV1::Promoted.as_str()) {
                    return Err(MemoryError::ProceduralMemoryRejected { reason: "candidate is not currently promoted".into() });
                }
                let old: Option<(String, String)> = tx.query_row("SELECT request_digest, payload_json FROM procedure_replay_inputs WHERE replay_id=?1", params![replay_id], |r| Ok((r.get(0)?, r.get(1)?))).optional()?;
                if let Some((d, _)) = old { if d != request_digest { return Err(MemoryError::ProceduralMemoryConflict { key: replay_id.clone() }); } }
                else {
                    let permit_used: Option<String> = tx.query_row("SELECT replay_id FROM procedure_replay_permit_uses WHERE permit_ref=?1", params![permit_id], |row| row.get(0)).optional()?;
                    if permit_used.is_some() {
                        return Err(MemoryError::ProceduralMemoryUnauthorized { principal: permit.principal.clone() });
                    }
                    tx.execute("INSERT INTO procedure_replay_inputs (replay_id,original_artifact_id,original_artifact_digest,patch_digest,source_tree_digest,verifier_digest,check_policy_digest,environment_digest,image_digest,store_identity_digest,retained_input_digest,promotion_receipt_ref,action_permit_ref,request_digest,payload_json,created_at) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,datetime('now'))", params![inputs.replay_id,inputs.original_artifact_id,inputs.original_artifact_digest,inputs.patch_digest,inputs.source_tree_digest,inputs.verifier_digest,inputs.check_policy_digest,inputs.environment_digest,inputs.image_digest,inputs.store_identity_digest,inputs.retained_input_digest,inputs.promotion_receipt_ref,inputs.action_permit_ref,request_digest,json])?;
                    tx.execute("INSERT INTO procedure_replay_permit_uses VALUES (?, ?, datetime('now'))", params![permit_id, inputs.replay_id])?;
                }
                let ad = digest(&(SCHEMA, &inputs.replay_id, &request_digest))?;
                let admission = ProcedureReplayAdmissionV1 { schema_version: SCHEMA.into(), replay_id: inputs.replay_id.clone(), admission_digest: ad.clone() };
                let admission_json = serde_json::to_string(&admission).map_err(|error| MemoryError::Other(error.to_string()))?;
                tx.execute("INSERT OR IGNORE INTO procedure_replay_admissions VALUES (?, ?, ?, datetime('now'))", params![inputs.replay_id, ad, admission_json])?;
                Ok(ProcedureReplayAdmissionV1 { schema_version: SCHEMA.into(), replay_id: inputs.replay_id, admission_digest: ad })
            })
        }).await
    }

    pub async fn load_retained_replay_inputs(
        &self,
        replay_id: impl Into<String>,
    ) -> Result<ProcedureReplayInputsV1, MemoryError> {
        let id = replay_id.into();
        self.with_read_conn(move |c| {
            c.query_row(
                "SELECT payload_json FROM procedure_replay_inputs WHERE replay_id=?1",
                params![id],
                |r| r.get::<_, String>(0),
            )
            .optional()?
            .ok_or_else(|| MemoryError::ProceduralMemoryNotFound {
                artifact_id: id.clone(),
            })
            .and_then(|j| {
                let parsed: ProcedureReplayInputsV1 =
                    serde_json::from_str(&j).map_err(|e| MemoryError::CorruptData {
                        table: "procedure_replay_inputs",
                        row_id: "payload".into(),
                        detail: e.to_string(),
                    })?;
                let expected = digest(&parsed)?;
                let stored: String = c.query_row(
                    "SELECT request_digest FROM procedure_replay_inputs WHERE replay_id=?1",
                    params![parsed.replay_id],
                    |r| r.get(0),
                )?;
                if expected != stored {
                    return Err(MemoryError::CorruptData {
                        table: "procedure_replay_inputs",
                        row_id: parsed.replay_id,
                        detail: "request digest mismatch".into(),
                    });
                }
                Ok(parsed)
            })
        })
        .await
    }

    pub async fn record_replay_result(
        &self,
        result: ProcedureReplayResultV1,
    ) -> Result<ProcedureReplayResultV1, MemoryError> {
        let r = result.clone();
        self.with_write_conn(move |c| {
            crate::db::with_transaction(c, |tx| {
                let exists: Option<String> = tx
                    .query_row(
                        "SELECT receipt_json FROM procedure_replay_results WHERE replay_id=?1",
                        params![r.replay_id],
                        |x| x.get(0),
                    )
                    .optional()?;
                let json = serde_json::to_string(&r)
                    .map_err(|error| MemoryError::Other(error.to_string()))?;
                if let Some(old) = exists {
                    if old != json {
                        return Err(MemoryError::ProceduralMemoryConflict {
                            key: r.replay_id.clone(),
                        });
                    }
                    return Ok(r);
                }
                tx.execute(
                    "INSERT INTO procedure_replay_results VALUES (?, ?, ?, ?, ?, datetime('now'))",
                    params![
                        r.replay_id,
                        r.result_digest,
                        serde_json::to_string(&r.outcome)
                            .map_err(|error| MemoryError::Other(error.to_string()))?,
                        serde_json::to_string(&r.reason_codes)
                            .map_err(|error| MemoryError::Other(error.to_string()))?,
                        json
                    ],
                )?;
                Ok(r)
            })
        })
        .await
    }
}

pub async fn admit_procedure_replay(
    store: &MemoryStore,
    inputs: ProcedureReplayInputsV1,
    permit: ProcedureActionPermitV1,
) -> Result<ProcedureReplayAdmissionV1, MemoryError> {
    store.admit_procedure_replay(inputs, permit).await
}
pub async fn admit_adjudicated_procedure_replay(
    store: &MemoryStore,
    forge: &dyn ForgeAdjudicationStore,
    replay_id: impl Into<String>,
    adjudication_id: &str,
    permit: ProcedureActionPermitV1,
) -> Result<ProcedureReplayAdmissionV1, MemoryError> {
    store
        .admit_adjudicated_procedure_replay(forge, replay_id, adjudication_id, permit)
        .await
}
pub async fn load_procedure_owner_snapshot(
    store: &MemoryStore,
    artifact_id: impl Into<String>,
) -> Result<ProcedureOwnerSnapshotV1, MemoryError> {
    store.load_procedure_owner_snapshot(artifact_id).await
}
pub async fn load_procedure_replay_snapshot(
    store: &MemoryStore,
    replay_id: impl Into<String>,
) -> Result<ProcedureReplaySnapshotV1, MemoryError> {
    store.load_procedure_replay_snapshot(replay_id).await
}
pub async fn load_retained_replay_inputs(
    store: &MemoryStore,
    id: impl Into<String>,
) -> Result<ProcedureReplayInputsV1, MemoryError> {
    store.load_retained_replay_inputs(id).await
}
pub async fn record_replay_result(
    store: &MemoryStore,
    result: ProcedureReplayResultV1,
) -> Result<ProcedureReplayResultV1, MemoryError> {
    store.record_replay_result(result).await
}
pub fn compare_replay(
    original: &ProcedureReplayInputsV1,
    replay: &ProcedureReplayInputsV1,
) -> ProcedureReplayComparisonV1 {
    compare_replay_observation(original, replay)
}

pub fn compare_replay_observation(
    original: &ProcedureReplayInputsV1,
    replay: &ProcedureReplayInputsV1,
) -> ProcedureReplayComparisonV1 {
    let original_material = observe_replay_material(original);
    let replay_material = observe_replay_material(replay);
    if original_material.is_none() || replay_material.is_none() {
        return ProcedureReplayComparisonV1 {
            outcome: ProcedureReplayOutcomeV1::Inconclusive,
            reason_codes: vec!["inconclusive_missing_replay_material".into()],
        };
    }
    let original_material = original_material.unwrap();
    let replay_material = replay_material.unwrap();
    let ids = [
        (
            "patch_digest",
            &original_material.patch_digest,
            &replay_material.patch_digest,
        ),
        (
            "source_tree_digest",
            &original_material.source_tree_digest,
            &replay_material.source_tree_digest,
        ),
        (
            "verifier_digest",
            &original_material.verifier_digest,
            &replay_material.verifier_digest,
        ),
        (
            "check_policy_digest",
            &original_material.check_policy_digest,
            &replay_material.check_policy_digest,
        ),
        (
            "environment_digest",
            &original_material.environment_digest,
            &replay_material.environment_digest,
        ),
        (
            "image_digest",
            &original_material.image_digest,
            &replay_material.image_digest,
        ),
        (
            "store_identity_digest",
            &original_material.store_identity_digest,
            &replay_material.store_identity_digest,
        ),
        (
            "retained_input_digest",
            &original_material.retained_input_digest,
            &replay_material.retained_input_digest,
        ),
    ];
    let reasons = ids
        .iter()
        .filter(|(_, a, b)| a != b)
        .map(|(n, _, _)| format!("drift_{n}"))
        .collect::<Vec<_>>();
    ProcedureReplayComparisonV1 {
        outcome: if reasons.is_empty() {
            ProcedureReplayOutcomeV1::ExactMatch
        } else {
            ProcedureReplayOutcomeV1::Drift
        },
        reason_codes: reasons,
    }
}

fn observe_replay_material(
    inputs: &ProcedureReplayInputsV1,
) -> Option<ProcedureReplayObservedIdentityV1> {
    Some(ProcedureReplayObservedIdentityV1 {
        patch_digest: canonicalize_replay_material(&inputs.patch_digest)?,
        source_tree_digest: canonicalize_replay_material(&inputs.source_tree_digest)?,
        verifier_digest: canonicalize_replay_material(&inputs.verifier_digest)?,
        check_policy_digest: canonicalize_replay_material(&inputs.check_policy_digest)?,
        environment_digest: canonicalize_replay_material(&inputs.environment_digest)?,
        image_digest: canonicalize_replay_material(&inputs.image_digest)?,
        store_identity_digest: canonicalize_replay_material(&inputs.store_identity_digest)?,
        retained_input_digest: canonicalize_replay_material(&inputs.retained_input_digest)?,
    })
}

fn canonicalize_replay_material(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        return None;
    }
    Some(
        value
            .trim()
            .trim_start_matches("blake3:")
            .to_ascii_lowercase(),
    )
}

fn rejected(error: impl ToString) -> MemoryError {
    MemoryError::ProceduralMemoryRejected {
        reason: error.to_string(),
    }
}

fn strip_blake3_prefix(value: &str) -> &str {
    value.strip_prefix("blake3:").unwrap_or(value)
}

fn derive_store_identity(
    artifact: &ProceduralMemoryArtifactV1,
    lifecycle_receipt: Option<&ProcedureLifecycleReceiptV1>,
    effectful_receipt: Option<&ProcedureEffectfulEvaluationReceiptV1>,
) -> Result<String, MemoryError> {
    digest(&(
        "procedure_replay_store_identity_v1",
        &artifact.artifact_id,
        &artifact.artifact_digest,
        lifecycle_receipt.map(|value| (&value.receipt_id, &value.receipt_digest)),
        effectful_receipt.map(|value| (&value.receipt_id, &value.receipt_digest)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> ProcedureReplayInputsV1 {
        ProcedureReplayInputsV1 {
            replay_id: "replay-1".into(),
            original_artifact_id: "artifact-1".into(),
            original_artifact_digest: "artifact-digest".into(),
            patch_digest: "patch".into(),
            source_tree_digest: "tree".into(),
            verifier_digest: "verifier".into(),
            check_policy_digest: "policy".into(),
            environment_digest: "environment".into(),
            image_digest: "image".into(),
            store_identity_digest: "store".into(),
            retained_input_digest: "retained".into(),
            promotion_receipt_ref: "promotion".into(),
            action_permit_ref: "permit".into(),
        }
    }

    #[test]
    fn identity_digest_is_deterministic_and_input_sensitive() -> Result<(), MemoryError> {
        let first = digest(&inputs())?;
        let second = digest(&inputs())?;
        let mut changed = inputs();
        changed.patch_digest = "changed".into();
        assert_eq!(first, second);
        let changed_digest = digest(&changed)?;
        assert_ne!(first, changed_digest);
        Ok(())
    }

    #[test]
    fn comparison_distinguishes_exact_match_and_drift() {
        let original = inputs();
        assert_eq!(
            compare_replay(&original, &original).outcome,
            ProcedureReplayOutcomeV1::ExactMatch
        );
        let mut replay = original.clone();
        replay.environment_digest = "changed".into();
        let comparison = compare_replay(&original, &replay);
        assert_eq!(comparison.outcome, ProcedureReplayOutcomeV1::Drift);
        assert_eq!(comparison.reason_codes, vec!["drift_environment_digest"]);
    }

    #[test]
    fn comparison_is_canonical_and_inconclusive_when_material_missing() {
        let mut original = inputs();
        original.verifier_digest = "blake3:ABCDEF".into();
        original.environment_digest = "blake3:123456".into();
        let mut replay = original.clone();
        replay.verifier_digest = "ABCDEF".into();
        replay.environment_digest = "123456".into();
        assert_eq!(
            compare_replay_observation(&original, &replay).outcome,
            ProcedureReplayOutcomeV1::ExactMatch
        );

        let mut missing = original;
        missing.verifier_digest = "".into();
        let comparison = compare_replay_observation(&missing, &replay);
        assert_eq!(comparison.outcome, ProcedureReplayOutcomeV1::Inconclusive);
    }
}
