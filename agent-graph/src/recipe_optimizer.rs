//! Offline, nonauthorizing recipe optimization governance.
//!
//! This module intentionally exposes only a read-only trial port. It can identify
//! and rank candidate recipes, but it cannot patch code, change evaluator policy,
//! grant effect permission, or activate a candidate.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq)]
pub struct BaselineRecipe {
    pub recipe_ref: String,
    pub digest: String,
    pub validation_score: f64,
}

impl BaselineRecipe {
    pub fn new(
        recipe_ref: impl Into<String>,
        digest: impl Into<String>,
        validation_score: f64,
    ) -> Self {
        Self {
            recipe_ref: recipe_ref.into(),
            digest: digest.into(),
            validation_score,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AcceptanceCriteria {
    pub digest: String,
    pub mandatory_checks: BTreeSet<String>,
    pub held_out_pass_margin: f64,
}

impl AcceptanceCriteria {
    pub fn new<I, S>(
        digest: impl Into<String>,
        mandatory_checks: I,
        held_out_pass_margin: f64,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            digest: digest.into(),
            mandatory_checks: mandatory_checks.into_iter().map(Into::into).collect(),
            held_out_pass_margin,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateRecipe {
    pub recipe_ref: String,
    pub baseline_ref: String,
    pub parameters: BTreeMap<String, String>,
}

impl CandidateRecipe {
    pub fn new<I, K, V>(
        recipe_ref: impl Into<String>,
        baseline_ref: impl Into<String>,
        parameters: I,
    ) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            recipe_ref: recipe_ref.into(),
            baseline_ref: baseline_ref.into(),
            parameters: parameters
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccessRequest {
    PublicInput(String),
    HeldOutRead(String),
    HeldOutWrite(String),
}

impl AccessRequest {
    pub fn is_held_out(&self) -> bool {
        matches!(self, Self::HeldOutRead(_) | Self::HeldOutWrite(_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadOnlyWork {
    pub work_ref: String,
    pub access_requests: Vec<AccessRequest>,
}

impl ReadOnlyWork {
    pub fn new<I>(work_ref: impl Into<String>, access_requests: I) -> Self
    where
        I: IntoIterator<Item = AccessRequest>,
    {
        Self {
            work_ref: work_ref.into(),
            access_requests: access_requests.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EvaluationDecision {
    Admissible,
    Qualified { score: f64 },
    Inconclusive,
    Contaminated,
    RejectedControlMutation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReadOnlyTrialReceipt {
    pub receipt_ref: String,
    pub work_ref: String,
    pub qualification_receipt_ref: String,
    pub evaluation: EvaluationDecision,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ControlMutation {
    RemoveMandatoryCheck(String),
    SetHeldOutPassMargin(f64),
    ReplaceEvaluator(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QualificationDecision {
    Qualified { receipt_ref: String },
    Denied { reason_ref: String },
    Unknown,
}

pub trait QualificationOwnerPort {
    fn qualification_for(&self, work: &ReadOnlyWork) -> QualificationDecision;
}

pub struct EvaluationEnvelope<'a> {
    pub candidate: &'a CandidateRecipe,
    pub acceptance_criteria: &'a AcceptanceCriteria,
    pub access_requests: &'a [AccessRequest],
    pub proposed_control_mutations: &'a [ControlMutation],
    pub trial_receipt: Option<&'a ReadOnlyTrialReceipt>,
}

pub trait EvaluatorIntegrityOwnerPort {
    fn assess_integrity(&self, envelope: &EvaluationEnvelope<'_>) -> EvaluationDecision;
}

pub trait ReadOnlyWorkOwnerPort {
    fn execute_read_only(
        &mut self,
        work: &ReadOnlyWork,
        candidate: &CandidateRecipe,
        qualification_receipt_ref: &str,
    ) -> ReadOnlyTrialReceipt;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecipeCapability {
    ReadQualifiedWork,
    RankCandidates,
    PatchEngine,
    ExternalEffect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipeCapabilityManifest {
    pub capabilities: BTreeSet<RecipeCapability>,
}

impl RecipeCapabilityManifest {
    fn offline_read_only() -> Self {
        Self {
            capabilities: BTreeSet::from([
                RecipeCapability::ReadQualifiedWork,
                RecipeCapability::RankCandidates,
            ]),
        }
    }

    pub fn allows(&self, capability: RecipeCapability) -> bool {
        self.capabilities.contains(&capability)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CandidateState {
    NonAuthorizing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateArtifact {
    pub recipe: CandidateRecipe,
    pub state: CandidateState,
    pub activation_ref: Option<String>,
}

impl CandidateArtifact {
    fn from_recipe(recipe: CandidateRecipe) -> Self {
        Self {
            recipe,
            state: CandidateState::NonAuthorizing,
            activation_ref: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrialDisposition {
    ExcludedBaselineMismatch,
    QualifiedImprovement,
    QualifiedRegression,
    Inconclusive,
    ExcludedUnqualified,
    ExcludedContaminated,
    RejectedControlMutation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrialRecord {
    pub candidate_ref: String,
    pub work_ref: String,
    pub disposition: TrialDisposition,
    pub evaluation: EvaluationDecision,
    pub validation_score: Option<f64>,
    pub qualification_receipt_ref: Option<String>,
    pub trial_receipt_ref: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OptimizationDisposition {
    QualifiedCandidateAvailable,
    NoQualifiedImprovement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromotionRequest {
    pub candidate_ref: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OptimizationReport {
    pub disposition: OptimizationDisposition,
    /// The selected active recipe remains the immutable baseline. A candidate
    /// requires a separate publisher action after this report.
    pub selected_recipe: BaselineRecipe,
    pub baseline: BaselineRecipe,
    pub acceptance_criteria: AcceptanceCriteria,
    pub candidates: Vec<CandidateArtifact>,
    pub trials: Vec<TrialRecord>,
    pub ranked_candidate_refs: Vec<String>,
    pub promotion_request: Option<PromotionRequest>,
    pub capability_manifest: RecipeCapabilityManifest,
    pub granted_new_effect_permission: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrialRequest {
    pub candidate: CandidateRecipe,
    pub work: ReadOnlyWork,
    pub proposed_control_mutations: Vec<ControlMutation>,
}

impl TrialRequest {
    pub fn new(candidate: CandidateRecipe, work: ReadOnlyWork) -> Self {
        Self {
            candidate,
            work,
            proposed_control_mutations: Vec::new(),
        }
    }

    pub fn with_control_mutations<I>(mut self, mutations: I) -> Self
    where
        I: IntoIterator<Item = ControlMutation>,
    {
        self.proposed_control_mutations = mutations.into_iter().collect();
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ActivationDecision {
    Authorized {
        authorization_ref: String,
    },
    Denied {
        reason_ref: String,
    },
    #[default]
    Unknown,
}

pub trait PublisherAuthorityOwnerPort {
    fn authorize_activation(&self, candidate_ref: &str) -> ActivationDecision;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivationRequestResult {
    pub candidate: CandidateArtifact,
    pub decision: ActivationDecision,
    /// Always false: this module has no activation sink.
    pub activated: bool,
}

#[derive(Clone, Debug)]
pub struct OfflineRecipeOptimizer {
    baseline: BaselineRecipe,
    acceptance_criteria: AcceptanceCriteria,
}

impl OfflineRecipeOptimizer {
    pub fn new(baseline: BaselineRecipe, acceptance_criteria: AcceptanceCriteria) -> Self {
        Self {
            baseline,
            acceptance_criteria,
        }
    }

    pub fn optimize<I>(
        &self,
        requests: I,
        qualification_owner: &dyn QualificationOwnerPort,
        evaluator_owner: &dyn EvaluatorIntegrityOwnerPort,
        work_owner: &mut dyn ReadOnlyWorkOwnerPort,
    ) -> OptimizationReport
    where
        I: IntoIterator<Item = TrialRequest>,
    {
        let mut candidates = Vec::new();
        let mut trials = Vec::new();
        let mut qualified_improvements = Vec::new();

        for request in requests {
            let candidate_ref = request.candidate.recipe_ref.clone();
            let work_ref = request.work.work_ref.clone();
            candidates.push(CandidateArtifact::from_recipe(request.candidate.clone()));

            if request.candidate.baseline_ref != self.baseline.recipe_ref {
                trials.push(TrialRecord {
                    candidate_ref,
                    work_ref,
                    disposition: TrialDisposition::ExcludedBaselineMismatch,
                    evaluation: EvaluationDecision::Inconclusive,
                    validation_score: None,
                    qualification_receipt_ref: None,
                    trial_receipt_ref: None,
                });
                continue;
            }
            let qualification_receipt_ref =
                match qualification_owner.qualification_for(&request.work) {
                    QualificationDecision::Qualified { receipt_ref } => receipt_ref,
                    QualificationDecision::Denied { .. } | QualificationDecision::Unknown => {
                        trials.push(TrialRecord {
                            candidate_ref,
                            work_ref,
                            disposition: TrialDisposition::ExcludedUnqualified,
                            evaluation: EvaluationDecision::Inconclusive,
                            validation_score: None,
                            qualification_receipt_ref: None,
                            trial_receipt_ref: None,
                        });
                        continue;
                    }
                };

            let preflight = evaluator_owner.assess_integrity(&EvaluationEnvelope {
                candidate: &request.candidate,
                acceptance_criteria: &self.acceptance_criteria,
                access_requests: &request.work.access_requests,
                proposed_control_mutations: &request.proposed_control_mutations,
                trial_receipt: None,
            });

            let denied = if !request.proposed_control_mutations.is_empty() {
                Some((
                    TrialDisposition::RejectedControlMutation,
                    EvaluationDecision::RejectedControlMutation,
                ))
            } else if request
                .work
                .access_requests
                .iter()
                .any(AccessRequest::is_held_out)
            {
                Some((
                    TrialDisposition::ExcludedContaminated,
                    EvaluationDecision::Contaminated,
                ))
            } else {
                match preflight {
                    EvaluationDecision::Contaminated => Some((
                        TrialDisposition::ExcludedContaminated,
                        EvaluationDecision::Contaminated,
                    )),
                    EvaluationDecision::RejectedControlMutation => Some((
                        TrialDisposition::RejectedControlMutation,
                        EvaluationDecision::RejectedControlMutation,
                    )),
                    _ => None,
                }
            };

            if let Some((disposition, evaluation)) = denied {
                trials.push(TrialRecord {
                    candidate_ref,
                    work_ref,
                    disposition,
                    evaluation,
                    validation_score: None,
                    qualification_receipt_ref: Some(qualification_receipt_ref),
                    trial_receipt_ref: None,
                });
                continue;
            }

            let receipt = work_owner.execute_read_only(
                &request.work,
                &request.candidate,
                &qualification_receipt_ref,
            );
            let evaluation = evaluator_owner.assess_integrity(&EvaluationEnvelope {
                candidate: &request.candidate,
                acceptance_criteria: &self.acceptance_criteria,
                access_requests: &request.work.access_requests,
                proposed_control_mutations: &request.proposed_control_mutations,
                trial_receipt: Some(&receipt),
            });

            let (disposition, score) = match evaluation {
                EvaluationDecision::Qualified { score }
                    if score
                        >= self.baseline.validation_score
                            + self.acceptance_criteria.held_out_pass_margin =>
                {
                    qualified_improvements.push((candidate_ref.clone(), score));
                    (TrialDisposition::QualifiedImprovement, Some(score))
                }
                EvaluationDecision::Qualified { score } => {
                    (TrialDisposition::QualifiedRegression, Some(score))
                }
                EvaluationDecision::Contaminated => (TrialDisposition::ExcludedContaminated, None),
                EvaluationDecision::RejectedControlMutation => {
                    (TrialDisposition::RejectedControlMutation, None)
                }
                EvaluationDecision::Admissible | EvaluationDecision::Inconclusive => {
                    (TrialDisposition::Inconclusive, None)
                }
            };
            trials.push(TrialRecord {
                candidate_ref,
                work_ref,
                disposition,
                evaluation,
                validation_score: score,
                qualification_receipt_ref: Some(qualification_receipt_ref),
                trial_receipt_ref: Some(receipt.receipt_ref),
            });
        }

        qualified_improvements.sort_by(|left, right| {
            right
                .1
                .total_cmp(&left.1)
                .then_with(|| left.0.cmp(&right.0))
        });
        let ranked_candidate_refs: Vec<_> = qualified_improvements
            .iter()
            .map(|(candidate_ref, _)| candidate_ref.clone())
            .collect();
        let promotion_request = ranked_candidate_refs
            .first()
            .cloned()
            .map(|candidate_ref| PromotionRequest { candidate_ref });
        let disposition = if promotion_request.is_some() {
            OptimizationDisposition::QualifiedCandidateAvailable
        } else {
            OptimizationDisposition::NoQualifiedImprovement
        };

        OptimizationReport {
            disposition,
            selected_recipe: self.baseline.clone(),
            baseline: self.baseline.clone(),
            acceptance_criteria: self.acceptance_criteria.clone(),
            candidates,
            trials,
            ranked_candidate_refs,
            promotion_request,
            capability_manifest: RecipeCapabilityManifest::offline_read_only(),
            granted_new_effect_permission: false,
        }
    }

    pub fn request_activation(
        &self,
        candidate: &CandidateRecipe,
        publisher_owner: &dyn PublisherAuthorityOwnerPort,
    ) -> ActivationRequestResult {
        ActivationRequestResult {
            candidate: CandidateArtifact::from_recipe(candidate.clone()),
            decision: publisher_owner.authorize_activation(&candidate.recipe_ref),
            activated: false,
        }
    }
}
