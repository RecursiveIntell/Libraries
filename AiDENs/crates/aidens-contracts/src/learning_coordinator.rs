//! Learning coordinator snapshot projection contracts.
//!
//! These contracts represent reconstructed workflow state only. Payload and permit
//! truth remains in owner-native receipts.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum LearningCoordinatorStageV1 {
    Preflighted,
    ExecutedVerified,
    CandidateTested,
    EffectfulEvidencePersisted,
    ForgePublished,
    Adjudicated,
    EligibleForLifecycleConsideration,
    Quarantined,
    Promoted,
    ReplayAdmitted,
    Replayed,
    RolledBack,
    Revoked,
    TerminalPublished,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum LearningCoordinatorDispositionV1 {
    Pending,
    Blocked,
    Indeterminate,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum NextLearningActionV1 {
    AwaitPreflight,
    ExecuteAndVerify,
    TestCandidate,
    PersistEffectfulEvidence,
    PublishForgeEvidence,
    SeekAdjudication,
    PromoteProcedure,
    AdmitForReplay,
    ReplayProcedure,
    RollbackProcedure,
    RevokeProcedure,
    PublishTerminalEvidence,
    AwaitLifecycleDecision,
    WaitForManualResolution,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OwnerReceiptPointerV1 {
    pub owner_crate: String,
    pub artifact_kind: ArtifactKindV1,
    pub receipt_id: String,
    pub receipt_digest: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OwnerBindingDigestsV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preflight: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executed: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_test: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effectful_evidence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forge: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adjudication: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle_eligibility: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle_promoted: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_admission: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoke: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LearningCoordinatorProjectionV1 {
    pub stage: LearningCoordinatorStageV1,
    pub disposition: LearningCoordinatorDispositionV1,
    pub owner_receipts: Vec<OwnerReceiptPointerV1>,
    pub owner_binding_digests: OwnerBindingDigestsV1,
    pub next_action: NextLearningActionV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl LearningCoordinatorProjectionV1 {
    pub const SCHEMA: &'static str = "AiDENsLearningCoordinatorProjectionV1";

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.owner_receipts.is_empty() {
            return Err("learning coordinator projection requires owner receipt pointers");
        }
        if self.owner_receipts.iter().any(|receipt| {
            receipt.owner_crate.is_empty()
                || receipt.receipt_id.is_empty()
                || receipt.receipt_digest.is_empty()
        }) {
            return Err("learning coordinator projection has invalid owner receipt pointer");
        }
        Ok(())
    }
}
