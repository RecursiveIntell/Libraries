use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CANDIDATE_PROMOTION_ADJUDICATION_V1_SCHEMA: &str = "CandidatePromotionAdjudicationV1";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct IdentityDigest(String);

impl IdentityDigest {
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        if value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
            Ok(Self(value.to_ascii_lowercase()))
        } else {
            Err("identity digest must be canonical 64-character hexadecimal BLAKE3".into())
        }
    }

    pub fn of(value: impl AsRef<[u8]>) -> Self {
        Self(blake3::hash(value.as_ref()).to_hex().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReceiptRef {
    pub receipt_id: String,
    pub receipt_digest: IdentityDigest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct UncertaintyV1 {
    pub estimate: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FamilyGateV1 {
    pub family: String,
    pub score: f64,
    pub passed: bool,
    pub admissible_pairs: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HoldoutGateV1 {
    pub score: f64,
    pub passed: bool,
    pub admissible_pairs: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FrozenPromotionThresholdsV1 {
    pub minimum_admissible_pairs: u64,
    pub minimum_family_score: f64,
    pub minimum_holdout_score: f64,
    pub maximum_uncertainty: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum AdjudicationDecisionV1 {
    EligibleForLifecycleConsideration,
    Quarantined,
    Inconclusive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CandidatePromotionAdjudicationV1 {
    pub schema_version: String,
    pub adjudication_id: String,
    pub adjudication_digest: IdentityDigest,
    pub candidate_id: String,
    pub candidate_digest: IdentityDigest,
    pub patch_digest: IdentityDigest,
    pub source_tree_digest: IdentityDigest,
    pub verifier_digest: IdentityDigest,
    pub check_policy_digest: IdentityDigest,
    pub environment_digest: IdentityDigest,
    pub image_digest: IdentityDigest,
    pub experiment_id: String,
    pub evidence_bundle_id: String,
    pub evidence_bundle_digest: IdentityDigest,
    pub assignment_digest: IdentityDigest,
    pub paired_denominator: u64,
    pub admissible_pairs: u64,
    pub excluded_pairs: u64,
    pub uncertainty: UncertaintyV1,
    pub family_results: Vec<FamilyGateV1>,
    pub holdout_result: HoldoutGateV1,
    pub thresholds: FrozenPromotionThresholdsV1,
    pub decision: AdjudicationDecisionV1,
    pub reason_codes: Vec<String>,
    pub source_receipt_refs: Vec<ReceiptRef>,
    pub created_at: String,
}

impl CandidatePromotionAdjudicationV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != CANDIDATE_PROMOTION_ADJUDICATION_V1_SCHEMA {
            return Err("invalid schema version".into());
        }
        for (name, value) in [
            ("candidate_id", &self.candidate_id),
            ("experiment_id", &self.experiment_id),
            ("evidence_bundle_id", &self.evidence_bundle_id),
            ("assignment_digest", &self.assignment_digest.0),
        ] {
            if value.is_empty() {
                return Err(format!("{name} must not be empty"));
            }
        }
        if self.paired_denominator == 0
            || self.admissible_pairs > self.paired_denominator
            || self.excluded_pairs > self.paired_denominator
        {
            return Err("invalid denominator or admissibility counts".into());
        }
        if self.admissible_pairs + self.excluded_pairs != self.paired_denominator {
            return Err("admissible and excluded pairs must partition denominator".into());
        }
        if self.family_results.is_empty() {
            return Err("family results are required".into());
        }
        if self
            .family_results
            .iter()
            .any(|gate| gate.family.is_empty())
        {
            return Err("family names must not be empty".into());
        }
        if self.source_receipt_refs.is_empty() {
            return Err("source receipt refs are required".into());
        }
        if !(0.0..=1.0).contains(&self.uncertainty.lower_bound)
            || !(0.0..=1.0).contains(&self.uncertainty.upper_bound)
            || self.uncertainty.lower_bound > self.uncertainty.upper_bound
        {
            return Err("invalid uncertainty bounds".into());
        }
        if self.canonical_digest()? != self.adjudication_digest {
            return Err("adjudication digest does not bind material fields".into());
        }
        Ok(())
    }

    pub fn canonical_digest(&self) -> Result<IdentityDigest, String> {
        let mut copy = self.clone();
        copy.adjudication_digest = IdentityDigest::of([]);
        copy.created_at.clear();
        let bytes = serde_json::to_vec(&copy)
            .map_err(|error| format!("failed to serialize canonical adjudication: {error}"))?;
        Ok(IdentityDigest::of(bytes))
    }
}

#[derive(Debug, Clone)]
pub struct CandidatePromotionInput {
    pub adjudication_id: String,
    pub candidate_id: String,
    pub candidate_digest: IdentityDigest,
    pub patch_digest: IdentityDigest,
    pub source_tree_digest: IdentityDigest,
    pub verifier_digest: IdentityDigest,
    pub check_policy_digest: IdentityDigest,
    pub environment_digest: IdentityDigest,
    pub image_digest: IdentityDigest,
    pub experiment_id: String,
    pub evidence_bundle_id: String,
    pub evidence_bundle_digest: IdentityDigest,
    pub assignment_digest: IdentityDigest,
    pub paired_denominator: u64,
    pub admissible_pairs: u64,
    pub excluded_pairs: u64,
    pub uncertainty: UncertaintyV1,
    pub family_results: Vec<FamilyGateV1>,
    pub holdout_result: HoldoutGateV1,
    pub thresholds: FrozenPromotionThresholdsV1,
    pub source_receipt_refs: Vec<ReceiptRef>,
    pub created_at: String,
}

pub fn adjudicate_candidate(
    input: CandidatePromotionInput,
) -> Result<CandidatePromotionAdjudicationV1, String> {
    let mut reasons = Vec::new();
    if input.paired_denominator == 0
        || input.admissible_pairs < input.thresholds.minimum_admissible_pairs
    {
        reasons.push("insufficient_admissible_pairs".into());
    }
    if input
        .family_results
        .iter()
        .any(|g| !g.passed || g.score < input.thresholds.minimum_family_score)
    {
        reasons.push("family_gate_failed".into());
    }
    if !input.holdout_result.passed
        || input.holdout_result.score < input.thresholds.minimum_holdout_score
    {
        reasons.push("holdout_gate_failed".into());
    }
    if input.uncertainty.upper_bound - input.uncertainty.lower_bound
        > input.thresholds.maximum_uncertainty
    {
        reasons.push("uncertainty_too_high".into());
    }
    let decision = if input.paired_denominator == 0 || input.admissible_pairs == 0 {
        AdjudicationDecisionV1::Inconclusive
    } else if reasons.is_empty() {
        AdjudicationDecisionV1::EligibleForLifecycleConsideration
    } else {
        AdjudicationDecisionV1::Quarantined
    };
    let mut output = CandidatePromotionAdjudicationV1 {
        schema_version: CANDIDATE_PROMOTION_ADJUDICATION_V1_SCHEMA.into(),
        adjudication_id: input.adjudication_id,
        adjudication_digest: IdentityDigest::of([]),
        candidate_id: input.candidate_id,
        candidate_digest: input.candidate_digest,
        patch_digest: input.patch_digest,
        source_tree_digest: input.source_tree_digest,
        verifier_digest: input.verifier_digest,
        check_policy_digest: input.check_policy_digest,
        environment_digest: input.environment_digest,
        image_digest: input.image_digest,
        experiment_id: input.experiment_id,
        evidence_bundle_id: input.evidence_bundle_id,
        evidence_bundle_digest: input.evidence_bundle_digest,
        assignment_digest: input.assignment_digest,
        paired_denominator: input.paired_denominator,
        admissible_pairs: input.admissible_pairs,
        excluded_pairs: input.excluded_pairs,
        uncertainty: input.uncertainty,
        family_results: input.family_results,
        holdout_result: input.holdout_result,
        thresholds: input.thresholds,
        decision,
        reason_codes: reasons,
        source_receipt_refs: input.source_receipt_refs,
        created_at: input.created_at,
    };
    output.adjudication_digest = output.canonical_digest()?;
    Ok(output)
}
