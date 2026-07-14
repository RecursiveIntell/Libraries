//! Evidence bundles, hypothesis edges, receipts, and verification plans.
//!
//! Phase 5 claim policy: this module produces PROVISIONAL LOCAL ATTRIBUTIONS
//! from one paired baseline/patched run on one fixed workload slice.
//! It does NOT produce robust causal confirmation, generalized effect claims,
//! or production-level statistical guarantees, but it does model first-class
//! verification trials and named falsification artifacts.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::ForgeResult;
use crate::experiment::{ExperimentDiff, TypedLocatedEffect};
use crate::lab::evaluate::ScoreVector;
use stack_ids::{AttemptId, ClaimVersionId, RelationVersionId, TrialId};

const LOCAL_AUTHORING_METADATA_KEY: &str = "living_memory_authoring";

// ── Claim strength ──

/// How strong a causal claim this bundle makes.
///
/// Phase 5 only supports `ProvisionalSinglePair`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimStrength {
    /// Provisional local attribution from one paired intervention on one fixed workload slice.
    #[default]
    ProvisionalSinglePair,
}

impl std::fmt::Display for ClaimStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProvisionalSinglePair => {
                write!(f, "provisional local attribution from one paired intervention on one fixed workload slice")
            }
        }
    }
}

// ── Bundle scope ──

/// Scope of the evidence bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleScope {
    /// Which workload was used.
    pub workload_id: String,
    /// Backend family used for execution.
    pub backend_family: String,
    /// Ordered list of checks executed.
    pub selected_checks: Vec<String>,
    /// Effective timeout class for the run.
    pub timeout_class: String,
    /// Sorted config-flag projection affecting execution.
    pub config_flags: Vec<String>,
}

// ── Covariates ──

/// Measured covariates for a paired experiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Covariates {
    /// Hash of backend kind + OS/runtime/toolchain versions + allowed env vars.
    pub env_fingerprint: String,
    /// Hash of lockfile or dependency manifest snapshot.
    pub dependency_fingerprint: Option<String>,
    /// Sorted projection of execution-affecting config flags.
    pub config_flags: Vec<String>,
    /// Hash of test/benchmark names + input corpus id + selection parameters.
    pub workload_id: String,
    /// Canonical ordered list of executed checks.
    pub selected_checks: Vec<String>,
    /// Whether non-primary edits were present in the patched workspace.
    pub adjacent_edits: bool,
    /// Optional list of non-primary edit signatures.
    #[serde(default)]
    pub adjacent_edit_signatures: Vec<String>,
}

// ── Treatment ──

/// The treatment applied in the experiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Treatment {
    /// "baseline" or "patch_applied".
    pub kind: String,
    /// Content hash of the patch.
    pub patch_hash: String,
    /// Summary of the patch (files touched, edit count).
    pub patch_summary: String,
}

// ── Receipt ──

/// A receipt proving the existence and integrity of a trial artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptRef {
    /// Unique receipt identifier.
    pub receipt_id: String,
    /// What kind of artifact this receipt covers.
    pub kind: ReceiptKind,
    /// Where the artifact is stored.
    pub storage: ReceiptStorage,
    /// Blake3 hash of the artifact content.
    pub content_hash: String,
    /// Cross-crate trace ID, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// Handle for replaying the artifact, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_handle: Option<String>,
}

impl ReceiptRef {
    /// Verify that the content hash matches the given content.
    pub fn verify_content(&self, content: &[u8]) -> bool {
        let hash = blake3::hash(content).to_hex().to_string();
        hash == self.content_hash
    }
}

/// Kind of artifact a receipt covers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptKind {
    TrialLog,
    TrialMetrics,
    CheckResult,
    PatchApplicationRecord,
    /// Serialized causal-graph update receipt from CEA.
    CausalUpdate,
    /// Serialized singleton-ablation receipt from CEA.
    Ablation,
}

/// Where a receipt's artifact is stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptStorage {
    /// Stored inline in the receipt itself.
    Inline(String),
    /// Stored in the forge DB.
    StoreRow { table: String, key: String },
    /// Stored at a filesystem path.
    ArtifactPath(String),
}

// ── Hypothesis edge ──

/// A typed edge linking an edit operation to an observed effect.
///
/// This is the primary hypothesis structure for Phase 5.
/// Edge: EditOpSignature -> LocatedEffect with a typed kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisEdge {
    /// Unique edge identifier.
    pub edge_id: String,
    /// Source: the edit operation signature (serialized).
    pub source_edit: String,
    /// Target: the observed effect (serialized).
    pub target_effect: String,
    /// What kind of causal relationship this edge represents.
    pub kind: HypothesisEdgeKind,
    /// Current status of this edge.
    pub status: HypothesisStatus,
    /// Confidence in this edge (0.0 to 1.0, heuristic for Phase 5).
    pub confidence: f64,
    /// Bundle IDs that provide evidence for this edge.
    pub evidence_ids: Vec<String>,
    /// Bundle IDs that contradict this edge.
    pub contradiction_ids: Vec<String>,
    /// Whether this edge has been verified by a follow-up experiment.
    pub verification_status: VerificationState,
}

/// What kind of causal relationship a hypothesis edge represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisEdgeKind {
    /// The edit causes a regression (new failure in patched).
    CausesRegression,
    /// The edit fixes a failure (failure in baseline, absent in patched).
    FixesFailure,
    /// The edit is associated with a failure that is stable across both.
    AssociatedWithStableFailure,
}

/// Verification state for an edge.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationState {
    #[default]
    Unverified,
    PlanGenerated,
    VerificationPending,
    Verified,
    VerificationFailed,
}

// ── Evidence bundle ──

/// Which execution context the trial belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaselineOrPatch {
    /// Trial ran on the baseline workspace.
    Baseline,
    /// Trial ran on the patched workspace.
    Patched,
}

/// Canonical execution trial for one side of a verification pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationTrial {
    /// Unique trial identifier in one attempt.
    pub trial_id: TrialId,
    /// Logical retry family for related retry attempts.
    pub attempt_id: AttemptId,
    /// Baseline vs patched execution lane.
    pub baseline_or_patch: BaselineOrPatch,
    /// Whether this trial completed successfully.
    pub completed: bool,
    /// Artifact receipts tied to this trial.
    #[serde(default)]
    pub receipts: Vec<String>,
}

/// Outcome of a named refutation artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefutationArtifactOutcome {
    /// Refutation did not weaken the original claim.
    Passed,
    /// Refutation produced an explicit failure signal.
    Failed {
        /// Why the refutation was failed (invariant name, numeric counterexample, etc).
        reason: String,
    },
    /// Refutation could not complete.
    Inconclusive {
        /// Why no definitive result was available.
        reason: String,
    },
    /// Refutation was intentionally skipped.
    Skipped {
        /// Skip reason.
        reason: String,
    },
}

/// Type of named falsification artifact emitted by verification infrastructure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefutationArtifactType {
    /// Inert control treatment/no-op on treatment assignment.
    Placebo,
    /// Outcome intentionally randomized/nullified.
    DummyOutcome,
    /// Subsample split stability check.
    SubsampleStability,
    /// A bounded removal of one patch operation, evaluated against the full patch.
    SingletonAblation,
}

/// Falsification artifact carried by the evidence bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefutationArtifact {
    /// Artifact identity for traceability.
    pub artifact_id: String,
    /// Artifact type.
    pub artifact_type: RefutationArtifactType,
    /// Trial that emitted this artifact, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trial_id: Option<TrialId>,
    /// Attempt that emitted this artifact, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_id: Option<AttemptId>,
    /// Refutation outcome, including fail/pass path.
    pub outcome: RefutationArtifactOutcome,
    /// Numeric stability or effect delta when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimate_delta: Option<f64>,
    /// Optional structured details for durable debugging.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Which effect-derived export relation a lineage hint applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectRelationLineageSource {
    /// The `primary_effect_*` export relation for the bundle.
    PrimaryEffect,
    /// One `all_effect_*` export relation for the bundle.
    AllEffect,
    /// One `experiment_diff_*` export relation for the bundle.
    ExperimentDiff,
}

impl EffectRelationLineageSource {
    /// Stable export source label used in relation predicates and metadata.
    pub fn export_source_key(self) -> &'static str {
        match self {
            Self::PrimaryEffect => "primary_effect",
            Self::AllEffect => "all_effect",
            Self::ExperimentDiff => "experiment_diff",
        }
    }
}

/// Explicit relation-lineage hint for an effect-derived export relation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectRelationLineageHint {
    /// Which effect export family this hint applies to.
    pub source: EffectRelationLineageSource,
    /// Effect kind to match.
    pub kind: crate::experiment::EffectKind,
    /// Effect file to match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
    /// Effect line to match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Effect message to match.
    pub message: String,
    /// Whether the effect was present in the baseline execution.
    pub in_baseline: bool,
    /// Whether the effect was present in the patched execution.
    pub in_patched: bool,
    /// Real prior relation version when known.
    pub supersedes_relation_version_id: RelationVersionId,
}

impl EffectRelationLineageHint {
    fn matches(
        &self,
        source: EffectRelationLineageSource,
        effect: &crate::experiment::TypedLocatedEffect,
    ) -> bool {
        self.source == source
            && self.kind == effect.kind
            && self.file == effect.file
            && self.line == effect.line
            && self.message == effect.message
            && self.in_baseline == effect.in_baseline
            && self.in_patched == effect.in_patched
    }
}

/// Explicit relation-lineage hint for a hypothesis-edge export relation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HypothesisRelationLineageHint {
    /// Edge identity to match.
    pub edge_id: String,
    /// Real prior relation version when known.
    pub supersedes_relation_version_id: RelationVersionId,
}

impl HypothesisRelationLineageHint {
    fn matches(&self, edge: &HypothesisEdge) -> bool {
        self.edge_id == edge.edge_id
    }
}

/// Explicit relation-lineage hint for a verification-trial export relation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationTrialRelationLineageHint {
    /// Trial identity to match.
    pub trial_id: TrialId,
    /// Trial lane to match.
    pub baseline_or_patch: BaselineOrPatch,
    /// Attempt identity, when known and needed to disambiguate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_id: Option<AttemptId>,
    /// Real prior relation version when known.
    pub supersedes_relation_version_id: RelationVersionId,
}

impl VerificationTrialRelationLineageHint {
    fn matches(&self, trial: &VerificationTrial) -> bool {
        self.trial_id == trial.trial_id
            && self.baseline_or_patch == trial.baseline_or_patch
            && match self.attempt_id.as_ref() {
                Some(attempt_id) => attempt_id == &trial.attempt_id,
                None => true,
            }
    }
}

/// Explicit relation-lineage hint for a refutation-artifact export relation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefutationRelationLineageHint {
    /// Artifact identity to match.
    pub artifact_id: String,
    /// Artifact type, when known and needed to disambiguate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_type: Option<RefutationArtifactType>,
    /// Real prior relation version when known.
    pub supersedes_relation_version_id: RelationVersionId,
}

impl RefutationRelationLineageHint {
    fn matches(&self, artifact: &RefutationArtifact) -> bool {
        self.artifact_id == artifact.artifact_id
            && match self.artifact_type {
                Some(artifact_type) => artifact_type == artifact.artifact_type,
                None => true,
            }
    }
}

/// Optional sidecar carrying known relation-version lineage for bundle export.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationLineageHints {
    /// Hints for effect-derived relations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effect_relations: Vec<EffectRelationLineageHint>,
    /// Hints for hypothesis-edge relations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hypothesis_relations: Vec<HypothesisRelationLineageHint>,
    /// Hints for verification-trial relations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verification_trial_relations: Vec<VerificationTrialRelationLineageHint>,
    /// Hints for refutation-artifact relations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refutation_relations: Vec<RefutationRelationLineageHint>,
}

impl RelationLineageHints {
    /// Returns `true` when no relation-lineage hints are present.
    pub fn is_empty(&self) -> bool {
        self.effect_relations.is_empty()
            && self.hypothesis_relations.is_empty()
            && self.verification_trial_relations.is_empty()
            && self.refutation_relations.is_empty()
    }

    fn effect_supersedes_relation_version_id(
        &self,
        source: EffectRelationLineageSource,
        effect: &crate::experiment::TypedLocatedEffect,
    ) -> Option<RelationVersionId> {
        self.effect_relations
            .iter()
            .find(|hint| hint.matches(source, effect))
            .map(|hint| hint.supersedes_relation_version_id.clone())
    }

    fn hypothesis_supersedes_relation_version_id(
        &self,
        edge: &HypothesisEdge,
    ) -> Option<RelationVersionId> {
        self.hypothesis_relations
            .iter()
            .find(|hint| hint.matches(edge))
            .map(|hint| hint.supersedes_relation_version_id.clone())
    }

    fn verification_trial_supersedes_relation_version_id(
        &self,
        trial: &VerificationTrial,
    ) -> Option<RelationVersionId> {
        self.verification_trial_relations
            .iter()
            .find(|hint| hint.matches(trial))
            .map(|hint| hint.supersedes_relation_version_id.clone())
    }

    fn refutation_supersedes_relation_version_id(
        &self,
        artifact: &RefutationArtifact,
    ) -> Option<RelationVersionId> {
        self.refutation_relations
            .iter()
            .find(|hint| hint.matches(artifact))
            .map(|hint| hint.supersedes_relation_version_id.clone())
    }
}

/// A bundle of evidence collected during a forge evaluation run.
///
/// One ExperimentEvidenceBundle = one paired experiment on one fixed workload slice.
/// One bundle may contain multiple effects but names exactly one primary effect.
/// Claim strength for Phase 5 is always `ProvisionalSinglePair`.
///
/// `semantic_memory_forge::EvidenceBundle` is the authoritative cross-crate
/// evidence contract. This local type is an authoring wrapper that carries
/// forge-engine-only working fields and round-trips them explicitly through the
/// canonical bundle metadata under [`LOCAL_AUTHORING_METADATA_KEY`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentEvidenceBundle {
    /// Unique identifier for this bundle.
    pub bundle_id: String,
    /// The candidate that produced this evidence.
    pub candidate_id: String,
    /// Eval run that generated the evidence.
    pub eval_id: String,
    /// Version of the algebra spec used.
    pub version_id: String,
    /// Real prior claim-lineage version when known.
    ///
    /// This is optional and carried through to the export envelope so that
    /// downstream projection imports can preserve version-aware supersession.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes_claim_version_id: Option<ClaimVersionId>,
    /// Explicit relation-version lineage hints for exportable relations, when known.
    ///
    /// These hints are advisory only. They do not mint lineage, and export leaves
    /// `supersedes_relation_version_id` unset when no exact hint is present.
    #[serde(default, skip_serializing_if = "RelationLineageHints::is_empty")]
    pub relation_lineage_hints: RelationLineageHints,
    /// Computed scores.
    pub scores: ScoreVector,
    /// Causal hypotheses derived from this run (legacy field, prefer hypothesis_edges).
    pub hypotheses: Vec<CausalHypothesis>,
    /// Verification plan for follow-up validation.
    pub verification: Option<VerificationPlan>,
    /// Trace ID for cross-crate correlation.
    pub trace_id: Option<String>,
    /// Typed experiment diff (baseline vs patched), if from a paired experiment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experiment_diff: Option<ExperimentDiff>,
    /// Attribution result from CEA, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution_json: Option<String>,
    /// Deterministic evidence assessment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assessment: Option<EvidenceAssessment>,
    /// Warnings generated during evidence collection.
    #[serde(default)]
    pub warnings: Vec<String>,
    /// When this bundle was recorded by Forge.
    #[serde(default = "default_bundle_created_at")]
    pub created_at: String,

    // ── Phase 5 fields (all have serde defaults for backward compat) ──
    /// Forge domain ID for the experiment run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    /// Forge domain ID for the patch attempt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_id: Option<String>,
    /// The causal question this bundle answers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causal_question: Option<String>,
    /// Explicit description of the unit of observation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_definition: Option<String>,
    /// Scope of this bundle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_scope: Option<BundleScope>,
    /// Explicit paired comparability verdict when the source captured it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pair_comparability: Option<PairComparability>,
    /// How strong a claim this bundle makes.
    #[serde(default)]
    pub claim_strength: ClaimStrength,
    /// Why we think confounding is controlled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identification_rationale: Option<String>,
    /// Known threats to validity.
    #[serde(default)]
    pub known_threats: Vec<String>,
    /// Content hash of the patch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch_hash: Option<String>,
    /// The treatment applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub treatment: Option<Treatment>,
    /// Outcome summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    /// Measured covariates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub covariates: Option<Covariates>,
    /// Promotion state for the claim represented by this bundle, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub promotion_state: Option<semantic_memory_forge::PromotionState>,
    /// The primary effect attributed to the patch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_effect: Option<TypedLocatedEffect>,
    /// All effects observed in the experiment.
    #[serde(default)]
    pub all_effects: Vec<TypedLocatedEffect>,
    /// Typed hypothesis edges (Phase 5 edge model).
    #[serde(default)]
    pub hypothesis_edges: Vec<HypothesisEdge>,
    /// Receipts proving artifact integrity.
    #[serde(default)]
    pub receipts: Vec<ReceiptRef>,
    /// Canonical per-side verification trials for this causal claim.
    #[serde(default)]
    pub verification_trials: Vec<VerificationTrial>,
    /// Refutation artifacts generated by verification infrastructure.
    #[serde(default)]
    pub refutation_artifacts: Vec<RefutationArtifact>,
    /// Whether this bundle is sealed (immutable after creation).
    #[serde(default)]
    pub sealed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalEvidenceBundleAuthoring {
    candidate_id: String,
    eval_id: String,
    version_id: String,
    supersedes_claim_version_id: Option<ClaimVersionId>,
    relation_lineage_hints: RelationLineageHints,
    scores: ScoreVector,
    hypotheses: Vec<CausalHypothesis>,
    verification: Option<VerificationPlan>,
    experiment_diff: Option<ExperimentDiff>,
    attribution_json: Option<String>,
    assessment: Option<EvidenceAssessment>,
    warnings: Vec<String>,
    run_id: Option<String>,
    attempt_id: Option<String>,
    pair_comparability: Option<PairComparability>,
    claim_strength: ClaimStrength,
    known_threats: Vec<String>,
    patch_hash: Option<String>,
    treatment: Option<Treatment>,
    covariates: Option<Covariates>,
    primary_effect: Option<TypedLocatedEffect>,
    all_effects: Vec<TypedLocatedEffect>,
    hypothesis_edges: Vec<HypothesisEdge>,
    receipts: Vec<ReceiptRef>,
    verification_trials: Vec<VerificationTrial>,
    refutation_artifacts: Vec<RefutationArtifact>,
    sealed: bool,
}

impl LocalEvidenceBundleAuthoring {
    fn from_bundle(bundle: &ExperimentEvidenceBundle) -> Self {
        Self {
            candidate_id: bundle.candidate_id.clone(),
            eval_id: bundle.eval_id.clone(),
            version_id: bundle.version_id.clone(),
            supersedes_claim_version_id: bundle.supersedes_claim_version_id.clone(),
            relation_lineage_hints: bundle.relation_lineage_hints.clone(),
            scores: bundle.scores.clone(),
            hypotheses: bundle.hypotheses.clone(),
            verification: bundle.verification.clone(),
            experiment_diff: bundle.experiment_diff.clone(),
            attribution_json: bundle.attribution_json.clone(),
            assessment: bundle.assessment.clone(),
            warnings: bundle.warnings.clone(),
            run_id: bundle.run_id.clone(),
            attempt_id: bundle.attempt_id.clone(),
            pair_comparability: bundle.pair_comparability.clone(),
            claim_strength: bundle.claim_strength,
            known_threats: bundle.known_threats.clone(),
            patch_hash: bundle.patch_hash.clone(),
            treatment: bundle.treatment.clone(),
            covariates: bundle.covariates.clone(),
            primary_effect: bundle.primary_effect.clone(),
            all_effects: bundle.all_effects.clone(),
            hypothesis_edges: bundle.hypothesis_edges.clone(),
            receipts: bundle.receipts.clone(),
            verification_trials: bundle.verification_trials.clone(),
            refutation_artifacts: bundle.refutation_artifacts.clone(),
            sealed: bundle.sealed,
        }
    }
}

impl ExperimentEvidenceBundle {
    /// Seal the bundle, making it logically immutable.
    /// Returns an error if the bundle is already sealed.
    pub fn seal(&mut self) -> ForgeResult<()> {
        if self.sealed {
            return Err(crate::error::ForgeError::Other(
                "bundle is already sealed".into(),
            ));
        }
        self.sealed = true;
        Ok(())
    }

    /// Returns an error if the bundle is sealed and mutation is attempted.
    pub fn check_not_sealed(&self) -> ForgeResult<()> {
        if self.sealed {
            return Err(crate::error::ForgeError::Other(
                "cannot mutate sealed bundle".into(),
            ));
        }
        Ok(())
    }

    /// Project this local bundle into the canonical Forge evidence contract.
    ///
    /// The canonical `semantic_memory_forge::EvidenceBundle` remains the
    /// authoritative cross-crate contract. Local forge-engine-only authoring
    /// fields are preserved explicitly inside canonical metadata rather than
    /// becoming a second drifting wire contract.
    pub fn to_canonical_evidence_bundle(&self) -> semantic_memory_forge::EvidenceBundle {
        let question = semantic_memory_forge::CausalQuestion {
            description: self.causal_question.clone().unwrap_or_else(|| {
                let treatment = self
                    .treatment
                    .as_ref()
                    .map(|t| t.patch_summary.as_str())
                    .unwrap_or("applied patch");
                let outcome = self.outcome.as_deref().unwrap_or("recorded outcome");
                let workload = self
                    .bundle_scope
                    .as_ref()
                    .map(|scope| scope.workload_id.as_str())
                    .unwrap_or("unspecified workload");
                let scope_summary = self
                    .bundle_scope
                    .as_ref()
                    .map(|scope| scope.backend_family.as_str())
                    .unwrap_or("unspecified scope");
                Self::render_causal_question(treatment, outcome, workload, scope_summary)
            }),
            unit_definition: self
                .unit_definition
                .clone()
                .unwrap_or_else(|| "paired experiment run".into()),
        };

        let treatment = semantic_memory_forge::TreatmentSpec {
            description: self
                .treatment
                .as_ref()
                .map(|t| format!("{} [{}]", t.patch_summary, t.patch_hash))
                .unwrap_or_else(|| "patch applied".into()),
            baseline_description: "baseline workspace without patch".into(),
            paired_trials: self.experiment_diff.is_some() || !self.verification_trials.is_empty(),
        };

        let outcome = semantic_memory_forge::OutcomeSpec {
            description: self
                .outcome
                .clone()
                .unwrap_or_else(|| "bundle outcome summary".into()),
            measurement_method: if self.verification_trials.is_empty() {
                "single_pair_execution_observation".into()
            } else {
                "paired_verification_trials".into()
            },
            outcome_type: if self.primary_effect.is_some() {
                "typed_effect".into()
            } else {
                "bundle_summary".into()
            },
        };

        let mut covariates = Vec::new();
        if let Some(bundle_scope) = &self.bundle_scope {
            covariates.push(format!("workload:{}", bundle_scope.workload_id));
            covariates.push(format!("backend:{}", bundle_scope.backend_family));
            covariates.push(format!("timeout_class:{}", bundle_scope.timeout_class));
            covariates.extend(
                bundle_scope
                    .selected_checks
                    .iter()
                    .map(|check| format!("check:{check}")),
            );
            covariates.extend(
                bundle_scope
                    .config_flags
                    .iter()
                    .map(|flag| format!("config_flag:{flag}")),
            );
        }
        if let Some(covariates_meta) = &self.covariates {
            covariates.push(format!(
                "env_fingerprint:{}",
                covariates_meta.env_fingerprint
            ));
            covariates.push(format!("workload:{}", covariates_meta.workload_id));
            covariates.extend(
                covariates_meta
                    .selected_checks
                    .iter()
                    .map(|check| format!("selected_check:{check}")),
            );
            covariates.extend(
                covariates_meta
                    .config_flags
                    .iter()
                    .map(|flag| format!("covariate_flag:{flag}")),
            );
            if let Some(dependency_fingerprint) = &covariates_meta.dependency_fingerprint {
                covariates.push(format!("dependency_fingerprint:{dependency_fingerprint}"));
            }
            if covariates_meta.adjacent_edits {
                covariates.push("adjacent_edits:true".into());
            }
            covariates.extend(
                covariates_meta
                    .adjacent_edit_signatures
                    .iter()
                    .map(|sig| format!("adjacent_edit:{sig}")),
            );
        }
        covariates.extend(
            self.known_threats
                .iter()
                .map(|threat| format!("known_threat:{threat}")),
        );

        let refutations = self
            .refutation_artifacts
            .iter()
            .map(|artifact| semantic_memory_forge::RefutationAttempt {
                method: format!("{:?}", artifact.artifact_type).to_lowercase(),
                result: match &artifact.outcome {
                    RefutationArtifactOutcome::Passed => {
                        semantic_memory_forge::RefutationResult::Passed {
                            estimate_change: artifact.estimate_delta,
                        }
                    }
                    RefutationArtifactOutcome::Failed { reason } => {
                        semantic_memory_forge::RefutationResult::Failed {
                            reason: reason.clone(),
                            estimate_change: artifact.estimate_delta,
                        }
                    }
                    RefutationArtifactOutcome::Inconclusive { reason } => {
                        semantic_memory_forge::RefutationResult::Inconclusive {
                            reason: reason.clone(),
                        }
                    }
                    RefutationArtifactOutcome::Skipped { reason } => {
                        semantic_memory_forge::RefutationResult::Skipped {
                            reason: reason.clone(),
                        }
                    }
                },
                estimator_kind: Some("living_memory_phase5_scorevector".into()),
                parameters: artifact.details.as_ref().map(|details| {
                    serde_json::json!({
                        "artifact_id": artifact.artifact_id,
                        "details": details,
                    })
                }),
            })
            .collect();

        let refutation_artifacts = self
            .refutation_artifacts
            .iter()
            .map(|artifact| semantic_memory_forge::RefutationArtifactRecord {
                artifact_id: artifact.artifact_id.clone(),
                artifact_type: Self::debug_name_to_snake_case(&format!(
                    "{:?}",
                    artifact.artifact_type
                )),
                trial_id: artifact.trial_id.clone(),
                attempt_id: artifact.attempt_id.clone(),
                result: match &artifact.outcome {
                    RefutationArtifactOutcome::Passed => {
                        semantic_memory_forge::RefutationResult::Passed {
                            estimate_change: artifact.estimate_delta,
                        }
                    }
                    RefutationArtifactOutcome::Failed { reason } => {
                        semantic_memory_forge::RefutationResult::Failed {
                            reason: reason.clone(),
                            estimate_change: artifact.estimate_delta,
                        }
                    }
                    RefutationArtifactOutcome::Inconclusive { reason } => {
                        semantic_memory_forge::RefutationResult::Inconclusive {
                            reason: reason.clone(),
                        }
                    }
                    RefutationArtifactOutcome::Skipped { reason } => {
                        semantic_memory_forge::RefutationResult::Skipped {
                            reason: reason.clone(),
                        }
                    }
                },
                estimate_delta: artifact.estimate_delta,
                details: artifact.details.clone(),
            })
            .collect::<Vec<_>>();

        let verification_trials = self
            .verification_trials
            .iter()
            .map(|trial| semantic_memory_forge::VerificationTrialRecord {
                trial_id: trial.trial_id.clone(),
                attempt_id: trial.attempt_id.clone(),
                side: match trial.baseline_or_patch {
                    BaselineOrPatch::Baseline => {
                        semantic_memory_forge::VerificationTrialSide::Baseline
                    }
                    BaselineOrPatch::Patched => {
                        semantic_memory_forge::VerificationTrialSide::Patched
                    }
                },
                completed: trial.completed,
                receipt_handles: trial.receipts.clone(),
            })
            .collect::<Vec<_>>();

        let comparability_snapshot =
            self.bundle_scope
                .as_ref()
                .map(|scope| semantic_memory_forge::ComparabilitySnapshot {
                    workload_id: scope.workload_id.clone(),
                    backend_family: scope.backend_family.clone(),
                    selected_checks: scope.selected_checks.clone(),
                    timeout_class: scope.timeout_class.clone(),
                    config_flags: scope.config_flags.clone(),
                    comparable: self.pair_comparability.as_ref().map(|pair| pair.valid),
                    violations: self
                        .pair_comparability
                        .as_ref()
                        .map(|pair| pair.violations.clone())
                        .unwrap_or_default(),
                });

        let raw_receipt_handle = self.receipts.first().map(|receipt| match &receipt.storage {
            ReceiptStorage::Inline(_) => format!("receipt:inline:{}", receipt.receipt_id),
            ReceiptStorage::StoreRow { table, key } => {
                format!("receipt:store:{table}:{key}")
            }
            ReceiptStorage::ArtifactPath(path) => format!("receipt:path:{path}"),
        });

        let confidence = self
            .assessment
            .as_ref()
            .map(|assessment| match assessment.sample_support {
                SampleSupport::Sufficient => 0.9,
                SampleSupport::Marginal => 0.6,
                SampleSupport::Insufficient => 0.3,
            })
            .unwrap_or_else(|| self.scores.weighted_total.clamp(0.0, 1.0) as f32);

        let completed_trial_count = self
            .verification_trials
            .iter()
            .filter(|trial| trial.completed)
            .count() as u32;
        let passed_refutation_count = self
            .refutation_artifacts
            .iter()
            .filter(|artifact| matches!(artifact.outcome, RefutationArtifactOutcome::Passed))
            .count() as u32;
        let failed_refutation_count = self
            .refutation_artifacts
            .iter()
            .filter(|artifact| matches!(artifact.outcome, RefutationArtifactOutcome::Failed { .. }))
            .count() as u32;
        let lifecycle_state = if self.assessment.as_ref().is_some_and(|assessment| {
            assessment.contradiction_state == ContradictionState::HasContradictions
        }) || failed_refutation_count > 0
        {
            semantic_memory_forge::VerificationLifecycleState::Contradicted
        } else if completed_trial_count > 0
            || self.assessment.as_ref().is_some_and(|assessment| {
                matches!(
                    assessment.reproducibility,
                    AssessmentCategory::Strong | AssessmentCategory::Adequate
                )
            })
        {
            semantic_memory_forge::VerificationLifecycleState::Verified
        } else {
            semantic_memory_forge::VerificationLifecycleState::Unverified
        };

        let derived_promotion_state = self.promotion_state.clone().unwrap_or_else(|| {
            if let Some(assessment) = self.assessment.as_ref() {
                if assessment.contradiction_state == ContradictionState::HasContradictions {
                    semantic_memory_forge::PromotionState::Blocked {
                        reason: "contradictions_present".into(),
                    }
                } else if assessment.sample_support == SampleSupport::Insufficient {
                    semantic_memory_forge::PromotionState::Blocked {
                        reason: "sample_support_insufficient".into(),
                    }
                } else if assessment.isolation != AssessmentCategory::Strong {
                    semantic_memory_forge::PromotionState::Blocked {
                        reason: "isolation_not_strong".into(),
                    }
                } else if matches!(
                    assessment.reproducibility,
                    AssessmentCategory::Strong | AssessmentCategory::Adequate
                ) {
                    semantic_memory_forge::PromotionState::Eligible
                } else {
                    semantic_memory_forge::PromotionState::NotPromoted
                }
            } else {
                semantic_memory_forge::PromotionState::NotPromoted
            }
        });

        let mut verification_notes = self.warnings.clone();
        if self.supersedes_claim_version_id.is_some() {
            verification_notes.push("supersedes_prior_claim_version".into());
        }

        let estimator_meta = Some(semantic_memory_forge::EstimatorMeta {
            kind: semantic_memory_forge::EstimatorKind::Custom(
                "living_memory_phase5_scorevector".into(),
            ),
            version: self.version_id.clone(),
            parameters: serde_json::json!({
                "weighted_total": self.scores.weighted_total,
                "correctness": self.scores.correctness,
                "novelty": self.scores.novelty,
                "stability": self.scores.stability,
                "claim_strength": format!("{}", self.claim_strength),
            }),
            random_seed: None,
            environment: self.covariates.as_ref().map(|covariates| {
                semantic_memory_forge::EnvironmentFingerprint {
                    python_version: None,
                    package_versions: serde_json::json!({
                        "selected_checks": covariates.selected_checks,
                        "config_flags": covariates.config_flags,
                    }),
                    platform: None,
                    env_hash: Some(covariates.env_fingerprint.clone()),
                }
            }),
            timeout_secs: None,
            failure_mode: None,
            request_schema_version: Some("living_memory.phase5.bundle.v1".into()),
            response_schema_version: Some("semantic_memory_forge.evidence_bundle.v3".into()),
        });

        semantic_memory_forge::EvidenceBundle {
            id: semantic_memory_forge::EvidenceBundleId::new(self.bundle_id.clone()),
            question,
            treatment,
            outcome,
            covariates,
            identification_rationale: self
                .identification_rationale
                .clone()
                .unwrap_or_else(|| "phase5_local_bundle_adapter".into()),
            estimator_kind: "living_memory_phase5_scorevector".into(),
            estimator_version: self.version_id.clone(),
            estimator_meta,
            estimate: self.scores.weighted_total,
            estimate_uncertainty: None,
            confidence,
            trial_count: self.verification_trials.len().max(1) as u32,
            variance_aware: self.verification_trials.len() > 1,
            verification_trials,
            comparability_snapshot,
            refutations,
            refutation_artifacts,
            verification_summary: Some(semantic_memory_forge::VerificationSummary {
                lifecycle_state,
                promotion_state: derived_promotion_state.clone(),
                completed_trial_count,
                passed_refutation_count,
                failed_refutation_count,
                comparability_snapshot_version: self.bundle_scope.as_ref().map(|scope| {
                    format!(
                        "{}:{}:{}",
                        scope.workload_id, scope.backend_family, scope.timeout_class
                    )
                }),
                notes: verification_notes,
            }),
            raw_receipt_handle,
            trace_ctx: self
                .trace_id
                .as_ref()
                .map(|trace_id| stack_ids::TraceCtx::from_trace_id(trace_id.as_str())),
            attempt_id: self
                .verification_trials
                .first()
                .map(|trial| trial.attempt_id.clone())
                .or_else(|| {
                    self.attempt_id
                        .as_ref()
                        .map(|attempt| AttemptId::new(attempt.clone()))
                }),
            trial_id: self
                .verification_trials
                .first()
                .map(|trial| trial.trial_id.clone()),
            replay_handle: self
                .receipts
                .iter()
                .find_map(|receipt| receipt.replay_handle.clone()),
            source_envelope_id: None,
            claim_ids: Vec::new(),
            created_at: self.created_at.clone(),
            comparability_snapshot_version: self.bundle_scope.as_ref().map(|scope| {
                format!(
                    "{}:{}:{}",
                    scope.workload_id, scope.backend_family, scope.timeout_class
                )
            }),
            metadata: Some(serde_json::json!({
                "bundle_id": self.bundle_id,
                "candidate_id": self.candidate_id,
                "eval_id": self.eval_id,
                "version_id": self.version_id,
                "claim_strength": format!("{}", self.claim_strength),
                "assessment": self.assessment,
                "pair_comparability": self.pair_comparability,
                "promotion_state": derived_promotion_state,
                "receipt_count": self.receipts.len(),
                "verification_trial_count": self.verification_trials.len(),
                "refutation_artifact_count": self.refutation_artifacts.len(),
                "canonical_owner": "semantic_memory_forge::EvidenceBundle",
                LOCAL_AUTHORING_METADATA_KEY: LocalEvidenceBundleAuthoring::from_bundle(self),
            })),
        }
    }

    /// Rebuild the local authoring wrapper from the canonical Forge evidence contract.
    ///
    /// The canonical bundle is authoritative. When local authoring metadata is
    /// present, it is used only to recover forge-engine-specific working fields
    /// that are intentionally outside the canonical contract.
    pub fn from_canonical_evidence_bundle(
        canonical: &semantic_memory_forge::EvidenceBundle,
    ) -> ForgeResult<Self> {
        let authoring = canonical
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(LOCAL_AUTHORING_METADATA_KEY))
            .cloned()
            .map(serde_json::from_value::<LocalEvidenceBundleAuthoring>)
            .transpose()?;

        let bundle_scope = canonical
            .comparability_snapshot
            .as_ref()
            .map(|snapshot| BundleScope {
                workload_id: snapshot.workload_id.clone(),
                backend_family: snapshot.backend_family.clone(),
                selected_checks: snapshot.selected_checks.clone(),
                timeout_class: snapshot.timeout_class.clone(),
                config_flags: snapshot.config_flags.clone(),
            });

        let pair_comparability = authoring
            .as_ref()
            .and_then(|authoring| authoring.pair_comparability.clone())
            .or_else(|| {
                canonical
                    .comparability_snapshot
                    .as_ref()
                    .map(|snapshot| PairComparability {
                        valid: snapshot.comparable.unwrap_or(false),
                        violations: snapshot.violations.clone(),
                    })
            });

        let treatment = authoring
            .as_ref()
            .and_then(|authoring| authoring.treatment.clone())
            .or_else(|| {
                if canonical.treatment.description.is_empty() {
                    None
                } else {
                    Some(Treatment {
                        kind: "patch_applied".into(),
                        patch_hash: String::new(),
                        patch_summary: canonical.treatment.description.clone(),
                    })
                }
            });

        let covariates = authoring
            .as_ref()
            .and_then(|authoring| authoring.covariates.clone());

        let score_fallback = ScoreVector {
            correctness: canonical.estimate,
            novelty: 0.0,
            stability: canonical.confidence as f64,
            weighted_total: canonical.estimate,
            cea_confidence: None,
            cea_predicted_correctness: None,
        };

        Ok(Self {
            bundle_id: canonical.id.as_str().to_string(),
            candidate_id: authoring
                .as_ref()
                .map(|authoring| authoring.candidate_id.clone())
                .or_else(|| {
                    canonical.metadata.as_ref().and_then(|metadata| {
                        metadata
                            .get("candidate_id")
                            .and_then(serde_json::Value::as_str)
                            .map(ToOwned::to_owned)
                    })
                })
                .unwrap_or_else(|| "unknown_candidate".into()),
            eval_id: authoring
                .as_ref()
                .map(|authoring| authoring.eval_id.clone())
                .or_else(|| {
                    canonical.metadata.as_ref().and_then(|metadata| {
                        metadata
                            .get("eval_id")
                            .and_then(serde_json::Value::as_str)
                            .map(ToOwned::to_owned)
                    })
                })
                .unwrap_or_else(|| "unknown_eval".into()),
            version_id: authoring
                .as_ref()
                .map(|authoring| authoring.version_id.clone())
                .or_else(|| {
                    canonical.metadata.as_ref().and_then(|metadata| {
                        metadata
                            .get("version_id")
                            .and_then(serde_json::Value::as_str)
                            .map(ToOwned::to_owned)
                    })
                })
                .unwrap_or_else(|| canonical.estimator_version.clone()),
            supersedes_claim_version_id: authoring
                .as_ref()
                .and_then(|authoring| authoring.supersedes_claim_version_id.clone()),
            relation_lineage_hints: authoring
                .as_ref()
                .map(|authoring| authoring.relation_lineage_hints.clone())
                .unwrap_or_default(),
            scores: authoring
                .as_ref()
                .map(|authoring| authoring.scores.clone())
                .unwrap_or(score_fallback),
            hypotheses: authoring
                .as_ref()
                .map(|authoring| authoring.hypotheses.clone())
                .unwrap_or_default(),
            verification: authoring
                .as_ref()
                .and_then(|authoring| authoring.verification.clone()),
            trace_id: canonical
                .trace_ctx
                .as_ref()
                .map(|trace_ctx| trace_ctx.trace_id.clone()),
            experiment_diff: authoring
                .as_ref()
                .and_then(|authoring| authoring.experiment_diff.clone()),
            attribution_json: authoring
                .as_ref()
                .and_then(|authoring| authoring.attribution_json.clone()),
            assessment: authoring
                .as_ref()
                .and_then(|authoring| authoring.assessment.clone()),
            warnings: authoring
                .as_ref()
                .map(|authoring| authoring.warnings.clone())
                .unwrap_or_default(),
            created_at: canonical.created_at.clone(),
            run_id: authoring
                .as_ref()
                .and_then(|authoring| authoring.run_id.clone()),
            attempt_id: canonical
                .attempt_id
                .as_ref()
                .map(|attempt_id| attempt_id.as_str().to_string())
                .or_else(|| {
                    authoring
                        .as_ref()
                        .and_then(|authoring| authoring.attempt_id.clone())
                }),
            causal_question: Some(canonical.question.description.clone()),
            unit_definition: Some(canonical.question.unit_definition.clone()),
            bundle_scope,
            pair_comparability,
            claim_strength: authoring
                .as_ref()
                .map(|authoring| authoring.claim_strength)
                .unwrap_or_default(),
            identification_rationale: Some(canonical.identification_rationale.clone()),
            known_threats: authoring
                .as_ref()
                .map(|authoring| authoring.known_threats.clone())
                .unwrap_or_default(),
            patch_hash: authoring
                .as_ref()
                .and_then(|authoring| authoring.patch_hash.clone()),
            treatment,
            outcome: Some(canonical.outcome.description.clone()),
            covariates,
            promotion_state: canonical
                .verification_summary
                .as_ref()
                .map(|summary| summary.promotion_state.clone()),
            primary_effect: authoring
                .as_ref()
                .and_then(|authoring| authoring.primary_effect.clone()),
            all_effects: authoring
                .as_ref()
                .map(|authoring| authoring.all_effects.clone())
                .unwrap_or_default(),
            hypothesis_edges: authoring
                .as_ref()
                .map(|authoring| authoring.hypothesis_edges.clone())
                .unwrap_or_default(),
            receipts: authoring
                .as_ref()
                .map(|authoring| authoring.receipts.clone())
                .unwrap_or_default(),
            verification_trials: if let Some(authoring) = authoring.as_ref() {
                authoring.verification_trials.clone()
            } else {
                canonical
                    .verification_trials
                    .iter()
                    .map(|trial| VerificationTrial {
                        trial_id: trial.trial_id.clone(),
                        attempt_id: trial.attempt_id.clone(),
                        baseline_or_patch: match trial.side {
                            semantic_memory_forge::VerificationTrialSide::Baseline => {
                                BaselineOrPatch::Baseline
                            }
                            semantic_memory_forge::VerificationTrialSide::Patched => {
                                BaselineOrPatch::Patched
                            }
                        },
                        completed: trial.completed,
                        receipts: trial.receipt_handles.clone(),
                    })
                    .collect()
            },
            refutation_artifacts: if let Some(authoring) = authoring.as_ref() {
                authoring.refutation_artifacts.clone()
            } else {
                canonical
                    .refutation_artifacts
                    .iter()
                    .map(|artifact| RefutationArtifact {
                        artifact_id: artifact.artifact_id.clone(),
                        artifact_type: match artifact.artifact_type.as_str() {
                            "placebo" => RefutationArtifactType::Placebo,
                            "dummy_outcome" => RefutationArtifactType::DummyOutcome,
                            _ => RefutationArtifactType::SubsampleStability,
                        },
                        trial_id: artifact.trial_id.clone(),
                        attempt_id: artifact.attempt_id.clone(),
                        outcome: match &artifact.result {
                            semantic_memory_forge::RefutationResult::Passed { .. } => {
                                RefutationArtifactOutcome::Passed
                            }
                            semantic_memory_forge::RefutationResult::Failed { reason, .. } => {
                                RefutationArtifactOutcome::Failed {
                                    reason: reason.clone(),
                                }
                            }
                            semantic_memory_forge::RefutationResult::Inconclusive { reason } => {
                                RefutationArtifactOutcome::Inconclusive {
                                    reason: reason.clone(),
                                }
                            }
                            semantic_memory_forge::RefutationResult::Skipped { reason } => {
                                RefutationArtifactOutcome::Skipped {
                                    reason: reason.clone(),
                                }
                            }
                        },
                        estimate_delta: artifact.estimate_delta,
                        details: artifact.details.clone(),
                    })
                    .collect()
            },
            sealed: authoring
                .as_ref()
                .map(|authoring| authoring.sealed)
                .unwrap_or(false),
        })
    }

    /// Render the causal question for this bundle.
    pub fn render_causal_question(
        patch_summary: &str,
        outcome_summary: &str,
        workload_id: &str,
        scope_summary: &str,
    ) -> String {
        format!(
            "Did patch {} change outcome {} on workload {} under scope {}?",
            patch_summary, outcome_summary, workload_id, scope_summary
        )
    }

    fn debug_name_to_snake_case(name: &str) -> String {
        let mut rendered = String::with_capacity(name.len() + 4);
        for (idx, ch) in name.chars().enumerate() {
            if ch.is_uppercase() {
                if idx > 0 {
                    rendered.push('_');
                }
                for lower in ch.to_lowercase() {
                    rendered.push(lower);
                }
            } else {
                rendered.push(ch);
            }
        }
        rendered
    }

    /// Return the prior relation version for an effect-derived export relation, when known.
    pub fn superseded_effect_relation_version_id(
        &self,
        source: EffectRelationLineageSource,
        effect: &crate::experiment::TypedLocatedEffect,
    ) -> Option<RelationVersionId> {
        self.relation_lineage_hints
            .effect_supersedes_relation_version_id(source, effect)
    }

    /// Return the prior relation version for a hypothesis-edge export relation, when known.
    pub fn superseded_hypothesis_relation_version_id(
        &self,
        edge: &HypothesisEdge,
    ) -> Option<RelationVersionId> {
        self.relation_lineage_hints
            .hypothesis_supersedes_relation_version_id(edge)
    }

    /// Return the prior relation version for a verification-trial export relation, when known.
    pub fn superseded_verification_trial_relation_version_id(
        &self,
        trial: &VerificationTrial,
    ) -> Option<RelationVersionId> {
        self.relation_lineage_hints
            .verification_trial_supersedes_relation_version_id(trial)
    }

    /// Return the prior relation version for a refutation-artifact export relation, when known.
    pub fn superseded_refutation_relation_version_id(
        &self,
        artifact: &RefutationArtifact,
    ) -> Option<RelationVersionId> {
        self.relation_lineage_hints
            .refutation_supersedes_relation_version_id(artifact)
    }

    /// Convert the bundle into episode metadata suitable for semantic-memory storage.
    pub fn to_episode_meta(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "forge_evidence",
            "bundle_id": self.bundle_id,
            "candidate_id": self.candidate_id,
            "eval_id": self.eval_id,
            "version_id": self.version_id,
            "supersedes_claim_version_id": self.supersedes_claim_version_id,
            "trace_id": self.trace_id,
            "hypothesis_count": self.hypotheses.len(),
            "hypothesis_edge_count": self.hypothesis_edges.len(),
            "has_verification": self.verification.is_some(),
            "verification_trial_count": self.verification_trials.len(),
            "refutation_artifact_count": self.refutation_artifacts.len(),
            "claim_strength": format!("{}", self.claim_strength),
            "run_id": self.run_id,
            "attempt_id": self.attempt_id,
            "treatment": self.treatment,
            "outcome": self.outcome,
            "confounders": self.known_threats,
            "identification_rationale": self.identification_rationale,
            "patch_hash": self.patch_hash,
            "estimator_metadata": serde_json::json!({
                "weighted_total": self.scores.weighted_total,
                "cea_confidence": self.scores.cea_confidence,
                "cea_predicted_correctness": self.scores.cea_predicted_correctness,
            }),
            "sealed": self.sealed,
        })
    }

    /// Convert the bundle into episode content text for embedding.
    ///
    /// Includes: patch summary, primary causal question, primary effect summary,
    /// key covariates, claim strength, and verification recommendations.
    /// Does NOT include: giant raw logs, unstable formatting, or missing identifiers.
    pub fn to_episode_content(&self) -> String {
        let mut parts = Vec::new();

        // Primary identifiers
        parts.push(format!(
            "Evidence bundle {} for candidate {} (eval {})",
            self.bundle_id, self.candidate_id, self.eval_id
        ));

        // Claim strength
        parts.push(format!("Claim strength: {}", self.claim_strength));

        if let Some(ref outcome) = self.outcome {
            parts.push(format!("Outcome: {outcome}"));
        }

        if let Some(ref rationale) = self.identification_rationale {
            parts.push(format!("Identification rationale: {rationale}"));
        }

        // Causal question
        if let Some(ref q) = self.causal_question {
            parts.push(format!("Causal question: {q}"));
        }

        // Patch summary
        if let Some(ref t) = self.treatment {
            parts.push(format!(
                "Treatment: kind={kind}, patch_hash={patch_hash}, summary={summary}",
                kind = t.kind,
                patch_hash = t.patch_hash,
                summary = t.patch_summary
            ));
        }

        // Primary effect
        if let Some(ref eff) = self.primary_effect {
            parts.push(format!("Primary effect: {:?} - {}", eff.kind, eff.message));
        }

        // Scores
        parts.push(format!(
            "Scores: correctness={:.2}, novelty={:.2}, stability={:.2}, total={:.2}",
            self.scores.correctness,
            self.scores.novelty,
            self.scores.stability,
            self.scores.weighted_total,
        ));

        // Key covariates
        if let Some(ref cov) = self.covariates {
            parts.push(format!("Workload: {}", cov.workload_id));
            parts.push(format!("Checks: {}", cov.selected_checks.join(", ")));
            if cov.adjacent_edits {
                parts.push("Adjacent edits present".to_string());
            }
            if !cov.adjacent_edit_signatures.is_empty() {
                parts.push(format!(
                    "Adjacent edits: {}",
                    cov.adjacent_edit_signatures.join(", ")
                ));
            }
        }

        if !self.known_threats.is_empty() {
            parts.push(format!(
                "Confounders / threats: {}",
                self.known_threats.join(", ")
            ));
        }

        // Hypothesis edges
        for edge in &self.hypothesis_edges {
            parts.push(format!(
                "Edge {}: {:?} {:?} (confidence={:.2})",
                edge.edge_id, edge.kind, edge.status, edge.confidence
            ));
        }

        // Legacy hypotheses (for backward compat)
        for h in &self.hypotheses {
            parts.push(format!(
                "Hypothesis {}: {:?} (confidence={:.2}, support={}, contradictions={})",
                h.hypothesis_id, h.status, h.confidence, h.support_count, h.contradiction_count
            ));
        }

        // Explicit trial lineage
        for trial in &self.verification_trials {
            parts.push(format!(
                "Trial {} / attempt {}: {:?} completed={}",
                trial.trial_id, trial.attempt_id, trial.baseline_or_patch, trial.completed
            ));
        }

        // Refutation artifacts (pass/fail surface)
        for artifact in &self.refutation_artifacts {
            let outcome = match &artifact.outcome {
                RefutationArtifactOutcome::Passed => "passed".to_string(),
                RefutationArtifactOutcome::Failed { reason } => format!("failed: {reason}"),
                RefutationArtifactOutcome::Inconclusive { reason } => {
                    format!("inconclusive: {reason}")
                }
                RefutationArtifactOutcome::Skipped { reason } => format!("skipped: {reason}"),
            };
            parts.push(format!(
                "Refutation {:?} {}",
                artifact.artifact_type, outcome
            ));
        }

        // Verification recommendations
        if let Some(ref plan) = self.verification {
            let required_count = plan
                .steps
                .iter()
                .filter(|s| s.requirement == StepRequirement::Required)
                .count();
            parts.push(format!(
                "Verification: {} steps ({} required)",
                plan.steps.len(),
                required_count,
            ));
            if let Some(ref budget) = plan.budget {
                parts.push(format!(
                    "Plan budget: max {} steps, est. {}s",
                    budget.max_steps, budget.estimated_duration_secs
                ));
            }
        }

        parts.join("\n")
    }
}

fn default_bundle_created_at() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[path = "evidence_analysis.rs"]
mod evidence_analysis;

#[allow(deprecated)]
pub use evidence_analysis::{
    build_hypothesis_edges, compute_assessment, compute_confidence, derive_status,
    generate_verification_plan, local_hypothesis_support_confidence, update_hypotheses_from_diff,
    AssessmentCategory, CausalHypothesis, ContradictionState, DroppedStep, EvidenceAssessment,
    ExperimentRunner, HypothesisStatus, LocalExperimentRunner, PairComparability, PlanBudget,
    SampleSupport, StepRequirement, VerificationPlan, VerificationPolicy, VerificationStep,
    VerificationType,
};
