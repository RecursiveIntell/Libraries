//! Bounded, non-authoritative composition for coding-learning evaluation.
use blake3::Hasher;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationEvidence {
    pub execution_mode: String,
    pub verification_positive: bool,
    pub safety_violations: u32,
    pub receipt_complete: bool,
}
impl EvaluationEvidence {
    pub fn fixture() -> Self {
        Self {
            execution_mode: "fixture".into(),
            verification_positive: true,
            safety_violations: 0,
            receipt_complete: true,
        }
    }
    pub fn eligible_for_learning(&self) -> bool {
        self.execution_mode == "real_sandbox"
            && self.verification_positive
            && self.safety_violations == 0
            && self.receipt_complete
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxRun {
    pub task: String,
    pub lineage: String,
    pub tools: Vec<String>,
    pub patch: String,
    pub verified: bool,
}
impl SandboxRun {
    pub fn verified(
        task: impl Into<String>,
        lineage: impl Into<String>,
        tools: Vec<String>,
    ) -> Self {
        Self {
            task: task.into(),
            lineage: lineage.into(),
            tools,
            patch: String::new(),
            verified: true,
        }
    }
    pub fn with_patch(mut self, patch: impl Into<String>) -> Self {
        self.patch = patch.into();
        self
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CandidateStatus {
    Quarantined,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub id: String,
    pub status: CandidateStatus,
    pub reasons: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateRejection {
    pub reasons: Vec<String>,
}

pub fn extract_candidate(run: &SandboxRun) -> Result<Candidate, CandidateRejection> {
    let known = ["read_file", "search_files", "patch", "write_file"];
    let mut reasons = Vec::new();
    if !run.verified {
        reasons.push("verification-not-positive".into());
    }
    if run.lineage.trim().is_empty() {
        reasons.push("missing-lineage".into());
    }
    if run.tools.iter().any(|t| !known.contains(&t.as_str())) {
        reasons.push("unknown-tool".into());
    }
    let lower = run.patch.to_ascii_lowercase();
    if ["secret", "token=", "password", "api_key"]
        .iter()
        .any(|s| lower.contains(s))
    {
        reasons.push("secret-detected".into());
    }
    if !reasons.is_empty() {
        reasons.sort();
        reasons.dedup();
        return Err(CandidateRejection { reasons });
    }
    let mut h = Hasher::new();
    h.update(run.task.as_bytes());
    h.update(run.lineage.as_bytes());
    h.update(run.patch.as_bytes());
    for tool in &run.tools {
        h.update(tool.as_bytes());
    }
    Ok(Candidate {
        id: h.finalize().to_hex().to_string(),
        status: CandidateStatus::Quarantined,
        reasons: vec!["quarantine-first".into()],
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrialSpec {
    pub task_digest: String,
    pub verifier_digest: String,
    pub environment_digest: String,
    pub policy_digest: String,
    pub seed: u64,
}
impl TrialSpec {
    pub fn new(
        t: impl Into<String>,
        v: impl Into<String>,
        e: impl Into<String>,
        p: impl Into<String>,
        seed: u64,
    ) -> Self {
        Self {
            task_digest: t.into(),
            verifier_digest: v.into(),
            environment_digest: e.into(),
            policy_digest: p.into(),
            seed,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trial {
    pub task_digest: String,
    pub verifier_digest: String,
    pub environment_digest: String,
    pub policy_digest: String,
    pub seed: u64,
    pub candidate: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairedTrials {
    pub trials: Vec<Trial>,
    pub denominator: usize,
    pub causal_unavailable: bool,
}
pub fn compose_paired_trials(spec: TrialSpec, families: usize) -> PairedTrials {
    let mut trials = Vec::new();
    for _ in 0..families {
        for candidate in [false, true] {
            trials.push(Trial {
                task_digest: spec.task_digest.clone(),
                verifier_digest: spec.verifier_digest.clone(),
                environment_digest: spec.environment_digest.clone(),
                policy_digest: spec.policy_digest.clone(),
                seed: spec.seed,
                candidate,
            });
        }
    }
    PairedTrials {
        denominator: trials.len(),
        trials,
        causal_unavailable: true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleDecision {
    Promote,
    Quarantine,
    Revoke,
    Rollback,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleInput {
    pub permit: bool,
    pub paired_trials: usize,
    pub families: usize,
    pub verification_positive: bool,
    pub safety_violations: usize,
    pub evaluator_suppression: bool,
    pub holdout_regression_points: u8,
    pub replay_agreement_percent: u8,
}
impl Default for LifecycleInput {
    fn default() -> Self {
        Self {
            permit: false,
            paired_trials: 0,
            families: 0,
            verification_positive: false,
            safety_violations: 0,
            evaluator_suppression: false,
            holdout_regression_points: 99,
            replay_agreement_percent: 0,
        }
    }
}
impl LifecycleInput {
    pub fn passing() -> Self {
        Self {
            permit: true,
            paired_trials: 20,
            families: 5,
            verification_positive: true,
            safety_violations: 0,
            evaluator_suppression: false,
            holdout_regression_points: 0,
            replay_agreement_percent: 95,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleRequest {
    pub decision: LifecycleDecision,
    pub reasons: Vec<String>,
}
pub fn lifecycle_decision(i: LifecycleInput) -> LifecycleRequest {
    let mut r = Vec::new();
    if !i.permit {
        r.push("lifecycle-permit-required".into());
    }
    if i.paired_trials < 20 {
        r.push("paired-trials-below-minimum".into());
    }
    if i.families < 5 {
        r.push("task-families-below-minimum".into());
    }
    if !i.verification_positive {
        r.push("verification-not-positive".into());
    }
    if i.safety_violations > 0 {
        r.push("safety-violations".into());
    }
    if i.evaluator_suppression {
        r.push("evaluator-suppression".into());
    }
    if i.holdout_regression_points > 2 {
        r.push("holdout-regression".into());
    }
    if i.replay_agreement_percent < 95 {
        r.push("replay-agreement-below-minimum".into());
    }
    LifecycleRequest {
        decision: if r.is_empty() {
            LifecycleDecision::Promote
        } else {
            LifecycleDecision::Quarantine
        },
        reasons: r,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayMode {
    NoReplay,
    StoreInputs,
    ReplayEvaluation,
    MetadataOnly,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayResult {
    ReplayMatch,
    ReplayMismatch,
    Drift,
    Inconclusive,
    NotAvailable,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayReport {
    pub result: ReplayResult,
    pub verifier_drift: bool,
    pub store_drift: bool,
    pub environment_drift: bool,
}
pub fn replay(_spec: TrialSpec, mode: ReplayMode) -> ReplayReport {
    match mode {
        ReplayMode::NoReplay => ReplayReport {
            result: ReplayResult::NotAvailable,
            verifier_drift: false,
            store_drift: false,
            environment_drift: false,
        },
        ReplayMode::StoreInputs => ReplayReport {
            result: ReplayResult::Inconclusive,
            verifier_drift: false,
            store_drift: false,
            environment_drift: false,
        },
        ReplayMode::MetadataOnly => ReplayReport {
            result: ReplayResult::NotAvailable,
            verifier_drift: false,
            store_drift: false,
            environment_drift: false,
        },
        ReplayMode::ReplayEvaluation => ReplayReport {
            result: ReplayResult::ReplayMismatch,
            verifier_drift: true,
            store_drift: false,
            environment_drift: false,
        },
    }
}

pub fn replay_with_observed(
    expected: TrialSpec,
    mode: ReplayMode,
    observed: TrialSpec,
) -> ReplayReport {
    if mode == ReplayMode::MetadataOnly {
        return replay(expected, mode);
    }
    if mode != ReplayMode::ReplayEvaluation {
        return replay(expected, mode);
    }
    let verifier_drift = expected.verifier_digest != observed.verifier_digest;
    let store_drift = expected.policy_digest != observed.policy_digest;
    let environment_drift = expected.environment_digest != observed.environment_digest;
    let task_drift = expected.task_digest != observed.task_digest || expected.seed != observed.seed;
    ReplayReport {
        result: if verifier_drift || store_drift || environment_drift || task_drift {
            ReplayResult::Drift
        } else {
            ReplayResult::ReplayMatch
        },
        verifier_drift,
        store_drift,
        environment_drift,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureVerticalSlice {
    pub fixture_evidence: EvaluationEvidence,
    pub candidate: Candidate,
    pub paired_trials: PairedTrials,
    pub lifecycle: LifecycleDecision,
    pub replay: ReplayReport,
}

pub fn run_fixture_vertical_slice() -> Result<FixtureVerticalSlice, CandidateRejection> {
    let evidence = EvaluationEvidence::fixture();
    let run = SandboxRun::verified(
        "medusa-task-12",
        "fixture-lineage",
        vec!["read_file".into()],
    )
    .with_patch("deterministic candidate");
    let candidate = extract_candidate(&run)?;
    let trials = compose_paired_trials(
        TrialSpec::new(
            "task-digest",
            "verifier-digest",
            "environment-digest",
            "policy-digest",
            12,
        ),
        5,
    );
    let promoted = lifecycle_decision(LifecycleInput::passing());
    let replay = replay_with_observed(
        TrialSpec::new(
            "task-digest",
            "verifier-digest",
            "environment-digest",
            "policy-digest",
            12,
        ),
        ReplayMode::ReplayEvaluation,
        TrialSpec::new(
            "task-digest",
            "verifier-digest",
            "environment-digest",
            "policy-digest",
            12,
        ),
    );
    Ok(FixtureVerticalSlice {
        fixture_evidence: evidence,
        candidate,
        paired_trials: trials,
        lifecycle: if promoted.decision == LifecycleDecision::Promote {
            LifecycleDecision::Revoke
        } else {
            promoted.decision
        },
        replay,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixture_evidence_can_never_promote() {
        assert!(!EvaluationEvidence::fixture().eligible_for_learning());
    }
    #[test]
    fn candidate_extraction_is_deterministic_and_quarantined() {
        let r = SandboxRun::verified("task-1", "lineage-1", vec!["read_file".into()]);
        let a = extract_candidate(&r).unwrap();
        assert_eq!(a, extract_candidate(&r).unwrap());
        assert_eq!(a.status, CandidateStatus::Quarantined);
    }
    #[test]
    fn candidate_rejects_secret_and_unknown_tool() {
        let r = SandboxRun::verified("task", "lineage", vec!["unknown".into()])
            .with_patch("token=secret");
        let e = extract_candidate(&r).unwrap_err();
        assert!(e.reasons.contains(&"unknown-tool".into()));
        assert!(e.reasons.contains(&"secret-detected".into()));
    }
    #[test]
    fn paired_trials_freeze_inputs_and_account_denominator() {
        let t = compose_paired_trials(
            TrialSpec::new("task", "verifier", "environment", "policy", 7),
            3,
        );
        assert_eq!(t.denominator, 6);
        assert!(t.causal_unavailable);
        assert!(t
            .trials
            .iter()
            .all(|x| x.task_digest == "task" && x.seed == 7));
    }
    #[test]
    fn lifecycle_requires_permit_and_all_hard_gates() {
        assert_eq!(
            lifecycle_decision(LifecycleInput::default()).decision,
            LifecycleDecision::Quarantine
        );
        assert_eq!(
            lifecycle_decision(LifecycleInput::passing()).decision,
            LifecycleDecision::Promote
        );
    }
    #[test]
    fn replay_modes_report_drift_and_exact_match() {
        let s = TrialSpec::new("t", "v", "e", "p", 1);
        assert_eq!(
            replay(s.clone(), ReplayMode::NoReplay).result,
            ReplayResult::NotAvailable
        );
        assert_eq!(
            replay(s.clone(), ReplayMode::StoreInputs).result,
            ReplayResult::Inconclusive
        );
        assert_eq!(
            replay(s, ReplayMode::ReplayEvaluation).result,
            ReplayResult::ReplayMismatch
        );
    }

    #[test]
    fn fixture_vertical_slice_is_deterministic_and_non_promoting() {
        let first = run_fixture_vertical_slice().unwrap();
        let second = run_fixture_vertical_slice().unwrap();
        assert_eq!(first, second);
        assert_eq!(first.lifecycle, LifecycleDecision::Revoke);
        assert_eq!(first.replay.result, ReplayResult::ReplayMatch);
        assert!(!first.fixture_evidence.eligible_for_learning());
    }

    #[test]
    fn replay_classifies_drift_and_old_metadata_as_unavailable() {
        let spec = TrialSpec::new("t", "v", "e", "p", 1);
        assert_eq!(
            replay_with_observed(spec.clone(), ReplayMode::ReplayEvaluation, spec.clone()).result,
            ReplayResult::ReplayMatch
        );
        let drift = TrialSpec::new("t", "changed", "e", "p", 1);
        assert_eq!(
            replay_with_observed(spec, ReplayMode::ReplayEvaluation, drift).result,
            ReplayResult::Drift
        );
        assert_eq!(
            replay_with_observed(
                TrialSpec::new("t", "v", "e", "p", 1),
                ReplayMode::MetadataOnly,
                TrialSpec::new("t", "v", "e", "p", 1)
            )
            .result,
            ReplayResult::NotAvailable
        );
    }
}
