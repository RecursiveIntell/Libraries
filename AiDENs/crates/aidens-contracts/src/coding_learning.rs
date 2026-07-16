//! Non-authoritative coding-learning terminal projection.
//!
//! Canonical task, policy, effect, verification, memory, attribution, export,
//! and replay truth remains in the owner artifacts referenced by this module.

use super::*;

const REQUIRED_CODING_LEARNING_OWNER_ROLES: [&str; 12] = [
    "task",
    "source-tree",
    "policy",
    "sandbox",
    "patch",
    "checks",
    "verification",
    "cea",
    "procedure-lifecycle",
    "forge-export-envelope-v3",
    "replay",
    "terminal-projection",
];

pub fn required_coding_learning_owner_roles() -> &'static [&'static str] {
    &REQUIRED_CODING_LEARNING_OWNER_ROLES
}

pub fn validate_durable_v3_identity(id: &ArtifactId) -> Result<(), String> {
    if id.as_str().contains("local-process-seq") {
        Err(format!(
            "display-only identity is not durable: {}",
            id.as_str()
        ))
    } else if id.as_str().trim().is_empty() {
        Err("durable identity is empty".into())
    } else {
        Ok(())
    }
}

fn backpointer_has_durable_owner_identity(backpointer: &CanonicalBackpointerV1) -> bool {
    match (&backpointer.artifact_id, &backpointer.external_id) {
        (Some(artifact_id), None) => validate_durable_v3_identity(artifact_id).is_ok(),
        (None, Some(external_id)) => {
            !external_id.trim().is_empty() && !external_id.contains("local-process-seq")
        }
        _ => false,
    }
}

pub fn validate_required_coding_learning_backpointers(
    backpointers: &[CanonicalBackpointerV1],
) -> Result<(), Vec<String>> {
    let mut reasons = Vec::new();
    for role in required_coding_learning_owner_roles() {
        let matching = backpointers
            .iter()
            .filter(|backpointer| backpointer.role == *role)
            .collect::<Vec<_>>();
        if matching.is_empty() {
            reasons.push(format!("required-owner-reference-missing:{role}"));
        } else if matching.len() != 1 {
            reasons.push(format!("required-owner-reference-role-not-unique:{role}"));
        } else if !backpointer_has_durable_owner_identity(matching[0]) {
            reasons.push(format!("required-owner-reference-not-durable:{role}"));
        }
    }
    reasons.sort();
    reasons.dedup();
    if reasons.is_empty() {
        Ok(())
    } else {
        Err(reasons)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CodingLearningTerminalStateV1 {
    SucceededVerified,
    SucceededDegraded,
    BlockedPolicy,
    BlockedMissingPermit,
    BlockedSandboxUnavailable,
    BlockedReceiptPersistence,
    BlockedEvidenceInsufficient,
    BlockedStaleEvidence,
    BlockedRevoked,
    FailedExecution,
    FailedVerification,
    FailedRollback,
    AbortedCancelled,
    InterruptedAwaitingApproval,
    Quarantined,
    BudgetExhausted,
    ReplayUnavailable,
    ReplayMismatch,
    ReplayDrift,
    ProviderUnavailable,
    MockOnly,
    FixtureOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CodingLearningTerminalProjectionV1 {
    pub state: CodingLearningTerminalStateV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CodingLearningEvidenceV1 {
    pub execution_mode: String,
    pub preflight_persisted: bool,
    pub permits_valid: bool,
    pub typed_patch_applied: bool,
    pub required_checks_executed: bool,
    pub verification_positive: bool,
    pub verification_degraded: bool,
    pub required_digests_present: bool,
    pub terminal_receipts_durable: bool,
    pub receipts_healthy: bool,
    pub publication_complete: bool,
    pub index_complete: bool,
    pub blocked: bool,
    pub revoked: bool,
    pub stale: bool,
    pub mock_only: bool,
    pub fixture_only: bool,
    pub indeterminate: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl Default for CodingLearningEvidenceV1 {
    fn default() -> Self {
        Self {
            execution_mode: "unknown".into(),
            preflight_persisted: false,
            permits_valid: false,
            typed_patch_applied: false,
            required_checks_executed: false,
            verification_positive: false,
            verification_degraded: false,
            required_digests_present: false,
            terminal_receipts_durable: false,
            receipts_healthy: false,
            publication_complete: false,
            index_complete: false,
            blocked: false,
            revoked: false,
            stale: false,
            mock_only: false,
            fixture_only: false,
            indeterminate: false,
            canonical_backpointers: Vec::new(),
            reason_codes: vec!["evidence-insufficient".into()],
        }
    }
}

impl CodingLearningEvidenceV1 {
    pub fn verified_candidate(canonical_backpointers: Vec<CanonicalBackpointerV1>) -> Self {
        Self {
            execution_mode: "real_sandbox".into(),
            preflight_persisted: true,
            permits_valid: true,
            typed_patch_applied: true,
            required_checks_executed: true,
            verification_positive: true,
            verification_degraded: false,
            required_digests_present: true,
            terminal_receipts_durable: true,
            receipts_healthy: true,
            publication_complete: true,
            index_complete: true,
            blocked: false,
            revoked: false,
            stale: false,
            mock_only: false,
            fixture_only: false,
            indeterminate: false,
            canonical_backpointers,
            reason_codes: Vec::new(),
        }
    }
}

pub fn succeeded_verified(evidence: &CodingLearningEvidenceV1) -> bool {
    evidence.execution_mode == "real_sandbox"
        && evidence.preflight_persisted
        && evidence.permits_valid
        && evidence.typed_patch_applied
        && evidence.required_checks_executed
        && evidence.verification_positive
        && !evidence.verification_degraded
        && evidence.required_digests_present
        && evidence.terminal_receipts_durable
        && evidence.receipts_healthy
        && evidence.publication_complete
        && evidence.index_complete
        && !evidence.blocked
        && !evidence.revoked
        && !evidence.stale
        && !evidence.mock_only
        && !evidence.fixture_only
        && !evidence.indeterminate
        && validate_required_coding_learning_backpointers(&evidence.canonical_backpointers).is_ok()
}

fn missing_success_evidence(evidence: &CodingLearningEvidenceV1) -> Vec<String> {
    let mut reasons = evidence.reason_codes.clone();
    let checks = [
        (evidence.preflight_persisted, "preflight-not-persisted"),
        (evidence.permits_valid, "permit-evidence-invalid"),
        (evidence.typed_patch_applied, "typed-patch-not-applied"),
        (
            evidence.required_checks_executed,
            "required-checks-not-executed",
        ),
        (
            evidence.verification_positive,
            "positive-verification-missing",
        ),
        (
            evidence.required_digests_present,
            "required-digests-missing",
        ),
        (
            evidence.terminal_receipts_durable,
            "terminal-receipts-not-durable",
        ),
        (evidence.receipts_healthy, "receipt-health-unverified"),
        (
            evidence.publication_complete,
            "bundle-publication-incomplete",
        ),
        (evidence.index_complete, "index-publication-incomplete"),
    ];
    for (present, reason) in checks {
        if !present {
            reasons.push(reason.into());
        }
    }
    if evidence.execution_mode != "real_sandbox" {
        reasons.push("execution-mode-not-real-sandbox".into());
    }
    if let Err(owner_reasons) =
        validate_required_coding_learning_backpointers(&evidence.canonical_backpointers)
    {
        reasons.extend(owner_reasons);
    }
    reasons.sort();
    reasons.dedup();
    reasons
}

pub fn project_terminal_state(
    evidence: &CodingLearningEvidenceV1,
) -> CodingLearningTerminalProjectionV1 {
    let state = if succeeded_verified(evidence) {
        CodingLearningTerminalStateV1::SucceededVerified
    } else if evidence.mock_only || evidence.execution_mode == "mock" {
        CodingLearningTerminalStateV1::MockOnly
    } else if evidence.fixture_only || evidence.execution_mode == "fixture" {
        CodingLearningTerminalStateV1::FixtureOnly
    } else if evidence.revoked {
        CodingLearningTerminalStateV1::BlockedRevoked
    } else if evidence.stale {
        CodingLearningTerminalStateV1::BlockedStaleEvidence
    } else if evidence.blocked {
        CodingLearningTerminalStateV1::BlockedPolicy
    } else if evidence.verification_degraded {
        CodingLearningTerminalStateV1::SucceededDegraded
    } else if evidence.indeterminate {
        CodingLearningTerminalStateV1::BlockedReceiptPersistence
    } else {
        CodingLearningTerminalStateV1::BlockedEvidenceInsufficient
    };
    let mut canonical_backpointers = evidence.canonical_backpointers.clone();
    canonical_backpointers.push(CanonicalBackpointerV1::owner_type(
        "aidens-contracts",
        "CodingLearningTerminalProjectionV1",
        "non-authoritative-terminal-projection-owner",
    ));
    CodingLearningTerminalProjectionV1 {
        state,
        reason_codes: if succeeded_verified(evidence) {
            vec!["all-success-evidence-gates-satisfied".into()]
        } else {
            missing_success_evidence(evidence)
        },
        canonical_backpointers,
    }
}
