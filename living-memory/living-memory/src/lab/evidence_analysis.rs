use super::{ExperimentEvidenceBundle, HypothesisEdge, HypothesisEdgeKind, VerificationState};
use crate::config::ForgeLimits;
use crate::error::ForgeResult;
use crate::experiment::ExperimentDiff;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A causal hypothesis linking an edit pattern to an observed effect.
///
/// Legacy structure. Prefer `HypothesisEdge` for new code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalHypothesis {
    /// Unique hypothesis identifier.
    pub hypothesis_id: String,
    /// The edit signature that is hypothesized to cause the effect.
    pub cause_signature: String,
    /// The effect signature observed.
    pub effect_signature: String,
    /// Confidence in this hypothesis (0.0 to 1.0).
    pub confidence: f64,
    /// Current status of the hypothesis.
    pub status: HypothesisStatus,
    /// Number of supporting observations.
    pub support_count: u64,
    /// Number of contradicting observations.
    pub contradiction_count: u64,
}

/// Lifecycle status of a causal hypothesis.
///
/// Phase 5 uses: Proposed, Supported, Contradicted, Neutral.
/// `Confirmed` and `Refuted` are preserved as serde aliases for backward compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisStatus {
    /// Newly proposed, awaiting evidence.
    Proposed,
    /// Partially supported by evidence from paired results.
    Supported,
    /// Contradicted by evidence from paired results.
    #[serde(alias = "Refuted")]
    Contradicted,
    /// Neutral: no evidence supports or contradicts.
    #[serde(alias = "Confirmed")]
    Neutral,
}

/// A structured follow-up plan for verifying hypotheses.
///
/// Phase 5: these are STRUCTURED FOLLOW-UP PLANS, not robust refutation engines.
/// Refutation artifacts (placebo, dummy outcome, subsample stability) are modeled
/// explicitly as first-class evidence-artifact records and can be attached to the
/// bundle without being executed in-plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationPlan {
    /// Unique plan identifier.
    pub plan_id: String,
    /// Hypotheses to verify.
    pub target_hypotheses: Vec<String>,
    /// Steps in the verification.
    pub steps: Vec<VerificationStep>,
    /// Budget constraints for this plan.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<PlanBudget>,
    /// Steps that were dropped due to budget constraints.
    #[serde(default)]
    pub dropped_steps: Vec<DroppedStep>,
}

/// Budget constraints for a verification plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanBudget {
    /// Maximum number of verification steps.
    pub max_steps: usize,
    /// Estimated total duration in seconds.
    pub estimated_duration_secs: u64,
}

/// A step that was dropped from a verification plan due to budget constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroppedStep {
    /// The step that was dropped.
    pub step: VerificationStep,
    /// Why it was dropped.
    pub reason: String,
}

/// A single step in a verification plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStep {
    /// What type of verification to perform.
    pub verification_type: VerificationType,
    /// Description of what this step checks.
    pub description: String,
    /// Expected outcome.
    pub expected_outcome: String,
    /// Whether this step is required for promotion.
    #[serde(default = "default_informational")]
    pub requirement: StepRequirement,
}

fn default_informational() -> StepRequirement {
    StepRequirement::Informational
}

/// Types of verification experiments.
///
/// Phase 5 supports: RunTest, CheckInvariant, ManualReview, CompareBaseline.
/// Reproduce/Generalize/Negate/CrossProject/Ablation are defined but not
/// executed by Phase 5 runners.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationType {
    /// Re-run the same edit and check for same effect.
    Reproduce,
    /// Apply a similar edit to see if effect generalizes.
    Generalize,
    /// Apply a counter-edit to see if effect disappears.
    Negate,
    /// Run on a different codebase to test portability.
    CrossProject,
    /// Run a specific test.
    RunTest,
    /// Check an invariant.
    CheckInvariant,
    /// Compare against baseline.
    CompareBaseline,
    /// Require manual review.
    ManualReview,
    /// Ablation: remove specific edit ops and re-run.
    Ablation { edit_op_signatures: Vec<String> },
}

/// Whether a verification step is required for promotion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepRequirement {
    Required,
    Informational,
}

/// Policy controlling verification plan generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationPolicy {
    /// Maximum ablation combinations to explore.
    #[serde(default = "default_max_ablation")]
    pub max_ablation_combinations: u32,
    /// Whether manual review steps are generated.
    #[serde(default)]
    pub generate_manual_review: bool,
    /// Whether to generate ablation steps for mixed-effect patches.
    #[serde(default = "default_true")]
    pub generate_ablation_for_mixed_effects: bool,
}

fn default_max_ablation() -> u32 {
    16
}

fn default_true() -> bool {
    true
}

impl Default for VerificationPolicy {
    fn default() -> Self {
        Self {
            max_ablation_combinations: default_max_ablation(),
            generate_manual_review: false,
            generate_ablation_for_mixed_effects: true,
        }
    }
}

/// Deterministic evidence assessment derived from policy, not vibes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceAssessment {
    /// Whether the evidence is reproducible (based on trial count).
    pub reproducibility: AssessmentCategory,
    /// Whether the execution was properly isolated.
    pub isolation: AssessmentCategory,
    /// Whether any hypotheses are contradicted.
    pub contradiction_state: ContradictionState,
    /// Sample support level.
    pub sample_support: SampleSupport,
}

/// Assessment category (deterministic, policy-derived).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentCategory {
    Strong,
    Adequate,
    Weak,
    Insufficient,
}

/// Whether hypotheses are contradicted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionState {
    /// No contradictions found.
    Clean,
    /// Some hypotheses have contradicting observations.
    HasContradictions,
    /// Insufficient data to determine.
    Undetermined,
}

/// Level of sample support for claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SampleSupport {
    /// Sufficient trials for statistical claims.
    Sufficient,
    /// Marginal — claims should carry warnings.
    Marginal,
    /// Insufficient — no scalar claims allowed.
    Insufficient,
}

/// Result of checking whether a baseline/patched pair is comparable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairComparability {
    /// Whether the pair is valid for attribution.
    pub valid: bool,
    /// Reasons why the pair is incomparable, if any.
    pub violations: Vec<String>,
}

impl PairComparability {
    /// Check pair comparability between baseline and patched trial contexts.
    ///
    /// A paired comparison is valid only if baseline and patched runs share:
    /// - same workload_id
    /// - same ordered selected_checks
    /// - same effective timeout class
    /// - same execution backend family
    /// - same config-flag projection
    #[allow(clippy::too_many_arguments)]
    pub fn check(
        baseline_workload: &str,
        patched_workload: &str,
        baseline_checks: &[String],
        patched_checks: &[String],
        baseline_timeout_class: &str,
        patched_timeout_class: &str,
        baseline_backend: &str,
        patched_backend: &str,
        baseline_config_flags: &[String],
        patched_config_flags: &[String],
    ) -> Self {
        let mut violations = Vec::new();

        if baseline_workload != patched_workload {
            violations.push(format!(
                "workload mismatch: baseline={baseline_workload}, patched={patched_workload}"
            ));
        }
        if baseline_checks != patched_checks {
            violations.push(format!(
                "selected_checks mismatch: baseline={baseline_checks:?}, patched={patched_checks:?}"
            ));
        }
        if baseline_timeout_class != patched_timeout_class {
            violations.push(format!(
                "timeout_class mismatch: baseline={baseline_timeout_class}, patched={patched_timeout_class}"
            ));
        }
        if baseline_backend != patched_backend {
            violations.push(format!(
                "backend mismatch: baseline={baseline_backend}, patched={patched_backend}"
            ));
        }
        if baseline_config_flags != patched_config_flags {
            violations.push(format!(
                "config_flags mismatch: baseline={baseline_config_flags:?}, patched={patched_config_flags:?}"
            ));
        }

        Self {
            valid: violations.is_empty(),
            violations,
        }
    }

    /// Check timeout asymmetry: if baseline times out and patched succeeds,
    /// the pair is invalid for attribution.
    pub fn check_timeout_asymmetry(
        baseline_timed_out: bool,
        patched_timed_out: bool,
    ) -> Option<String> {
        if baseline_timed_out && !patched_timed_out {
            Some("baseline timed out but patched succeeded: pair invalid for attribution".into())
        } else {
            None
        }
    }
}

/// Trait for running verification experiments.
#[async_trait::async_trait]
pub trait ExperimentRunner: Send + Sync {
    /// Execute a verification plan and return updated hypotheses.
    async fn run_plan(
        &self,
        plan: &VerificationPlan,
        bundle: &ExperimentEvidenceBundle,
    ) -> ForgeResult<Vec<CausalHypothesis>>;
}

/// Local experiment runner that executes experiments on the host machine.
///
/// Phase 5 limitation: this runner logs verification steps but does NOT
/// actually execute real checks. It records steps as planned, without
/// fabricating support/contradiction counts.
pub struct LocalExperimentRunner {
    /// Working directory for experiments.
    pub workspace_dir: PathBuf,
    /// Maximum concurrent experiments.
    pub max_parallelism: usize,
}

impl LocalExperimentRunner {
    /// Create a new local experiment runner.
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self {
            workspace_dir,
            max_parallelism: 4,
        }
    }

    /// Set the maximum parallelism.
    pub fn with_max_parallelism(mut self, max: usize) -> Self {
        self.max_parallelism = max.clamp(1, 32);
        self
    }
}

#[async_trait::async_trait]
impl ExperimentRunner for LocalExperimentRunner {
    async fn run_plan(
        &self,
        plan: &VerificationPlan,
        bundle: &ExperimentEvidenceBundle,
    ) -> ForgeResult<Vec<CausalHypothesis>> {
        tracing::info!(
            plan_id = %plan.plan_id,
            bundle_id = %bundle.bundle_id,
            steps = plan.steps.len(),
            workspace = %self.workspace_dir.display(),
            "recording verification plan (Phase 5: plan-only, no execution)"
        );

        let updated = bundle.hypotheses.clone();
        for step in &plan.steps {
            tracing::info!(
                verification_type = ?step.verification_type,
                description = %step.description,
                "recorded verification step (not executed in Phase 5)"
            );
        }

        Ok(updated)
    }
}

/// Derive hypothesis status from observation counts.
pub fn derive_status(support: u64, contradictions: u64) -> HypothesisStatus {
    if support == 0 && contradictions == 0 {
        HypothesisStatus::Proposed
    } else if contradictions > support {
        HypothesisStatus::Contradicted
    } else if support > contradictions {
        HypothesisStatus::Supported
    } else {
        HypothesisStatus::Neutral
    }
}

/// Compute hypothesis confidence from observation counts.
pub fn local_hypothesis_support_confidence(support: u64, contradictions: u64) -> f64 {
    let prior = 1.0_f64;
    let total = support as f64 + contradictions as f64 + prior;
    (support as f64 / total).clamp(0.0, 1.0)
}

/// Compatibility alias for historical call sites.
///
/// Phase status: migration-only
/// Removal condition: remove when all consumers have migrated to `local_hypothesis_support_confidence`
#[deprecated(note = "Use `local_hypothesis_support_confidence` for explicit local semantics")]
pub fn compute_confidence(support: u64, contradictions: u64) -> f64 {
    local_hypothesis_support_confidence(support, contradictions)
}

/// Update hypotheses from typed experiment effects.
pub fn update_hypotheses_from_diff(hypotheses: &mut [CausalHypothesis], diff: &ExperimentDiff) {
    for effect in &diff.effects {
        for hypothesis in hypotheses.iter_mut() {
            let matches = effect.message.contains(&hypothesis.effect_signature)
                || hypothesis.effect_signature.contains(&effect.message);
            if !matches {
                continue;
            }

            if effect.in_patched != effect.in_baseline {
                hypothesis.support_count += 1;
            }

            hypothesis.status =
                derive_status(hypothesis.support_count, hypothesis.contradiction_count);
            hypothesis.confidence = local_hypothesis_support_confidence(
                hypothesis.support_count,
                hypothesis.contradiction_count,
            );
        }
    }
}

/// Build hypothesis edges from an ExperimentDiff.
pub fn build_hypothesis_edges(diff: &ExperimentDiff, bundle_id: &str) -> Vec<HypothesisEdge> {
    let mut edges = Vec::new();

    for effect in &diff.effects {
        let kind = if effect.in_patched && !effect.in_baseline {
            HypothesisEdgeKind::CausesRegression
        } else if effect.in_baseline && !effect.in_patched {
            HypothesisEdgeKind::FixesFailure
        } else if effect.in_baseline && effect.in_patched {
            HypothesisEdgeKind::AssociatedWithStableFailure
        } else {
            continue;
        };

        let status = match kind {
            HypothesisEdgeKind::CausesRegression | HypothesisEdgeKind::FixesFailure => {
                HypothesisStatus::Supported
            }
            HypothesisEdgeKind::AssociatedWithStableFailure => HypothesisStatus::Neutral,
        };
        let confidence = match kind {
            HypothesisEdgeKind::CausesRegression | HypothesisEdgeKind::FixesFailure => {
                local_hypothesis_support_confidence(1, 0)
            }
            HypothesisEdgeKind::AssociatedWithStableFailure => 0.0,
        };
        let edge_id = format!(
            "edge-{}-{}",
            bundle_id,
            blake3::hash(effect.message.as_bytes())
                .to_hex()
                .to_string()
                .get(..8)
                .unwrap_or("00000000")
        );

        edges.push(HypothesisEdge {
            edge_id,
            source_edit: String::new(),
            target_effect: serde_json::to_string(effect).unwrap_or_default(),
            kind,
            status,
            confidence,
            evidence_ids: vec![bundle_id.to_string()],
            contradiction_ids: vec![],
            verification_status: VerificationState::Unverified,
        });
    }

    edges
}

/// Generate a verification plan from experiment results and hypotheses.
pub fn generate_verification_plan(
    bundle: &ExperimentEvidenceBundle,
    edit_op_signatures: &[String],
    policy: &VerificationPolicy,
    limits: &ForgeLimits,
) -> VerificationPlan {
    let plan_id = uuid::Uuid::new_v4().to_string();
    let target_hypotheses = bundle
        .hypotheses
        .iter()
        .map(|hypothesis| hypothesis.hypothesis_id.clone())
        .collect();

    let mut all_steps = Vec::new();
    let mut dropped_steps = Vec::new();

    if let Some(diff) = bundle.experiment_diff.as_ref() {
        for effect in &diff.effects {
            if effect.in_patched && !effect.in_baseline {
                all_steps.push(VerificationStep {
                    verification_type: VerificationType::RunTest,
                    description: format!("Re-run test for new effect: {}", effect.message),
                    expected_outcome: "Effect reproduces consistently".to_string(),
                    requirement: StepRequirement::Required,
                });
            }
        }
    }

    all_steps.push(VerificationStep {
        verification_type: VerificationType::CompareBaseline,
        description: "Verify baseline results are stable".to_string(),
        expected_outcome: "Baseline unchanged".to_string(),
        requirement: StepRequirement::Required,
    });
    all_steps.push(VerificationStep {
        verification_type: VerificationType::CheckInvariant,
        description: "Verify no invariant violations".to_string(),
        expected_outcome: "Zero violations".to_string(),
        requirement: StepRequirement::Required,
    });

    if bundle
        .hypotheses
        .iter()
        .any(|hypothesis| hypothesis.confidence < 0.3)
    {
        all_steps.push(VerificationStep {
            verification_type: VerificationType::ManualReview,
            description: "Manual review for low-confidence hypotheses".to_string(),
            expected_outcome: "Reviewer confirms or rejects".to_string(),
            requirement: StepRequirement::Informational,
        });
    }

    if policy.generate_ablation_for_mixed_effects {
        if let Some(diff) = bundle.experiment_diff.as_ref() {
            if diff.regressions > 0 && diff.improvements > 0 && !edit_op_signatures.is_empty() {
                let max_ablations = policy
                    .max_ablation_combinations
                    .min(edit_op_signatures.len() as u32);
                for (index, signature) in edit_op_signatures.iter().enumerate() {
                    if index >= max_ablations as usize {
                        break;
                    }
                    all_steps.push(VerificationStep {
                        verification_type: VerificationType::Ablation {
                            edit_op_signatures: vec![signature.clone()],
                        },
                        description: format!("Ablation: remove edit op {index} and re-run"),
                        expected_outcome: "Identify which edit caused each effect".to_string(),
                        requirement: StepRequirement::Informational,
                    });
                }
            }
        }
    }

    if policy.generate_manual_review {
        all_steps.push(VerificationStep {
            verification_type: VerificationType::ManualReview,
            description: "Manual review of patch and effects".to_string(),
            expected_outcome: "Reviewer approves".to_string(),
            requirement: StepRequirement::Informational,
        });
    }

    let max_steps = limits.max_verification_steps;
    let mut steps = Vec::new();
    for step in all_steps {
        if steps.len() >= max_steps {
            dropped_steps.push(DroppedStep {
                step,
                reason: format!("exceeded max_verification_steps={max_steps}"),
            });
        } else {
            steps.push(step);
        }
    }

    VerificationPlan {
        plan_id,
        target_hypotheses,
        budget: Some(PlanBudget {
            max_steps,
            estimated_duration_secs: steps.len() as u64 * 60,
        }),
        steps,
        dropped_steps,
    }
}

/// Compute a deterministic evidence assessment from the bundle.
pub fn compute_assessment(
    bundle: &ExperimentEvidenceBundle,
    trial_count: u32,
    isolated: bool,
    min_trials: u32,
) -> EvidenceAssessment {
    let reproducibility = if trial_count >= min_trials {
        AssessmentCategory::Strong
    } else if trial_count >= 2 {
        AssessmentCategory::Adequate
    } else if trial_count == 1 {
        AssessmentCategory::Weak
    } else {
        AssessmentCategory::Insufficient
    };
    let isolation = if isolated {
        AssessmentCategory::Strong
    } else {
        AssessmentCategory::Weak
    };
    let has_contradictions = bundle
        .hypotheses
        .iter()
        .any(|hypothesis| hypothesis.contradiction_count > 0);
    let contradiction_state = if bundle.hypotheses.is_empty() {
        ContradictionState::Undetermined
    } else if has_contradictions {
        ContradictionState::HasContradictions
    } else {
        ContradictionState::Clean
    };
    let total_observations: u64 = bundle
        .hypotheses
        .iter()
        .map(|hypothesis| hypothesis.support_count + hypothesis.contradiction_count)
        .sum();
    let sample_support = if total_observations >= 10 {
        SampleSupport::Sufficient
    } else if total_observations >= 3 {
        SampleSupport::Marginal
    } else {
        SampleSupport::Insufficient
    };

    EvidenceAssessment {
        reproducibility,
        isolation,
        contradiction_state,
        sample_support,
    }
}
