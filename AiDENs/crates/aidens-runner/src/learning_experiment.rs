//! Thin, owner-backed Medusa paired-evaluation adapter.
//! This module derives advice from canonical owner evidence; it never owns lifecycle truth.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Baseline,
    Candidate,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationPlan {
    pub task_digest: String,
    pub family_digest: String,
    pub split_digest: String,
    pub verifier_digest: String,
    pub environment_digest: String,
    pub policy_digest: String,
    pub candidate_digest: String,
    pub baseline_digest: String,
    pub order_seed: u64,
}
impl EvaluationPlan {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        t: impl Into<String>,
        f: impl Into<String>,
        s: impl Into<String>,
        v: impl Into<String>,
        e: impl Into<String>,
        p: impl Into<String>,
        c: impl Into<String>,
        b: impl Into<String>,
        seed: u64,
    ) -> Self {
        Self {
            task_digest: t.into(),
            family_digest: f.into(),
            split_digest: s.into(),
            verifier_digest: v.into(),
            environment_digest: e.into(),
            policy_digest: p.into(),
            candidate_digest: c.into(),
            baseline_digest: b.into(),
            order_seed: seed,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerTrialEvidence {
    pub trial_id: String,
    pub family: String,
    pub split: String,
    pub side: Side,
    pub reward: i32,
    pub verifier_digest: String,
    pub environment_digest: String,
    pub policy_digest: String,
    pub run_present: bool,
    pub verification_positive: bool,
    pub safety_violations: u32,
    pub evaluator_suppressed: bool,
    pub holdout_regression_points: u8,
    pub replay_agreement_percent: u8,
}
impl OwnerTrialEvidence {
    pub fn new(
        id: impl Into<String>,
        family: impl Into<String>,
        split: impl Into<String>,
        side: Side,
        reward: i32,
    ) -> Self {
        Self {
            trial_id: id.into(),
            family: family.into(),
            split: split.into(),
            side,
            reward,
            verifier_digest: "verifier".into(),
            environment_digest: "environment".into(),
            policy_digest: "policy".into(),
            run_present: true,
            verification_positive: true,
            safety_violations: 0,
            evaluator_suppressed: false,
            holdout_regression_points: 0,
            replay_agreement_percent: 100,
        }
    }
    pub fn with_verifier(mut self, value: impl Into<String>) -> Self {
        self.verifier_digest = value.into();
        self
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    Quarantine,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionEvidence {
    pub decision: Decision,
    pub denominator: usize,
    pub paired_trials: usize,
    pub families: usize,
    pub reward_vector: Vec<i32>,
    pub reasons: Vec<String>,
}

pub fn evaluate(plan: &EvaluationPlan, evidence: &[OwnerTrialEvidence]) -> DecisionEvidence {
    let mut reasons = Vec::new();
    let mut seen = BTreeSet::new();
    let mut groups: BTreeMap<(&str, &str), (usize, usize, i32)> = BTreeMap::new();
    for e in evidence {
        if !seen.insert(e.trial_id.as_str()) {
            reasons.push("duplicate-trial".into());
        }
        if e.verifier_digest != plan.verifier_digest
            || e.environment_digest != plan.environment_digest
            || e.policy_digest != plan.policy_digest
        {
            reasons.push("changed-verifier-or-budget".into());
        }
        if e.family.is_empty() || e.split.is_empty() {
            reasons.push("family-or-split-leakage".into());
        }
        let g = groups
            .entry((e.family.as_str(), e.split.as_str()))
            .or_insert((0, 0, 0));
        match e.side {
            Side::Baseline => g.0 += 1,
            Side::Candidate => {
                g.1 += 1;
                g.2 += e.reward;
            }
        }
        if !e.run_present {
            reasons.push("missing-run".into());
        }
        if !e.verification_positive {
            reasons.push("verification-not-positive".into());
        }
        if e.safety_violations > 0 {
            reasons.push("safety-violations".into());
        }
        if e.evaluator_suppressed {
            reasons.push("evaluator-suppression".into());
        }
        if e.split == "holdout" && e.holdout_regression_points > 2 {
            reasons.push("holdout-regression".into());
        }
        if e.replay_agreement_percent < 95 {
            reasons.push("replay-agreement-below-minimum".into());
        }
    }
    let denominator = groups.values().map(|(b, c, _)| (*b).max(*c)).sum();
    let paired_trials = groups.values().map(|(b, c, _)| (*b).min(*c)).sum();
    let families = groups
        .keys()
        .map(|(f, _)| *f)
        .collect::<BTreeSet<_>>()
        .len();
    if denominator < 20 {
        reasons.push("paired-trials-below-minimum".into());
    }
    if families < 5 {
        reasons.push("task-families-below-minimum".into());
    }
    let mut reward_vector = groups.values().map(|(_, _, r)| *r).collect::<Vec<_>>();
    reward_vector.sort();
    reasons.sort();
    reasons.dedup();
    if reasons.is_empty() {
        reasons.push("qualifying-advisory-evidence".into());
    }
    DecisionEvidence {
        decision: Decision::Quarantine,
        denominator,
        paired_trials,
        families,
        reward_vector,
        reasons,
    }
}
