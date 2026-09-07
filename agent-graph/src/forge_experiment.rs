//! Forge experiment orchestration with external native authority.
//!
//! Agent Graph coordinates isolated candidate workspaces, immutable test-set
//! bindings, trial history, and deterministic selection. It does not mint
//! operation identity, execution permits, effect accounting, containment
//! evidence, canonical memory support, or publication authority.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExperimentSpec {
    pub experiment_id: String,
    pub base_revision: String,
    pub test_sets: TestSetSnapshot,
    pub allowed_patch_scope: BTreeSet<String>,
    pub max_patch_delta_lines: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestSetSnapshot {
    pub public_digest: String,
    pub evaluator_digest: String,
    pub withheld_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestSetKind {
    Public,
    Evaluator,
    Withheld,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestSetAssessment {
    pub accepted: bool,
    pub independent_test_change_review_required: bool,
    pub changed_sets: Vec<TestSetKind>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StagingStatus {
    Pending,
    Passed,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatchProposal {
    pub suggested_delta_lines: usize,
    pub actual_delta_lines: usize,
    pub touched_scopes: BTreeSet<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionChecks {
    NotRequired,
    Required,
    Satisfied,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateWorkspace {
    pub candidate_id: String,
    pub workspace_id: String,
    pub base_revision: String,
    pub provenance_ref: String,
    /// Advisory projection only; never containment or authorization evidence.
    pub staging_status: StagingStatus,
    pub containment_evidence_ref: Option<String>,
    pub patch: PatchProposal,
    pub parent_candidate_ids: Vec<String>,
    pub interaction_checks: InteractionChecks,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateLifecycle {
    Active,
    Selected,
    Published,
    Historical,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateRecord {
    pub candidate_id: String,
    pub workspace_id: String,
    pub base_revision: String,
    pub provenance_ref: String,
    pub staging_status: StagingStatus,
    pub containment_evidence_ref: Option<String>,
    pub patch: PatchProposal,
    pub parent_candidate_ids: Vec<String>,
    pub interaction_checks: InteractionChecks,
    pub interaction_check_receipt_ref: Option<String>,
    pub lifecycle: CandidateLifecycle,
}

impl From<CandidateWorkspace> for CandidateRecord {
    fn from(candidate: CandidateWorkspace) -> Self {
        Self {
            candidate_id: candidate.candidate_id,
            workspace_id: candidate.workspace_id,
            base_revision: candidate.base_revision,
            provenance_ref: candidate.provenance_ref,
            staging_status: candidate.staging_status,
            containment_evidence_ref: candidate.containment_evidence_ref,
            patch: candidate.patch,
            parent_candidate_ids: candidate.parent_candidate_ids,
            interaction_checks: candidate.interaction_checks,
            interaction_check_receipt_ref: None,
            lifecycle: CandidateLifecycle::Active,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainmentQuery {
    pub experiment_id: String,
    pub candidate_id: String,
    pub workspace_id: String,
    pub base_revision: String,
    pub evidence_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationAuthorizationQuery {
    pub experiment_id: String,
    pub candidate_id: String,
    pub workspace_id: String,
    pub base_revision: String,
    pub provenance_ref: String,
    pub actual_delta_lines: usize,
    pub touched_scopes: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeOperationGrant {
    pub operation_identity_ref: String,
    pub permit_ref: String,
    pub effect_accounting_ref: String,
}

impl NativeOperationGrant {
    fn is_complete(&self) -> bool {
        !self.operation_identity_ref.is_empty()
            && !self.permit_ref.is_empty()
            && !self.effect_accounting_ref.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnerDecision {
    Authorized,
    Rejected,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnerAuthorization {
    Granted(NativeOperationGrant),
    Rejected,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicationRequest {
    pub experiment_id: String,
    pub candidate_id: String,
    pub workspace_id: String,
    pub provenance_ref: String,
    pub authorization: NativeOperationGrant,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicationReceipt {
    pub publication_ref: String,
    pub operation_identity_ref: String,
    pub permit_ref: String,
    pub effect_accounting_ref: String,
}

/// Non-serializable authority port. Forge consumes owner decisions; it never
/// manufactures their identity, permit, containment, effects, or publication.
pub trait ForgeNativeOwner {
    fn verify_containment(&self, query: &ContainmentQuery) -> OwnerDecision;
    fn authorize_operation(&self, query: &OperationAuthorizationQuery) -> OwnerAuthorization;
    fn publish(&self, request: &PublicationRequest) -> Result<PublicationReceipt, OwnerDecision>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GovernanceReason {
    UnknownCandidate,
    StagingIncompleteOrFailed,
    TrialEvidenceIncompleteOrFailed,
    StaleBase,
    RetestRequired,
    ContainmentEvidenceMissing,
    ContainmentRejected,
    NativeOwnerUnavailable,
    ActualDeltaExceedsClamp,
    PatchScopeExceeded,
    OperationRejected,
    NativeGrantIncomplete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateDisposition {
    Ready,
    Blocked,
    StaleBase,
    RetestRequired,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateAssessment {
    pub disposition: CandidateDisposition,
    pub reasons: BTreeSet<GovernanceReason>,
    pub native_containment_verified: bool,
    pub authorization: Option<NativeOperationGrant>,
}

impl CandidateAssessment {
    fn with(
        disposition: CandidateDisposition,
        reasons: impl IntoIterator<Item = GovernanceReason>,
    ) -> Self {
        Self {
            disposition,
            reasons: reasons.into_iter().collect(),
            native_containment_verified: false,
            authorization: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryTier {
    CandidateQuarantine,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateMemoryRecord {
    pub record_id: String,
    pub candidate_id: String,
    pub content: String,
    pub tier: MemoryTier,
    pub canonical_supported: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CeaProposal {
    pub proposal_id: String,
    pub suspect: String,
    pub hypothesis: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CausalState {
    HypothesisOnly,
    SupportedByPairedIntervention,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CausalRecord {
    proposal: CeaProposal,
    state: CausalState,
    intervention_ref: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AblationEvidence {
    pub valid: bool,
    pub comparable: bool,
    pub control_environment: String,
    pub treatment_environment: String,
    pub effect_observed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CausalComparison {
    Supported,
    Refuted,
    InconclusiveInvalidAblation,
    InconclusiveNonComparable,
    BlockedEnvironmentConfound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestAccess {
    Public,
    Evaluator,
    WithheldDiscriminator,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrialPlan {
    pub trial_id: String,
    pub candidate_id: String,
    pub requested_test_access: TestAccess,
}

impl TrialPlan {
    pub fn public(trial_id: impl Into<String>, candidate_id: impl Into<String>) -> Self {
        Self {
            trial_id: trial_id.into(),
            candidate_id: candidate_id.into(),
            requested_test_access: TestAccess::Public,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrialOutcome {
    Passed,
    Failed(String),
    WithheldAccessDenied,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrialCost {
    pub wall_millis: u64,
    pub compute_units: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrialCompletion {
    pub outcome: TrialOutcome,
    pub cost: TrialCost,
    pub environment_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrialRecord {
    pub trial_id: String,
    pub candidate_id: String,
    pub requested_test_access: TestAccess,
    pub outcome: Option<TrialOutcome>,
    pub cost: Option<TrialCost>,
    pub environment_ref: Option<String>,
    pub contaminated: bool,
    pub usable: bool,
    pub public_tests_remain_valid: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ForgeError {
    DuplicateCandidate(String),
    WorkspaceNotIsolated(String),
    UnknownCandidate(String),
    DuplicateTrial(String),
    UnknownTrial(String),
    TrialAlreadyCompleted(String),
    UnknownCeaProposal(String),
    DuplicateCeaProposal(String),
    CanonicalMemoryOwnerRequired,
    CandidateAlreadySelected,
    NoSelectedCandidate,
    PublicationAlreadyCompleted,
    CandidateNotPublishable,
    OwnerRejected,
    OwnerUnavailable,
    InvalidOwnerGrant,
    PublicationReceiptMismatch,
    EmptyCombination,
    MixedCombinationBase,
}

impl fmt::Display for ForgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ForgeError {}

/// In-memory orchestration projection. All failed trials and candidate history
/// are append-preserving for the lifetime of this experiment value.
#[derive(Debug)]
pub struct ForgeExperiment {
    spec: ExperimentSpec,
    candidates: BTreeMap<String, CandidateRecord>,
    workspace_ids: BTreeSet<String>,
    selected_candidate: Option<String>,
    publication: Option<PublicationReceipt>,
    candidate_memory: Vec<CandidateMemoryRecord>,
    causal_records: BTreeMap<String, CausalRecord>,
    trials: Vec<TrialRecord>,
    trial_indexes: BTreeMap<String, usize>,
}

impl ForgeExperiment {
    pub fn new(spec: ExperimentSpec) -> Self {
        Self {
            spec,
            candidates: BTreeMap::new(),
            workspace_ids: BTreeSet::new(),
            selected_candidate: None,
            publication: None,
            candidate_memory: Vec::new(),
            causal_records: BTreeMap::new(),
            trials: Vec::new(),
            trial_indexes: BTreeMap::new(),
        }
    }

    pub fn test_sets(&self) -> &TestSetSnapshot {
        &self.spec.test_sets
    }

    pub fn assess_test_sets(&self, observed: &TestSetSnapshot) -> TestSetAssessment {
        let mut changed_sets = Vec::new();
        if observed.public_digest != self.spec.test_sets.public_digest {
            changed_sets.push(TestSetKind::Public);
        }
        if observed.evaluator_digest != self.spec.test_sets.evaluator_digest {
            changed_sets.push(TestSetKind::Evaluator);
        }
        if observed.withheld_digest != self.spec.test_sets.withheld_digest {
            changed_sets.push(TestSetKind::Withheld);
        }
        TestSetAssessment {
            accepted: changed_sets.is_empty(),
            independent_test_change_review_required: changed_sets
                .iter()
                .any(|kind| matches!(kind, TestSetKind::Evaluator | TestSetKind::Withheld)),
            changed_sets,
        }
    }

    pub fn register_candidate(&mut self, candidate: CandidateWorkspace) -> Result<(), ForgeError> {
        if self.candidates.contains_key(&candidate.candidate_id) {
            return Err(ForgeError::DuplicateCandidate(candidate.candidate_id));
        }
        if self.workspace_ids.contains(&candidate.workspace_id) {
            return Err(ForgeError::WorkspaceNotIsolated(candidate.workspace_id));
        }
        self.workspace_ids.insert(candidate.workspace_id.clone());
        self.candidates
            .insert(candidate.candidate_id.clone(), candidate.into());
        Ok(())
    }

    pub fn candidate(&self, candidate_id: &str) -> Option<&CandidateRecord> {
        self.candidates.get(candidate_id)
    }

    pub fn assess_candidate(
        &self,
        candidate_id: &str,
        current_base_revision: &str,
        owner: &dyn ForgeNativeOwner,
    ) -> CandidateAssessment {
        let Some(candidate) = self.candidates.get(candidate_id) else {
            return CandidateAssessment::with(
                CandidateDisposition::Blocked,
                [GovernanceReason::UnknownCandidate],
            );
        };
        if candidate.base_revision != current_base_revision {
            return CandidateAssessment::with(
                CandidateDisposition::StaleBase,
                [GovernanceReason::StaleBase],
            );
        }
        if candidate.interaction_checks == InteractionChecks::Required {
            return CandidateAssessment::with(
                CandidateDisposition::RetestRequired,
                [GovernanceReason::RetestRequired],
            );
        }

        let mut reasons = BTreeSet::new();
        if candidate.staging_status != StagingStatus::Passed {
            reasons.insert(GovernanceReason::StagingIncompleteOrFailed);
        }
        let trials: Vec<_> = self
            .trials
            .iter()
            .filter(|trial| trial.candidate_id == candidate_id)
            .collect();
        if trials.is_empty()
            || trials.iter().any(|trial| {
                trial.outcome != Some(TrialOutcome::Passed) || !trial.usable || trial.contaminated
            })
        {
            reasons.insert(GovernanceReason::TrialEvidenceIncompleteOrFailed);
        }
        if candidate.patch.actual_delta_lines > self.spec.max_patch_delta_lines
            || candidate.patch.actual_delta_lines > candidate.patch.suggested_delta_lines
        {
            reasons.insert(GovernanceReason::ActualDeltaExceedsClamp);
        }
        if candidate
            .patch
            .touched_scopes
            .iter()
            .any(|scope| !scope_is_allowed(scope, &self.spec.allowed_patch_scope))
        {
            reasons.insert(GovernanceReason::PatchScopeExceeded);
        }
        if !reasons.is_empty() {
            return CandidateAssessment::with(CandidateDisposition::Blocked, reasons);
        }

        let Some(evidence_ref) = candidate.containment_evidence_ref.as_ref() else {
            return CandidateAssessment::with(
                CandidateDisposition::Blocked,
                [GovernanceReason::ContainmentEvidenceMissing],
            );
        };
        let containment = owner.verify_containment(&ContainmentQuery {
            experiment_id: self.spec.experiment_id.clone(),
            candidate_id: candidate.candidate_id.clone(),
            workspace_id: candidate.workspace_id.clone(),
            base_revision: candidate.base_revision.clone(),
            evidence_ref: evidence_ref.clone(),
        });
        match containment {
            OwnerDecision::Rejected => {
                return CandidateAssessment::with(
                    CandidateDisposition::Blocked,
                    [GovernanceReason::ContainmentRejected],
                );
            }
            OwnerDecision::Unavailable => {
                return CandidateAssessment::with(
                    CandidateDisposition::Blocked,
                    [GovernanceReason::NativeOwnerUnavailable],
                );
            }
            OwnerDecision::Authorized => {}
        }

        let authorization = owner.authorize_operation(&OperationAuthorizationQuery {
            experiment_id: self.spec.experiment_id.clone(),
            candidate_id: candidate.candidate_id.clone(),
            workspace_id: candidate.workspace_id.clone(),
            base_revision: candidate.base_revision.clone(),
            provenance_ref: candidate.provenance_ref.clone(),
            actual_delta_lines: candidate.patch.actual_delta_lines,
            touched_scopes: candidate.patch.touched_scopes.clone(),
        });
        match authorization {
            OwnerAuthorization::Granted(grant) if grant.is_complete() => CandidateAssessment {
                disposition: CandidateDisposition::Ready,
                reasons,
                native_containment_verified: true,
                authorization: Some(grant),
            },
            OwnerAuthorization::Granted(_) => CandidateAssessment {
                disposition: CandidateDisposition::Blocked,
                reasons: [GovernanceReason::NativeGrantIncomplete]
                    .into_iter()
                    .collect(),
                native_containment_verified: true,
                authorization: None,
            },
            OwnerAuthorization::Rejected => CandidateAssessment {
                disposition: CandidateDisposition::Blocked,
                reasons: [GovernanceReason::OperationRejected].into_iter().collect(),
                native_containment_verified: true,
                authorization: None,
            },
            OwnerAuthorization::Unavailable => CandidateAssessment {
                disposition: CandidateDisposition::Blocked,
                reasons: [GovernanceReason::NativeOwnerUnavailable]
                    .into_iter()
                    .collect(),
                native_containment_verified: true,
                authorization: None,
            },
        }
    }

    pub fn rebase_candidate(
        &mut self,
        source_candidate_id: &str,
        new_candidate_id: &str,
        new_workspace_id: &str,
        new_base_revision: &str,
        provenance_ref: &str,
    ) -> Result<&CandidateRecord, ForgeError> {
        let source = self
            .candidates
            .get(source_candidate_id)
            .cloned()
            .ok_or_else(|| ForgeError::UnknownCandidate(source_candidate_id.to_owned()))?;
        let rebased = CandidateWorkspace {
            candidate_id: new_candidate_id.to_owned(),
            workspace_id: new_workspace_id.to_owned(),
            base_revision: new_base_revision.to_owned(),
            provenance_ref: provenance_ref.to_owned(),
            staging_status: StagingStatus::Pending,
            containment_evidence_ref: None,
            patch: source.patch,
            parent_candidate_ids: vec![source_candidate_id.to_owned()],
            interaction_checks: InteractionChecks::Required,
        };
        self.register_candidate(rebased)?;
        if let Some(source) = self.candidates.get_mut(source_candidate_id) {
            source.lifecycle = CandidateLifecycle::Historical;
        }
        self.candidates
            .get(new_candidate_id)
            .ok_or_else(|| ForgeError::UnknownCandidate(new_candidate_id.to_owned()))
    }

    pub fn combine_candidates(
        &mut self,
        parent_candidate_ids: &[&str],
        new_candidate_id: &str,
        new_workspace_id: &str,
        provenance_ref: &str,
    ) -> Result<&CandidateRecord, ForgeError> {
        if parent_candidate_ids.is_empty() {
            return Err(ForgeError::EmptyCombination);
        }
        let parents = parent_candidate_ids
            .iter()
            .map(|id| {
                self.candidates
                    .get(*id)
                    .cloned()
                    .ok_or_else(|| ForgeError::UnknownCandidate((*id).to_owned()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let base_revision = parents[0].base_revision.clone();
        if parents
            .iter()
            .any(|parent| parent.base_revision != base_revision)
        {
            return Err(ForgeError::MixedCombinationBase);
        }
        let mut scopes = BTreeSet::new();
        let mut suggested_delta_lines = 0usize;
        let mut actual_delta_lines = 0usize;
        for parent in &parents {
            scopes.extend(parent.patch.touched_scopes.iter().cloned());
            suggested_delta_lines =
                suggested_delta_lines.saturating_add(parent.patch.suggested_delta_lines);
            actual_delta_lines = actual_delta_lines.saturating_add(parent.patch.actual_delta_lines);
        }
        self.register_candidate(CandidateWorkspace {
            candidate_id: new_candidate_id.to_owned(),
            workspace_id: new_workspace_id.to_owned(),
            base_revision,
            provenance_ref: provenance_ref.to_owned(),
            staging_status: StagingStatus::Pending,
            containment_evidence_ref: None,
            patch: PatchProposal {
                suggested_delta_lines,
                actual_delta_lines,
                touched_scopes: scopes,
            },
            parent_candidate_ids: parent_candidate_ids
                .iter()
                .map(|id| (*id).to_owned())
                .collect(),
            interaction_checks: InteractionChecks::Required,
        })?;
        self.candidates
            .get(new_candidate_id)
            .ok_or_else(|| ForgeError::UnknownCandidate(new_candidate_id.to_owned()))
    }

    /// Record staging evidence for this exact candidate; native verification is still required.
    pub fn record_staging(
        &mut self,
        candidate_id: &str,
        status: StagingStatus,
        containment_evidence_ref: Option<String>,
    ) -> Result<(), ForgeError> {
        let candidate = self
            .candidates
            .get_mut(candidate_id)
            .ok_or_else(|| ForgeError::UnknownCandidate(candidate_id.to_owned()))?;
        candidate.staging_status = status;
        candidate.containment_evidence_ref = containment_evidence_ref;
        Ok(())
    }

    pub fn record_interaction_checks(
        &mut self,
        candidate_id: &str,
        receipt_ref: &str,
    ) -> Result<(), ForgeError> {
        let candidate = self
            .candidates
            .get_mut(candidate_id)
            .ok_or_else(|| ForgeError::UnknownCandidate(candidate_id.to_owned()))?;
        candidate.interaction_checks = InteractionChecks::Satisfied;
        candidate.interaction_check_receipt_ref = Some(receipt_ref.to_owned());
        Ok(())
    }

    pub fn record_candidate_memory(
        &mut self,
        candidate_id: &str,
        content: &str,
    ) -> CandidateMemoryRecord {
        let record = CandidateMemoryRecord {
            record_id: format!("candidate-memory-{}", self.candidate_memory.len() + 1),
            candidate_id: candidate_id.to_owned(),
            content: content.to_owned(),
            tier: MemoryTier::CandidateQuarantine,
            canonical_supported: false,
        };
        self.candidate_memory.push(record.clone());
        record
    }

    pub fn promote_candidate_memory(&self, _record_id: &str) -> Result<(), ForgeError> {
        Err(ForgeError::CanonicalMemoryOwnerRequired)
    }

    pub fn candidate_memory(&self) -> &[CandidateMemoryRecord] {
        &self.candidate_memory
    }

    pub fn select_candidate(&mut self, candidate_id: &str) -> Result<(), ForgeError> {
        if self.selected_candidate.is_some() {
            return Err(ForgeError::CandidateAlreadySelected);
        }
        if !self.candidates.contains_key(candidate_id) {
            return Err(ForgeError::UnknownCandidate(candidate_id.to_owned()));
        }
        for (id, candidate) in &mut self.candidates {
            candidate.lifecycle = if id == candidate_id {
                CandidateLifecycle::Selected
            } else {
                CandidateLifecycle::Historical
            };
        }
        self.selected_candidate = Some(candidate_id.to_owned());
        Ok(())
    }

    pub fn publish_selected(
        &mut self,
        current_base_revision: &str,
        owner: &dyn ForgeNativeOwner,
    ) -> Result<PublicationReceipt, ForgeError> {
        if self.publication.is_some() {
            return Err(ForgeError::PublicationAlreadyCompleted);
        }
        let candidate_id = self
            .selected_candidate
            .clone()
            .ok_or(ForgeError::NoSelectedCandidate)?;
        let assessment = self.assess_candidate(&candidate_id, current_base_revision, owner);
        if assessment.disposition != CandidateDisposition::Ready {
            return Err(ForgeError::CandidateNotPublishable);
        }
        let authorization = assessment
            .authorization
            .ok_or(ForgeError::InvalidOwnerGrant)?;
        let candidate = self
            .candidates
            .get(&candidate_id)
            .ok_or_else(|| ForgeError::UnknownCandidate(candidate_id.clone()))?;
        let request = PublicationRequest {
            experiment_id: self.spec.experiment_id.clone(),
            candidate_id: candidate_id.clone(),
            workspace_id: candidate.workspace_id.clone(),
            provenance_ref: candidate.provenance_ref.clone(),
            authorization: authorization.clone(),
        };
        let receipt = owner.publish(&request).map_err(|decision| match decision {
            OwnerDecision::Unavailable => ForgeError::OwnerUnavailable,
            OwnerDecision::Rejected | OwnerDecision::Authorized => ForgeError::OwnerRejected,
        })?;
        if receipt.operation_identity_ref != authorization.operation_identity_ref
            || receipt.permit_ref != authorization.permit_ref
            || receipt.effect_accounting_ref != authorization.effect_accounting_ref
        {
            return Err(ForgeError::PublicationReceiptMismatch);
        }
        if let Some(candidate) = self.candidates.get_mut(&candidate_id) {
            candidate.lifecycle = CandidateLifecycle::Published;
        }
        self.publication = Some(receipt.clone());
        Ok(receipt)
    }

    pub fn record_cea_proposal(&mut self, proposal: CeaProposal) {
        let proposal_id = proposal.proposal_id.clone();
        self.causal_records
            .entry(proposal_id)
            .or_insert(CausalRecord {
                proposal,
                state: CausalState::HypothesisOnly,
                intervention_ref: None,
            });
    }

    pub fn record_paired_intervention(
        &mut self,
        proposal_id: &str,
        intervention_ref: &str,
        evidence: AblationEvidence,
    ) -> Result<(), ForgeError> {
        let record = self
            .causal_records
            .get_mut(proposal_id)
            .ok_or_else(|| ForgeError::UnknownCeaProposal(proposal_id.to_owned()))?;
        if !intervention_ref.is_empty()
            && Self::evaluate_ablation(evidence) == CausalComparison::Supported
        {
            record.state = CausalState::SupportedByPairedIntervention;
            record.intervention_ref = Some(intervention_ref.to_owned());
        }
        Ok(())
    }

    pub fn causal_state(&self, proposal_id: &str) -> Option<CausalState> {
        self.causal_records
            .get(proposal_id)
            .map(|record| record.state)
    }

    pub fn diagnosis(&self, proposal_id: &str) -> Option<&str> {
        self.causal_records.get(proposal_id).and_then(|record| {
            (record.state == CausalState::SupportedByPairedIntervention)
                .then_some(record.proposal.hypothesis.as_str())
        })
    }

    pub fn evaluate_ablation(evidence: AblationEvidence) -> CausalComparison {
        if !evidence.valid {
            return CausalComparison::InconclusiveInvalidAblation;
        }
        if !evidence.comparable {
            return CausalComparison::InconclusiveNonComparable;
        }
        if evidence.control_environment != evidence.treatment_environment {
            return CausalComparison::BlockedEnvironmentConfound;
        }
        if evidence.effect_observed {
            CausalComparison::Supported
        } else {
            CausalComparison::Refuted
        }
    }

    pub fn schedule_trial(&mut self, plan: TrialPlan) -> Result<(), ForgeError> {
        if self.trial_indexes.contains_key(&plan.trial_id) {
            return Err(ForgeError::DuplicateTrial(plan.trial_id));
        }
        let index = self.trials.len();
        self.trial_indexes.insert(plan.trial_id.clone(), index);
        self.trials.push(TrialRecord {
            trial_id: plan.trial_id,
            candidate_id: plan.candidate_id,
            requested_test_access: plan.requested_test_access,
            outcome: None,
            cost: None,
            environment_ref: None,
            contaminated: false,
            usable: true,
            public_tests_remain_valid: true,
        });
        Ok(())
    }

    pub fn complete_trial(
        &mut self,
        trial_id: &str,
        completion: TrialCompletion,
    ) -> Result<(), ForgeError> {
        let index = *self
            .trial_indexes
            .get(trial_id)
            .ok_or_else(|| ForgeError::UnknownTrial(trial_id.to_owned()))?;
        let trial = &mut self.trials[index];
        if trial.outcome.is_some() {
            return Err(ForgeError::TrialAlreadyCompleted(trial_id.to_owned()));
        }
        trial.cost = Some(completion.cost);
        trial.environment_ref = Some(completion.environment_ref);
        if trial.requested_test_access == TestAccess::WithheldDiscriminator {
            trial.outcome = Some(TrialOutcome::WithheldAccessDenied);
            trial.contaminated = true;
            trial.usable = false;
        } else {
            trial.outcome = Some(completion.outcome);
        }
        Ok(())
    }

    pub fn trials(&self) -> &[TrialRecord] {
        &self.trials
    }
}

fn scope_is_allowed(scope: &str, allowed_scopes: &BTreeSet<String>) -> bool {
    allowed_scopes.iter().any(|allowed| {
        scope == allowed
            || scope
                .strip_prefix(allowed)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}
