//! Non-authoritative coding-learning terminal projection.
//!
//! Canonical task, policy, effect, verification, memory, attribution, export,
//! and replay truth remains in the owner artifacts referenced by this module.

use super::*;

const REQUIRED_CODING_LEARNING_OWNER_ROLES: [&str; 11] = [
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
];

const REQUIRED_CODING_LEARNING_TERMINAL_ROLES: [&str; 12] = [
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
    "published-run-bundle",
];

const REQUIRED_CODING_LEARNING_CHILD_OWNERS: [&str; 8] = [
    "owner:effectful-evaluation",
    "owner:procedure-lifecycle-tested",
    "owner:procedure-effectful-prerequisite",
    "owner:forge-evidence-bundle",
    "owner:forge-export-receipt",
    "owner:semantic-memory-projection-import",
    "owner:procedure-lifecycle-promoted",
    "owner:promoted-procedure-replay",
];

pub fn required_coding_learning_owner_roles() -> &'static [&'static str] {
    &REQUIRED_CODING_LEARNING_OWNER_ROLES
}

pub fn required_coding_learning_terminal_roles() -> &'static [&'static str] {
    &REQUIRED_CODING_LEARNING_TERMINAL_ROLES
}

pub fn required_coding_learning_child_owners() -> &'static [&'static str] {
    &REQUIRED_CODING_LEARNING_CHILD_OWNERS
}

pub fn validate_required_coding_learning_children(
    children: &[AiDENsRunChildReceiptV1],
) -> Result<(), Vec<String>> {
    let mut reasons = Vec::new();
    let observed = children
        .iter()
        .map(|child| child.owner_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let required = required_coding_learning_child_owners()
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    if children.len() != required.len() || observed != required {
        reasons.push("coding-learning-required-child-owner-set-mismatch".into());
    }
    for child in children {
        let receipt = &child.receipt;
        let valid = match child.owner_id.as_str() {
            "owner:effectful-evaluation" => {
                receipt
                    .pointer("/schema")
                    .and_then(serde_json::Value::as_str)
                    == Some("AiDENsEffectfulEvaluationReportV1")
                    && receipt
                        .pointer("/verified")
                        .and_then(serde_json::Value::as_bool)
                        == Some(true)
            }
            "owner:procedure-lifecycle-tested" => {
                lifecycle_receipt_matches(receipt, "tested", "test")
            }
            "owner:procedure-effectful-prerequisite" => {
                receipt
                    .pointer("/schema_version")
                    .and_then(serde_json::Value::as_str)
                    == Some("procedure_effectful_evaluation_receipt_v1")
                    && receipt
                        .pointer("/verified")
                        .and_then(serde_json::Value::as_bool)
                        == Some(true)
                    && nonempty_json_string(receipt, "/receipt_id")
                    && nonempty_json_string(receipt, "/receipt_digest")
            }
            "owner:forge-evidence-bundle" => {
                receipt
                    .pointer("/version_id")
                    .and_then(serde_json::Value::as_str)
                    == Some("aidens-exact-source-execution-evidence-v1")
                    && nonempty_json_string(receipt, "/bundle_id")
                    && nonempty_json_string(receipt, "/candidate_id")
            }
            "owner:forge-export-receipt" => {
                receipt
                    .pointer("/rendering_version")
                    .and_then(serde_json::Value::as_u64)
                    == Some(3)
                    && nonempty_json_string(receipt, "/export_key")
                    && nonempty_json_string(receipt, "/bundle_id")
                    && nonempty_json_string(receipt, "/namespace")
            }
            "owner:semantic-memory-projection-import" => {
                receipt
                    .pointer("/status")
                    .and_then(serde_json::Value::as_str)
                    == Some("complete")
                    && receipt
                        .pointer("/direct_write")
                        .and_then(serde_json::Value::as_bool)
                        == Some(false)
                    && nonempty_json_string(receipt, "/source_envelope_id")
                    && nonempty_json_string(receipt, "/content_digest")
            }
            "owner:procedure-lifecycle-promoted" => {
                lifecycle_receipt_matches(receipt, "promoted", "promote")
                    && nonempty_json_string(receipt, "/adjudication_digest")
                    && nonempty_json_string(receipt, "/permit_digest")
            }
            "owner:promoted-procedure-replay" => {
                receipt
                    .pointer("/schema")
                    .and_then(serde_json::Value::as_str)
                    == Some("AiDENsPromotedProcedureReplayOutcomeV1")
                    && receipt
                        .pointer("/action_allowed")
                        .and_then(serde_json::Value::as_bool)
                        == Some(true)
                    && receipt
                        .pointer("/retained_patch_exact")
                        .and_then(serde_json::Value::as_bool)
                        == Some(true)
                    && receipt
                        .pointer("/report/verified")
                        .and_then(serde_json::Value::as_bool)
                        == Some(true)
            }
            _ => false,
        };
        if !valid {
            reasons.push(format!(
                "coding-learning-child-signature-invalid:{}",
                child.owner_id
            ));
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

fn lifecycle_receipt_matches(
    receipt: &serde_json::Value,
    disposition: &str,
    operation: &str,
) -> bool {
    receipt
        .pointer("/schema_version")
        .and_then(serde_json::Value::as_str)
        == Some("procedure_lifecycle_receipt_v1")
        && receipt
            .pointer("/disposition")
            .and_then(serde_json::Value::as_str)
            == Some(disposition)
        && receipt
            .pointer("/operation")
            .and_then(serde_json::Value::as_str)
            == Some(operation)
        && nonempty_json_string(receipt, "/receipt_id")
        && nonempty_json_string(receipt, "/receipt_digest")
}

fn nonempty_json_string(receipt: &serde_json::Value, pointer: &str) -> bool {
    receipt
        .pointer(pointer)
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
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
    validate_coding_learning_backpointer_roles(backpointers, required_coding_learning_owner_roles())
}

pub fn validate_required_coding_learning_terminal_backpointers(
    backpointers: &[CanonicalBackpointerV1],
) -> Result<(), Vec<String>> {
    validate_coding_learning_backpointer_roles(
        backpointers,
        required_coding_learning_terminal_roles(),
    )
}

fn validate_coding_learning_backpointer_roles(
    backpointers: &[CanonicalBackpointerV1],
    required_roles: &[&str],
) -> Result<(), Vec<String>> {
    let mut reasons = Vec::new();
    for role in required_roles {
        let matching = backpointers
            .iter()
            .filter(|backpointer| backpointer.role == *role)
            .collect::<Vec<_>>();
        if matching.is_empty() {
            reasons.push(format!("required-owner-reference-missing:{role}"));
        } else if matching.len() > 1 {
            reasons.push(format!("required-owner-reference-duplicate:{role}"));
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
        && validate_required_coding_learning_terminal_backpointers(&evidence.canonical_backpointers)
            .is_ok()
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
        validate_required_coding_learning_terminal_backpointers(&evidence.canonical_backpointers)
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
