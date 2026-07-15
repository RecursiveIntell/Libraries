//! Verification target types for the orient phase.
//!
//! Each `TargetKind` variant represents a distinct class of verification
//! work the pilot can select, from active syndromes to stale scopes.

use crate::types::{BudgetClass, CanonicalCaseClass, LawfulStepKind};
use serde::{Deserialize, Serialize};
use stack_ids::ClaimVersionId;
use verification_control::VerificationCaseClass;

/// Scheduling priority of a verification target, lowest numeric value wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetPriority {
    ActiveSyndrome = 0,
    FragileNode = 1,
    ThinExport = 2,
    UnverifiedClaimVersion = 3,
    RefutationGap = 4,
    SupersessionVerification = 5,
    ComparabilityDrift = 6,
    CalibrationCaveat = 7,
    ScopeStale = 8,
}

/// A specific class of verification work identified during orientation.
///
/// Each variant carries the data the act phase needs to execute the
/// corresponding oracle or advisory plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    FragileNode {
        node_id: String,
        belief_micros: u64,
    },
    ActiveSyndrome {
        signature: String,
    },
    ThinExport {
        marker: String,
    },
    UnverifiedClaimVersion {
        claim_version_id: ClaimVersionId,
    },
    RefutationGap {
        target_node_id: String,
        claim_version_id: Option<ClaimVersionId>,
    },
    SupersessionVerification {
        claim_version_id: ClaimVersionId,
        supersedes_claim_version_id: ClaimVersionId,
    },
    ComparabilityDrift {
        detail: String,
    },
    CalibrationCaveat {
        marker: String,
    },
    ScopeStale {
        last_import_at: Option<String>,
    },
}

impl TargetKind {
    /// Returns the canonical case class associated with this target kind.
    pub fn canonical_case_class(&self) -> CanonicalCaseClass {
        match self {
            Self::ActiveSyndrome { .. } => CanonicalCaseClass::ContradictionInvestigation,
            Self::FragileNode { .. } => CanonicalCaseClass::PromotionCandidate,
            Self::ThinExport { .. } => CanonicalCaseClass::ThinExportGap,
            Self::UnverifiedClaimVersion { .. } => CanonicalCaseClass::PromotionCandidate,
            Self::RefutationGap { .. } => CanonicalCaseClass::PromotionCandidate,
            Self::SupersessionVerification { .. } => CanonicalCaseClass::SupersessionVerification,
            Self::ComparabilityDrift { .. } => CanonicalCaseClass::ComparabilityDrift,
            Self::CalibrationCaveat { .. } => CanonicalCaseClass::CalibrationCaveat,
            Self::ScopeStale { .. } => CanonicalCaseClass::ScopeFreshness,
        }
    }

    /// Returns the verification case class associated with this target kind.
    pub fn case_class(&self) -> VerificationCaseClass {
        match self {
            Self::FragileNode { .. } => VerificationCaseClass::FragileNode,
            Self::ActiveSyndrome { .. } => VerificationCaseClass::ActiveSyndrome,
            Self::ThinExport { .. } => VerificationCaseClass::ThinExport,
            Self::UnverifiedClaimVersion { .. } => VerificationCaseClass::UnverifiedClaimVersion,
            Self::RefutationGap { .. } => VerificationCaseClass::RefutationGap,
            Self::SupersessionVerification { .. } => {
                VerificationCaseClass::SupersessionVerification
            }
            Self::ComparabilityDrift { .. } => VerificationCaseClass::ComparabilityDrift,
            Self::CalibrationCaveat { .. } => VerificationCaseClass::CalibrationCaveat,
            Self::ScopeStale { .. } => VerificationCaseClass::ScopeStale,
        }
    }

    /// Returns the primary claim version carried by this target, if any.
    pub fn primary_claim_version_id(&self) -> Option<ClaimVersionId> {
        match self {
            Self::UnverifiedClaimVersion { claim_version_id }
            | Self::RefutationGap {
                claim_version_id: Some(claim_version_id),
                ..
            }
            | Self::SupersessionVerification {
                claim_version_id, ..
            } => Some(claim_version_id.clone()),
            _ => None,
        }
    }

    /// Returns the stable key used for history tracking and receipts.
    pub fn stable_key(&self) -> String {
        match self {
            Self::FragileNode { node_id, .. } => format!("fragile-{node_id}"),
            Self::ActiveSyndrome { signature } => format!("syndrome-{signature}"),
            Self::ThinExport { marker } => format!("thin-export-{marker}"),
            Self::UnverifiedClaimVersion { claim_version_id } => {
                format!("unverified-{}", claim_version_id.as_str().split_once(':').map(|(_, p)| p).unwrap_or(claim_version_id.as_str()))
            }
            Self::RefutationGap {
                target_node_id,
                claim_version_id,
            } => format!(
                "refutation-gap-{}-{}",
                target_node_id.as_str().split_once(':').map(|(_, p)| p).unwrap_or(target_node_id.as_str()),
                claim_version_id
                    .as_ref()
                    .map(|id| id.as_str().split_once(':').map(|(_, p)| p).unwrap_or(id.as_str()))
                    .unwrap_or("none")
            ),
            Self::SupersessionVerification {
                claim_version_id,
                supersedes_claim_version_id,
            } => format!(
                "supersession-{}-{}",
                claim_version_id.as_str().split_once(':').map(|(_, p)| p).unwrap_or(claim_version_id.as_str()),
                supersedes_claim_version_id.as_str().split_once(':').map(|(_, p)| p).unwrap_or(supersedes_claim_version_id.as_str())
            ),
            Self::ComparabilityDrift { detail } => format!("comparability-{detail}"),
            Self::CalibrationCaveat { marker } => format!("calibration-{marker}"),
            Self::ScopeStale { last_import_at } => {
                format!(
                    "scope-stale-{}",
                    last_import_at.as_deref().unwrap_or("none")
                )
            }
        }
    }

    /// Returns the scheduling priority implied by this target.
    pub fn priority(&self) -> TargetPriority {
        match self {
            Self::ActiveSyndrome { .. } => TargetPriority::ActiveSyndrome,
            Self::FragileNode { .. } => TargetPriority::FragileNode,
            Self::ThinExport { .. } => TargetPriority::ThinExport,
            Self::UnverifiedClaimVersion { .. } => TargetPriority::UnverifiedClaimVersion,
            Self::RefutationGap { .. } => TargetPriority::RefutationGap,
            Self::SupersessionVerification { .. } => TargetPriority::SupersessionVerification,
            Self::ComparabilityDrift { .. } => TargetPriority::ComparabilityDrift,
            Self::CalibrationCaveat { .. } => TargetPriority::CalibrationCaveat,
            Self::ScopeStale { .. } => TargetPriority::ScopeStale,
        }
    }

    /// Returns the budget class implied by this target.
    pub fn budget_class(&self) -> BudgetClass {
        match self {
            Self::ThinExport { .. } | Self::ScopeStale { .. } => BudgetClass::Micro,
            Self::ActiveSyndrome { .. }
            | Self::FragileNode { .. }
            | Self::UnverifiedClaimVersion { .. }
            | Self::RefutationGap { .. }
            | Self::SupersessionVerification { .. }
            | Self::ComparabilityDrift { .. }
            | Self::CalibrationCaveat { .. } => BudgetClass::Standard,
        }
    }

    /// Returns the artifact families required to execute this target lawfully.
    pub fn required_artifact_families(&self) -> Vec<String> {
        let mut families = vec![
            "ExecutionContextV1".into(),
            "RuntimeQueryProvenanceV1".into(),
        ];
        match self {
            Self::ThinExport { .. } => {}
            Self::ComparabilityDrift { .. } => families.push("ComparableWitnessSet".into()),
            Self::SupersessionVerification { .. } => families.push("ReplayArtifact".into()),
            Self::RefutationGap { .. } => families.push("FalsifierWitness".into()),
            _ => families.push("VerificationPlanV1".into()),
        }
        families
    }

    /// Returns the lawful step ladder implied by this target.
    pub fn lawful_step_ladder(&self) -> Vec<LawfulStepKind> {
        match self {
            Self::ActiveSyndrome { .. } => vec![
                LawfulStepKind::ExactOracleSlice,
                LawfulStepKind::ExactReplay,
                LawfulStepKind::MinimalPerturbationRefuter,
                LawfulStepKind::HumanReviewRequest,
            ],
            Self::FragileNode { .. } => vec![
                LawfulStepKind::ContractSchemaCheck,
                LawfulStepKind::ProvenanceReceiptAudit,
                LawfulStepKind::MinimalPerturbationRefuter,
                LawfulStepKind::HumanReviewRequest,
            ],
            Self::ThinExport { .. } => vec![
                LawfulStepKind::ContractSchemaCheck,
                LawfulStepKind::ProvenanceReceiptAudit,
                LawfulStepKind::HumanReviewRequest,
            ],
            Self::UnverifiedClaimVersion { .. } => vec![
                LawfulStepKind::ContractSchemaCheck,
                LawfulStepKind::ProvenanceReceiptAudit,
                LawfulStepKind::TemporalConsistencyCheck,
                LawfulStepKind::ExactOracleSlice,
                LawfulStepKind::ConservativeOracleSlice,
                LawfulStepKind::HumanReviewRequest,
            ],
            Self::RefutationGap { .. } => vec![
                LawfulStepKind::MinimalPerturbationRefuter,
                LawfulStepKind::ExactOracleSlice,
                LawfulStepKind::HumanReviewRequest,
            ],
            Self::SupersessionVerification { .. } => vec![
                LawfulStepKind::TemporalConsistencyCheck,
                LawfulStepKind::ExactReplay,
                LawfulStepKind::HumanReviewRequest,
            ],
            Self::ComparabilityDrift { .. } => vec![
                LawfulStepKind::NuisanceComparabilityAudit,
                LawfulStepKind::PairedComparativeCheck,
                LawfulStepKind::HumanReviewRequest,
            ],
            Self::CalibrationCaveat { .. } => vec![
                LawfulStepKind::ProvenanceReceiptAudit,
                LawfulStepKind::ConservativeOracleSlice,
                LawfulStepKind::HumanReviewRequest,
            ],
            Self::ScopeStale { .. } => vec![
                LawfulStepKind::TemporalConsistencyCheck,
                LawfulStepKind::CanonicalExportRequest,
                LawfulStepKind::CanonicalImportRequest,
                LawfulStepKind::HumanReviewRequest,
            ],
        }
    }
}
