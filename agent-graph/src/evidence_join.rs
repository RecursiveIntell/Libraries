//! Owner-resolved, bounded evidence joins.
//!
//! This module owns only orchestration concerns: bounded branch projection,
//! deterministic source grouping, join state, and terminal disposition. It does
//! not interpret receipts or decide whether evidence supports claims. Those
//! decisions are delegated to an injected [`EvidenceJoinOracle`].

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use thiserror::Error;

/// Hard ceilings applied to an evidence-join projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceJoinLimits {
    /// Maximum number of branch projections admitted to one join.
    pub max_branches: usize,
    /// Maximum serialized size of the complete bounded input.
    pub max_projection_bytes: usize,
}

impl Default for EvidenceJoinLimits {
    fn default() -> Self {
        Self {
            max_branches: 64,
            max_projection_bytes: 65_536,
        }
    }
}

/// Claim surface supplied to the external evidence-support owner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimProjection {
    pub claim_ref: String,
    pub scope: ClaimScope,
}

/// Scope of the claim under adjudication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ClaimScope {
    NaturalLanguage,
    NumericalMethod { method_ref: String },
    Other { scope_ref: String },
}

/// Scope in which an evidence item was produced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum EvidenceScope {
    NaturalLanguage,
    NumericalMethod { method_ref: String },
    Other { scope_ref: String },
}

/// Bounded evidence reference retained for owner verification and drill-down.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRefProjection {
    pub evidence_ref: String,
    /// Correlation data only. Digest presence is not evidence support.
    pub digest: Option<String>,
    pub scope: EvidenceScope,
}

/// Bounded required-check reference retained for owner verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredCheckProjection {
    pub check_ref: String,
    pub receipt_ref: Option<String>,
    /// Display/correlation data only; the oracle decides receipt validity.
    pub status: Option<String>,
    /// Display/correlation data passed to the receipt owner for verification.
    pub executed_count: Option<u64>,
}

/// Unbounded branch candidate accepted only by the projection boundary.
///
/// `payload` is deliberately discarded. Only the typed obligation, evidence,
/// check, source, gap, and counterexample references cross into join state.
#[derive(Debug, Clone)]
pub struct BranchEvidenceCandidate {
    pub branch_ref: String,
    pub obligation_refs: Vec<String>,
    pub evidence: Vec<EvidenceRefProjection>,
    pub required_checks: Vec<RequiredCheckProjection>,
    pub source_group_ref: String,
    pub source_epoch: String,
    pub mandatory_gap_refs: Vec<String>,
    pub counterexample_refs: Vec<String>,
    pub abstained: bool,
    pub native_completed: bool,
    /// Display/correlation data only.
    pub status: Option<String>,
    /// Display/correlation data only.
    pub support_score: Option<f64>,
    pub payload: Value,
}

/// Serializable branch projection used by the join.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BranchEvidenceProjection {
    pub branch_ref: String,
    pub obligation_refs: Vec<String>,
    pub evidence: Vec<EvidenceRefProjection>,
    pub required_checks: Vec<RequiredCheckProjection>,
    pub source_group_ref: String,
    pub source_epoch: String,
    pub mandatory_gap_refs: Vec<String>,
    pub counterexample_refs: Vec<String>,
    pub abstained: bool,
    pub native_completed: bool,
    /// Display/correlation data only.
    pub status: Option<String>,
    /// Display/correlation data only.
    pub support_score: Option<f64>,
}

impl From<BranchEvidenceCandidate> for BranchEvidenceProjection {
    fn from(candidate: BranchEvidenceCandidate) -> Self {
        let BranchEvidenceCandidate {
            branch_ref,
            obligation_refs,
            evidence,
            required_checks,
            source_group_ref,
            source_epoch,
            mandatory_gap_refs,
            counterexample_refs,
            abstained,
            native_completed,
            status,
            support_score,
            payload: _,
        } = candidate;
        Self {
            branch_ref,
            obligation_refs,
            evidence,
            required_checks,
            source_group_ref,
            source_epoch,
            mandatory_gap_refs,
            counterexample_refs,
            abstained,
            native_completed,
            status,
            support_score,
        }
    }
}

/// Budget state projected into the join by the runtime owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JoinBudgetProjection {
    pub exhausted: bool,
    pub required_checks_remaining: usize,
}

/// Pre-projection request. Large branch payloads exist only on this side of the
/// bounded owner boundary.
#[derive(Debug, Clone)]
pub struct EvidenceJoinSeed {
    pub target_revision: String,
    pub claim: ClaimProjection,
    pub branches: Vec<BranchEvidenceCandidate>,
    pub cross_cutting_obligation_refs: Vec<String>,
    pub budget: JoinBudgetProjection,
}

/// Serializable bounded input to an owner-resolved evidence join.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceJoinInput {
    pub target_revision: String,
    pub claim: ClaimProjection,
    pub branches: Vec<BranchEvidenceProjection>,
    pub cross_cutting_obligation_refs: Vec<String>,
    pub budget: JoinBudgetProjection,
}

/// External owner decision. Unavailable decisions always fail closed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OracleDecision {
    Confirmed,
    Rejected,
    #[default]
    Unavailable,
}

/// Receipt-verification query sent to the canonical check owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckReceiptVerification {
    pub target_revision: String,
    pub branch_ref: String,
    pub check_ref: String,
    pub receipt_ref: String,
    pub declared_status: Option<String>,
    pub executed_count: Option<u64>,
}

/// Evidence-support query sent to the canonical claim/evidence owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceSupportVerification {
    pub target_revision: String,
    pub branch_ref: String,
    pub claim_ref: String,
    pub claim_scope: ClaimScope,
    pub evidence_ref: String,
    pub evidence_digest: Option<String>,
    pub evidence_scope: EvidenceScope,
}

/// Source-epoch compatibility query sent to the canonical source owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEpochCompatibilityVerification {
    pub target_revision: String,
    pub source_epochs: Vec<String>,
}

/// Parent obligation query sent to the canonical obligation owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossCuttingObligationVerification {
    pub target_revision: String,
    pub claim_ref: String,
    pub obligation_ref: String,
}

/// Counterexample query sent to the canonical claim/evidence owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterexampleVerification {
    pub target_revision: String,
    pub claim_ref: String,
    pub branch_ref: String,
    pub counterexample_ref: String,
}

/// Non-serializable authority boundary for evidence-join semantics.
///
/// Agent Graph never mints, verifies, or interprets receipts, claims, source
/// epochs, obligations, or counterexamples itself.
pub trait EvidenceJoinOracle: Send + Sync {
    fn verify_check_receipt(&self, query: &CheckReceiptVerification) -> OracleDecision;

    fn verify_evidence_support(&self, query: &EvidenceSupportVerification) -> OracleDecision;

    fn verify_source_epoch_compatibility(
        &self,
        query: &SourceEpochCompatibilityVerification,
    ) -> OracleDecision;

    fn verify_cross_cutting_obligation(
        &self,
        query: &CrossCuttingObligationVerification,
    ) -> OracleDecision;

    fn validate_counterexample(&self, query: &CounterexampleVerification) -> OracleDecision;
}

/// Terminal join disposition owned by Agent Graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JoinDisposition {
    Pass,
    Blocked,
    Unsupported,
    Reopen,
}

/// Stable reason codes explaining a non-pass disposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JoinReasonCode {
    MissingCheckReceipt,
    CheckReceiptRejected,
    CheckVerifierUnavailable,
    EvidenceUnsupported,
    EvidenceSupportUnavailable,
    SourceEpochIncompatible,
    SourceEpochCompatibilityUnavailable,
    CrossCuttingObligationRejected,
    CrossCuttingObligationUnavailable,
    BranchAbstained,
    MandatoryGap,
    CounterexampleValidated,
    CounterexampleValidationUnavailable,
    BudgetExhausted,
}

/// One disposition reason with a stable drill-down reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JoinReason {
    pub code: JoinReasonCode,
    pub reference: String,
}

/// Per-branch drill-down projection retained in the result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BranchJoinResolution {
    pub branch_ref: String,
    pub obligation_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub required_check_refs: Vec<String>,
    pub source_group_ref: String,
    pub source_epoch: String,
    pub mandatory_gap_refs: Vec<String>,
    pub counterexample_refs: Vec<String>,
    pub abstained: bool,
    pub native_completed: bool,
}

/// Serializable result of an owner-resolved evidence join.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceJoinResult {
    pub disposition: JoinDisposition,
    pub unknown_or_unavailable: bool,
    pub reasons: Vec<JoinReason>,
    pub branches: Vec<BranchJoinResolution>,
    pub cross_cutting_obligation_refs: Vec<String>,
    pub source_group_refs: Vec<String>,
    pub source_epochs: Vec<String>,
    pub independent_source_count: usize,
    pub abstained_branch_refs: Vec<String>,
    pub mandatory_gap_refs: Vec<String>,
    pub validated_counterexample_refs: Vec<String>,
    pub native_completed_branch_refs: Vec<String>,
}

/// Projection or admission failure at the bounded Graph-owned boundary.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EvidenceJoinError {
    #[error("evidence join has {actual} branches; maximum is {max}")]
    TooManyBranches { actual: usize, max: usize },
    #[error("evidence join projection is {actual} bytes; maximum is {max}")]
    ProjectionTooLarge { actual: usize, max: usize },
    #[error("evidence join projection serialization failed: {0}")]
    Serialization(String),
}

/// Bounded evidence-join owner surface.
#[derive(Debug, Clone, Copy)]
pub struct EvidenceJoin {
    limits: EvidenceJoinLimits,
}

impl Default for EvidenceJoin {
    fn default() -> Self {
        Self::new(EvidenceJoinLimits::default())
    }
}

impl EvidenceJoin {
    pub const fn new(limits: EvidenceJoinLimits) -> Self {
        Self { limits }
    }

    pub const fn limits(&self) -> EvidenceJoinLimits {
        self.limits
    }

    /// Discard branch payloads and admit only a bounded typed projection.
    pub fn project(&self, seed: EvidenceJoinSeed) -> Result<EvidenceJoinInput, EvidenceJoinError> {
        let input = EvidenceJoinInput {
            target_revision: seed.target_revision,
            claim: seed.claim,
            branches: seed.branches.into_iter().map(Into::into).collect(),
            cross_cutting_obligation_refs: seed.cross_cutting_obligation_refs,
            budget: seed.budget,
        };
        self.validate_bounds(&input)?;
        Ok(input)
    }

    /// Resolve a bounded join exclusively through injected owner decisions.
    pub fn resolve(
        &self,
        input: &EvidenceJoinInput,
        oracle: &dyn EvidenceJoinOracle,
    ) -> Result<EvidenceJoinResult, EvidenceJoinError> {
        self.validate_bounds(input)?;

        let mut reasons = Vec::new();
        let mut branches = Vec::with_capacity(input.branches.len());
        let mut source_groups = BTreeSet::new();
        let mut source_epochs = BTreeSet::new();
        let mut abstained_branch_refs = Vec::new();
        let mut mandatory_gap_refs = Vec::new();
        let mut validated_counterexample_refs = Vec::new();
        let mut native_completed_branch_refs = Vec::new();
        let mut blocked = false;
        let mut unsupported = false;
        let mut reopen = false;
        let mut unknown_or_unavailable = false;

        for branch in &input.branches {
            source_groups.insert(branch.source_group_ref.clone());
            source_epochs.insert(branch.source_epoch.clone());

            if branch.abstained {
                blocked = true;
                abstained_branch_refs.push(branch.branch_ref.clone());
                push_reason(
                    &mut reasons,
                    JoinReasonCode::BranchAbstained,
                    &branch.branch_ref,
                );
            }
            for gap_ref in &branch.mandatory_gap_refs {
                blocked = true;
                mandatory_gap_refs.push(gap_ref.clone());
                push_reason(&mut reasons, JoinReasonCode::MandatoryGap, gap_ref);
            }
            if branch.native_completed {
                native_completed_branch_refs.push(branch.branch_ref.clone());
            }

            for check in &branch.required_checks {
                let Some(receipt_ref) = &check.receipt_ref else {
                    blocked = true;
                    push_reason(
                        &mut reasons,
                        JoinReasonCode::MissingCheckReceipt,
                        &check.check_ref,
                    );
                    continue;
                };
                let query = CheckReceiptVerification {
                    target_revision: input.target_revision.clone(),
                    branch_ref: branch.branch_ref.clone(),
                    check_ref: check.check_ref.clone(),
                    receipt_ref: receipt_ref.clone(),
                    declared_status: check.status.clone(),
                    executed_count: check.executed_count,
                };
                match oracle.verify_check_receipt(&query) {
                    OracleDecision::Confirmed => {}
                    OracleDecision::Rejected => {
                        blocked = true;
                        push_reason(
                            &mut reasons,
                            JoinReasonCode::CheckReceiptRejected,
                            receipt_ref,
                        );
                    }
                    OracleDecision::Unavailable => {
                        blocked = true;
                        unknown_or_unavailable = true;
                        push_reason(
                            &mut reasons,
                            JoinReasonCode::CheckVerifierUnavailable,
                            receipt_ref,
                        );
                    }
                }
            }

            for evidence in &branch.evidence {
                let query = EvidenceSupportVerification {
                    target_revision: input.target_revision.clone(),
                    branch_ref: branch.branch_ref.clone(),
                    claim_ref: input.claim.claim_ref.clone(),
                    claim_scope: input.claim.scope.clone(),
                    evidence_ref: evidence.evidence_ref.clone(),
                    evidence_digest: evidence.digest.clone(),
                    evidence_scope: evidence.scope.clone(),
                };
                match oracle.verify_evidence_support(&query) {
                    OracleDecision::Confirmed => {}
                    OracleDecision::Rejected => {
                        unsupported = true;
                        push_reason(
                            &mut reasons,
                            JoinReasonCode::EvidenceUnsupported,
                            &evidence.evidence_ref,
                        );
                    }
                    OracleDecision::Unavailable => {
                        blocked = true;
                        unknown_or_unavailable = true;
                        push_reason(
                            &mut reasons,
                            JoinReasonCode::EvidenceSupportUnavailable,
                            &evidence.evidence_ref,
                        );
                    }
                }
            }

            for counterexample_ref in &branch.counterexample_refs {
                let query = CounterexampleVerification {
                    target_revision: input.target_revision.clone(),
                    claim_ref: input.claim.claim_ref.clone(),
                    branch_ref: branch.branch_ref.clone(),
                    counterexample_ref: counterexample_ref.clone(),
                };
                match oracle.validate_counterexample(&query) {
                    OracleDecision::Confirmed => {
                        reopen = true;
                        validated_counterexample_refs.push(counterexample_ref.clone());
                        push_reason(
                            &mut reasons,
                            JoinReasonCode::CounterexampleValidated,
                            counterexample_ref,
                        );
                    }
                    OracleDecision::Rejected => {}
                    OracleDecision::Unavailable => {
                        blocked = true;
                        unknown_or_unavailable = true;
                        push_reason(
                            &mut reasons,
                            JoinReasonCode::CounterexampleValidationUnavailable,
                            counterexample_ref,
                        );
                    }
                }
            }

            branches.push(BranchJoinResolution {
                branch_ref: branch.branch_ref.clone(),
                obligation_refs: branch.obligation_refs.clone(),
                evidence_refs: branch
                    .evidence
                    .iter()
                    .map(|evidence| evidence.evidence_ref.clone())
                    .collect(),
                required_check_refs: branch
                    .required_checks
                    .iter()
                    .map(|check| check.check_ref.clone())
                    .collect(),
                source_group_ref: branch.source_group_ref.clone(),
                source_epoch: branch.source_epoch.clone(),
                mandatory_gap_refs: branch.mandatory_gap_refs.clone(),
                counterexample_refs: branch.counterexample_refs.clone(),
                abstained: branch.abstained,
                native_completed: branch.native_completed,
            });
        }

        let source_epochs: Vec<_> = source_epochs.into_iter().collect();
        if source_epochs.len() > 1 {
            let query = SourceEpochCompatibilityVerification {
                target_revision: input.target_revision.clone(),
                source_epochs: source_epochs.clone(),
            };
            match oracle.verify_source_epoch_compatibility(&query) {
                OracleDecision::Confirmed => {}
                OracleDecision::Rejected => {
                    blocked = true;
                    push_reason(
                        &mut reasons,
                        JoinReasonCode::SourceEpochIncompatible,
                        &source_epochs.join(","),
                    );
                }
                OracleDecision::Unavailable => {
                    blocked = true;
                    unknown_or_unavailable = true;
                    push_reason(
                        &mut reasons,
                        JoinReasonCode::SourceEpochCompatibilityUnavailable,
                        &source_epochs.join(","),
                    );
                }
            }
        }

        for obligation_ref in &input.cross_cutting_obligation_refs {
            let query = CrossCuttingObligationVerification {
                target_revision: input.target_revision.clone(),
                claim_ref: input.claim.claim_ref.clone(),
                obligation_ref: obligation_ref.clone(),
            };
            match oracle.verify_cross_cutting_obligation(&query) {
                OracleDecision::Confirmed => {}
                OracleDecision::Rejected => {
                    blocked = true;
                    push_reason(
                        &mut reasons,
                        JoinReasonCode::CrossCuttingObligationRejected,
                        obligation_ref,
                    );
                }
                OracleDecision::Unavailable => {
                    blocked = true;
                    unknown_or_unavailable = true;
                    push_reason(
                        &mut reasons,
                        JoinReasonCode::CrossCuttingObligationUnavailable,
                        obligation_ref,
                    );
                }
            }
        }

        if input.budget.exhausted && input.budget.required_checks_remaining > 0 {
            blocked = true;
            push_reason(
                &mut reasons,
                JoinReasonCode::BudgetExhausted,
                &input.budget.required_checks_remaining.to_string(),
            );
        }

        if !input
            .branches
            .iter()
            .any(|branch| !branch.evidence.is_empty())
        {
            unsupported = true;
            push_reason(
                &mut reasons,
                JoinReasonCode::EvidenceUnsupported,
                &input.claim.claim_ref,
            );
        }
        let disposition = if reopen {
            JoinDisposition::Reopen
        } else if unsupported {
            JoinDisposition::Unsupported
        } else if blocked {
            JoinDisposition::Blocked
        } else {
            JoinDisposition::Pass
        };
        let source_group_refs: Vec<_> = source_groups.into_iter().collect();

        Ok(EvidenceJoinResult {
            disposition,
            unknown_or_unavailable,
            reasons,
            branches,
            cross_cutting_obligation_refs: input.cross_cutting_obligation_refs.clone(),
            independent_source_count: source_group_refs.len(),
            source_group_refs,
            source_epochs,
            abstained_branch_refs,
            mandatory_gap_refs,
            validated_counterexample_refs,
            native_completed_branch_refs,
        })
    }

    fn validate_bounds(&self, input: &EvidenceJoinInput) -> Result<(), EvidenceJoinError> {
        if input.branches.len() > self.limits.max_branches {
            return Err(EvidenceJoinError::TooManyBranches {
                actual: input.branches.len(),
                max: self.limits.max_branches,
            });
        }
        let actual = serde_json::to_vec(input)
            .map_err(|error| EvidenceJoinError::Serialization(error.to_string()))?
            .len();
        if actual > self.limits.max_projection_bytes {
            return Err(EvidenceJoinError::ProjectionTooLarge {
                actual,
                max: self.limits.max_projection_bytes,
            });
        }
        Ok(())
    }
}

fn push_reason(reasons: &mut Vec<JoinReason>, code: JoinReasonCode, reference: &str) {
    reasons.push(JoinReason {
        code,
        reference: reference.to_owned(),
    });
}
