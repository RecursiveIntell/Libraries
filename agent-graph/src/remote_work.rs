use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteAttemptV1 {
    pub logical_work_id: String,
    pub attempt_id: String,
    pub publication_key: String,
    pub view_revision: String,
    pub budget_microunits: u64,
    pub result_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicationStatusV1 {
    pub facts_complete: bool,
    pub evidence_complete: bool,
    pub authority_complete: bool,
    pub operation_gate_refs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteDispositionV1 {
    Published,
    Deduplicated,
    Stale,
    Revoked,
    BlockedReplication,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteSettlementV1 {
    pub logical_work_id: String,
    pub retained_attempt_ids: Vec<String>,
    pub retained_budget_microunits: u64,
    pub selected_result_ref: Option<String>,
    pub disposition: RemoteDispositionV1,
    pub publication_count: u64,
    pub limitations: Vec<String>,
}

pub trait RemoteOwnerPort: Send + Sync {
    fn current_view_revision(&self, logical_work_id: &str) -> Option<String>;
    fn current_authority(&self, logical_work_id: &str) -> bool;
    fn replication_gate_passes(&self, gate_ref: &str) -> bool;
    fn publish_once(&self, publication_key: &str, result_ref: &str) -> bool;
    fn publication_count(&self, publication_key: &str) -> u64;
}

pub fn settle_remote_work(
    attempts: &[RemoteAttemptV1],
    replication: &ReplicationStatusV1,
    owner: &dyn RemoteOwnerPort,
) -> RemoteSettlementV1 {
    let logical_work_id = attempts
        .first()
        .map(|attempt| attempt.logical_work_id.clone())
        .unwrap_or_default();
    let retained_attempt_ids = attempts
        .iter()
        .map(|attempt| attempt.attempt_id.clone())
        .collect::<Vec<_>>();
    let retained_budget_microunits = attempts
        .iter()
        .map(|attempt| attempt.budget_microunits)
        .sum();
    let mut result = RemoteSettlementV1 {
        logical_work_id: logical_work_id.clone(),
        retained_attempt_ids,
        retained_budget_microunits,
        selected_result_ref: None,
        disposition: RemoteDispositionV1::BlockedReplication,
        publication_count: 0,
        limitations: Vec::new(),
    };
    if attempts.is_empty() {
        result.limitations.push("no remote attempts".into());
        return result;
    }
    if !owner.current_authority(&logical_work_id) {
        result.disposition = RemoteDispositionV1::Revoked;
        result.limitations.push("current authority revoked".into());
        return result;
    }
    let Some(current_revision) = owner.current_view_revision(&logical_work_id) else {
        result.disposition = RemoteDispositionV1::Stale;
        result.limitations.push("current view unavailable".into());
        return result;
    };
    if attempts
        .iter()
        .any(|attempt| attempt.view_revision != current_revision)
    {
        result.disposition = RemoteDispositionV1::Stale;
        result.limitations.push("remote view is stale".into());
        return result;
    }
    let unique_gates = replication
        .operation_gate_refs
        .iter()
        .collect::<BTreeSet<_>>();
    if !replication.facts_complete
        || !replication.evidence_complete
        || !replication.authority_complete
        || unique_gates.is_empty()
        || unique_gates
            .iter()
            .any(|gate| !owner.replication_gate_passes(gate))
    {
        result.disposition = RemoteDispositionV1::BlockedReplication;
        result
            .limitations
            .push("operation-specific replication gates incomplete".into());
        return result;
    }
    let selected = &attempts[0];
    let inserted = owner.publish_once(&selected.publication_key, &selected.result_ref);
    result.selected_result_ref = Some(selected.result_ref.clone());
    result.publication_count = owner.publication_count(&selected.publication_key);
    result.disposition = if inserted {
        RemoteDispositionV1::Published
    } else {
        RemoteDispositionV1::Deduplicated
    };
    result
}
