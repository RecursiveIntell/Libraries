//! Canonical owner for exact coding-procedure replay retention (V38).
use crate::procedural_memory::ProcedureLifecycleDispositionV1;
use crate::{MemoryError, MemoryStore, ProcedureActionPermitV1};
use blake3::Hasher;
use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

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
    let ids = [
        ("patch_digest", &original.patch_digest, &replay.patch_digest),
        (
            "source_tree_digest",
            &original.source_tree_digest,
            &replay.source_tree_digest,
        ),
        (
            "verifier_digest",
            &original.verifier_digest,
            &replay.verifier_digest,
        ),
        (
            "check_policy_digest",
            &original.check_policy_digest,
            &replay.check_policy_digest,
        ),
        (
            "environment_digest",
            &original.environment_digest,
            &replay.environment_digest,
        ),
        ("image_digest", &original.image_digest, &replay.image_digest),
        (
            "store_identity_digest",
            &original.store_identity_digest,
            &replay.store_identity_digest,
        ),
        (
            "retained_input_digest",
            &original.retained_input_digest,
            &replay.retained_input_digest,
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
}
