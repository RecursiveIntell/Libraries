//! Receipt-bearing causal experiment orchestration.
//!
//! This module records what was executed and what the differential checker
//! observed.  Its receipts are integrity evidence, not a general proof of
//! causality; proximity attribution remains observational unless an ablation
//! classification says otherwise.

use serde::{Deserialize, Serialize};

use crate::adapters::ProjectAdapter;
use crate::cea::{
    attribute_effects, load_graph, predict, update_graph, CausalPrediction, CoverageSummary,
    EditOpSignature, UpdateResult,
};
use crate::config::ForgeConfig;
use crate::error::{ForgeError, ForgeResult};
use crate::exec::backend::{CheckResult, ExecutionBackend};
use crate::experiment::{
    ExperimentConfig, ExperimentResult, PairedExperimentRunner, PairedTrialResult,
};
use crate::runtime::patch::types::StructuredPatch;
use crate::store::ForgeStore;
use cea_core::{AttributedRunResult, EvidenceKind, ObservationIdentity};

/// Result of the mandatory prediction-policy decision.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PredictionDisposition {
    RunChecks,
    MaySkipChecks,
}

/// Reasons that prevent a prediction from replacing verification.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PredictionGateReason {
    DisabledOptIn,
    InsufficientIndependentRuns,
    LowCoverage,
    FuzzyOnlyEvidence,
    ScopeOrConfigMismatch,
    MissingInterventionalEvidence,
    RiskFlags,
    UnknownEffects,
}

/// Explicit check gate; association-only graph data never returns `MaySkipChecks`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionGate {
    pub disposition: PredictionDisposition,
    pub reasons: Vec<PredictionGateReason>,
}

/// Advisory prediction plus its compulsory execution gate and digest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionReceipt {
    pub prediction: CausalPrediction,
    pub gate: PredictionGate,
    pub digest: String,
}

/// Prediction values observed at one side of a causal graph update.
///
/// The prediction remains advisory; this summary exists so downstream
/// evidence consumers never have to infer values from a prediction digest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionSummary {
    pub digest: String,
    pub predicted_correctness: f64,
    pub confidence: f64,
    pub coverage_fraction: f64,
}

/// A deterministic, tamper-evident receipt for one graph update attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalUpdateReceipt {
    pub receipt_digest: String,
    /// Compatibility alias for the experiment evidence kind.
    pub evidence_kind: EvidenceKind,
    /// The matched baseline/full-patch execution is a patch-level intervention.
    pub experiment_evidence_kind: EvidenceKind,
    /// Proximity triples are edit-localization hypotheses, never per-edit proof.
    pub attribution_evidence_kind: EvidenceKind,
    pub observation_identity: ObservationIdentity,
    pub run_digest: String,
    pub trial_digest: String,
    pub patch_digest: String,
    pub base_digest: String,
    pub config_digest: String,
    pub triple_digest: String,
    pub triple_count: usize,
    pub regression_count: u32,
    pub improvement_count: u32,
    pub stable_count: u32,
    pub pre_prediction_digest: String,
    pub post_prediction_digest: String,
    pub pre_prediction: PredictionSummary,
    pub post_prediction: PredictionSummary,
    pub update_disposition: String,
    pub coverage: CoverageSummary,
    pub prediction_disposition: PredictionDisposition,
    pub degradation_reasons: Vec<String>,
}

impl CausalUpdateReceipt {
    /// Verify the deterministic receipt binding after transport or storage.
    pub fn verify_integrity(&self) -> bool {
        self.receipt_digest == receipt_digest_for(self)
    }
}

/// An executed experiment with the update receipts it produced.
#[derive(Debug, Clone)]
pub struct CausalExperimentResult {
    pub experiment: ExperimentResult,
    pub receipts: Vec<CausalUpdateReceipt>,
}

/// Intervention outcome for a singleton ablation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AblationClassification {
    Supported,
    Contradicted,
    Inconclusive,
}

/// Replayable record for one attempted operation removal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AblationReceipt {
    pub operation_index: usize,
    pub classification: AblationClassification,
    pub evidence_kind: EvidenceKind,
    pub verification_method: String,
    pub comparable: bool,
    pub full_patch_digest: String,
    pub ablated_patch_digest: Option<String>,
    pub baseline_digest: String,
    pub config_digest: String,
    pub baseline_effect_digest: String,
    pub full_outcome_digest: String,
    pub ablated_outcome_digest: String,
    pub removed_effect_count: usize,
    pub persisting_effect_count: usize,
    pub new_effect_count: usize,
    pub receipt_digest: String,
    pub degradation_reasons: Vec<String>,
}

impl AblationReceipt {
    /// Verify the exact-effect-set ablation receipt after transport.
    pub fn verify_integrity(&self) -> bool {
        self.receipt_digest == ablation_receipt_digest(self)
    }
}

/// Engine bound to the canonical Forge store and one configured version.
pub struct CausalAttributionEngine<'a> {
    store: &'a ForgeStore,
    backend: &'a dyn ExecutionBackend,
    adapter: &'a dyn ProjectAdapter,
    config: &'a ForgeConfig,
    version_id: String,
}

impl<'a> CausalAttributionEngine<'a> {
    pub fn new(
        store: &'a ForgeStore,
        backend: &'a dyn ExecutionBackend,
        adapter: &'a dyn ProjectAdapter,
        config: &'a ForgeConfig,
        version_id: impl Into<String>,
    ) -> Self {
        Self {
            store,
            backend,
            adapter,
            config,
            version_id: version_id.into(),
        }
    }

    pub fn coverage(&self) -> ForgeResult<CoverageSummary> {
        Ok(load_graph(self.store, Some(&self.version_id))?.coverage_summary())
    }

    pub fn prediction_gate(
        &self,
        prediction: &CausalPrediction,
        independent_runs: usize,
        fuzzy_only: bool,
        scope_matches: bool,
        has_interventional_evidence: bool,
    ) -> PredictionGate {
        let mut reasons = Vec::new();
        if !self.config.cea.enable_zero_shot {
            reasons.push(PredictionGateReason::DisabledOptIn);
        }
        if independent_runs < self.config.cea.min_runs_before_prediction {
            reasons.push(PredictionGateReason::InsufficientIndependentRuns);
        }
        if prediction.coverage_fraction < self.config.cea.zero_shot_coverage_threshold {
            reasons.push(PredictionGateReason::LowCoverage);
        }
        if fuzzy_only {
            reasons.push(PredictionGateReason::FuzzyOnlyEvidence);
        }
        if !scope_matches {
            reasons.push(PredictionGateReason::ScopeOrConfigMismatch);
        }
        if !has_interventional_evidence {
            reasons.push(PredictionGateReason::MissingInterventionalEvidence);
        }
        if !prediction.risk_flags.is_empty() {
            reasons.push(PredictionGateReason::RiskFlags);
        }
        if prediction.coverage_fraction < 1.0 {
            reasons.push(PredictionGateReason::UnknownEffects);
        }
        // cea-core intentionally does not make graph association zero-shot eligible.
        if !prediction.zero_shot_eligible {
            reasons.push(PredictionGateReason::MissingInterventionalEvidence);
        }
        reasons.sort();
        reasons.dedup();
        PredictionGate {
            disposition: if reasons.is_empty() {
                PredictionDisposition::MaySkipChecks
            } else {
                PredictionDisposition::RunChecks
            },
            reasons,
        }
    }

    pub fn predict_patch(&self, signatures: &[EditOpSignature]) -> ForgeResult<PredictionReceipt> {
        let graph = load_graph(self.store, Some(&self.version_id))?;
        let prediction = predict(signatures, &graph, &self.config.cea);
        let gate = self.prediction_gate(&prediction, 0, false, true, false);
        let digest = digest(&(prediction.clone(), gate.clone()));
        Ok(PredictionReceipt {
            prediction,
            gate,
            digest,
        })
    }

    /// Persist only effects that are differentially new in the patched arm.
    pub fn observe_pair(
        &self,
        patch: &StructuredPatch,
        pair: &PairedTrialResult,
        run_id: &str,
        eval_id: &str,
    ) -> ForgeResult<CausalUpdateReceipt> {
        if !pair.comparable {
            return Err(ForgeError::PairIncomparable {
                reasons: pair.comparability_reasons.clone(),
            });
        }
        let patch_digest = digest(patch);
        let base_digest = pair.baseline_descriptor.fingerprint();
        let config_digest = digest(self.config);
        let identity = ObservationIdentity {
            observation_id: format!("{run_id}:{}", pair.pair_index),
            run_id: run_id.to_string(),
            trial_id: pair.pair_index.to_string(),
            patch_digest: Some(patch_digest.clone()),
            base_digest: Some(base_digest.clone()),
            config_digest: Some(config_digest.clone()),
        };
        let differential = differential_patched_result(&pair.baseline_result, &pair.patched_result);
        let triples = attribute_effects(
            patch,
            &differential,
            &pair.line_map,
            self.config.cea.max_line_distance_for_attribution,
        )?;
        // The execution itself is paired-interventional. The per-edit links
        // produced by line proximity are persisted only in the explicitly
        // observational association graph. Prediction over this graph stays
        // advisory and cannot satisfy the independent interventional gate.
        let attributed = AttributedRunResult::with_observation(
            triples.clone(),
            differential,
            EvidenceKind::Observational,
            identity.clone(),
        );
        let pre_prediction = self.predict_patch(&signatures_for_patch(patch)?)?;
        let update_disposition = if triples.is_empty() {
            "no_observational_edges".to_string()
        } else {
            match update_graph(
                self.store,
                &attributed,
                eval_id,
                &self.version_id,
                self.config,
            )? {
                UpdateResult::Applied {
                    edges_added,
                    edges_updated,
                } => format!("applied:added={edges_added},updated={edges_updated}"),
                UpdateResult::AlreadyProcessed => "already_processed".to_string(),
            }
        };
        let coverage = self.coverage()?;
        let prediction = self.predict_patch(&signatures_for_patch(patch)?)?;
        let mut degradation_reasons = Vec::new();
        if triples.is_empty() {
            degradation_reasons.push(
                "no differentially new located effects; no proximity triples persisted".to_string(),
            );
        } else {
            degradation_reasons.push(
                "proximity triples persisted only as observational edit-localization hypotheses; they cannot authorize check skipping".to_string(),
            );
        }
        let mut receipt = CausalUpdateReceipt {
            receipt_digest: String::new(),
            evidence_kind: EvidenceKind::PairedInterventional,
            experiment_evidence_kind: EvidenceKind::PairedInterventional,
            attribution_evidence_kind: EvidenceKind::Observational,
            observation_identity: identity,
            run_digest: attributed.run_hash,
            trial_digest: digest(&pair.pair_index),
            patch_digest,
            base_digest,
            config_digest,
            triple_digest: digest(&triples),
            triple_count: triples.len(),
            regression_count: pair.diff.regressions,
            improvement_count: pair.diff.improvements,
            stable_count: stable_effect_count(&pair.baseline_result, &pair.patched_result),
            pre_prediction_digest: pre_prediction.digest.clone(),
            post_prediction_digest: prediction.digest.clone(),
            pre_prediction: prediction_summary(&pre_prediction),
            post_prediction: prediction_summary(&prediction),
            update_disposition,
            coverage,
            prediction_disposition: prediction.gate.disposition,
            degradation_reasons,
        };
        receipt.receipt_digest = receipt_digest_for(&receipt);
        Ok(receipt)
    }

    pub async fn run_and_observe(
        &self,
        fixture: &std::path::Path,
        patch: &StructuredPatch,
        experiment_config: &ExperimentConfig,
        eval_id: &str,
    ) -> ForgeResult<CausalExperimentResult> {
        let runner = PairedExperimentRunner::new(self.backend, self.adapter, self.config);
        let experiment = runner.run(fixture, patch, experiment_config).await?;
        let receipts = experiment
            .pairs
            .iter()
            .map(|pair| self.observe_pair(patch, pair, &experiment.run_id, eval_id))
            .collect::<ForgeResult<Vec<_>>>()?;
        Ok(CausalExperimentResult {
            experiment,
            receipts,
        })
    }

    /// Run at most the configured number of singleton removals on fresh workspaces.
    pub async fn run_singleton_ablations(
        &self,
        fixture: &std::path::Path,
        patch: &StructuredPatch,
    ) -> ForgeResult<Vec<AblationReceipt>> {
        let runner = PairedExperimentRunner::new(self.backend, self.adapter, self.config);
        let mut full_trials = Vec::new();
        let full = runner
            .run_pair(fixture, Some(patch), 0, &mut full_trials)
            .await?;
        let full_patch_digest = digest(patch);
        let baseline_digest = full.baseline_descriptor.fingerprint();
        let config_digest = digest(self.config);
        let op_count = patch.edits.iter().map(|edit| edit.ops.len()).sum::<usize>();
        let count = op_count.min(self.config.cea.max_singleton_ablations);
        let mut receipts = Vec::with_capacity(count);
        for operation_index in 0..count {
            let ablated = remove_operation(patch, operation_index);
            let mut trials = Vec::new();
            let outcome = runner
                .run_pair(
                    fixture,
                    ablated.as_ref(),
                    operation_index as u32 + 1,
                    &mut trials,
                )
                .await;
            let baseline_effect_digest = digest(&effect_identities(&full.baseline_result));
            let (
                classification,
                ablated_digest,
                comparable,
                removed,
                persisting,
                new,
                mut degradations,
            ) = match outcome {
                Ok(pair) => {
                    let assessment = assess_ablation_pair(&full, &pair);
                    (
                        assessment.classification,
                        assessment.ablated_outcome_digest,
                        assessment.comparable,
                        assessment.removed_effect_count,
                        assessment.persisting_effect_count,
                        assessment.new_effect_count,
                        assessment.degradation_reasons,
                    )
                }
                Err(error) => (
                    AblationClassification::Inconclusive,
                    digest(&error.to_string()),
                    false,
                    0,
                    0,
                    0,
                    vec![format!("ablation infeasible: {error}")],
                ),
            };
            if ablated.is_none() {
                degradations
                    .push("empty ablation executed as an unpatched baseline arm".to_string());
            }
            let mut receipt = AblationReceipt {
                operation_index,
                classification,
                evidence_kind: EvidenceKind::Ablation,
                verification_method: "exact_normalized_differential_effect_identity_sets"
                    .to_string(),
                comparable,
                full_patch_digest: full_patch_digest.clone(),
                ablated_patch_digest: ablated.as_ref().map(digest),
                baseline_digest: baseline_digest.clone(),
                config_digest: config_digest.clone(),
                baseline_effect_digest,
                full_outcome_digest: digest(&full.diff),
                ablated_outcome_digest: ablated_digest,
                removed_effect_count: removed,
                persisting_effect_count: persisting,
                new_effect_count: new,
                receipt_digest: String::new(),
                degradation_reasons: degradations,
            };
            receipt.receipt_digest = ablation_receipt_digest(&receipt);
            receipts.push(receipt);
        }
        Ok(receipts)
    }
}

fn prediction_summary(receipt: &PredictionReceipt) -> PredictionSummary {
    PredictionSummary {
        digest: receipt.digest.clone(),
        predicted_correctness: receipt.prediction.predicted_correctness,
        confidence: receipt.prediction.confidence,
        coverage_fraction: receipt.prediction.coverage_fraction,
    }
}

fn differential_patched_result(baseline: &CheckResult, patched: &CheckResult) -> CheckResult {
    let mut result = patched.clone();
    for (base, target) in [
        (&baseline.fmt_output, &mut result.fmt_output),
        (&baseline.clippy_output, &mut result.clippy_output),
        (&baseline.test_output, &mut result.test_output),
    ] {
        let stable = base
            .effects
            .iter()
            .map(effect_identity)
            .collect::<std::collections::BTreeSet<_>>();
        target
            .effects
            .retain(|effect| !stable.contains(&effect_identity(effect)));
    }
    // `cea-core` represents a passing check as a synthetic pass effect.  A
    // pass on both arms is stable evidence, so do not let it enter the
    // differential attribution view.
    if baseline.fmt_pass && patched.fmt_pass {
        result.fmt_pass = false;
    }
    if baseline.clippy_pass && patched.clippy_pass {
        result.clippy_pass = false;
    }
    if baseline.test_pass && patched.test_pass {
        result.test_pass = false;
    }
    // A fixed baseline failure is patch-level improvement evidence.  It is not
    // a synthetic "pass caused by this edit" effect and has no proximity edge.
    if result.fmt_output.effects.is_empty() {
        result.fmt_pass = false;
    }
    if result.clippy_output.effects.is_empty() {
        result.clippy_pass = false;
    }
    if result.test_output.effects.is_empty() {
        result.test_pass = false;
    }
    result
}

fn signatures_for_patch(patch: &StructuredPatch) -> ForgeResult<Vec<EditOpSignature>> {
    let files = patch.edits.len();
    let mut signatures = Vec::new();
    for (file_index, edit) in patch.edits.iter().enumerate() {
        let extension = edit.path.extension().and_then(|v| v.to_str()).unwrap_or("");
        for (op_index, op) in edit.ops.iter().enumerate() {
            signatures.push(crate::cea::build_edit_op_signature(
                op,
                op_index,
                edit.ops.len(),
                file_index,
                files,
                extension,
            )?);
        }
    }
    Ok(signatures)
}

fn remove_operation(patch: &StructuredPatch, remove: usize) -> Option<StructuredPatch> {
    let mut next = patch.clone();
    let mut current = 0;
    for edit in &mut next.edits {
        edit.ops.retain(|_| {
            let keep = current != remove;
            current += 1;
            keep
        });
    }
    next.edits.retain(|edit| !edit.ops.is_empty());
    if next.edits.is_empty() {
        None
    } else {
        Some(next)
    }
}

fn classify_ablation_sets(
    full: &std::collections::BTreeSet<String>,
    ablated: &std::collections::BTreeSet<String>,
) -> AblationClassification {
    if full.is_empty() {
        return AblationClassification::Inconclusive;
    }
    let removed = full.difference(ablated).next().is_some();
    let contradictory_new = ablated.difference(full).next().is_some();
    if removed && !contradictory_new {
        AblationClassification::Supported
    } else if full.is_subset(ablated) {
        AblationClassification::Contradicted
    } else {
        AblationClassification::Inconclusive
    }
}

struct AblationPairAssessment {
    classification: AblationClassification,
    ablated_outcome_digest: String,
    comparable: bool,
    removed_effect_count: usize,
    persisting_effect_count: usize,
    new_effect_count: usize,
    degradation_reasons: Vec<String>,
}

fn assess_ablation_pair(
    full: &PairedTrialResult,
    ablated: &PairedTrialResult,
) -> AblationPairAssessment {
    let full_effects = regression_identities(&full.baseline_result, &full.patched_result);
    if !full.comparable || !ablated.comparable {
        let mut reasons = full.comparability_reasons.clone();
        reasons.extend(ablated.comparability_reasons.clone());
        reasons.sort();
        reasons.dedup();
        return AblationPairAssessment {
            classification: AblationClassification::Inconclusive,
            ablated_outcome_digest: digest(&ablated.diff),
            comparable: false,
            removed_effect_count: 0,
            persisting_effect_count: 0,
            new_effect_count: 0,
            degradation_reasons: reasons,
        };
    }
    if full.baseline_descriptor.fingerprint() != ablated.baseline_descriptor.fingerprint() {
        return AblationPairAssessment {
            classification: AblationClassification::Inconclusive,
            ablated_outcome_digest: digest(&ablated.diff),
            comparable: false,
            removed_effect_count: 0,
            persisting_effect_count: 0,
            new_effect_count: 0,
            degradation_reasons: vec![
                "full and ablation baselines differ; intervention is not comparable".to_string(),
            ],
        };
    }

    let ablated_effects = regression_identities(&full.baseline_result, &ablated.patched_result);
    AblationPairAssessment {
        classification: classify_ablation_sets(&full_effects, &ablated_effects),
        ablated_outcome_digest: digest(&ablated_effects),
        comparable: true,
        removed_effect_count: full_effects.difference(&ablated_effects).count(),
        persisting_effect_count: full_effects.intersection(&ablated_effects).count(),
        new_effect_count: ablated_effects.difference(&full_effects).count(),
        degradation_reasons: Vec::new(),
    }
}

fn effect_identity(effect: &crate::exec::backend::LocatedEffect) -> String {
    digest(&(
        &effect.sig.check_kind,
        &effect.sig.outcome,
        &effect.sig.severity,
        &effect.sig.message_class,
        &effect.file,
        effect.line,
    ))
}

fn effect_identities(result: &CheckResult) -> std::collections::BTreeSet<String> {
    result
        .fmt_output
        .effects
        .iter()
        .chain(&result.clippy_output.effects)
        .chain(&result.test_output.effects)
        .map(effect_identity)
        .collect()
}

fn regression_identities(
    baseline: &CheckResult,
    patched: &CheckResult,
) -> std::collections::BTreeSet<String> {
    let base = effect_identities(baseline);
    effect_identities(patched)
        .difference(&base)
        .cloned()
        .collect()
}

fn stable_effect_count(baseline: &CheckResult, patched: &CheckResult) -> u32 {
    let baseline = effect_identities(baseline);
    let patched = effect_identities(patched);
    baseline.intersection(&patched).count() as u32
}

fn receipt_digest_for(receipt: &CausalUpdateReceipt) -> String {
    let mut unsigned = receipt.clone();
    unsigned.receipt_digest.clear();
    digest(&unsigned)
}

fn ablation_receipt_digest(receipt: &AblationReceipt) -> String {
    digest(&(
        receipt.operation_index,
        receipt.classification,
        receipt.evidence_kind,
        &receipt.verification_method,
        receipt.comparable,
        &receipt.full_patch_digest,
        &receipt.ablated_patch_digest,
        &receipt.baseline_digest,
        &receipt.config_digest,
        &receipt.baseline_effect_digest,
        &receipt.full_outcome_digest,
        &receipt.ablated_outcome_digest,
        receipt.removed_effect_count,
        receipt.persisting_effect_count,
        receipt.new_effect_count,
        &receipt.degradation_reasons,
    ))
}

fn digest<T: Serialize>(value: &T) -> String {
    blake3::hash(&serde_json::to_vec(value).unwrap_or_default())
        .to_hex()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::baseline::{BaselineDescriptor, BaselineSourceKind};
    use crate::exec::backend::{CheckKind, ParsedCheckOutput};
    use crate::experiment::ExperimentDiff;

    fn descriptor(commit: &str) -> BaselineDescriptor {
        BaselineDescriptor {
            source_kind: BaselineSourceKind::Explicit,
            commit_sha: Some(commit.to_string()),
            dirty: false,
            untracked_count: 0,
            lockfile_hash: None,
            rustc_version: "rustc".into(),
            cargo_version: "cargo".into(),
            target_triple: "host".into(),
            env_fingerprint: "env".into(),
            submodule_state: vec![],
        }
    }

    fn passing_check() -> CheckResult {
        CheckResult {
            fmt_pass: true,
            clippy_pass: true,
            test_pass: true,
            fmt_output: ParsedCheckOutput {
                check_kind: CheckKind::Fmt,
                ..ParsedCheckOutput::default()
            },
            clippy_output: ParsedCheckOutput {
                check_kind: CheckKind::Clippy,
                ..ParsedCheckOutput::default()
            },
            test_output: ParsedCheckOutput {
                check_kind: CheckKind::Test,
                ..ParsedCheckOutput::default()
            },
            total_duration_ms: 1,
        }
    }

    fn pair(commit: &str) -> PairedTrialResult {
        let baseline = passing_check();
        let patched = passing_check();
        PairedTrialResult {
            pair_index: 0,
            baseline_descriptor: descriptor(commit),
            patched_descriptor: descriptor(commit),
            baseline_result: baseline.clone(),
            patched_result: patched.clone(),
            line_map: Default::default(),
            diff: ExperimentDiff::from_paired(&baseline, &patched),
            comparable: true,
            comparability_reasons: Vec::new(),
        }
    }

    #[test]
    fn cross_run_baseline_drift_makes_ablation_inconclusive() {
        let assessment = assess_ablation_pair(&pair("full-base"), &pair("drifted-base"));
        assert_eq!(
            assessment.classification,
            AblationClassification::Inconclusive
        );
        assert!(!assessment.comparable);
        assert!(assessment
            .degradation_reasons
            .iter()
            .any(|reason| reason.contains("full and ablation baselines differ")));
    }
}
