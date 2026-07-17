//! RL-trained retrieval routing on receipt replay data.
//!
//! Uses the system's receipt-driven replay as training signal for a
//! simple tabular/linear routing policy. The policy maps query profile
//! features to pipeline stage selection probabilities, learning from
//! past search outcomes.
//!
//! Degrades gracefully to heuristic routing when untrained.
//!
//! Behind `#[cfg(feature = "rl-routing")]` which depends on `routing`.

#![cfg(feature = "rl-routing")]

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::routing::{QueryProfile, RetrievalRouter, RoutingDecision};

// ─── Policy ─────────────────────────────────────────────────────────────

/// A simple tabular routing policy trained on receipt replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingPolicy {
    /// Maps feature name to weight.
    pub weights: HashMap<String, f64>,
    /// Learning rate for policy updates.
    pub learning_rate: f64,
    /// Baseline outcome score (exponential moving average).
    pub baseline: f64,
    /// Number of training examples seen.
    pub trained_examples: usize,
    /// RFC 3339 timestamp of the most recent training update.
    #[serde(default)]
    pub last_updated: Option<String>,
}

impl Default for RoutingPolicy {
    fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert("bm25_coarse".to_string(), 1.0);
        weights.insert("vector_medium".to_string(), 1.0);
        weights.insert("rerank_fine".to_string(), 0.5);
        weights.insert("graph_expansion".to_string(), 0.3);
        weights.insert("decoder".to_string(), 0.2);
        weights.insert("discord".to_string(), 0.2);
        Self {
            weights,
            learning_rate: 0.01,
            baseline: 0.5,
            trained_examples: 0,
            last_updated: None,
        }
    }
}

// ─── Training example ───────────────────────────────────────────────────

/// A training example extracted from a search receipt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingExample {
    pub query_profile: QueryProfile,
    pub decision: RoutingDecision,
    pub outcome_score: f64,
}

/// Extract a training example from a search outcome.
pub fn extract_training_example(
    profile: &QueryProfile,
    decision: &RoutingDecision,
    outcome_score: f64,
) -> TrainingExample {
    TrainingExample {
        query_profile: profile.clone(),
        decision: decision.clone(),
        outcome_score,
    }
}

// ─── Policy update ──────────────────────────────────────────────────────

/// Update the policy from a batch of training examples.
///
/// For each example:
/// - reward = outcome_score - baseline
/// - If reward > 0, increase weights for stages that were enabled
/// - If reward < 0, decrease weights for stages that were enabled
/// - Update baseline as exponential moving average
pub fn update_policy(policy: &mut RoutingPolicy, examples: &[TrainingExample]) {
    if examples.is_empty() {
        return;
    }

    let lr = policy.learning_rate;
    let alpha = 0.1; // EMA smoothing

    for ex in examples {
        let reward = ex.outcome_score - policy.baseline;

        // Update weights for each stage.
        let stages = [
            ("bm25_coarse", ex.decision.bm25_coarse),
            ("vector_medium", ex.decision.vector_medium),
            ("rerank_fine", ex.decision.rerank_fine),
            ("graph_expansion", ex.decision.graph_expansion),
            ("decoder", ex.decision.decoder),
            ("discord", ex.decision.discord),
        ];

        for (name, enabled) in stages {
            if enabled {
                let w = policy.weights.entry(name.to_string()).or_insert(0.5);
                *w += lr * reward;
                *w = w.clamp(0.0, 2.0);
            }
        }

        // Update baseline.
        policy.baseline = alpha * ex.outcome_score + (1.0 - alpha) * policy.baseline;
        policy.trained_examples += 1;
    }
}

// ─── Routing with RL ────────────────────────────────────────────────────

/// Route a query using learned policy weights.
///
/// Callers should use [`is_trained`] before selecting this production path.
/// Heuristic fallback belongs at the routing dispatch boundary, where the
/// caller's configured [`RetrievalRouter`] is available.
pub fn route_with_policy(policy: &RoutingPolicy, profile: &QueryProfile) -> RoutingDecision {
    // Compute stage scores from learned weights.
    let bm25_w = *policy.weights.get("bm25_coarse").unwrap_or(&1.0);
    let vector_w = *policy.weights.get("vector_medium").unwrap_or(&1.0);
    let rerank_w = *policy.weights.get("rerank_fine").unwrap_or(&0.5);
    let graph_w = *policy.weights.get("graph_expansion").unwrap_or(&0.3);
    let decoder_w = *policy.weights.get("decoder").unwrap_or(&0.2);
    let discord_w = *policy.weights.get("discord").unwrap_or(&0.2);

    // Enable stages with weight > 0.5.
    let bm25_coarse = bm25_w > 0.5;
    let vector_medium = vector_w > 0.5 && profile.specificity >= 0.15;
    let rerank_fine = rerank_w > 0.5;
    let graph_expansion = graph_w > 0.5 && profile.has_entities;
    let decoder = decoder_w > 0.5 && profile.contradiction_risk;
    let discord = discord_w > 0.5 && profile.has_entities;

    let no_retrieval = !bm25_coarse && !vector_medium && profile.token_count < 3;

    RoutingDecision {
        bm25_coarse,
        vector_medium,
        rerank_fine,
        graph_expansion,
        decoder,
        discord,
        no_retrieval,
        reasoning: format!(
            "RL policy (trained={}): bm25_w={:.2}, vec_w={:.2}, rerank_w={:.2}",
            policy.trained_examples, bm25_w, vector_w, rerank_w
        ),
    }
}

/// Backward-compatible routing entry point.
///
/// Learning remains shadow-only in this MVP. The executed decision is therefore
/// always the configured heuristic baseline; callers may separately call
/// [`route_with_policy`] to record a no-effect shadow prediction.
pub fn route_with_rl(_policy: &RoutingPolicy, profile: &QueryProfile) -> RoutingDecision {
    RetrievalRouter::default().route(profile)
}

/// Legacy active-routing readiness predicate.
///
/// A `RoutingPolicy` alone cannot prove verified outcome provenance or authorize
/// activation. The MVP emits promotion proposals only, so this predicate always
/// fails closed. Use [`evaluate_shadow_candidate`] for proposal eligibility.
pub fn is_trained(_policy: &RoutingPolicy) -> bool {
    false
}

// ─── Receipt-driven RL routing feedback ─────────────────────────────────

/// The outcome quality of a routing decision, as reported by the caller
/// after observing search results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingOutcome {
    /// The routing decision produced good results.
    Good,
    /// The routing decision produced poor results.
    Bad,
    /// The routing decision produced acceptable results.
    Neutral,
}

impl RoutingOutcome {
    /// Convert the outcome to a numeric score for policy updates.
    fn to_score(self) -> f64 {
        match self {
            RoutingOutcome::Good => 0.9,
            RoutingOutcome::Neutral => 0.5,
            RoutingOutcome::Bad => 0.1,
        }
    }
}

/// Alias for the tabular routing policy used in receipt-driven RL feedback.
/// This is the same `RoutingPolicy` struct, aliased for clarity in the
/// receipt-driven feedback context.
pub type TabularRoutingPolicy = RoutingPolicy;

/// Record a routing outcome and update the tabular routing policy's Q-table.
///
/// This is the receipt-driven RL feedback loop: after a search is performed
/// using a routing decision, the caller reports whether the outcome was good,
/// bad, or neutral. This function converts the outcome to a score and updates
/// the policy weights accordingly.
///
/// Returns the updated policy.
pub fn record_routing_outcome(
    policy: &mut TabularRoutingPolicy,
    profile: &QueryProfile,
    decision: &RoutingDecision,
    outcome: RoutingOutcome,
) -> TabularRoutingPolicy {
    let score = outcome.to_score();
    let example = extract_training_example(profile, decision, score);
    update_policy(policy, &[example]);
    policy.last_updated = Some(chrono::Utc::now().to_rfc3339());
    policy.clone()
}

/// Minimum verified examples before a shadow policy may emit a proposal.
pub const MIN_SHADOW_EXAMPLES: usize = 100;
/// Minimum examples in every observed route bucket.
pub const MIN_SHADOW_ROUTE_BUCKET_EXAMPLES: usize = 30;
/// Minimum distinct task families represented by verified evidence.
pub const MIN_SHADOW_FAMILIES: usize = 5;
/// Required holdout utility gain in basis points (3%).
pub const MIN_SHADOW_HOLDOUT_GAIN_BPS: i64 = 300;

/// Verification-owned evidence projected into routing evaluation.
///
/// This is an immutable backpointer/value summary, not a caller sentiment label.
/// Missing or duplicate owner receipt identity is rejected by the evaluator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedRoutingOutcomeV1 {
    pub verification_receipt_id: String,
    pub verification_receipt_digest: String,
    pub route_bucket: String,
    pub family: String,
    pub baseline_score_bps: i64,
    pub candidate_score_bps: i64,
    pub holdout: bool,
    pub correctness_regression: bool,
}

/// No-effect evaluation report for an immutable routing-policy candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowRoutingCandidateV1 {
    pub schema: String,
    pub policy_digest: String,
    pub eligible_examples: usize,
    pub route_bucket_counts: BTreeMap<String, usize>,
    pub family_count: usize,
    pub holdout_utility_delta_bps: i64,
    pub effect_percent: u8,
    pub active_route_changed: bool,
    pub promotion_proposal: bool,
    pub reason_codes: Vec<String>,
}

/// Evaluate a policy in shadow mode from verification-owned outcome receipts.
///
/// The report can only propose promotion. It cannot mutate the policy, router,
/// active route, or any canonical lifecycle state.
pub fn evaluate_shadow_candidate(
    policy: &RoutingPolicy,
    outcomes: &[VerifiedRoutingOutcomeV1],
) -> ShadowRoutingCandidateV1 {
    let mut reason_codes = Vec::new();
    let mut seen_receipts = BTreeSet::new();
    let mut route_bucket_counts = BTreeMap::<String, usize>::new();
    let mut families = BTreeSet::new();
    let mut eligible_examples = 0usize;
    let mut holdout_delta_sum = 0i64;
    let mut holdout_count = 0i64;
    let mut correctness_regression = false;

    for outcome in outcomes {
        let receipt_id = outcome.verification_receipt_id.trim();
        let receipt_digest = outcome.verification_receipt_digest.trim();
        if receipt_id.is_empty() {
            reason_codes.push("verification-receipt-id-missing".to_string());
            continue;
        }
        if receipt_digest.is_empty() {
            reason_codes.push("verification-receipt-digest-missing".to_string());
            continue;
        }
        if !seen_receipts.insert(receipt_id.to_string()) {
            reason_codes.push("duplicate-verification-receipt".to_string());
            continue;
        }
        if outcome.route_bucket.trim().is_empty() {
            reason_codes.push("route-bucket-missing".to_string());
            continue;
        }
        if outcome.family.trim().is_empty() {
            reason_codes.push("task-family-missing".to_string());
            continue;
        }
        if !(0..=10_000).contains(&outcome.baseline_score_bps)
            || !(0..=10_000).contains(&outcome.candidate_score_bps)
        {
            reason_codes.push("verified-outcome-score-out-of-range".to_string());
            continue;
        }

        eligible_examples += 1;
        *route_bucket_counts
            .entry(outcome.route_bucket.clone())
            .or_default() += 1;
        families.insert(outcome.family.clone());
        correctness_regression |= outcome.correctness_regression;
        if outcome.holdout {
            holdout_delta_sum += outcome.candidate_score_bps - outcome.baseline_score_bps;
            holdout_count += 1;
        }
    }

    if eligible_examples < MIN_SHADOW_EXAMPLES {
        reason_codes.push("eligible-examples-below-minimum".to_string());
    }
    if families.len() < MIN_SHADOW_FAMILIES {
        reason_codes.push("task-families-below-minimum".to_string());
    }
    if route_bucket_counts
        .values()
        .any(|count| *count < MIN_SHADOW_ROUTE_BUCKET_EXAMPLES)
    {
        reason_codes.push("route-bucket-examples-below-minimum".to_string());
    }
    if correctness_regression {
        reason_codes.push("correctness-regression".to_string());
    }
    let holdout_utility_delta_bps = if holdout_count == 0 {
        reason_codes.push("holdout-evidence-missing".to_string());
        0
    } else {
        holdout_delta_sum / holdout_count
    };
    if holdout_utility_delta_bps < MIN_SHADOW_HOLDOUT_GAIN_BPS {
        reason_codes.push("holdout-utility-gain-below-minimum".to_string());
    }
    reason_codes.sort();
    reason_codes.dedup();

    ShadowRoutingCandidateV1 {
        schema: "ShadowRoutingCandidateV1".to_string(),
        policy_digest: routing_policy_digest(policy),
        eligible_examples,
        route_bucket_counts,
        family_count: families.len(),
        holdout_utility_delta_bps,
        effect_percent: 0,
        active_route_changed: false,
        promotion_proposal: reason_codes.is_empty(),
        reason_codes,
    }
}

/// Content address only behavior-bearing policy material; volatile timestamps
/// and training counters are evidence metadata and deliberately excluded.
pub fn routing_policy_digest(policy: &RoutingPolicy) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"semantic-memory-shadow-routing-policy-v1\0");
    for (name, weight) in policy.weights.iter().collect::<BTreeMap<_, _>>() {
        hash_framed(&mut hasher, name.as_bytes());
        hasher.update(&weight.to_le_bytes());
    }
    hasher.update(&policy.learning_rate.to_le_bytes());
    hasher.update(&policy.baseline.to_le_bytes());
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn hash_framed(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

#[cfg(test)]
fn same_stages(left: &RoutingDecision, right: &RoutingDecision) -> bool {
    left.bm25_coarse == right.bm25_coarse
        && left.vector_medium == right.vector_medium
        && left.rerank_fine == right.rerank_fine
        && left.graph_expansion == right.graph_expansion
        && left.decoder == right.decoder
        && left.discord == right.discord
        && left.no_retrieval == right.no_retrieval
}

// ─── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn untrained_policy_falls_back_to_heuristic() {
        let policy = RoutingPolicy::default();
        assert_eq!(policy.trained_examples, 0);
        let profile = QueryProfile::from_query("what is rust");
        let decision = route_with_rl(&policy, &profile);
        // Heuristic routing should enable BM25 for a 4-token query.
        assert!(decision.bm25_coarse);
    }

    #[test]
    fn positive_outcome_increases_weights() {
        let mut policy = RoutingPolicy::default();
        let profile = QueryProfile::from_query("compare rust vs python");
        let decision = RoutingDecision {
            bm25_coarse: true,
            vector_medium: true,
            rerank_fine: true,
            graph_expansion: false,
            decoder: true,
            discord: false,
            no_retrieval: false,
            reasoning: "test".to_string(),
        };
        let example = TrainingExample {
            query_profile: profile,
            decision,
            outcome_score: 0.9, // positive
        };
        let initial_bm25_w = *policy.weights.get("bm25_coarse").unwrap();
        update_policy(&mut policy, &[example]);
        let updated_bm25_w = *policy.weights.get("bm25_coarse").unwrap();
        assert!(
            updated_bm25_w > initial_bm25_w,
            "positive outcome should increase weight: {} -> {}",
            initial_bm25_w,
            updated_bm25_w
        );
    }

    #[test]
    fn negative_outcome_decreases_weights() {
        let mut policy = RoutingPolicy::default();
        let profile = QueryProfile::from_query("compare rust vs python");
        let decision = RoutingDecision {
            bm25_coarse: true,
            vector_medium: true,
            rerank_fine: true,
            graph_expansion: false,
            decoder: true,
            discord: false,
            no_retrieval: false,
            reasoning: "test".to_string(),
        };
        let example = TrainingExample {
            query_profile: profile,
            decision,
            outcome_score: 0.1, // negative (below baseline 0.5)
        };
        let initial_bm25_w = *policy.weights.get("bm25_coarse").unwrap();
        update_policy(&mut policy, &[example]);
        let updated_bm25_w = *policy.weights.get("bm25_coarse").unwrap();
        assert!(
            updated_bm25_w < initial_bm25_w,
            "negative outcome should decrease weight: {} -> {}",
            initial_bm25_w,
            updated_bm25_w
        );
    }

    #[test]
    fn baseline_updates_correctly() {
        let mut policy = RoutingPolicy::default();
        let initial_baseline = policy.baseline;
        let profile = QueryProfile::from_query("test");
        let decision = RoutingDecision {
            bm25_coarse: true,
            vector_medium: false,
            rerank_fine: false,
            graph_expansion: false,
            decoder: false,
            discord: false,
            no_retrieval: false,
            reasoning: "test".to_string(),
        };
        let example = TrainingExample {
            query_profile: profile,
            decision,
            outcome_score: 0.8,
        };
        update_policy(&mut policy, &[example]);
        // Baseline should move toward 0.8 from 0.5.
        assert!(
            policy.baseline > initial_baseline,
            "baseline should increase: {} -> {}",
            initial_baseline,
            policy.baseline
        );
    }

    #[test]
    fn is_trained_returns_false_for_new_policy() {
        let policy = RoutingPolicy::default();
        assert!(!is_trained(&policy));
    }

    #[test]
    fn caller_scored_examples_never_authorize_active_routing() {
        let mut policy = RoutingPolicy::default();
        let profile = QueryProfile::from_query("test query here");
        let heuristic = RetrievalRouter::default().route(&profile);
        for _ in 0..100 {
            record_routing_outcome(&mut policy, &profile, &heuristic, RoutingOutcome::Good);
        }
        assert_eq!(policy.trained_examples, 100);
        assert!(!is_trained(&policy));
        assert!(same_stages(&route_with_rl(&policy, &profile), &heuristic));
    }

    #[test]
    fn verified_shadow_candidate_is_content_addressed_and_proposal_only() {
        let policy = RoutingPolicy::default();
        let outcomes = (0..100)
            .map(|index| VerifiedRoutingOutcomeV1 {
                verification_receipt_id: format!("verification:{index}"),
                verification_receipt_digest: format!("blake3:{index:064x}"),
                route_bucket: format!("bucket-{}", index % 3),
                family: format!("family-{}", index % 5),
                baseline_score_bps: 8_000,
                candidate_score_bps: 8_400,
                holdout: true,
                correctness_regression: false,
            })
            .collect::<Vec<_>>();

        let first = evaluate_shadow_candidate(&policy, &outcomes);
        let second = evaluate_shadow_candidate(&policy, &outcomes);
        assert_eq!(first, second);
        assert!(first.policy_digest.starts_with("blake3:"));
        assert_eq!(first.eligible_examples, 100);
        assert_eq!(first.effect_percent, 0);
        assert!(first.promotion_proposal);
        assert!(!first.active_route_changed);
    }

    #[test]
    fn shadow_candidate_rejects_sparse_poisoned_or_self_reported_evidence() {
        let policy = RoutingPolicy::default();
        let mut outcomes = (0..99)
            .map(|index| VerifiedRoutingOutcomeV1 {
                verification_receipt_id: format!("verification:{index}"),
                verification_receipt_digest: format!("blake3:{index:064x}"),
                route_bucket: "bucket-a".into(),
                family: format!("family-{}", index % 4),
                baseline_score_bps: 8_000,
                candidate_score_bps: 8_400,
                holdout: true,
                correctness_regression: false,
            })
            .collect::<Vec<_>>();
        outcomes.push(outcomes[0].clone());
        outcomes[1].verification_receipt_digest.clear();
        outcomes[2].correctness_regression = true;

        let report = evaluate_shadow_candidate(&policy, &outcomes);
        assert!(!report.promotion_proposal);
        assert_eq!(report.effect_percent, 0);
        assert!(report
            .reason_codes
            .iter()
            .any(|reason| reason == "duplicate-verification-receipt"));
        assert!(report
            .reason_codes
            .iter()
            .any(|reason| reason == "verification-receipt-digest-missing"));
        assert!(report
            .reason_codes
            .iter()
            .any(|reason| reason == "correctness-regression"));
    }

    #[test]
    fn trained_policy_produces_different_decisions() {
        let mut policy = RoutingPolicy::default();

        // Train with high outcome for decoder-enabled decisions.
        // Use a high learning rate to ensure weights cross the 0.5 threshold.
        policy.learning_rate = 0.1;
        let profile = QueryProfile::from_query("compare rust vs python");
        let decision = RoutingDecision {
            bm25_coarse: true,
            vector_medium: true,
            rerank_fine: true,
            graph_expansion: false,
            decoder: true,
            discord: false,
            no_retrieval: false,
            reasoning: "test".to_string(),
        };
        for _ in 0..50 {
            let example = TrainingExample {
                query_profile: profile.clone(),
                decision: decision.clone(),
                outcome_score: 0.9,
            };
            update_policy(&mut policy, &[example]);
        }

        // Now route a similar query — decoder should be enabled.
        let test_profile = QueryProfile::from_query("compare go vs rust differences");
        let rl_decision = route_with_policy(&policy, &test_profile);
        assert!(
            rl_decision.decoder,
            "trained policy should enable decoder for contradiction queries (decoder weight: {})",
            policy.weights.get("decoder").unwrap_or(&0.0)
        );
    }

    #[test]
    fn trained_policy_prediction_differs_but_executed_route_stays_heuristic() {
        let profile = QueryProfile::from_query("compare rust vs python performance");
        let heuristic = RetrievalRouter::default().route(&profile);
        assert!(heuristic.rerank_fine);

        let mut policy = RoutingPolicy::default();
        let decision = heuristic.clone();
        for _ in 0..11 {
            record_routing_outcome(&mut policy, &profile, &decision, RoutingOutcome::Bad);
        }

        assert!(!is_trained(&policy));
        let shadow_prediction = route_with_policy(&policy, &profile);
        assert_ne!(shadow_prediction.rerank_fine, heuristic.rerank_fine);
        assert!(shadow_prediction.reasoning.starts_with("RL policy"));
        assert!(same_stages(&route_with_rl(&policy, &profile), &heuristic));
    }

    #[test]
    fn empty_examples_does_nothing() {
        let mut policy = RoutingPolicy::default();
        let initial = policy.clone();
        update_policy(&mut policy, &[]);
        assert_eq!(policy.trained_examples, initial.trained_examples);
        assert!((policy.baseline - initial.baseline).abs() < 0.001);
    }

    #[test]
    fn record_routing_outcome_good_increases_weights() {
        let mut policy = RoutingPolicy::default();
        let profile = QueryProfile::from_query("compare rust vs python");
        let decision = RoutingDecision {
            bm25_coarse: true,
            vector_medium: true,
            rerank_fine: true,
            graph_expansion: false,
            decoder: true,
            discord: false,
            no_retrieval: false,
            reasoning: "test".to_string(),
        };
        let initial_bm25_w = *policy.weights.get("bm25_coarse").unwrap();
        record_routing_outcome(&mut policy, &profile, &decision, RoutingOutcome::Good);
        let updated_bm25_w = *policy.weights.get("bm25_coarse").unwrap();
        assert!(
            updated_bm25_w > initial_bm25_w,
            "good outcome should increase weight: {} -> {}",
            initial_bm25_w,
            updated_bm25_w
        );
    }

    #[test]
    fn record_routing_outcome_bad_decreases_weights() {
        let mut policy = RoutingPolicy::default();
        let profile = QueryProfile::from_query("compare rust vs python");
        let decision = RoutingDecision {
            bm25_coarse: true,
            vector_medium: true,
            rerank_fine: true,
            graph_expansion: false,
            decoder: true,
            discord: false,
            no_retrieval: false,
            reasoning: "test".to_string(),
        };
        let initial_bm25_w = *policy.weights.get("bm25_coarse").unwrap();
        record_routing_outcome(&mut policy, &profile, &decision, RoutingOutcome::Bad);
        let updated_bm25_w = *policy.weights.get("bm25_coarse").unwrap();
        assert!(
            updated_bm25_w < initial_bm25_w,
            "bad outcome should decrease weight: {} -> {}",
            initial_bm25_w,
            updated_bm25_w
        );
    }

    #[test]
    fn record_routing_outcome_neutral_does_not_change_much() {
        let mut policy = RoutingPolicy::default();
        let profile = QueryProfile::from_query("test query here");
        let decision = RoutingDecision {
            bm25_coarse: true,
            vector_medium: false,
            rerank_fine: false,
            graph_expansion: false,
            decoder: false,
            discord: false,
            no_retrieval: false,
            reasoning: "test".to_string(),
        };
        let initial_trained = policy.trained_examples;
        record_routing_outcome(&mut policy, &profile, &decision, RoutingOutcome::Neutral);
        assert_eq!(policy.trained_examples, initial_trained + 1);
    }
}
