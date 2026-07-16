//! Non-authoritative coding-learning projections and material-bound writers.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CodingLearningBundleV3 {
    pub schema: String,
    pub bundle_id: ArtifactId,
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
}

impl CodingLearningBundleV3 {
    pub fn new_material_bound(material: &str) -> Self {
        Self {
            schema: "CodingLearningBundleV3".into(),
            bundle_id: generated_artifact_id_from_material("coding-learning-bundle-v3", material),
            canonical_backpointers: coding_learning_backpointers(),
        }
    }
}

pub fn validate_durable_v3_identity(id: &ArtifactId) -> Result<(), String> {
    if id.as_str().contains("local-process-seq") {
        Err("display-only identity is not durable".into())
    } else {
        Ok(())
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
pub struct CodingLearningEvidenceV1 {
    pub required_owner_receipts: bool,
    pub receipts_healthy: bool,
    pub publication_complete: bool,
    pub index_complete: bool,
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub reason_codes: Vec<String>,
}
impl Default for CodingLearningEvidenceV1 {
    fn default() -> Self {
        Self {
            required_owner_receipts: false,
            receipts_healthy: false,
            publication_complete: false,
            index_complete: false,
            canonical_backpointers: coding_learning_backpointers(),
            reason_codes: vec!["evidence-insufficient".into()],
        }
    }
}

pub fn succeeded_verified(e: &CodingLearningEvidenceV1) -> bool {
    e.required_owner_receipts
        && e.receipts_healthy
        && e.publication_complete
        && e.index_complete
        && e.canonical_backpointers.len() >= 12
}
pub fn project_terminal_state(e: &CodingLearningEvidenceV1) -> CodingLearningTerminalStateV1 {
    if succeeded_verified(e) {
        CodingLearningTerminalStateV1::SucceededVerified
    } else {
        CodingLearningTerminalStateV1::BlockedEvidenceInsufficient
    }
}
fn coding_learning_backpointers() -> Vec<CanonicalBackpointerV1> {
    [
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
    ]
    .into_iter()
    .map(|role| CanonicalBackpointerV1::owner_type("canonical-owner", "ExternalArtifact", role))
    .collect()
}
