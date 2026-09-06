use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use agent_graph::remote_work::{
    settle_remote_work, RemoteAttemptV1, RemoteDispositionV1, RemoteOwnerPort, ReplicationStatusV1,
};

#[derive(Default)]
struct Owners {
    revisions: BTreeMap<String, String>,
    authority: BTreeSet<String>,
    gates: BTreeSet<String>,
    publications: Mutex<BTreeMap<String, String>>,
}

impl RemoteOwnerPort for Owners {
    fn current_view_revision(&self, logical_work_id: &str) -> Option<String> {
        self.revisions.get(logical_work_id).cloned()
    }

    fn current_authority(&self, logical_work_id: &str) -> bool {
        self.authority.contains(logical_work_id)
    }

    fn replication_gate_passes(&self, gate_ref: &str) -> bool {
        self.gates.contains(gate_ref)
    }

    fn publish_once(&self, publication_key: &str, result_ref: &str) -> bool {
        let mut publications = self.publications.lock().unwrap();
        if publications.contains_key(publication_key) {
            false
        } else {
            publications.insert(publication_key.into(), result_ref.into());
            true
        }
    }

    fn publication_count(&self, publication_key: &str) -> u64 {
        u64::from(
            self.publications
                .lock()
                .unwrap()
                .contains_key(publication_key),
        )
    }
}

fn attempts() -> Vec<RemoteAttemptV1> {
    vec![
        RemoteAttemptV1 {
            logical_work_id: "work:1".into(),
            attempt_id: "attempt:1".into(),
            publication_key: "publication:1".into(),
            view_revision: "view:2".into(),
            budget_microunits: 7,
            result_ref: "result:one".into(),
        },
        RemoteAttemptV1 {
            logical_work_id: "work:1".into(),
            attempt_id: "attempt:2".into(),
            publication_key: "publication:1".into(),
            view_revision: "view:2".into(),
            budget_microunits: 11,
            result_ref: "result:two".into(),
        },
    ]
}

fn complete_replication() -> ReplicationStatusV1 {
    ReplicationStatusV1 {
        facts_complete: true,
        evidence_complete: true,
        authority_complete: true,
        operation_gate_refs: vec!["gate:join".into(), "gate:effect".into()],
    }
}

fn owners() -> Owners {
    Owners {
        revisions: BTreeMap::from([("work:1".into(), "view:2".into())]),
        authority: BTreeSet::from(["work:1".into()]),
        gates: BTreeSet::from(["gate:join".into(), "gate:effect".into()]),
        publications: Mutex::new(BTreeMap::new()),
    }
}

#[test]
fn adv_03_remote_duplicate_work_deduplicates_publication_and_retains_attempts_and_budgets() {
    let owners = owners();
    let first = settle_remote_work(&attempts(), &complete_replication(), &owners);
    let second = settle_remote_work(&attempts(), &complete_replication(), &owners);
    assert_eq!(first.disposition, RemoteDispositionV1::Published);
    assert_eq!(second.disposition, RemoteDispositionV1::Deduplicated);
    assert_eq!(first.retained_attempt_ids, ["attempt:1", "attempt:2"]);
    assert_eq!(first.retained_budget_microunits, 18);
    assert_eq!(first.publication_count, 1);
    assert_eq!(second.publication_count, 1);
}

#[test]
fn adv_04_remote_stale_or_revoked_view_is_rejected_by_current_owner() {
    let owners = owners();
    let mut stale = attempts();
    stale[0].view_revision = "view:1".into();
    assert_eq!(
        settle_remote_work(&stale, &complete_replication(), &owners).disposition,
        RemoteDispositionV1::Stale
    );
    let revoked = Owners {
        revisions: owners.revisions,
        authority: BTreeSet::new(),
        gates: owners.gates,
        publications: Mutex::new(BTreeMap::new()),
    };
    assert_eq!(
        settle_remote_work(&attempts(), &complete_replication(), &revoked).disposition,
        RemoteDispositionV1::Revoked
    );
}

#[test]
fn adv_05_partial_memory_replication_remains_blocked_until_operation_gates_pass() {
    let owners = owners();
    for partial in [
        ReplicationStatusV1 {
            facts_complete: false,
            ..complete_replication()
        },
        ReplicationStatusV1 {
            evidence_complete: false,
            ..complete_replication()
        },
        ReplicationStatusV1 {
            authority_complete: false,
            ..complete_replication()
        },
        ReplicationStatusV1 {
            operation_gate_refs: vec!["gate:missing".into()],
            ..complete_replication()
        },
    ] {
        let result = settle_remote_work(&attempts(), &partial, &owners);
        assert_eq!(result.disposition, RemoteDispositionV1::BlockedReplication);
        assert_eq!(result.publication_count, 0);
    }
}
