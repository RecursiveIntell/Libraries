//! P12 — Verification plans, claim evidence bundles, and governance control.
//!
//! Risk-bearing claims require verification plans and governance disposition
//! before promotion. Contradictions are tracked through repair records.

use serde::{Deserialize, Serialize};

/// A verification plan attached to a risk-bearing claim.
/// Defines what checks must pass before the claim can be promoted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationPlanV1 {
    /// Unique plan identifier.
    pub plan_id: String,
    /// Claim this plan applies to.
    pub claim_id: String,
    /// Risk class: low, medium, high, critical.
    pub risk_class: RiskClass,
    /// Required cheap checks (fast, always run).
    pub required_cheap_checks: Vec<String>,
    /// Required replay checks (deterministic replay of receipts).
    pub required_replay_checks: Vec<String>,
    /// Required falsification checks (attempt to disprove).
    pub required_falsification_checks: Vec<String>,
    /// Treatment references (what treatment was applied).
    pub treatment_refs: Vec<String>,
    /// Deadline for verification (RFC3339).
    pub deadline: Option<String>,
    /// Current disposition.
    pub disposition: VerificationDisposition,
}

/// Risk classification for claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskClass {
    /// No side effects, no external claims.
    Low,
    /// May affect local state but not external systems.
    Medium,
    /// Affects external systems or makes public claims.
    High,
    /// Affects safety, security, or irreversible operations.
    Critical,
}

/// Verification disposition state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationDisposition {
    /// Plan created, checks not yet run.
    Pending,
    /// Checks are running.
    InProgress,
    /// All checks passed, claim can be promoted.
    Verified,
    /// One or more checks failed.
    Failed,
    /// Contradiction detected, claim quarantined.
    Quarantined,
    /// Verification expired without completion.
    Expired,
}

impl VerificationPlanV1 {
    /// Create a new verification plan for a claim.
    pub fn new(claim_id: &str, risk_class: RiskClass) -> Self {
        let plan_id = format!("vp:{claim_id}:{}", chrono::Utc::now().timestamp());
        Self {
            plan_id,
            claim_id: claim_id.to_string(),
            risk_class,
            required_cheap_checks: vec!["schema_validation".into(), "digest_check".into()],
            required_replay_checks: if risk_class >= RiskClass::Medium {
                vec!["receipt_replay".into()]
            } else {
                vec![]
            },
            required_falsification_checks: if risk_class >= RiskClass::High {
                vec!["counterexample_search".into()]
            } else {
                vec![]
            },
            treatment_refs: vec![],
            deadline: None,
            disposition: VerificationDisposition::Pending,
        }
    }

    /// Check if the plan can be promoted (all checks passed).
    pub fn can_promote(&self) -> bool {
        self.disposition == VerificationDisposition::Verified
    }

    /// Check if the plan is blocked (failed or quarantined).
    pub fn is_blocked(&self) -> bool {
        matches!(
            self.disposition,
            VerificationDisposition::Failed | VerificationDisposition::Quarantined
        )
    }
}

impl PartialOrd for RiskClass {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.ordinal().cmp(&other.ordinal()))
    }
}

impl RiskClass {
    fn ordinal(&self) -> u8 {
        match self {
            RiskClass::Low => 0,
            RiskClass::Medium => 1,
            RiskClass::High => 2,
            RiskClass::Critical => 3,
        }
    }
}

/// A claim evidence bundle — ties a claim to its supporting evidence,
/// treatment, outcome, and potential confounders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimEvidenceBundleV1 {
    /// Unique bundle identifier.
    pub bundle_id: String,
    /// The claim being evidenced.
    pub claim_id: String,
    /// What treatment was applied (e.g., "code_change", "experiment", "analysis").
    pub treatment: String,
    /// What was the outcome (e.g., "pass", "fail", "inconclusive").
    pub outcome: String,
    /// Potential confounders that could explain the result.
    pub confounders: Vec<String>,
    /// Evidence references (receipt IDs, fact IDs, source paths).
    pub evidence_refs: Vec<String>,
    /// Refutation references (attempts to disprove).
    pub refutation_refs: Vec<String>,
    /// Whether the bundle supports or refutes the claim.
    pub supports: bool,
}

impl ClaimEvidenceBundleV1 {
    /// Create a new evidence bundle for a claim.
    pub fn new(claim_id: &str, treatment: &str, outcome: &str) -> Self {
        Self {
            bundle_id: format!("ceb:{claim_id}:{}", chrono::Utc::now().timestamp()),
            claim_id: claim_id.to_string(),
            treatment: treatment.to_string(),
            outcome: outcome.to_string(),
            confounders: vec![],
            evidence_refs: vec![],
            refutation_refs: vec![],
            supports: outcome == "pass",
        }
    }

    /// Add evidence reference.
    pub fn with_evidence(mut self, ref_id: &str) -> Self {
        self.evidence_refs.push(ref_id.to_string());
        self
    }

    /// Add confounder.
    pub fn with_confounder(mut self, confounder: &str) -> Self {
        self.confounders.push(confounder.to_string());
        self
    }
}

/// Governance decision on whether a claim can be promoted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceDecisionV1 {
    /// The decision ID.
    pub decision_id: String,
    /// Claim being decided.
    pub claim_id: String,
    /// Verification plan reference.
    pub verification_plan_id: String,
    /// Decision: promote, reject, quarantine, defer.
    pub decision: GovernanceAction,
    /// Rationale.
    pub rationale: String,
    /// Timestamp (RFC3339).
    pub timestamp: String,
}

/// Governance action types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernanceAction {
    /// Claim verified, promote to accepted status.
    Promote,
    /// Claim failed verification, reject.
    Reject,
    /// Contradiction detected, quarantine.
    Quarantine,
    /// Defer decision pending more evidence.
    Defer,
}

/// Evaluate whether a claim can be promoted given its verification plan
/// and evidence bundles.
pub fn evaluate_promotion(
    plan: &VerificationPlanV1,
    evidence: &[ClaimEvidenceBundleV1],
) -> GovernanceDecisionV1 {
    let decision = if plan.is_blocked() {
        if plan.disposition == VerificationDisposition::Quarantined {
            GovernanceAction::Quarantine
        } else {
            GovernanceAction::Reject
        }
    } else if plan.can_promote() {
        // Check if any evidence refutes the claim
        let has_refutation = evidence.iter().any(|e| !e.supports);
        if has_refutation {
            GovernanceAction::Quarantine
        } else {
            GovernanceAction::Promote
        }
    } else {
        GovernanceAction::Defer
    };

    let rationale = match decision {
        GovernanceAction::Promote => "All checks passed, no refutations found".to_string(),
        GovernanceAction::Reject => "Verification failed".to_string(),
        GovernanceAction::Quarantine => "Contradiction detected or refutation found".to_string(),
        GovernanceAction::Defer => "Verification pending".to_string(),
    };

    GovernanceDecisionV1 {
        decision_id: format!("gd:{}:{}", plan.claim_id, chrono::Utc::now().timestamp()),
        claim_id: plan.claim_id.clone(),
        verification_plan_id: plan.plan_id.clone(),
        decision,
        rationale,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verification_plan_created_pending() {
        let plan = VerificationPlanV1::new("claim-01", RiskClass::High);
        assert_eq!(plan.disposition, VerificationDisposition::Pending);
        assert!(!plan.can_promote());
        assert!(!plan.is_blocked());
    }

    #[test]
    fn high_risk_requires_falsification() {
        let plan = VerificationPlanV1::new("claim-01", RiskClass::High);
        assert!(!plan.required_falsification_checks.is_empty());
    }

    #[test]
    fn low_risk_skips_falsification() {
        let plan = VerificationPlanV1::new("claim-01", RiskClass::Low);
        assert!(plan.required_falsification_checks.is_empty());
        assert!(plan.required_replay_checks.is_empty());
    }

    #[test]
    fn verified_plan_can_promote() {
        let mut plan = VerificationPlanV1::new("claim-01", RiskClass::Low);
        plan.disposition = VerificationDisposition::Verified;
        assert!(plan.can_promote());
    }

    #[test]
    fn failed_plan_is_blocked() {
        let mut plan = VerificationPlanV1::new("claim-01", RiskClass::Low);
        plan.disposition = VerificationDisposition::Failed;
        assert!(plan.is_blocked());
        assert!(!plan.can_promote());
    }

    #[test]
    fn claim_evidence_bundle_marks_support() {
        let bundle = ClaimEvidenceBundleV1::new("claim-01", "experiment", "pass");
        assert!(bundle.supports);
    }

    #[test]
    fn claim_evidence_bundle_marks_refutation() {
        let bundle = ClaimEvidenceBundleV1::new("claim-01", "experiment", "fail");
        assert!(!bundle.supports);
    }

    #[test]
    fn evaluate_promotion_promotes_verified() {
        let mut plan = VerificationPlanV1::new("claim-01", RiskClass::Low);
        plan.disposition = VerificationDisposition::Verified;
        let evidence = vec![ClaimEvidenceBundleV1::new("claim-01", "test", "pass")];
        let decision = evaluate_promotion(&plan, &evidence);
        assert_eq!(decision.decision, GovernanceAction::Promote);
    }

    #[test]
    fn evaluate_promotion_quarantines_with_refutation() {
        let mut plan = VerificationPlanV1::new("claim-01", RiskClass::Low);
        plan.disposition = VerificationDisposition::Verified;
        let evidence = vec![
            ClaimEvidenceBundleV1::new("claim-01", "test", "pass"),
            ClaimEvidenceBundleV1::new("claim-01", "counterexample", "fail"),
        ];
        let decision = evaluate_promotion(&plan, &evidence);
        assert_eq!(decision.decision, GovernanceAction::Quarantine);
    }

    #[test]
    fn evaluate_promotion_defers_pending() {
        let plan = VerificationPlanV1::new("claim-01", RiskClass::Low);
        let evidence = vec![];
        let decision = evaluate_promotion(&plan, &evidence);
        assert_eq!(decision.decision, GovernanceAction::Defer);
    }

    #[test]
    fn evaluate_promotion_rejects_failed() {
        let mut plan = VerificationPlanV1::new("claim-01", RiskClass::Low);
        plan.disposition = VerificationDisposition::Failed;
        let decision = evaluate_promotion(&plan, &[]);
        assert_eq!(decision.decision, GovernanceAction::Reject);
    }

    #[test]
    fn risk_class_ordering() {
        assert!(RiskClass::Critical > RiskClass::High);
        assert!(RiskClass::High > RiskClass::Medium);
        assert!(RiskClass::Medium > RiskClass::Low);
    }
}
