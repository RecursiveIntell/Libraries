//! Read-only operator evidence and agency projections.
//!
//! This module projects decisions made by canonical owners. It does not own
//! evidence, approvals, applicability, purpose policy, or durable run state,
//! and it exposes no mutation path except routing an already owner-approved
//! command through an explicitly injected command port.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Explicit state carried by an operator projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectionState {
    /// A source fact or decision was observed.
    Observed,
    /// Runtime execution reached its terminal state.
    CompletedExecution,
    /// The referenced evidence supports semantic closure.
    Supported,
    /// The referenced evidence does not support semantic closure.
    Unsupported,
    /// An owner decision prevents the requested result.
    Blocked,
    /// Data or a feature was intentionally withheld.
    Withheld,
    /// A canonical owner could not classify the result.
    Unknown,
}

/// Read-only durable run state supplied by its canonical owner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableRunState {
    /// Stable run reference.
    pub run_ref: String,
    /// Latest sequence durably recorded by the run owner.
    pub durable_sequence: u64,
    /// Whether the durable run is terminal.
    pub terminal: bool,
    /// Digest of the durable owner state.
    pub state_digest: String,
}

/// One best-effort observation sink event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    /// Monotonic sink sequence.
    pub sequence: u64,
    /// Reference to the event, not its payload.
    pub event_ref: String,
}

impl Observation {
    /// Constructs an observation.
    pub fn new(sequence: u64, event_ref: impl Into<String>) -> Self {
        Self {
            sequence,
            event_ref: event_ref.into(),
        }
    }
}

/// Exact inclusive interval missing from a best-effort observation stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceGap {
    /// First missing sequence.
    pub missing_from: u64,
    /// Last missing sequence.
    pub missing_through: u64,
    /// First observed sequence after the gap.
    pub next_observed: u64,
}

/// Read-only view over sink observations and canonical durable state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationProjection {
    /// Projection state.
    pub state: ProjectionState,
    /// Observed event references in sequence order.
    pub observed_event_refs: Vec<String>,
    /// Exact detected sequence gaps.
    pub gaps: Vec<SequenceGap>,
    /// Unmodified snapshot from the durable run owner.
    pub canonical_run_state: DurableRunState,
}

/// Builds a read-only observation projection without changing durable state.
pub fn project_observations(
    canonical_run_state: &DurableRunState,
    observations: &[Observation],
) -> ObservationProjection {
    let mut ordered = observations.to_vec();
    ordered.sort_by_key(|observation| observation.sequence);
    ordered.dedup_by_key(|observation| observation.sequence);

    let gaps = ordered
        .windows(2)
        .filter_map(|pair| {
            let previous = pair[0].sequence;
            let next = pair[1].sequence;
            (next > previous.saturating_add(1)).then(|| SequenceGap {
                missing_from: previous.saturating_add(1),
                missing_through: next - 1,
                next_observed: next,
            })
        })
        .collect();

    ObservationProjection {
        state: ProjectionState::Observed,
        observed_event_refs: ordered
            .into_iter()
            .map(|observation| observation.event_ref)
            .collect(),
        gaps,
        canonical_run_state: canonical_run_state.clone(),
    }
}

/// Inputs required to project runtime and semantic terminal status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalInput {
    /// Stable run reference.
    pub run_ref: String,
    /// Owner-issued runtime terminal receipt reference.
    pub terminal_receipt_ref: String,
    /// Whether runtime execution completed.
    pub execution_completed: bool,
    /// Evidence references required for semantic closure.
    pub required_evidence_refs: Vec<String>,
    /// Required evidence references confirmed as supporting by the owner.
    pub supported_evidence_refs: Vec<String>,
}

/// Split runtime-execution and semantic-closure status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalProjection {
    /// Stable run reference.
    pub run_ref: String,
    /// Runtime terminal receipt reference.
    pub terminal_receipt_ref: String,
    /// Runtime status, separate from semantic status.
    pub execution_state: ProjectionState,
    /// Semantic support status, separate from runtime completion.
    pub semantic_state: ProjectionState,
    /// Required evidence references not supported by the owner input.
    pub missing_evidence_refs: Vec<String>,
}

/// Projects terminal execution separately from semantic evidence closure.
pub fn project_terminal(input: TerminalInput) -> TerminalProjection {
    let supported: BTreeSet<_> = input.supported_evidence_refs.iter().collect();
    let mut missing_evidence_refs: Vec<_> = input
        .required_evidence_refs
        .iter()
        .filter(|reference| !supported.contains(reference))
        .cloned()
        .collect();
    missing_evidence_refs.sort();
    missing_evidence_refs.dedup();

    let execution_state = if input.execution_completed {
        ProjectionState::CompletedExecution
    } else {
        ProjectionState::Observed
    };
    let semantic_state = if !input.execution_completed {
        ProjectionState::Blocked
    } else if missing_evidence_refs.is_empty() {
        ProjectionState::Supported
    } else {
        ProjectionState::Unsupported
    };

    TerminalProjection {
        run_ref: input.run_ref,
        terminal_receipt_ref: input.terminal_receipt_ref,
        execution_state,
        semantic_state,
        missing_evidence_refs,
    }
}

/// Coverage details supplied by the applicability owner.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageProjection {
    /// Scope references covered by the invalidation analysis.
    pub covered_scope_refs: Vec<String>,
    /// Scope references not covered by the invalidation analysis.
    pub uncovered_scope_refs: Vec<String>,
}

/// Canonical source-invalidation decision returned by its owner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceInvalidationDecision {
    /// Reference to the source change.
    pub change_ref: String,
    /// Coverage of the owner decision.
    pub coverage: CoverageProjection,
    /// Reference-only dependency chain.
    pub dependency_chain_refs: Vec<String>,
    /// Result references affected by the source change.
    pub affected_result_refs: Vec<String>,
}

/// Port to the canonical applicability/source-invalidation owner.
pub trait SourceInvalidationOwnerPort {
    /// Returns the current owner decision for a source reference.
    fn source_invalidation(&self, source_ref: &str) -> Option<SourceInvalidationDecision>;
}

/// Public source-change projection containing references, never transcripts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceInvalidationProjection {
    /// Whether the owner supplied a current decision.
    pub state: ProjectionState,
    /// Source reference queried.
    pub source_ref: String,
    /// Owner-issued source-change reference.
    pub change_ref: Option<String>,
    /// Explicit coverage of the owner decision.
    pub coverage: CoverageProjection,
    /// Reference-only dependency chain.
    pub dependency_chain_refs: Vec<String>,
    /// Result references affected by the change.
    pub affected_result_refs: Vec<String>,
}

/// Projects a source invalidation decision without reproducing source content.
pub fn project_source_invalidation(
    source_ref: &str,
    owner: &dyn SourceInvalidationOwnerPort,
) -> SourceInvalidationProjection {
    match owner.source_invalidation(source_ref) {
        Some(decision) => SourceInvalidationProjection {
            state: ProjectionState::Observed,
            source_ref: source_ref.to_owned(),
            change_ref: Some(decision.change_ref),
            coverage: decision.coverage,
            dependency_chain_refs: decision.dependency_chain_refs,
            affected_result_refs: decision.affected_result_refs,
        },
        None => SourceInvalidationProjection {
            state: ProjectionState::Unknown,
            source_ref: source_ref.to_owned(),
            change_ref: None,
            coverage: CoverageProjection::default(),
            dependency_chain_refs: Vec::new(),
            affected_result_refs: Vec::new(),
        },
    }
}

/// Evidence input retained only long enough to apply redaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceReference {
    /// Stable evidence reference.
    pub evidence_ref: String,
    /// Whether public export must redact its bytes.
    pub sensitive: bool,
    /// Evidence bytes; never copied into the export projection.
    pub bytes: Vec<u8>,
}

impl EvidenceReference {
    /// Constructs a sensitive evidence input.
    pub fn sensitive(evidence_ref: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            evidence_ref: evidence_ref.into(),
            sensitive: true,
            bytes,
        }
    }
}

/// Inputs for a clean-room regression export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegressionExportInput {
    /// Failure reference being reproduced.
    pub failure_ref: String,
    /// Clean-room fixture and replay-scope references.
    pub replay_scope_refs: Vec<String>,
    /// Evidence inputs subject to export redaction.
    pub evidence: Vec<EvidenceReference>,
    /// Data references omitted from the clean-room export.
    pub missing_data_refs: Vec<String>,
}

/// Redacted evidence reference safe for a public projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedEvidenceReference {
    /// Stable evidence reference.
    pub evidence_ref: String,
    /// Whether evidence bytes were removed.
    pub redacted: bool,
}

/// Clean-room, reference-only regression export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegressionExportProjection {
    /// Withheld when any source evidence is sensitive.
    pub state: ProjectionState,
    /// Failure reference being reproduced.
    pub failure_ref: String,
    /// Retained clean-room replay scope.
    pub clean_room_replay_scope_refs: Vec<String>,
    /// Reference-only evidence inventory.
    pub evidence_refs: Vec<RedactedEvidenceReference>,
    /// Explicit missing-data limitations.
    pub limitations: Vec<String>,
}

/// Applies the public redaction policy to a regression export.
pub fn project_regression_export(input: RegressionExportInput) -> RegressionExportProjection {
    let has_sensitive = input.evidence.iter().any(|evidence| evidence.sensitive);
    RegressionExportProjection {
        state: if has_sensitive {
            ProjectionState::Withheld
        } else {
            ProjectionState::Observed
        },
        failure_ref: input.failure_ref,
        clean_room_replay_scope_refs: input.replay_scope_refs,
        evidence_refs: input
            .evidence
            .into_iter()
            .map(|evidence| RedactedEvidenceReference {
                evidence_ref: evidence.evidence_ref,
                redacted: evidence.sensitive,
            })
            .collect(),
        limitations: input
            .missing_data_refs
            .into_iter()
            .map(|reference| format!("missing-data:{reference}"))
            .collect(),
    }
}

/// Typed owner approval decision; there is intentionally no boolean shortcut.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "kebab-case")]
pub enum ApprovalDecision {
    /// An owner-issued approval is still required.
    Required { requirement_ref: String },
    /// An owner issued approval for the action.
    Granted { approval_ref: String },
    /// The owner denied the action.
    Denied { reason_ref: String },
    /// The owner could not provide a current decision.
    #[default]
    Unknown,
}

/// Port to the canonical approval owner.
pub trait ApprovalOwnerPort {
    /// Returns the current typed approval decision for an action identity.
    fn approval_for(&self, action_identity: &str) -> ApprovalDecision;
}

/// Explicit operator request classes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "request", rename_all = "kebab-case")]
pub enum OperatorRequest {
    /// Mutation requested directly through the observer surface.
    ObserverMutation { command_ref: String },
    /// Permit requested directly from the projection.
    DirectPermit { action_identity: String },
    /// Command routed through the explicit authorized command port.
    AuthorizedCommand {
        /// Command reference, not command payload.
        command_ref: String,
        /// Canonical action identity checked with the approval owner.
        action_identity: String,
    },
}

/// The only command-capable port accepted by this projection module.
pub trait AuthorizedCommandPort {
    /// Executes a command already authorized by the canonical approval owner.
    fn execute_authorized(&mut self, command_ref: &str, approval_ref: &str);
}

/// Read-only result of routing an operator request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorRequestProjection {
    /// Routing result.
    pub state: ProjectionState,
    /// Request reference visible to the operator.
    pub request_ref: String,
    /// Owner approval decision retained in full.
    pub approval: ApprovalDecision,
    /// Optional authorized command owner receipt reference.
    pub command_receipt_ref: Option<String>,
}

/// Denies observer authority and routes only explicitly owner-approved commands.
pub fn route_operator_request(
    request: OperatorRequest,
    approval_owner: &dyn ApprovalOwnerPort,
    command_port: &mut dyn AuthorizedCommandPort,
) -> OperatorRequestProjection {
    match request {
        OperatorRequest::ObserverMutation { command_ref } => OperatorRequestProjection {
            state: ProjectionState::Blocked,
            request_ref: command_ref,
            approval: ApprovalDecision::Denied {
                reason_ref: "observer-cannot-mutate".into(),
            },
            command_receipt_ref: None,
        },
        OperatorRequest::DirectPermit { action_identity } => OperatorRequestProjection {
            state: ProjectionState::Blocked,
            request_ref: action_identity,
            approval: ApprovalDecision::Denied {
                reason_ref: "projection-cannot-grant-permit".into(),
            },
            command_receipt_ref: None,
        },
        OperatorRequest::AuthorizedCommand {
            command_ref,
            action_identity,
        } => {
            let approval = approval_owner.approval_for(&action_identity);
            if let ApprovalDecision::Granted { approval_ref } = &approval {
                command_port.execute_authorized(&command_ref, approval_ref);
                OperatorRequestProjection {
                    state: ProjectionState::Observed,
                    request_ref: command_ref,
                    approval,
                    command_receipt_ref: None,
                }
            } else {
                OperatorRequestProjection {
                    state: match approval {
                        ApprovalDecision::Unknown => ProjectionState::Unknown,
                        _ => ProjectionState::Blocked,
                    },
                    request_ref: command_ref,
                    approval,
                    command_receipt_ref: None,
                }
            }
        }
    }
}

/// A recommendation's stance toward an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecommendationStance {
    /// Recommends the action.
    Agree,
    /// Disagrees with the action.
    Dissent,
    /// Abstains from a recommendation.
    Abstain,
}

/// Reference-only recommendation input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recommendation {
    /// Stable recommendation reference.
    pub recommendation_ref: String,
    /// Canonical action identity supplied by the recommendation owner.
    pub action_identity: String,
    /// Shared provenance group reference.
    pub provenance_group_ref: String,
    /// Shared dependence group reference.
    pub dependence_group_ref: String,
    /// Recommendation stance.
    pub stance: RecommendationStance,
}

/// Dependency-aware recommendation projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecommendationProjection {
    /// Blocked unless the approval owner has granted approval.
    pub state: ProjectionState,
    /// Canonical action identity.
    pub action_identity: String,
    /// Raw number of agreeing recommendations.
    pub agreeing_recommendation_count: usize,
    /// Number of unique dependence groups among agreements.
    pub independent_agreement_count: usize,
    /// Deduplicated provenance groups among agreements.
    pub provenance_group_refs: Vec<String>,
    /// Deduplicated dependence groups among agreements.
    pub dependence_group_refs: Vec<String>,
    /// Dissenting recommendation references.
    pub dissent_refs: Vec<String>,
    /// Abstaining recommendation references.
    pub abstention_refs: Vec<String>,
    /// Typed decision from the approval owner; agreement never creates it.
    pub approval: ApprovalDecision,
}

/// Collapses common provenance/dependence without multiplying authority.
pub fn aggregate_recommendations(
    action_identity: &str,
    recommendations: &[Recommendation],
    approval_owner: &dyn ApprovalOwnerPort,
) -> RecommendationProjection {
    let matching: Vec<_> = recommendations
        .iter()
        .filter(|recommendation| recommendation.action_identity == action_identity)
        .collect();
    let agreeing: Vec<_> = matching
        .iter()
        .copied()
        .filter(|recommendation| recommendation.stance == RecommendationStance::Agree)
        .collect();
    let provenance_group_refs: BTreeSet<_> = agreeing
        .iter()
        .map(|recommendation| recommendation.provenance_group_ref.clone())
        .collect();
    let dependence_group_refs: BTreeSet<_> = agreeing
        .iter()
        .map(|recommendation| recommendation.dependence_group_ref.clone())
        .collect();
    let dissent_refs = matching
        .iter()
        .copied()
        .filter(|recommendation| recommendation.stance == RecommendationStance::Dissent)
        .map(|recommendation| recommendation.recommendation_ref.clone())
        .collect();
    let abstention_refs = matching
        .iter()
        .copied()
        .filter(|recommendation| recommendation.stance == RecommendationStance::Abstain)
        .map(|recommendation| recommendation.recommendation_ref.clone())
        .collect();
    let approval = approval_owner.approval_for(action_identity);
    let state = match approval {
        ApprovalDecision::Granted { .. } => ProjectionState::Observed,
        ApprovalDecision::Unknown => ProjectionState::Unknown,
        _ => ProjectionState::Blocked,
    };

    RecommendationProjection {
        state,
        action_identity: action_identity.to_owned(),
        agreeing_recommendation_count: agreeing.len(),
        independent_agreement_count: dependence_group_refs.len(),
        provenance_group_refs: provenance_group_refs.into_iter().collect(),
        dependence_group_refs: dependence_group_refs.into_iter().collect(),
        dissent_refs,
        abstention_refs,
        approval,
    }
}

/// One action nudge observed in an interaction round.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionNudge {
    /// Interaction round.
    pub round: u64,
    /// Nudge text supplied only to the canonical classifier.
    pub text: String,
}

impl ActionNudge {
    /// Constructs an action nudge.
    pub fn new(round: u64, text: impl Into<String>) -> Self {
        Self {
            round,
            text: text.into(),
        }
    }
}

/// Canonical owner decision for an action nudge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "classification", rename_all = "kebab-case")]
pub enum ActionIdentityDecision {
    /// Nudge maps to a canonical action identity.
    Known {
        /// Canonical action identity.
        action_identity: String,
        /// Owner-issued classification reference.
        classification_ref: String,
    },
    /// The owner cannot classify the action.
    Unknown,
}

/// Port to the canonical action-identity classifier.
pub trait ActionIdentityOwnerPort {
    /// Classifies one action nudge.
    fn classify_action(&self, nudge: &ActionNudge) -> ActionIdentityDecision;
}

/// Bound on distinct interaction-policy entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractionPolicyLimits {
    /// Maximum projected action entries.
    pub max_policy_entries: usize,
}

/// One canonical action entry across interaction rounds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionInteractionProjection {
    /// Known canonical identity, or `None` for explicit unknown classification.
    pub action_identity: Option<String>,
    /// Owner-issued classification references.
    pub classification_refs: Vec<String>,
    /// Rounds in which this action was nudged.
    pub rounds: Vec<u64>,
    /// Classification status.
    pub state: ProjectionState,
}

/// Bounded interaction-policy projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionPolicyProjection {
    /// Unknown if any nudge was unclassifiable, otherwise observed.
    pub state: ProjectionState,
    /// Bounded canonical action entries.
    pub actions: Vec<ActionInteractionProjection>,
    /// Number of retained policy entries.
    pub interaction_policy_count: usize,
    /// Number of entries omitted to satisfy the bound.
    pub omitted_policy_entry_count: usize,
}

/// Groups paraphrases using only canonical owner classifications.
pub fn project_action_nudges(
    nudges: &[ActionNudge],
    identity_owner: &dyn ActionIdentityOwnerPort,
    limits: InteractionPolicyLimits,
) -> ActionPolicyProjection {
    let mut known: BTreeMap<String, (BTreeSet<String>, BTreeSet<u64>)> = BTreeMap::new();
    let mut unknown_rounds = BTreeSet::new();

    for nudge in nudges {
        match identity_owner.classify_action(nudge) {
            ActionIdentityDecision::Known {
                action_identity,
                classification_ref,
            } => {
                let (classifications, rounds) = known.entry(action_identity).or_default();
                classifications.insert(classification_ref);
                rounds.insert(nudge.round);
            }
            ActionIdentityDecision::Unknown => {
                unknown_rounds.insert(nudge.round);
            }
        }
    }

    let has_unknown = !unknown_rounds.is_empty();
    let mut actions: Vec<_> = known
        .into_iter()
        .map(
            |(action_identity, (classification_refs, rounds))| ActionInteractionProjection {
                action_identity: Some(action_identity),
                classification_refs: classification_refs.into_iter().collect(),
                rounds: rounds.into_iter().collect(),
                state: ProjectionState::Observed,
            },
        )
        .collect();
    if has_unknown {
        actions.push(ActionInteractionProjection {
            action_identity: None,
            classification_refs: Vec::new(),
            rounds: unknown_rounds.into_iter().collect(),
            state: ProjectionState::Unknown,
        });
    }

    let total = actions.len();
    actions.truncate(limits.max_policy_entries);
    ActionPolicyProjection {
        state: if has_unknown {
            ProjectionState::Unknown
        } else {
            ProjectionState::Observed
        },
        interaction_policy_count: actions.len(),
        omitted_policy_entry_count: total.saturating_sub(actions.len()),
        actions,
    }
}

/// Reference to a sensitive personalization feature and its evidence receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureReference {
    /// Stable feature reference.
    pub feature_ref: String,
    /// Sensitive receipt reference; receipt text is never projected.
    pub sensitive_receipt_ref: String,
}

/// Sensitive feature input. Receipt text exists only at the owner boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SensitiveFeatureInput {
    /// Reference-only feature identity.
    pub feature: FeatureReference,
    /// Sensitive receipt text, deliberately absent from output types.
    pub sensitive_receipt_text: String,
}

/// Purpose decision made by the canonical restriction owner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "kebab-case")]
pub enum PurposeDecision {
    /// Feature use is allowed for the requested purpose.
    Allowed { disclosure_ref: String },
    /// Feature use is withheld for the requested purpose.
    Withheld { disclosure_ref: String },
    /// The owner could not classify the purpose.
    Unknown,
}

/// Port to the canonical purpose-restriction owner.
pub trait PurposeRestrictionOwnerPort {
    /// Decides whether a feature may be used for a purpose.
    fn decide_purpose(&self, feature_ref: &str, purpose: &str) -> PurposeDecision;
}

/// Public, reference-only purpose-gated feature projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurposeGatedFeatureProjection {
    /// Allowed, withheld, or unknown state.
    pub state: ProjectionState,
    /// Stable feature reference.
    pub feature_ref: String,
    /// Sensitive evidence receipt reference, never receipt text.
    pub sensitive_receipt_ref: String,
    /// Owner disclosure reference preserving traceability.
    pub disclosure_ref: Option<String>,
}

/// Projects sensitive feature references under canonical purpose restrictions.
pub fn project_sensitive_features(
    purpose: &str,
    features: &[SensitiveFeatureInput],
    purpose_owner: &dyn PurposeRestrictionOwnerPort,
) -> Vec<PurposeGatedFeatureProjection> {
    features
        .iter()
        .map(|input| {
            let (state, disclosure_ref) =
                match purpose_owner.decide_purpose(&input.feature.feature_ref, purpose) {
                    PurposeDecision::Allowed { disclosure_ref } => {
                        (ProjectionState::Observed, Some(disclosure_ref))
                    }
                    PurposeDecision::Withheld { disclosure_ref } => {
                        (ProjectionState::Withheld, Some(disclosure_ref))
                    }
                    PurposeDecision::Unknown => (ProjectionState::Unknown, None),
                };
            PurposeGatedFeatureProjection {
                state,
                feature_ref: input.feature.feature_ref.clone(),
                sensitive_receipt_ref: input.feature.sensitive_receipt_ref.clone(),
                disclosure_ref,
            }
        })
        .collect()
}
