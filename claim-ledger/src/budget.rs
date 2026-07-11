//! Proof-debt budget accounting.
//!
//! FEUT-004: proof obligations + entropy/budget accounting + local artifact
//! receipts. This module makes proof debt a first-class runtime budget rather
//! than a categorical tag.
//!
//! ## Model
//!
//! Every claim or operation that makes an assertion without complete proof
//! incurs proof debt. The debt is quantified in micros (1_000_000 = one full
//! proof unit). A budget envelope sets the maximum debt allowed before gates
//! fire. Operations consume debt; admissions and evidence replenish it.
//!
//! ```text
//! ProofDebtBudgetV1 {
//!     budget_micros: 500_000,       // max allowed debt
//!     consumed_micros: 120_000,     // current debt
//!     available_micros: 380_000,    // budget - consumed
//! }
//! ```
//!
//! When `consumed_micros > budget_micros`, the budget is exhausted and the
//! gate fires: the claim must be retracted, degraded, or explicitly waived
//! by an operator with a receipt.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids;
use crate::types::ProofDebt;

/// Configurable weights and gate thresholds for proof-debt budgeting.
///
/// All values are in micros (1_000_000 = one full proof unit).
/// This struct allows operators to tune the budget system without changing
/// the code. The defaults are conservative: source basis is the heaviest
/// debt, and gates fire at 80% / 100% / 120%.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofDebtBudgetConfig {
    /// Weight for MissingSourceBasis debt.
    pub weight_missing_source_basis: u64,
    /// Weight for MissingRepro debt.
    pub weight_missing_repro: u64,
    /// Weight for MissingBenchmark debt.
    pub weight_missing_benchmark: u64,
    /// Weight for MissingExternalValidation debt.
    pub weight_missing_external_validation: u64,
    /// Gate threshold for Warn (percentage, 0-100).
    pub warn_threshold_pct: u64,
    /// Gate threshold for Degrade (percentage, 0-100).
    pub degrade_threshold_pct: u64,
    /// Gate threshold for Retract (percentage, 0-100+).
    pub retract_threshold_pct: u64,
}

impl Default for ProofDebtBudgetConfig {
    fn default() -> Self {
        Self {
            weight_missing_source_basis: 250_000,
            weight_missing_repro: 150_000,
            weight_missing_benchmark: 100_000,
            weight_missing_external_validation: 75_000,
            warn_threshold_pct: 80,
            degrade_threshold_pct: 100,
            retract_threshold_pct: 120,
        }
    }
}

impl ProofDebtBudgetConfig {
    /// Get the weight for a ProofDebt kind under this config.
    pub fn weight(&self, debt: &ProofDebt) -> u64 {
        match debt {
            ProofDebt::MissingSourceBasis => self.weight_missing_source_basis,
            ProofDebt::MissingRepro => self.weight_missing_repro,
            ProofDebt::MissingBenchmark => self.weight_missing_benchmark,
            ProofDebt::MissingExternalValidation => self.weight_missing_external_validation,
            ProofDebt::None => 0,
        }
    }
}

/// Proof-debt weight per debt kind in micros (1_000_000 = one full proof unit).
///
/// Uses default config weights. For custom weights, use
/// [`ProofDebtBudgetConfig::weight`].
pub fn proof_debt_weight(debt: &ProofDebt) -> u64 {
    ProofDebtBudgetConfig::default().weight(debt)
}

/// A proof-debt budget envelope.
///
/// Sets the maximum allowed proof debt for a scope (claim, case, or session).
/// The budget is consumed by operations that incur proof debt and replenished
/// by evidence and admissions. Waivers authorize bounded proceeding but never
/// alter the outstanding debt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofDebtBudgetV1 {
    /// Schema version.
    pub schema_version: String,
    /// Unique identifier for this budget envelope.
    pub budget_id: String,
    /// Scope this budget applies to (e.g., claim_id, case_id, "session").
    pub scope: String,
    /// Maximum proof debt allowed in micros.
    pub budget_micros: u64,
    /// Current consumed proof debt in micros.
    pub consumed_micros: u64,
    /// When this budget was created.
    pub created_at: DateTime<Utc>,
    /// When this budget was last updated.
    pub updated_at: DateTime<Utc>,
}

impl ProofDebtBudgetV1 {
    /// Create a new proof-debt budget for a scope.
    pub fn new(scope: &str, budget_micros: u64) -> Self {
        Self::new_at(scope, budget_micros, Utc::now())
    }

    /// Create a budget with an explicit recorded time for reproducible artifacts.
    pub fn new_at(scope: &str, budget_micros: u64, recorded_time: DateTime<Utc>) -> Self {
        Self {
            schema_version: "ProofDebtBudgetV1".to_string(),
            budget_id: ids::proof_debt_budget_id(scope),
            scope: scope.to_string(),
            budget_micros,
            consumed_micros: 0,
            created_at: recorded_time,
            updated_at: recorded_time,
        }
    }

    /// Available proof debt remaining in the budget.
    pub fn available_micros(&self) -> u64 {
        self.budget_micros.saturating_sub(self.consumed_micros)
    }

    /// Whether the budget is exhausted (consumed >= budget).
    pub fn is_exhausted(&self) -> bool {
        self.consumed_micros >= self.budget_micros
    }

    /// Percentage of budget consumed (0-100).
    pub fn consumed_pct(&self) -> u64 {
        if self.budget_micros == 0 {
            return 100;
        }
        (self.consumed_micros * 100) / self.budget_micros
    }

    /// Consume proof debt from the budget. Returns the debit record.
    ///
    /// Returns `None` if the consumption would overflow the budget and
    /// `strict` is true. In non-strict mode, the budget is marked exhausted
    /// and the debit is recorded with `overdrawn: true`.
    pub fn consume(
        &mut self,
        amount_micros: u64,
        source: &str,
        rationale: &str,
        strict: bool,
    ) -> Option<ProofDebtDebitV1> {
        self.consume_at(amount_micros, source, rationale, strict, Utc::now())
    }

    /// Consume proof debt with an explicit recorded time.
    pub fn consume_at(
        &mut self,
        amount_micros: u64,
        source: &str,
        rationale: &str,
        strict: bool,
        recorded_time: DateTime<Utc>,
    ) -> Option<ProofDebtDebitV1> {
        let new_consumed = self.consumed_micros.saturating_add(amount_micros);
        let overdrawn = new_consumed > self.budget_micros;
        if overdrawn && strict {
            return None;
        }
        let remaining = self.budget_micros.saturating_sub(new_consumed);
        self.consumed_micros = new_consumed;
        self.updated_at = recorded_time;
        Some(ProofDebtDebitV1::new_at(
            &self.budget_id,
            amount_micros,
            remaining,
            overdrawn,
            source,
            rationale,
            recorded_time,
        ))
    }

    /// Replenish proof debt (e.g., when evidence is added or a claim is admitted).
    pub fn replenish(
        &mut self,
        amount_micros: u64,
        source: &str,
        rationale: &str,
    ) -> ProofDebtCreditV1 {
        self.replenish_at(amount_micros, source, rationale, Utc::now())
    }

    /// Replenish proof debt with an explicit recorded time.
    pub fn replenish_at(
        &mut self,
        amount_micros: u64,
        source: &str,
        rationale: &str,
        recorded_time: DateTime<Utc>,
    ) -> ProofDebtCreditV1 {
        self.consumed_micros = self.consumed_micros.saturating_sub(amount_micros);
        self.updated_at = recorded_time;
        ProofDebtCreditV1::new_at(
            &self.budget_id,
            amount_micros,
            source,
            rationale,
            recorded_time,
        )
    }

    /// Consume debt for a specific ProofDebt kind, using the standard weight.
    pub fn consume_debt(
        &mut self,
        debt: &ProofDebt,
        source: &str,
        rationale: &str,
        strict: bool,
    ) -> Option<ProofDebtDebitV1> {
        let weight = proof_debt_weight(debt);
        if weight == 0 {
            return None;
        }
        self.consume(weight, source, rationale, strict)
    }

    /// Consume debt for a specific ProofDebt kind, using a custom config's weight.
    pub fn consume_debt_with_config(
        &mut self,
        debt: &ProofDebt,
        config: &ProofDebtBudgetConfig,
        source: &str,
        rationale: &str,
        strict: bool,
    ) -> Option<ProofDebtDebitV1> {
        let weight = config.weight(debt);
        if weight == 0 {
            return None;
        }
        self.consume(weight, source, rationale, strict)
    }

    /// Replenish debt when a ProofDebt is resolved (e.g., source basis found).
    pub fn resolve_debt(
        &mut self,
        debt: &ProofDebt,
        source: &str,
        rationale: &str,
    ) -> Option<ProofDebtCreditV1> {
        let weight = proof_debt_weight(debt);
        if weight == 0 {
            return None;
        }
        Some(self.replenish(weight, source, rationale))
    }

    /// Replenish debt when a ProofDebt is resolved, using a custom config's weight.
    pub fn resolve_debt_with_config(
        &mut self,
        debt: &ProofDebt,
        config: &ProofDebtBudgetConfig,
        source: &str,
        rationale: &str,
    ) -> Option<ProofDebtCreditV1> {
        let weight = config.weight(debt);
        if weight == 0 {
            return None;
        }
        Some(self.replenish(weight, source, rationale))
    }
}

/// A debit against a proof-debt budget.
///
/// Every operation that incurs proof debt produces a debit record. This is
/// the receipt for budget consumption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofDebtDebitV1 {
    /// Schema version.
    pub schema_version: String,
    /// Unique identifier for this debit.
    pub debit_id: String,
    /// Budget this debit was charged against.
    pub budget_id: String,
    /// Amount consumed in micros.
    pub amount_micros: u64,
    /// Remaining budget after this debit.
    pub remaining_micros: u64,
    /// Whether this debit caused the budget to be overdrawn.
    pub overdrawn: bool,
    /// Source of the debit (e.g., "claim:clm_abc", "operation:verify_node").
    pub source: String,
    /// Why the debit was incurred.
    pub rationale: String,
    /// When this debit was recorded.
    pub recorded_time: DateTime<Utc>,
}

impl ProofDebtDebitV1 {
    /// Create a new proof-debt debit record.
    pub fn new(
        budget_id: &str,
        amount_micros: u64,
        remaining_micros: u64,
        overdrawn: bool,
        source: &str,
        rationale: &str,
    ) -> Self {
        Self::new_at(
            budget_id,
            amount_micros,
            remaining_micros,
            overdrawn,
            source,
            rationale,
            Utc::now(),
        )
    }

    /// Create a debit record with an explicit recorded time.
    pub fn new_at(
        budget_id: &str,
        amount_micros: u64,
        remaining_micros: u64,
        overdrawn: bool,
        source: &str,
        rationale: &str,
        recorded_time: DateTime<Utc>,
    ) -> Self {
        Self {
            schema_version: "ProofDebtDebitV1".to_string(),
            debit_id: ids::proof_debt_debit_id(budget_id, source, amount_micros),
            budget_id: budget_id.to_string(),
            amount_micros,
            remaining_micros,
            overdrawn,
            source: source.to_string(),
            rationale: rationale.to_string(),
            recorded_time,
        }
    }
}

/// A credit (replenishment) to a proof-debt budget.
///
/// When evidence is added, a claim is admitted, or proof debt is waived,
/// the budget is replenished. This is the receipt for budget restoration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofDebtCreditV1 {
    /// Schema version.
    pub schema_version: String,
    /// Unique identifier for this credit.
    pub credit_id: String,
    /// Budget this credit was applied to.
    pub budget_id: String,
    /// Amount replenished in micros.
    pub amount_micros: u64,
    /// Source of the credit (e.g., "admission:clm_abc", "evidence:evb_xyz").
    pub source: String,
    /// Why the credit was applied.
    pub rationale: String,
    /// When this credit was recorded.
    pub recorded_time: DateTime<Utc>,
}

impl ProofDebtCreditV1 {
    /// Create a new proof-debt credit record.
    pub fn new(budget_id: &str, amount_micros: u64, source: &str, rationale: &str) -> Self {
        Self::new_at(budget_id, amount_micros, source, rationale, Utc::now())
    }

    /// Create a credit record with an explicit recorded time.
    pub fn new_at(
        budget_id: &str,
        amount_micros: u64,
        source: &str,
        rationale: &str,
        recorded_time: DateTime<Utc>,
    ) -> Self {
        Self {
            schema_version: "ProofDebtCreditV1".to_string(),
            credit_id: ids::proof_debt_credit_id(budget_id, source, amount_micros),
            budget_id: budget_id.to_string(),
            amount_micros,
            source: source.to_string(),
            rationale: rationale.to_string(),
            recorded_time,
        }
    }
}

/// Gate decision when a proof-debt budget is checked.
///
/// The gate fires when the budget is exhausted or near exhaustion. The
/// decision determines what happens to the claim or operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofDebtGateDecision {
    /// Budget has sufficient remaining — proceed normally.
    Proceed,
    /// Budget is near exhaustion (>= 80% consumed) — warn but proceed.
    Warn,
    /// Budget is exhausted — the claim must be degraded to a weaker state.
    Degrade,
    /// Budget is exhausted — the claim must be retracted.
    Retract,
    /// Budget is exhausted — an operator has explicitly waived the debt.
    Waived,
}

impl ProofDebtGateDecision {
    /// Returns true if the decision allows the operation to proceed.
    pub fn allows_proceed(&self) -> bool {
        matches!(self, Self::Proceed | Self::Warn | Self::Waived)
    }

    /// Returns true if the decision blocks the operation.
    pub fn blocks(&self) -> bool {
        matches!(self, Self::Degrade | Self::Retract)
    }
}

/// Result of checking a proof-debt budget gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofDebtGateResult {
    /// The budget that was checked.
    pub budget_id: String,
    /// The gate decision.
    pub decision: ProofDebtGateDecision,
    /// Budget consumed percentage at gate check time.
    pub consumed_pct: u64,
    /// Whether the budget was exhausted at gate check time.
    pub exhausted: bool,
    /// Applicable waiver that authorized this result, if any. Debt remains outstanding.
    pub waiver_id: Option<String>,
    /// Amount authorized by the applicable waiver, if any.
    pub waived_amount_micros: Option<u64>,
    /// Human-readable summary of the gate evaluation.
    pub summary: String,
}

/// Evaluate a proof-debt budget gate using default thresholds.
///
/// Returns the gate decision based on the budget's current state:
/// - Proceed: consumed < 80%
/// - Warn: consumed >= 80% but < 100%
/// - Degrade: consumed >= 100% (exhausted)
/// - Retract: consumed >= 120% (severely overdrawn)
pub fn evaluate_proof_debt_gate(budget: &ProofDebtBudgetV1) -> ProofDebtGateResult {
    evaluate_proof_debt_gate_with_waiver_and_config(budget, None, &ProofDebtBudgetConfig::default())
}

/// Evaluate a proof-debt gate, allowing a matching bounded waiver to authorize proceeding.
/// The waiver is only an authorization: it does not change `consumed_micros`.
pub fn evaluate_proof_debt_gate_with_waiver(
    budget: &ProofDebtBudgetV1,
    waiver: Option<&ProofDebtWaiverReceipt>,
) -> ProofDebtGateResult {
    evaluate_proof_debt_gate_with_waiver_and_config(
        budget,
        waiver,
        &ProofDebtBudgetConfig::default(),
    )
}

/// Evaluate a proof-debt budget gate using custom thresholds.
pub fn evaluate_proof_debt_gate_with_config(
    budget: &ProofDebtBudgetV1,
    config: &ProofDebtBudgetConfig,
) -> ProofDebtGateResult {
    evaluate_proof_debt_gate_with_waiver_and_config(budget, None, config)
}

/// Evaluate a proof-debt gate using custom thresholds and an optional waiver.
pub fn evaluate_proof_debt_gate_with_waiver_and_config(
    budget: &ProofDebtBudgetV1,
    waiver: Option<&ProofDebtWaiverReceipt>,
    config: &ProofDebtBudgetConfig,
) -> ProofDebtGateResult {
    let pct = budget.consumed_pct();
    let exhausted = budget.is_exhausted();
    let _overdrawn = budget.consumed_micros > budget.budget_micros;

    let (decision, summary, waiver_id, waived_amount_micros) = if let Some(receipt) =
        waiver.filter(|receipt| receipt.applies_to(budget))
    {
        (
            ProofDebtGateDecision::Waived,
            format!(
                "proof debt waived for bounded proceeding: {}% consumed ({} / {} micros), waiver {} authorizes {} micros",
                pct, budget.consumed_micros, budget.budget_micros, receipt.waiver_id, receipt.waived_amount_micros
            ),
            Some(receipt.waiver_id.clone()),
            Some(receipt.waived_amount_micros),
        )
    } else if pct >= config.retract_threshold_pct {
        (
            ProofDebtGateDecision::Retract,
            format!(
                "proof debt severely overdrawn: {}% consumed ({} / {} micros)",
                pct, budget.consumed_micros, budget.budget_micros
            ),
            None,
            None,
        )
    } else if pct >= config.degrade_threshold_pct {
        (
            ProofDebtGateDecision::Degrade,
            format!(
                "proof debt exhausted: {}% consumed ({} / {} micros)",
                pct, budget.consumed_micros, budget.budget_micros
            ),
            None,
            None,
        )
    } else if pct >= config.warn_threshold_pct {
        (
            ProofDebtGateDecision::Warn,
            format!(
                "proof debt warning: {}% consumed ({} / {} micros)",
                pct, budget.consumed_micros, budget.budget_micros
            ),
            None,
            None,
        )
    } else {
        (
            ProofDebtGateDecision::Proceed,
            format!(
                "proof debt ok: {}% consumed ({} / {} micros)",
                pct, budget.consumed_micros, budget.budget_micros
            ),
            None,
            None,
        )
    };

    ProofDebtGateResult {
        budget_id: budget.budget_id.clone(),
        decision,
        consumed_pct: pct,
        exhausted,
        waiver_id,
        waived_amount_micros,
        summary,
    }
}

/// A proof-debt waiver receipt.
///
/// When an operator explicitly waives proof debt (accepting the risk),
/// this receipt records the waiver with full provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofDebtWaiverReceipt {
    /// Schema version.
    pub schema_version: String,
    /// Unique identifier for this waiver.
    pub waiver_id: String,
    /// Budget this waiver applies to.
    pub budget_id: String,
    /// Scope covered by this waiver.
    pub scope: String,
    /// Budget ceiling in micros at the time the waiver was issued.
    pub budget_micros: u64,
    /// Amount of debt being waived in micros.
    pub waived_amount_micros: u64,
    /// Operator who approved the waiver.
    pub operator_ref: String,
    /// Why the waiver was granted.
    pub rationale: String,
    /// When this waiver was recorded.
    pub recorded_time: DateTime<Utc>,
}

impl ProofDebtWaiverReceipt {
    /// Create a new proof-debt waiver receipt.
    pub fn new(
        budget: &ProofDebtBudgetV1,
        waived_amount_micros: u64,
        operator_ref: &str,
        rationale: &str,
    ) -> Self {
        Self::new_at(
            budget,
            waived_amount_micros,
            operator_ref,
            rationale,
            Utc::now(),
        )
    }

    /// Create a waiver with an explicit recorded time for reproducible artifacts.
    pub fn new_at(
        budget: &ProofDebtBudgetV1,
        waived_amount_micros: u64,
        operator_ref: &str,
        rationale: &str,
        recorded_time: DateTime<Utc>,
    ) -> Self {
        Self {
            schema_version: "ProofDebtWaiverReceiptV1".to_string(),
            waiver_id: ids::proof_debt_waiver_id(
                &budget.budget_id,
                operator_ref,
                waived_amount_micros,
            ),
            budget_id: budget.budget_id.clone(),
            scope: budget.scope.clone(),
            budget_micros: budget.budget_micros,
            waived_amount_micros,
            operator_ref: operator_ref.to_string(),
            rationale: rationale.to_string(),
            recorded_time,
        }
    }

    /// Whether this waiver authorizes the current outstanding amount on this budget.
    pub fn applies_to(&self, budget: &ProofDebtBudgetV1) -> bool {
        self.budget_id == budget.budget_id
            && self.scope == budget.scope
            && self.budget_micros == budget.budget_micros
            && self.waived_amount_micros >= budget.consumed_micros
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ProofDebt;

    #[test]
    fn budget_new_has_id() {
        let budget = ProofDebtBudgetV1::new("claim:clm_abc", 500_000);
        assert!(budget.budget_id.starts_with("pdb_"));
        assert_eq!(budget.budget_micros, 500_000);
        assert_eq!(budget.consumed_micros, 0);
        assert_eq!(budget.available_micros(), 500_000);
        assert!(!budget.is_exhausted());
    }

    #[test]
    fn consume_debt_reduces_available() {
        let mut budget = ProofDebtBudgetV1::new("claim:clm_abc", 500_000);
        let debit = budget.consume_debt(
            &ProofDebt::MissingSourceBasis,
            "claim:clm_abc",
            "claim lacks source basis",
            false,
        );
        assert!(debit.is_some());
        let debit = debit.unwrap();
        assert_eq!(debit.amount_micros, 250_000); // weight of MissingSourceBasis
        assert_eq!(budget.consumed_micros, 250_000);
        assert_eq!(budget.available_micros(), 250_000);
        assert!(!budget.is_exhausted());
    }

    #[test]
    fn consume_strict_blocks_overdraw() {
        let mut budget = ProofDebtBudgetV1::new("claim:clm_abc", 100_000);
        // Try to consume more than budget in strict mode
        let result = budget.consume(150_000, "test", "overdraw attempt", true);
        assert!(result.is_none());
        // Budget should be unchanged
        assert_eq!(budget.consumed_micros, 0);
    }

    #[test]
    fn consume_non_strict_allows_overdraw() {
        let mut budget = ProofDebtBudgetV1::new("claim:clm_abc", 100_000);
        let debit = budget.consume(150_000, "test", "overdraw", false);
        assert!(debit.is_some());
        let debit = debit.unwrap();
        assert!(debit.overdrawn);
        assert!(budget.is_exhausted());
    }

    #[test]
    fn resolve_debt_replenishes() {
        let mut budget = ProofDebtBudgetV1::new("claim:clm_abc", 500_000);
        budget.consume_debt(
            &ProofDebt::MissingSourceBasis,
            "claim:clm_abc",
            "initial",
            false,
        );
        assert_eq!(budget.consumed_micros, 250_000);

        let credit = budget.resolve_debt(
            &ProofDebt::MissingSourceBasis,
            "evidence:evb_xyz",
            "source basis found",
        );
        assert!(credit.is_some());
        assert_eq!(budget.consumed_micros, 0);
    }

    #[test]
    fn gate_proceed_when_under_80() {
        let mut budget = ProofDebtBudgetV1::new("claim:clm_abc", 500_000);
        budget.consume(100_000, "test", "20%", false); // 20%
        let result = evaluate_proof_debt_gate(&budget);
        assert_eq!(result.decision, ProofDebtGateDecision::Proceed);
        assert!(!result.exhausted);
    }

    #[test]
    fn gate_warn_at_80() {
        let mut budget = ProofDebtBudgetV1::new("claim:clm_abc", 500_000);
        budget.consume(400_000, "test", "80%", false); // 80%
        let result = evaluate_proof_debt_gate(&budget);
        assert_eq!(result.decision, ProofDebtGateDecision::Warn);
    }

    #[test]
    fn gate_degrade_when_exhausted() {
        let mut budget = ProofDebtBudgetV1::new("claim:clm_abc", 500_000);
        budget.consume(500_000, "test", "100%", false); // 100%
        let result = evaluate_proof_debt_gate(&budget);
        assert_eq!(result.decision, ProofDebtGateDecision::Degrade);
        assert!(result.exhausted);
    }

    #[test]
    fn gate_retract_when_severely_overdrawn() {
        let mut budget = ProofDebtBudgetV1::new("claim:clm_abc", 500_000);
        budget.consume(600_000, "test", "120%", false); // 120%
        let result = evaluate_proof_debt_gate(&budget);
        assert_eq!(result.decision, ProofDebtGateDecision::Retract);
        assert!(result.decision.blocks());
    }

    #[test]
    fn waiver_preserves_debt_and_authorizes_bounded_proceeding() {
        let mut budget = ProofDebtBudgetV1::new("claim:clm_abc", 500_000);
        budget.consume(400_000, "test", "debt", false);
        assert_eq!(budget.available_micros(), 100_000);

        let waiver = ProofDebtWaiverReceipt::new(
            &budget,
            400_000,
            "operator:josh",
            "accepting risk on missing benchmark",
        );
        assert_eq!(budget.consumed_micros, 400_000);
        assert_eq!(budget.available_micros(), 100_000);
        let gate = evaluate_proof_debt_gate_with_waiver(&budget, Some(&waiver));
        assert_eq!(gate.decision, ProofDebtGateDecision::Waived);
    }

    #[test]
    fn proof_debt_weights_are_nonzero_except_none() {
        assert_eq!(proof_debt_weight(&ProofDebt::None), 0);
        assert!(proof_debt_weight(&ProofDebt::MissingSourceBasis) > 0);
        assert!(proof_debt_weight(&ProofDebt::MissingBenchmark) > 0);
        assert!(proof_debt_weight(&ProofDebt::MissingRepro) > 0);
        assert!(proof_debt_weight(&ProofDebt::MissingExternalValidation) > 0);
    }

    #[test]
    fn source_basis_is_heaviest_debt() {
        // Missing source basis should be the heaviest debt kind
        // because a claim with no source is the most dangerous.
        let source_weight = proof_debt_weight(&ProofDebt::MissingSourceBasis);
        assert!(source_weight > proof_debt_weight(&ProofDebt::MissingRepro));
        assert!(source_weight > proof_debt_weight(&ProofDebt::MissingBenchmark));
        assert!(source_weight > proof_debt_weight(&ProofDebt::MissingExternalValidation));
    }

    #[test]
    fn gate_decision_allows_proceed() {
        assert!(ProofDebtGateDecision::Proceed.allows_proceed());
        assert!(ProofDebtGateDecision::Warn.allows_proceed());
        assert!(ProofDebtGateDecision::Waived.allows_proceed());
        assert!(!ProofDebtGateDecision::Degrade.allows_proceed());
        assert!(!ProofDebtGateDecision::Retract.allows_proceed());
    }

    #[test]
    fn custom_config_changes_thresholds() {
        let mut budget = ProofDebtBudgetV1::new("claim:clm_custom", 1_000_000);
        // Consume 50% = 500k
        budget.consume(500_000, "test", "50%", false);

        // Default config: 50% < 80% -> Proceed
        let result = evaluate_proof_debt_gate(&budget);
        assert_eq!(result.decision, ProofDebtGateDecision::Proceed);

        // Custom config with warn at 40%: 50% >= 40% -> Warn
        let config = ProofDebtBudgetConfig {
            warn_threshold_pct: 40,
            ..ProofDebtBudgetConfig::default()
        };
        let result = evaluate_proof_debt_gate_with_config(&budget, &config);
        assert_eq!(result.decision, ProofDebtGateDecision::Warn);
    }

    #[test]
    fn custom_config_weights_change_consumption() {
        let config = ProofDebtBudgetConfig {
            weight_missing_source_basis: 500_000,
            ..ProofDebtBudgetConfig::default()
        };

        let mut budget = ProofDebtBudgetV1::new("claim:clm_weight", 1_000_000);
        let debit = budget.consume_debt_with_config(
            &ProofDebt::MissingSourceBasis,
            &config,
            "test",
            "custom weight",
            false,
        );
        assert!(debit.is_some());
        // With custom weight 500k instead of default 250k
        assert_eq!(budget.consumed_micros, 500_000);
        assert_eq!(budget.consumed_pct(), 50);
    }
}

/// Summary of proof-debt state for a scope (claim, case, or session).
///
/// This is a serializable snapshot of the budget state, suitable for
/// inclusion in context packs, verification reports, or dashboards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofDebtSummaryV1 {
    /// Schema version.
    pub schema_version: String,
    /// Budget ID this summary covers.
    pub budget_id: String,
    /// Scope this budget applies to.
    pub scope: String,
    /// Maximum proof debt allowed in micros.
    pub budget_micros: u64,
    /// Current consumed proof debt in micros.
    pub consumed_micros: u64,
    /// Available proof debt remaining.
    pub available_micros: u64,
    /// Percentage of budget consumed (0-100).
    pub consumed_pct: u64,
    /// Whether the budget is exhausted.
    pub exhausted: bool,
    /// Current gate decision.
    pub gate_decision: ProofDebtGateDecision,
    /// Human-readable gate summary.
    pub gate_summary: String,
    /// Applicable waiver ID, if any. This never changes outstanding debt.
    pub waiver_id: Option<String>,
    /// Amount authorized by the applicable waiver, if any.
    pub waived_amount_micros: Option<u64>,
}

impl ProofDebtSummaryV1 {
    /// Build a summary from a budget and its gate evaluation.
    pub fn from_budget(budget: &ProofDebtBudgetV1) -> Self {
        Self::from_budget_with_waiver(budget, None)
    }

    /// Build a summary that exposes both outstanding debt and an optional waiver.
    pub fn from_budget_with_waiver(
        budget: &ProofDebtBudgetV1,
        waiver: Option<&ProofDebtWaiverReceipt>,
    ) -> Self {
        let gate = evaluate_proof_debt_gate_with_waiver(budget, waiver);
        Self {
            schema_version: "ProofDebtSummaryV1".to_string(),
            budget_id: budget.budget_id.clone(),
            scope: budget.scope.clone(),
            budget_micros: budget.budget_micros,
            consumed_micros: budget.consumed_micros,
            available_micros: budget.available_micros(),
            consumed_pct: budget.consumed_pct(),
            exhausted: budget.is_exhausted(),
            gate_decision: gate.decision,
            gate_summary: gate.summary,
            waiver_id: gate.waiver_id,
            waived_amount_micros: gate.waived_amount_micros,
        }
    }
}

/// Compute the total proof-debt weight for a list of ProofDebt values.
///
/// This is useful for computing the initial debt of a claim from its
/// `proof_debt` Vec without creating a full budget.
pub fn total_proof_debt_weight(debts: &[ProofDebt]) -> u64 {
    debts.iter().map(proof_debt_weight).sum()
}

/// Compute total proof-debt weight using a custom config.
pub fn total_proof_debt_weight_with_config(
    debts: &[ProofDebt],
    config: &ProofDebtBudgetConfig,
) -> u64 {
    debts.iter().map(|d| config.weight(d)).sum()
}

/// Create a budget for a claim and immediately consume all its existing proof debt.
///
/// This is the standard entry point for computing a claim's proof-debt budget:
/// take the claim's `proof_debt` Vec, create a budget, and consume all debts.
/// The budget amount defaults to 500_000 micros (enough for 2 missing source
/// basis debts or ~6 mixed debts).
pub fn budget_for_claim(
    claim_id: &str,
    debts: &[ProofDebt],
    budget_micros: u64,
) -> (ProofDebtBudgetV1, Vec<ProofDebtDebitV1>) {
    let mut budget = ProofDebtBudgetV1::new(claim_id, budget_micros);
    let mut debits = Vec::new();
    for debt in debts {
        if let Some(debit) = budget.consume_debt(debt, claim_id, &format!("{:?}", debt), false) {
            debits.push(debit);
        }
    }
    (budget, debits)
}

#[cfg(test)]
mod summary_tests {
    use super::*;

    #[test]
    fn summary_reflects_budget_state() {
        let mut budget = ProofDebtBudgetV1::new("claim:clm_summary", 500_000);
        budget.consume(400_000, "test", "80%", false);

        let summary = ProofDebtSummaryV1::from_budget(&budget);
        assert_eq!(summary.scope, "claim:clm_summary");
        assert_eq!(summary.consumed_micros, 400_000);
        assert_eq!(summary.available_micros, 100_000);
        assert_eq!(summary.consumed_pct, 80);
        assert!(!summary.exhausted);
        assert_eq!(summary.gate_decision, ProofDebtGateDecision::Warn);
        assert!(!summary.gate_summary.is_empty());
    }

    #[test]
    fn total_weight_sums_all_debts() {
        let debts = vec![
            ProofDebt::MissingSourceBasis,
            ProofDebt::MissingRepro,
            ProofDebt::MissingBenchmark,
        ];
        let total = total_proof_debt_weight(&debts);
        // 250k + 150k + 100k = 500k
        assert_eq!(total, 500_000);
    }

    #[test]
    fn budget_for_claim_consumes_all_debts() {
        let debts = vec![ProofDebt::MissingSourceBasis, ProofDebt::MissingBenchmark];
        let (budget, debits) = budget_for_claim("claim:clm_auto", &debts, 1_000_000);
        // 250k + 100k = 350k consumed
        assert_eq!(budget.consumed_micros, 350_000);
        assert_eq!(debits.len(), 2);
        assert_eq!(budget.available_micros(), 650_000);
    }

    #[test]
    fn budget_for_claim_with_no_debts() {
        let debts = vec![ProofDebt::None];
        let (budget, debits) = budget_for_claim("claim:clm_clean", &debts, 500_000);
        assert_eq!(budget.consumed_micros, 0);
        assert!(debits.is_empty());
    }
}
