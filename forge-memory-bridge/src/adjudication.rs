//! Owner-facing, persistence-neutral boundary for Forge adjudication artifacts.

use serde::{Deserialize, Serialize};
use verification_adjudication::{CandidatePromotionAdjudicationV1, IdentityDigest};

use crate::BridgeError;

pub const FORGE_ADJUDICATION_PERSISTENCE_RECEIPT_V1_SCHEMA: &str =
    "ForgeAdjudicationPersistenceReceiptV1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForgeAdjudicationPersistenceReceiptV1 {
    pub schema_version: String,
    pub adjudication_id: String,
    pub adjudication_digest: IdentityDigest,
    pub owner: String,
    pub persisted_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjudicationBindingV1 {
    pub candidate_id: String,
    pub candidate_digest: IdentityDigest,
    pub evidence_bundle_id: String,
    pub evidence_bundle_digest: IdentityDigest,
}

pub trait ForgeAdjudicationStore {
    fn persist_adjudication(
        &self,
        adjudication: &CandidatePromotionAdjudicationV1,
    ) -> Result<ForgeAdjudicationPersistenceReceiptV1, BridgeError>;

    fn read_verified_adjudication(
        &self,
        adjudication_id: &str,
    ) -> Result<CandidatePromotionAdjudicationV1, BridgeError>;

    fn verify_adjudication_binding(
        &self,
        adjudication_id: &str,
        expected: &AdjudicationBindingV1,
    ) -> Result<ForgeAdjudicationPersistenceReceiptV1, BridgeError>;
}
