//! P16 — Lawful subtraction, compaction, and invariant-preserving reduction.
//!
//! Subtraction is the dual of accumulation: removal/compaction must preserve
//! declared invariants and emit receipts.

use serde::{Deserialize, Serialize};

/// Type of subtractive operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubtractionOperation {
    /// Remove exact duplicates.
    Dedupe,
    /// Replace detail with summary.
    Summarize,
    /// Compress stored representation.
    Compact,
    /// Remove outdated artifacts.
    Retire,
    /// Move to quarantine (still queryable but excluded).
    Quarantine,
    /// Reduce to minimal support set.
    Minimize,
    /// Extract support core (minimal retained support).
    SupportCoreExtraction,
}

/// A plan for a subtractive operation. Declares what will be removed
/// and what invariants must be preserved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtractionPlanV1 {
    /// Plan identifier.
    pub plan_id: String,
    /// Operation type.
    pub operation: SubtractionOperation,
    /// Target artifact IDs to subtract from.
    pub target_ids: Vec<String>,
    /// Invariants that must be preserved.
    pub preserved_invariants: Vec<InvariantBudgetV1>,
    /// Whether this is a dry-run (no actual mutation).
    pub dry_run: bool,
}

/// Budget for what can be lost during subtraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantBudgetV1 {
    /// What kind of budget.
    pub budget_type: BudgetType,
    /// Maximum items that can be lost.
    pub max_loss: usize,
    /// Current items at risk.
    pub current_at_risk: usize,
    /// Whether the budget is exceeded.
    pub exceeded: bool,
}

/// Types of invariant budgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetType {
    /// Replay budget (can we still replay history?).
    Replay,
    /// As-of query budget (can we still answer temporal queries?).
    AsOfQuery,
    /// Claim support budget (can we still support accepted claims?).
    ClaimSupport,
    /// Receipt lineage budget (can we still trace receipts?).
    ReceiptLineage,
    /// Legal retention budget (are we legally required to keep this?).
    LegalRetention,
}

/// Minimal retained support preserving an invariant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportCoreV1 {
    /// The invariant this support core preserves.
    pub invariant: String,
    /// Minimal artifact IDs that must be retained.
    pub retained_ids: Vec<String>,
    /// Artifact IDs that can be safely removed.
    pub removable_ids: Vec<String>,
    /// Whether the support core was verified.
    pub verified: bool,
}

/// Minimal removal that breaks an invariant. Used for dry-run checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovalFrontierV1 {
    /// The invariant that would be broken.
    pub invariant: String,
    /// The minimal set of removals that breaks it.
    pub removal_set: Vec<String>,
    /// Whether the frontier was found (false = invariant is robust).
    pub found: bool,
}

/// Receipt emitted after a compaction operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionReceiptV1 {
    /// Receipt ID.
    pub receipt_id: String,
    /// Digest before compaction.
    pub before_digest: String,
    /// Digest after compaction.
    pub after_digest: String,
    /// What was lost in compaction.
    pub what_lost: Vec<String>,
    /// What was preserved.
    pub what_preserved: Vec<String>,
    /// Whether as-of queries remain correct.
    pub as_of_queries_correct: bool,
    /// Timestamp.
    pub timestamp: String,
}

impl SubtractionPlanV1 {
    /// Create a new subtraction plan.
    pub fn new(operation: SubtractionOperation, target_ids: Vec<String>) -> Self {
        Self {
            plan_id: format!(
                "sp:{}:{}",
                match operation {
                    SubtractionOperation::Dedupe => "dedupe",
                    SubtractionOperation::Summarize => "summarize",
                    SubtractionOperation::Compact => "compact",
                    SubtractionOperation::Retire => "retire",
                    SubtractionOperation::Quarantine => "quarantine",
                    SubtractionOperation::Minimize => "minimize",
                    SubtractionOperation::SupportCoreExtraction => "support-core",
                },
                chrono::Utc::now().timestamp()
            ),
            operation,
            target_ids,
            preserved_invariants: vec![],
            dry_run: true,
        }
    }

    /// Add an invariant budget.
    pub fn with_invariant(mut self, budget: InvariantBudgetV1) -> Self {
        self.preserved_invariants.push(budget);
        self
    }

    /// Check if the plan can proceed (no budgets exceeded).
    pub fn can_proceed(&self) -> bool {
        !self.preserved_invariants.iter().any(|b| b.exceeded)
    }
}

impl SupportCoreV1 {
    /// Check if removing an artifact would break this support core.
    pub fn would_break(&self, artifact_id: &str) -> bool {
        self.retained_ids.contains(&artifact_id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtraction_plan_starts_as_dry_run() {
        let plan = SubtractionPlanV1::new(SubtractionOperation::Compact, vec!["a1".into()]);
        assert!(plan.dry_run);
        assert!(plan.can_proceed());
    }

    #[test]
    fn plan_blocked_when_budget_exceeded() {
        let plan = SubtractionPlanV1::new(SubtractionOperation::Compact, vec!["a1".into()])
            .with_invariant(InvariantBudgetV1 {
                budget_type: BudgetType::Replay,
                max_loss: 10,
                current_at_risk: 15,
                exceeded: true,
            });
        assert!(!plan.can_proceed());
    }

    #[test]
    fn support_core_would_break_on_retained() {
        let core = SupportCoreV1 {
            invariant: "replay".into(),
            retained_ids: vec!["a1".into(), "a2".into()],
            removable_ids: vec!["a3".into()],
            verified: true,
        };
        assert!(core.would_break("a1"));
        assert!(!core.would_break("a3"));
    }

    #[test]
    fn removal_frontier_found() {
        let frontier = RemovalFrontierV1 {
            invariant: "claim_support".into(),
            removal_set: vec!["a1".into()],
            found: true,
        };
        assert!(frontier.found);
    }

    #[test]
    fn compaction_receipt_records_before_after() {
        let receipt = CompactionReceiptV1 {
            receipt_id: "cr:1".into(),
            before_digest: "abc".into(),
            after_digest: "def".into(),
            what_lost: vec!["old_receipts".into()],
            what_preserved: vec!["current_claims".into()],
            as_of_queries_correct: true,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        assert_ne!(receipt.before_digest, receipt.after_digest);
        assert!(receipt.as_of_queries_correct);
    }

    #[test]
    fn subtraction_preserves_invariant_or_fails() {
        // If budget is exceeded, plan cannot proceed
        let plan = SubtractionPlanV1::new(SubtractionOperation::Retire, vec!["a1".into()])
            .with_invariant(InvariantBudgetV1 {
                budget_type: BudgetType::ClaimSupport,
                max_loss: 0,
                current_at_risk: 1,
                exceeded: true,
            });
        assert!(
            !plan.can_proceed(),
            "subtraction must not proceed when invariant budget is exceeded"
        );
    }
}
