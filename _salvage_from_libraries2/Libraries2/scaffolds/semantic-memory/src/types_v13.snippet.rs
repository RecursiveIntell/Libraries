use serde::{Deserialize, Serialize};
use stack_ids::{
    ClaimId, ClaimVersionId, ContentDigest, ContradictionWitnessId, SupportSetId,
};

/// Additive public read shape for v13-aware callers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionClaimVersionV13 {
    pub claim_version_id: ClaimVersionId,
    pub claim_id: ClaimId,
    pub bilattice_truth: String,
    pub support_set_id: Option<SupportSetId>,
    pub support_set_digest: Option<ContentDigest>,
    pub contradiction_witness_id: Option<ContradictionWitnessId>,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub tx_from: String,
    pub tx_to: Option<String>,
    pub quality_vector_json: Option<serde_json::Value>,
}
