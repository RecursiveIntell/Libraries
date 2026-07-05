//! Proof-debt entropy budget — tracks cumulative unverified claims.
//!
//! Every time a fact is promoted to the knowledge base, proof-debt is
//! incurred. Debt is paid when the claim is verified (test passed, audit
//! passed, no contradictions found, external evidence, superseded, or
//! quarantined). When outstanding debt exceeds the threshold for any
//! risk class, the loop shifts from additive to subtractive mode.
//!
//! Maps to FEUT-004: proof-debt / entropy-budget runtime.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Risk class of a claim — determines verification requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskClass {
    /// Low risk: observational, internal state. Cheap to verify.
    Low,
    /// Medium risk: cross-domain claim, requires checking.
    Medium,
    /// High risk: public claim, benchmark claim, requires falsification.
    High,
    /// Critical risk: requires replay AND falsification.
    Critical,
}

impl std::fmt::Display for RiskClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => f.write_str("low"),
            Self::Medium => f.write_str("medium"),
            Self::High => f.write_str("high"),
            Self::Critical => f.write_str("critical"),
        }
    }
}

/// How proof-debt was paid off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentMethod {
    /// Verified by test execution.
    TestPassed,
    /// Verified by hostile audit.
    AuditPassed,
    /// Verified by contradiction check (no contradictions found).
    NoContradictions,
    /// Verified by external evidence.
    ExternalEvidence,
    /// Superseded by a better claim.
    Superseded,
    /// Quarantined (debt forgiven, claim isolated).
    Quarantined,
}

/// A single proof-debt entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofDebtEntry {
    /// Unique entry id.
    pub entry_id: String,
    /// The fact/claim id in semantic-memory.
    pub claim_id: String,
    /// Namespace the claim belongs to.
    pub namespace: String,
    /// When the debt was incurred (ISO-8601).
    pub incurred_at: String,
    /// When the debt was paid (if verified). None = still outstanding.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid_at: Option<String>,
    /// Risk class of the claim.
    pub risk_class: RiskClass,
    /// How the debt was paid (if paid).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method: Option<PaymentMethod>,
}

/// Snapshot of the current proof-debt state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofDebtReceipt {
    pub total_incurred: usize,
    pub total_paid: usize,
    pub total_outstanding: usize,
    pub debt_ratio: f64,
    pub outstanding_by_risk: HashMap<String, usize>,
    pub exceeds_threshold: bool,
    pub exceeded_risk_classes: Vec<String>,
    pub timestamp: String,
}

// ---------------------------------------------------------------------------
// Budget
// ---------------------------------------------------------------------------

/// Tracks proof-debt across loop cycles.
#[derive(Debug, Clone)]
pub struct ProofDebtBudget {
    /// Outstanding debt entries, keyed by entry_id.
    outstanding: HashMap<String, ProofDebtEntry>,
    /// Paid debt entries (for audit trail).
    paid: Vec<ProofDebtEntry>,
    /// Debt threshold per risk class before triggering subtractive mode.
    thresholds: HashMap<RiskClass, usize>,
    /// Total debt incurred since loop start.
    total_incurred: usize,
    /// Counter for generating unique entry ids.
    counter: u64,
}

impl ProofDebtBudget {
    /// Create a new budget with default thresholds.
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert(RiskClass::Low, 50);
        thresholds.insert(RiskClass::Medium, 20);
        thresholds.insert(RiskClass::High, 10);
        thresholds.insert(RiskClass::Critical, 5);
        Self {
            outstanding: HashMap::new(),
            paid: Vec::new(),
            thresholds,
            total_incurred: 0,
            counter: 0,
        }
    }

    /// Set a custom debt threshold for a risk class.
    pub fn set_threshold(&mut self, risk_class: RiskClass, threshold: usize) {
        self.thresholds.insert(risk_class, threshold);
    }

    /// Incur debt for a new claim. Returns the entry id.
    pub fn incur(&mut self, claim_id: &str, namespace: &str, risk_class: RiskClass) -> String {
        self.counter += 1;
        let entry_id = format!("debt-{:06}", self.counter);
        let entry = ProofDebtEntry {
            entry_id: entry_id.clone(),
            claim_id: claim_id.to_string(),
            namespace: namespace.to_string(),
            incurred_at: Utc::now().to_rfc3339(),
            paid_at: None,
            risk_class,
            payment_method: None,
        };
        self.outstanding.insert(entry_id.clone(), entry);
        self.total_incurred += 1;
        entry_id
    }

    /// Pay off a specific debt entry.
    pub fn pay(&mut self, entry_id: &str, method: PaymentMethod) -> Result<(), String> {
        let mut entry = self
            .outstanding
            .remove(entry_id)
            .ok_or_else(|| format!("debt entry {} not found", entry_id))?;
        entry.paid_at = Some(Utc::now().to_rfc3339());
        entry.payment_method = Some(method);
        self.paid.push(entry);
        Ok(())
    }

    /// Pay all debt entries for a given claim. Returns the number paid.
    pub fn pay_for_claim(&mut self, claim_id: &str, method: PaymentMethod) -> usize {
        let ids: Vec<String> = self
            .outstanding
            .values()
            .filter(|e| e.claim_id == claim_id)
            .map(|e| e.entry_id.clone())
            .collect();
        let count = ids.len();
        for id in ids {
            let _ = self.pay(&id, method);
        }
        count
    }

    /// Pay all low-risk debt (typically done immediately for observations).
    pub fn pay_all_low_risk(&mut self, method: PaymentMethod) -> usize {
        let ids: Vec<String> = self
            .outstanding
            .values()
            .filter(|e| e.risk_class == RiskClass::Low)
            .map(|e| e.entry_id.clone())
            .collect();
        let count = ids.len();
        for id in ids {
            let _ = self.pay(&id, method);
        }
        count
    }

    /// Current outstanding debt count by risk class.
    pub fn outstanding_by_risk(&self) -> HashMap<RiskClass, usize> {
        let mut counts = HashMap::new();
        for entry in self.outstanding.values() {
            *counts.entry(entry.risk_class).or_default() += 1;
        }
        counts
    }

    /// Total outstanding debt.
    pub fn total_outstanding(&self) -> usize {
        self.outstanding.len()
    }

    /// Total paid debt.
    pub fn total_paid(&self) -> usize {
        self.paid.len()
    }

    /// Debt ratio: outstanding / total_incurred. Returns 0.0 if nothing
    /// has been incurred yet.
    pub fn debt_ratio(&self) -> f64 {
        if self.total_incurred == 0 {
            return 0.0;
        }
        self.outstanding.len() as f64 / self.total_incurred as f64
    }

    /// Whether any risk class has exceeded its threshold.
    pub fn exceeds_threshold(&self) -> bool {
        let counts = self.outstanding_by_risk();
        for (risk_class, count) in &counts {
            if let Some(threshold) = self.thresholds.get(risk_class) {
                if count >= threshold {
                    return true;
                }
            }
        }
        false
    }

    /// Which risk classes have exceeded their threshold.
    pub fn exceeded_risk_classes(&self) -> Vec<RiskClass> {
        let counts = self.outstanding_by_risk();
        let mut exceeded = Vec::new();
        for (risk_class, count) in &counts {
            if let Some(threshold) = self.thresholds.get(risk_class) {
                if count >= threshold {
                    exceeded.push(*risk_class);
                }
            }
        }
        exceeded
    }

    /// Whether the loop should shift to subtractive mode.
    pub fn should_shift_to_subtractive(&self) -> bool {
        self.exceeds_threshold() || self.debt_ratio() > 0.6
    }

    /// Whether the loop can return to additive mode.
    pub fn can_return_to_additive(&self) -> bool {
        !self.exceeds_threshold() && self.debt_ratio() < 0.3
    }

    /// Emit a receipt for the current debt state.
    pub fn debt_receipt(&self) -> ProofDebtReceipt {
        let counts = self.outstanding_by_risk();
        let mut outstanding_by_risk = HashMap::new();
        for (risk, count) in &counts {
            outstanding_by_risk.insert(risk.to_string(), *count);
        }
        let exceeded = self.exceeded_risk_classes();
        let exceeded_names: Vec<String> = exceeded.iter().map(|r| r.to_string()).collect();
        ProofDebtReceipt {
            total_incurred: self.total_incurred,
            total_paid: self.paid.len(),
            total_outstanding: self.outstanding.len(),
            debt_ratio: self.debt_ratio(),
            outstanding_by_risk,
            exceeds_threshold: self.exceeds_threshold(),
            exceeded_risk_classes: exceeded_names,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    /// Get all outstanding entries (for subtractive cycle processing).
    pub fn outstanding_entries(&self) -> Vec<&ProofDebtEntry> {
        self.outstanding.values().collect()
    }

    /// Get outstanding entries for a specific risk class.
    pub fn outstanding_for_risk(&self, risk: RiskClass) -> Vec<&ProofDebtEntry> {
        self.outstanding
            .values()
            .filter(|e| e.risk_class == risk)
            .collect()
    }
}

impl Default for ProofDebtBudget {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Risk classification heuristic
// ---------------------------------------------------------------------------

/// Classify the risk of a claim based on its content and context.
/// Low: observational facts, internal state, captured from execution.
/// Medium: cross-domain claims, new patterns, extracted from research.
/// High: public claims, benchmark claims, novelty claims.
/// Critical: claims that could influence identity or public positioning.
pub fn classify_risk(content: &str, namespace: &str) -> RiskClass {
    let content_lower = content.to_lowercase();

    // Critical: identity, public positioning, grandiosity triggers.
    let critical_markers = [
        "groundbreaking",
        "world-novel",
        "world first",
        "solved",
        "polymathic",
        "genius",
        "princeton accepted",
        "peer reviewed",
        "novel physics",
    ];
    if critical_markers.iter().any(|m| content_lower.contains(m)) {
        return RiskClass::Critical;
    }

    // High: public claims, benchmark superiority, novelty.
    let high_markers = [
        "novel",
        "first",
        "breakthrough",
        "benchmark",
        "superior",
        "best",
        "outperforms",
        "unprecedented",
        "never seen before",
    ];
    if high_markers.iter().any(|m| content_lower.contains(m)) {
        return RiskClass::High;
    }

    // Medium: cross-domain claims, patterns, research-derived.
    let medium_markers = [
        "pattern",
        "transfer",
        "analogy",
        "cross-domain",
        "physics to",
        "maps to",
        "inspired by",
        "research",
        "hypothesis",
        "suggests",
    ];
    if medium_markers.iter().any(|m| content_lower.contains(m)) {
        return RiskClass::Medium;
    }

    // Research and behavioral namespaces are medium by default.
    if namespace == "research" || namespace == "behavioral" || namespace == "mixed" {
        return RiskClass::Medium;
    }

    // Low: everything else (observations, internal state, execution output).
    RiskClass::Low
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incur_and_pay() {
        let mut budget = ProofDebtBudget::new();
        let id = budget.incur("fact-1", "projects", RiskClass::Low);
        assert_eq!(budget.total_outstanding(), 1);
        // total_incurred is a private field, use debt_receipt for testing.
        assert_eq!(budget.debt_receipt().total_incurred, 1);
        budget.pay(&id, PaymentMethod::NoContradictions).unwrap();
        assert_eq!(budget.total_outstanding(), 0);
        assert_eq!(budget.total_paid(), 1);
    }

    #[test]
    fn test_threshold_exceeded() {
        let mut budget = ProofDebtBudget::new();
        budget.set_threshold(RiskClass::High, 2);
        budget.incur("fact-1", "projects", RiskClass::High);
        assert!(!budget.exceeds_threshold());
        budget.incur("fact-2", "projects", RiskClass::High);
        assert!(budget.exceeds_threshold());
        assert!(budget.should_shift_to_subtractive());
    }

    #[test]
    fn test_debt_ratio() {
        let mut budget = ProofDebtBudget::new();
        budget.incur("f1", "ns", RiskClass::Low);
        budget.incur("f2", "ns", RiskClass::Low);
        budget.incur("f3", "ns", RiskClass::Low);
        assert_eq!(budget.debt_ratio(), 1.0);
        budget.pay_all_low_risk(PaymentMethod::NoContradictions);
        assert_eq!(budget.debt_ratio(), 0.0);
    }

    #[test]
    fn test_can_return_to_additive() {
        let mut budget = ProofDebtBudget::new();
        budget.set_threshold(RiskClass::Medium, 3);
        for i in 0..5 {
            budget.incur(&format!("f{i}"), "ns", RiskClass::Medium);
        }
        assert!(budget.should_shift_to_subtractive());
        // Pay off enough to get below threshold and ratio.
        let ids: Vec<String> = budget.outstanding.keys().take(4).cloned().collect();
        for id in ids {
            budget.pay(&id, PaymentMethod::NoContradictions).unwrap();
        }
        assert!(budget.can_return_to_additive());
    }

    #[test]
    fn test_classify_risk() {
        assert_eq!(
            classify_risk("groundbreaking discovery", "projects"),
            RiskClass::Critical
        );
        assert_eq!(
            classify_risk("novel architecture pattern", "research"),
            RiskClass::High
        );
        assert_eq!(
            classify_risk("pattern transfer from physics", "research"),
            RiskClass::Medium
        );
        assert_eq!(
            classify_risk("task completed successfully", "autonomous"),
            RiskClass::Low
        );
    }
}
