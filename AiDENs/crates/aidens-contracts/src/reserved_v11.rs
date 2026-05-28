//! Reserved v11B/v11C-adjacent display artifacts.
//!
//! These families remain draft/advisory unless activated by explicit future evidence and owner-crate admission.

use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum V11ActivationLevelV1 {
    #[default]
    ReservedDraft,
    AdvisoryOnly,
    Quarantined,
    ActiveByFutureOwner,
}

impl V11ActivationLevelV1 {
    pub fn is_active(self) -> bool {
        matches!(self, Self::ActiveByFutureOwner)
    }

    pub fn allows_truth_promotion(self) -> bool {
        self.is_active()
    }
}

fn v11_activation_reserved_draft() -> V11ActivationLevelV1 {
    V11ActivationLevelV1::ReservedDraft
}

fn bool_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ExternalAdmissionDispositionV1 {
    Quarantined,
    Rejected,
    AdvisoryOnly,
    AdmittedByCanonicalOwner,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExternalArtifactAdmissionDecisionV1 {
    pub decision_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub source_artifact_id: ArtifactId,
    pub disposition: ExternalAdmissionDispositionV1,
    pub activation_level: V11ActivationLevelV1,
    pub truth_promotion_allowed: bool,
    pub proof_waiver_allowed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub decided_at: DateTime<Utc>,
}

impl ExternalArtifactAdmissionDecisionV1 {
    pub fn default_quarantine(source_artifact_id: ArtifactId) -> Self {
        Self {
            decision_id: generated_artifact_id_from_material(
                "external-admission-decision",
                source_artifact_id.as_str(),
            ),
            kind: ArtifactKindV1::AdmissionDecision,
            source_artifact_id,
            disposition: ExternalAdmissionDispositionV1::Quarantined,
            activation_level: V11ActivationLevelV1::Quarantined,
            truth_promotion_allowed: false,
            proof_waiver_allowed: false,
            reason_codes: vec![
                "external-admission-defaults-to-quarantine".into(),
                "canonical-owner-admission-required".into(),
            ],
            canonical_backpointers: canonical_owner_backpointer(
                "remote-oracle-admission",
                "LocalAdmissionRecommendationV1",
                "canonical-external-admission-owner",
            ),
            decided_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisorySystemKindV1 {
    LearnedRanker,
    HeuristicAdvisor,
    FixtureBackedEvaluator,
    HumanDisplay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AdvisorySystemPromotionGuardV1 {
    pub guard_id: ArtifactId,
    pub system_kind: AdvisorySystemKindV1,
    pub activation_level: V11ActivationLevelV1,
    pub truth_promotion_allowed: bool,
    pub proof_waiver_allowed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub evaluated_at: DateTime<Utc>,
}

impl AdvisorySystemPromotionGuardV1 {
    pub fn advisory(system_kind: AdvisorySystemKindV1) -> Self {
        let material = format!("{system_kind:?}:advisory");
        Self {
            guard_id: generated_artifact_id_from_material("advisory-promotion-guard", &material),
            system_kind,
            activation_level: V11ActivationLevelV1::AdvisoryOnly,
            truth_promotion_allowed: false,
            proof_waiver_allowed: false,
            reason_codes: vec![
                "advisory-system-cannot-promote-truth".into(),
                "advisory-system-cannot-waive-proof".into(),
            ],
            evaluated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RegionGraphKindV1 {
    Storage,
    Retrieval,
    Inference,
    Repair,
    Control,
}

impl RegionGraphKindV1 {
    pub fn can_execute_kernel(self) -> bool {
        matches!(self, Self::Inference | Self::Repair)
    }
}

impl fmt::Display for RegionGraphKindV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Storage => "storage",
            Self::Retrieval => "retrieval",
            Self::Inference => "inference",
            Self::Repair => "repair",
            Self::Control => "control",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RegionNodeKindV1 {
    Claim,
    Evidence,
    Factor,
    Boundary,
    Nuisance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RegionNodeV1 {
    pub node_id: ArtifactId,
    pub node_kind: RegionNodeKindV1,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_artifact_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl RegionNodeV1 {
    pub fn new(
        node_kind: RegionNodeKindV1,
        label: impl Into<String>,
        source_artifact_id: Option<ArtifactId>,
    ) -> Self {
        Self {
            node_id: display_only_unstable_id("region-node"),
            node_kind,
            label: label.into(),
            source_artifact_id,
            reason_codes: vec!["region-node-defined".into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RegionEdgeV1 {
    pub edge_id: ArtifactId,
    pub from_node_id: ArtifactId,
    pub to_node_id: ArtifactId,
    pub relation: String,
}

impl RegionEdgeV1 {
    pub fn new(
        from_node_id: ArtifactId,
        to_node_id: ArtifactId,
        relation: impl Into<String>,
    ) -> Self {
        Self {
            edge_id: display_only_unstable_id("region-edge"),
            from_node_id,
            to_node_id,
            relation: relation.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RegionHyperedgeV1 {
    pub hyperedge_id: ArtifactId,
    pub node_ids: Vec<ArtifactId>,
    pub relation: String,
}

impl RegionHyperedgeV1 {
    pub fn new(mut node_ids: Vec<ArtifactId>, relation: impl Into<String>) -> Self {
        node_ids.sort();
        node_ids.dedup();
        Self {
            hyperedge_id: display_only_unstable_id("region-hyperedge"),
            node_ids,
            relation: relation.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RegionFactorV1 {
    pub factor_id: ArtifactId,
    pub scope_node_ids: Vec<ArtifactId>,
    pub factor_kind: String,
    pub weight: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl RegionFactorV1 {
    pub fn new(
        mut scope_node_ids: Vec<ArtifactId>,
        factor_kind: impl Into<String>,
        weight: f64,
    ) -> Self {
        scope_node_ids.sort();
        scope_node_ids.dedup();
        Self {
            factor_id: display_only_unstable_id("region-factor"),
            scope_node_ids,
            factor_kind: factor_kind.into(),
            weight,
            reason_codes: vec!["bounded-region-factor".into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RegionContractV1 {
    pub contract_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub graph_id: ArtifactId,
    pub graph_kind: RegionGraphKindV1,
    pub region_id: ArtifactId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_region_id: Option<StackRegionId>,
    pub node_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boundary_node_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub factor_ids: Vec<ArtifactId>,
    pub max_nodes: u32,
    pub local_repair_allowed: bool,
    #[serde(default = "v11_activation_reserved_draft")]
    pub activation_level: V11ActivationLevelV1,
    #[serde(default = "bool_true")]
    pub advisory_only: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub created_at: DateTime<Utc>,
}

impl RegionContractV1 {
    pub fn new(
        graph_id: ArtifactId,
        graph_kind: RegionGraphKindV1,
        region_id: ArtifactId,
        mut node_ids: Vec<ArtifactId>,
        mut boundary_node_ids: Vec<ArtifactId>,
        mut factor_ids: Vec<ArtifactId>,
        max_nodes: u32,
    ) -> Self {
        node_ids.sort();
        node_ids.dedup();
        boundary_node_ids.sort();
        boundary_node_ids.dedup();
        factor_ids.sort();
        factor_ids.dedup();
        let local_repair_allowed =
            graph_kind == RegionGraphKindV1::Repair || graph_kind == RegionGraphKindV1::Inference;
        Self {
            contract_id: display_only_unstable_id("region-contract"),
            kind: ArtifactKindV1::RegionContract,
            graph_id,
            graph_kind,
            region_id,
            canonical_region_id: None,
            node_ids,
            boundary_node_ids,
            factor_ids,
            max_nodes,
            local_repair_allowed,
            activation_level: V11ActivationLevelV1::ReservedDraft,
            advisory_only: true,
            reason_codes: vec![
                "region-boundary-contract".into(),
                "v11b-region-contract-reserved-draft".into(),
            ],
            canonical_backpointers: canonical_owner_backpointer(
                "constraint-compiler",
                "CompiledRegion",
                "canonical-region-contract-owner",
            ),
            created_at: Utc::now(),
        }
    }

    pub fn is_bounded(&self) -> bool {
        self.node_ids.len() <= self.max_nodes as usize
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RegionBoundaryMessageV1 {
    pub message_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub source_region_id: ArtifactId,
    pub destination_region_id: ArtifactId,
    pub artifact_family: String,
    pub payload_ref: ArtifactId,
    pub payload_digest: DisplayDigestV1,
    pub boundary_policy: String,
    pub budget_impact: String,
    #[serde(default = "v11_activation_reserved_draft")]
    pub activation_level: V11ActivationLevelV1,
    #[serde(default = "bool_true")]
    pub advisory_only: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub emitted_at: DateTime<Utc>,
}

impl RegionBoundaryMessageV1 {
    pub fn seed(
        source_region_id: ArtifactId,
        destination_region_id: ArtifactId,
        artifact_family: impl Into<String>,
        payload_ref: ArtifactId,
        payload_digest: DisplayDigestV1,
    ) -> Self {
        let artifact_family = artifact_family.into();
        let material = format!(
            "{}|{}|{}|{}|{}",
            source_region_id.0,
            destination_region_id.0,
            artifact_family,
            payload_ref.0,
            payload_digest.digest
        );
        Self {
            message_id: generated_artifact_id_from_material("boundary-message", &material),
            kind: ArtifactKindV1::BoundaryMessage,
            source_region_id,
            destination_region_id,
            artifact_family,
            payload_ref,
            payload_digest,
            boundary_policy: "receipt-required-advisory-seed".into(),
            budget_impact: "declared-no-runtime-debit".into(),
            activation_level: V11ActivationLevelV1::AdvisoryOnly,
            advisory_only: true,
            reason_codes: vec![
                "v11b-boundary-message-executable-seed".into(),
                "boundary-message-advisory-only".into(),
            ],
            canonical_backpointers: canonical_owner_backpointer(
                "kernel-execution",
                "BoundaryMessage",
                "canonical-region-boundary-owner",
            ),
            emitted_at: Utc::now(),
        }
    }

    pub fn can_cross_runtime_boundary(&self) -> bool {
        self.activation_level.is_active() && !self.advisory_only
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RegionBoundaryReceiptV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub message_id: ArtifactId,
    #[serde(default = "region_boundary_receipt_disposition_accepted")]
    pub disposition: RegionBoundaryReceiptDispositionV1,
    pub accepted: bool,
    pub replay_required: bool,
    pub canonicalization_profile: String,
    #[serde(default = "v11_activation_reserved_draft")]
    pub activation_level: V11ActivationLevelV1,
    #[serde(default = "bool_true")]
    pub advisory_only: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quarantine_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RegionBoundaryReceiptDispositionV1 {
    Accepted,
    Rejected,
    Quarantined,
}

fn region_boundary_receipt_disposition_accepted() -> RegionBoundaryReceiptDispositionV1 {
    RegionBoundaryReceiptDispositionV1::Accepted
}

impl RegionBoundaryReceiptV1 {
    pub fn seed(message: &RegionBoundaryMessageV1, accepted: bool) -> Self {
        let material = format!("{}|accepted={accepted}", message.message_id.0);
        Self {
            receipt_id: generated_artifact_id_from_material("boundary-receipt", &material),
            kind: ArtifactKindV1::BoundaryReceipt,
            message_id: message.message_id.clone(),
            disposition: if accepted {
                RegionBoundaryReceiptDispositionV1::Accepted
            } else {
                RegionBoundaryReceiptDispositionV1::Rejected
            },
            accepted,
            replay_required: true,
            canonicalization_profile: "stack-ids-json-c14n-v1".into(),
            activation_level: V11ActivationLevelV1::AdvisoryOnly,
            advisory_only: true,
            reason_codes: if accepted {
                vec![
                    "v11b-boundary-receipt-executable-seed".into(),
                    "boundary-message-admitted-advisory-only".into(),
                ]
            } else {
                vec![
                    "v11b-boundary-receipt-executable-seed".into(),
                    "boundary-message-rejected-advisory-only".into(),
                ]
            },
            quarantine_reason: None,
            canonical_backpointers: canonical_owner_backpointer(
                "kernel-execution",
                "BoundaryReceipt",
                "canonical-region-boundary-receipt-owner",
            ),
            recorded_at: Utc::now(),
        }
    }

    pub fn quarantined(message: &RegionBoundaryMessageV1, reason: impl Into<String>) -> Self {
        let reason = reason.into();
        let material = format!("{}|quarantined|{reason}", message.message_id.0);
        Self {
            receipt_id: generated_artifact_id_from_material("boundary-receipt", &material),
            kind: ArtifactKindV1::BoundaryReceipt,
            message_id: message.message_id.clone(),
            disposition: RegionBoundaryReceiptDispositionV1::Quarantined,
            accepted: false,
            replay_required: true,
            canonicalization_profile: "stack-ids-json-c14n-v1".into(),
            activation_level: V11ActivationLevelV1::Quarantined,
            advisory_only: true,
            reason_codes: vec![
                "v11b-boundary-receipt-executable-seed".into(),
                "boundary-message-quarantined-advisory-only".into(),
            ],
            quarantine_reason: Some(reason),
            canonical_backpointers: canonical_owner_backpointer(
                "kernel-execution",
                "BoundaryReceipt",
                "canonical-region-boundary-receipt-owner",
            ),
            recorded_at: Utc::now(),
        }
    }

    pub fn can_admit_runtime_payload(&self) -> bool {
        self.disposition == RegionBoundaryReceiptDispositionV1::Accepted
            && self.accepted
            && self.activation_level.is_active()
            && !self.advisory_only
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CompiledRegionGraphV1 {
    pub graph_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub graph_kind: RegionGraphKindV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_graph_kind: Option<RegionGraphKindV1>,
    pub max_region_size: u32,
    pub right_graph_law_satisfied: bool,
    pub nodes: Vec<RegionNodeV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<RegionEdgeV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hyperedges: Vec<RegionHyperedgeV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub factors: Vec<RegionFactorV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nuisance_node_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boundary_node_ids: Vec<ArtifactId>,
    pub regions: Vec<RegionContractV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_claim_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_projection_ids: Vec<ArtifactId>,
    #[serde(default = "v11_activation_reserved_draft")]
    pub activation_level: V11ActivationLevelV1,
    #[serde(default = "bool_true")]
    pub advisory_only: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub compiled_at: DateTime<Utc>,
}

impl CompiledRegionGraphV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        graph_id: ArtifactId,
        graph_kind: RegionGraphKindV1,
        source_graph_kind: Option<RegionGraphKindV1>,
        max_region_size: u32,
        nodes: Vec<RegionNodeV1>,
        edges: Vec<RegionEdgeV1>,
        hyperedges: Vec<RegionHyperedgeV1>,
        factors: Vec<RegionFactorV1>,
        nuisance_node_ids: Vec<ArtifactId>,
        boundary_node_ids: Vec<ArtifactId>,
        regions: Vec<RegionContractV1>,
        source_claim_ids: Vec<ArtifactId>,
        source_projection_ids: Vec<ArtifactId>,
    ) -> Self {
        let right_graph_law_satisfied =
            graph_kind.can_execute_kernel() && regions.iter().all(RegionContractV1::is_bounded);
        let mut reason_codes = vec![
            "compiled-region-graph".into(),
            format!("graph-kind:{graph_kind}"),
            "v11b-region-graph-reserved-draft".into(),
        ];
        if source_graph_kind == Some(RegionGraphKindV1::Storage) {
            reason_codes.push("storage-evidence-compiled-not-used-directly".into());
        }
        if !right_graph_law_satisfied {
            reason_codes.push("right-graph-law-blocked".into());
        }
        Self {
            graph_id,
            kind: ArtifactKindV1::CompiledRegionGraph,
            graph_kind,
            source_graph_kind,
            max_region_size,
            right_graph_law_satisfied,
            nodes,
            edges,
            hyperedges,
            factors,
            nuisance_node_ids,
            boundary_node_ids,
            regions,
            source_claim_ids,
            source_projection_ids,
            activation_level: V11ActivationLevelV1::ReservedDraft,
            advisory_only: true,
            reason_codes,
            canonical_backpointers: canonical_owner_backpointer(
                "constraint-compiler",
                "CompileOutput",
                "canonical-region-graph-owner",
            ),
            compiled_at: Utc::now(),
        }
    }

    pub fn is_bounded_region_graph(&self) -> bool {
        self.right_graph_law_satisfied && self.regions.iter().all(RegionContractV1::is_bounded)
    }

    pub fn can_claim_active_v11b_runtime(&self) -> bool {
        self.activation_level.is_active() && !self.advisory_only
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct KernelStopRuleReportV1 {
    pub stop_state: CanonicalKernelStopReason,
    pub iteration: u32,
    pub max_iterations: u32,
    pub residual_threshold: f64,
    pub damping: f64,
    pub observed_residual: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl KernelStopRuleReportV1 {
    pub fn new(
        stop_state: CanonicalKernelStopReason,
        iteration: u32,
        max_iterations: u32,
        residual_threshold: f64,
        damping: f64,
        observed_residual: f64,
    ) -> Self {
        Self {
            stop_state,
            iteration,
            max_iterations,
            residual_threshold,
            damping,
            observed_residual,
            reason_codes: vec!["kernel-stop-rule-evidence".into()],
        }
    }

    pub fn is_terminal(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct KernelResidualReportV1 {
    pub residual_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub graph_id: ArtifactId,
    pub region_id: ArtifactId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_residual_id: Option<StackResidualId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_region_id: Option<StackRegionId>,
    pub iteration: u32,
    pub previous_value: f64,
    pub current_value: f64,
    pub delta: f64,
    pub threshold: f64,
    pub converged: bool,
    pub stop_rule_evidence: KernelStopRuleReportV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub measured_at: DateTime<Utc>,
}

impl KernelResidualReportV1 {
    pub fn new(
        graph_id: ArtifactId,
        region_id: ArtifactId,
        iteration: u32,
        previous_value: f64,
        current_value: f64,
        threshold: f64,
        stop_rule_evidence: KernelStopRuleReportV1,
    ) -> Self {
        let delta = (previous_value - current_value).abs();
        let converged = current_value <= threshold;
        Self {
            residual_id: display_only_unstable_id("residual"),
            kind: ArtifactKindV1::Residual,
            graph_id,
            region_id,
            canonical_residual_id: None,
            canonical_region_id: None,
            iteration,
            previous_value,
            current_value,
            delta,
            threshold,
            converged,
            stop_rule_evidence,
            reason_codes: if converged {
                vec!["residual-within-threshold".into()]
            } else {
                vec!["residual-above-threshold".into()]
            },
            canonical_backpointers: vec![
                CanonicalBackpointerV1::owner_type(
                    "recursive-kernel-core",
                    "ResidualArtifact",
                    "canonical-residual-owner",
                ),
                CanonicalBackpointerV1::owner_type(
                    "kernel-execution",
                    "ResidualSample",
                    "canonical-residual-execution-owner",
                ),
            ],
            measured_at: Utc::now(),
        }
    }

    pub fn blocks_promotion_as_exact(&self) -> bool {
        !self.converged
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum KernelSyndromeKindDisplayV1 {
    Contradiction,
    HighResidual,
    BoundaryViolation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct KernelSyndromeReportV1 {
    pub syndrome_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub syndrome_kind: KernelSyndromeKindDisplayV1,
    pub graph_id: ArtifactId,
    pub region_id: ArtifactId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_syndrome_id: Option<StackSyndromeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_region_id: Option<StackRegionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contradiction_witness_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub residual_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_claim_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_repair_record_ids: Vec<StackBoundaryRepairRecordId>,
    pub global_recompute_required: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub emitted_at: DateTime<Utc>,
}

impl KernelSyndromeReportV1 {
    pub fn contradiction(
        graph_id: ArtifactId,
        region_id: ArtifactId,
        contradiction_witness_id: ArtifactId,
        affected_claim_ids: Vec<ArtifactId>,
    ) -> Self {
        Self {
            syndrome_id: display_only_unstable_id("syndrome"),
            kind: ArtifactKindV1::Syndrome,
            syndrome_kind: KernelSyndromeKindDisplayV1::Contradiction,
            graph_id,
            region_id,
            canonical_syndrome_id: None,
            canonical_region_id: None,
            contradiction_witness_id: Some(contradiction_witness_id),
            residual_id: None,
            affected_claim_ids,
            canonical_repair_record_ids: Vec::new(),
            global_recompute_required: true,
            reason_codes: vec!["canonical-repair-required".into()],
            canonical_backpointers: vec![
                CanonicalBackpointerV1::owner_type(
                    "recursive-kernel-core",
                    "Syndrome",
                    "canonical-syndrome-owner",
                ),
                CanonicalBackpointerV1::owner_type(
                    "verification-control",
                    "SyndromeRouteRecord",
                    "canonical-syndrome-repair-routing-owner",
                ),
            ],
            emitted_at: Utc::now(),
        }
    }

    pub fn requires_canonical_repair(&self) -> bool {
        self.global_recompute_required && self.canonical_repair_record_ids.is_empty()
    }

    pub fn blocks_promotion_as_exact(&self) -> bool {
        self.global_recompute_required || self.canonical_repair_record_ids.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum OracleAgreementV1 {
    Agrees,
    BoundedDisagreement,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OracleSliceRequestV1 {
    pub request_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub graph_id: ArtifactId,
    pub region_id: ArtifactId,
    pub requested_node_ids: Vec<ArtifactId>,
    pub max_exact_nodes: u32,
    pub approximate_digest: DisplayDigestV1,
    pub oracle_digest: DisplayDigestV1,
    pub agreement: OracleAgreementV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disagreement_bound: Option<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub requested_at: DateTime<Utc>,
}

impl OracleSliceRequestV1 {
    pub fn new(
        graph_id: ArtifactId,
        region_id: ArtifactId,
        mut requested_node_ids: Vec<ArtifactId>,
        max_exact_nodes: u32,
        approximate_digest: DisplayDigestV1,
        oracle_digest: DisplayDigestV1,
        disagreement_bound: Option<f64>,
    ) -> Self {
        requested_node_ids.sort();
        requested_node_ids.dedup();
        let agreement = if approximate_digest == oracle_digest {
            OracleAgreementV1::Agrees
        } else {
            OracleAgreementV1::BoundedDisagreement
        };
        Self {
            request_id: display_only_unstable_id("oracle-slice-request"),
            kind: ArtifactKindV1::OracleSliceRequest,
            graph_id,
            region_id,
            requested_node_ids,
            max_exact_nodes,
            approximate_digest,
            oracle_digest,
            agreement,
            disagreement_bound,
            reason_codes: if agreement == OracleAgreementV1::Agrees {
                vec!["oracle-slice-agrees-with-approximate-path".into()]
            } else {
                vec!["oracle-slice-bounded-disagreement".into()]
            },
            requested_at: Utc::now(),
        }
    }

    pub fn has_bounded_semantic_diff(&self) -> bool {
        self.agreement == OracleAgreementV1::Agrees
            || self
                .disagreement_bound
                .map(|bound| bound.is_finite() && bound >= 0.0)
                .unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConvergenceReportV1 {
    pub report_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub graph_id: ArtifactId,
    pub graph_kind: RegionGraphKindV1,
    pub converged: bool,
    pub degraded: bool,
    pub stop_state: CanonicalKernelStopReason,
    pub iteration_count: u32,
    pub max_iterations: u32,
    pub residual_threshold: f64,
    pub damping: f64,
    pub final_residual: f64,
    pub residual_ids: Vec<ArtifactId>,
    pub stop_rule_evidence: KernelStopRuleReportV1,
    pub oscillation_detected: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

impl ConvergenceReportV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        graph_id: ArtifactId,
        graph_kind: RegionGraphKindV1,
        stop_state: CanonicalKernelStopReason,
        iteration_count: u32,
        max_iterations: u32,
        residual_threshold: f64,
        damping: f64,
        final_residual: f64,
        residual_ids: Vec<ArtifactId>,
        stop_rule_evidence: KernelStopRuleReportV1,
        oscillation_detected: bool,
    ) -> Self {
        let converged = matches!(
            stop_state,
            CanonicalKernelStopReason::FixedPoint | CanonicalKernelStopReason::AcyclicCompletion
        );
        let residual_degraded = final_residual.is_finite()
            && residual_threshold.is_finite()
            && final_residual > residual_threshold;
        let degraded = !converged || oscillation_detected || residual_degraded;
        let mut reason_codes = if degraded {
            vec!["kernel-convergence-degraded".into()]
        } else {
            vec!["kernel-converged-with-stop-rule".into()]
        };
        if !converged {
            reason_codes.push("kernel-non-converged-stop-state".into());
        }
        if oscillation_detected {
            reason_codes.push("kernel-oscillation-detected".into());
        }
        if residual_degraded {
            reason_codes.push("kernel-residual-above-threshold".into());
        }
        Self {
            report_id: display_only_unstable_id("convergence-report"),
            kind: ArtifactKindV1::ConvergenceReport,
            graph_id,
            graph_kind,
            converged,
            degraded,
            stop_state,
            iteration_count,
            max_iterations,
            residual_threshold,
            damping,
            final_residual,
            residual_ids,
            stop_rule_evidence,
            oscillation_detected,
            reason_codes,
            recorded_at: Utc::now(),
        }
    }

    pub fn has_explicit_stop_rule_evidence(&self) -> bool {
        self.stop_rule_evidence.is_terminal()
    }

    pub fn blocks_promotion_as_exact(&self) -> bool {
        self.degraded
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct KernelRunDisplayReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub graph_id: ArtifactId,
    pub graph_kind: RegionGraphKindV1,
    pub region_ids: Vec<ArtifactId>,
    pub converged: bool,
    pub degraded: bool,
    pub convergence_report_id: ArtifactId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub syndrome_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub residual_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub oracle_request_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_repair_record_ids: Vec<StackBoundaryRepairRecordId>,
    pub stop_rule_evidence: KernelStopRuleReportV1,
    pub used_global_recompute: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub recorded_at: DateTime<Utc>,
}

impl KernelRunDisplayReportV1 {
    pub fn new(
        graph: &CompiledRegionGraphV1,
        convergence_report: &ConvergenceReportV1,
        syndromes: &[KernelSyndromeReportV1],
        oracle_requests: &[OracleSliceRequestV1],
    ) -> Self {
        let mut region_ids = graph
            .regions
            .iter()
            .map(|region| region.region_id.clone())
            .collect::<Vec<_>>();
        region_ids.sort();
        region_ids.dedup();
        let mut canonical_repair_record_ids = syndromes
            .iter()
            .flat_map(|syndrome| syndrome.canonical_repair_record_ids.iter().cloned())
            .collect::<Vec<_>>();
        canonical_repair_record_ids.sort();
        canonical_repair_record_ids.dedup();
        let used_global_recompute = syndromes
            .iter()
            .any(|syndrome| syndrome.global_recompute_required);
        let mut reason_codes = if convergence_report.converged {
            vec!["kernel-run-converged".into()]
        } else {
            vec!["kernel-run-degraded".into()]
        };
        if !syndromes.is_empty() {
            reason_codes.push("kernel-run-has-syndrome-evidence".into());
        }
        Self {
            receipt_id: display_only_unstable_id("kernel-run-report"),
            kind: ArtifactKindV1::KernelRunReport,
            graph_id: graph.graph_id.clone(),
            graph_kind: graph.graph_kind,
            region_ids,
            converged: convergence_report.converged,
            degraded: convergence_report.degraded,
            convergence_report_id: convergence_report.report_id.clone(),
            syndrome_ids: syndromes
                .iter()
                .map(|syndrome| syndrome.syndrome_id.clone())
                .collect(),
            residual_ids: convergence_report.residual_ids.clone(),
            oracle_request_ids: oracle_requests
                .iter()
                .map(|request| request.request_id.clone())
                .collect(),
            canonical_repair_record_ids,
            stop_rule_evidence: convergence_report.stop_rule_evidence.clone(),
            used_global_recompute,
            reason_codes,
            canonical_backpointers: vec![
                CanonicalBackpointerV1::owner_type(
                    "recursive-kernel-core",
                    "KernelRun",
                    "canonical-kernel-run-owner",
                ),
                CanonicalBackpointerV1::owner_type(
                    "kernel-execution",
                    "ExecutionReport",
                    "canonical-kernel-execution-owner",
                ),
            ],
            recorded_at: Utc::now(),
        }
    }

    pub fn convergence_is_evidenced(&self) -> bool {
        !self.converged
            || matches!(
                self.stop_rule_evidence.stop_state,
                CanonicalKernelStopReason::FixedPoint
                    | CanonicalKernelStopReason::AcyclicCompletion
            )
    }

    pub fn can_promote_as_exact_seed_result(&self) -> bool {
        self.converged && !self.degraded && self.syndrome_ids.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SubtractionOperatorV1 {
    Dedupe,
    Summarize,
    Compact,
    Retire,
    Quarantine,
    Minimize,
    SupportCoreExtraction,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum PreservedInvariantV1 {
    Replay,
    AsOfQuery,
    ClaimSupport,
    ReceiptLineage,
    LegalRetention,
    OperatorProof,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InvariantBudgetV1 {
    pub budget_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub history_budget_label: String,
    pub as_of_query_horizon: String,
    pub preserved_invariants: Vec<PreservedInvariantV1>,
    pub max_removed_claims: u32,
    pub max_removed_evidence_records: u32,
    pub max_compacted_receipts: u32,
    pub requires_receipt_retention: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_retention_policy_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub approval_receipt_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub declared_at: DateTime<Utc>,
}

impl InvariantBudgetV1 {
    pub fn full_history() -> Self {
        Self {
            budget_id: display_only_unstable_id("invariant-budget"),
            kind: ArtifactKindV1::InvariantBudget,
            history_budget_label: "full-history-preserved".into(),
            as_of_query_horizon: "all-recorded-history".into(),
            preserved_invariants: vec![
                PreservedInvariantV1::Replay,
                PreservedInvariantV1::AsOfQuery,
                PreservedInvariantV1::ClaimSupport,
                PreservedInvariantV1::ReceiptLineage,
                PreservedInvariantV1::LegalRetention,
                PreservedInvariantV1::OperatorProof,
            ],
            max_removed_claims: 0,
            max_removed_evidence_records: 0,
            max_compacted_receipts: 0,
            requires_receipt_retention: true,
            receipt_retention_policy_id: None,
            approval_receipt_ids: Vec::new(),
            reason_codes: vec!["full-history-budget-preserves-as-of-queries".into()],
            declared_at: Utc::now(),
        }
    }

    pub fn with_removal_limits(
        mut self,
        max_removed_claims: u32,
        max_removed_evidence_records: u32,
        max_compacted_receipts: u32,
    ) -> Self {
        self.max_removed_claims = max_removed_claims;
        self.max_removed_evidence_records = max_removed_evidence_records;
        self.max_compacted_receipts = max_compacted_receipts;
        self.reason_codes
            .push("explicit-reduction-limits-declared".into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }

    pub fn with_receipt_retention_policy(
        mut self,
        retention_policy_id: ArtifactId,
        approval_receipt_ids: Vec<ArtifactId>,
    ) -> Self {
        self.receipt_retention_policy_id = Some(retention_policy_id);
        self.approval_receipt_ids = sorted_unique_artifact_ids(approval_receipt_ids);
        self.reason_codes
            .push("receipt-retention-policy-and-approval-declared".into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }

    pub fn preserves(&self, invariant: PreservedInvariantV1) -> bool {
        self.preserved_invariants.contains(&invariant)
    }

    pub fn preserves_as_of_queries(&self) -> bool {
        self.preserves(PreservedInvariantV1::AsOfQuery)
    }

    pub fn preserves_claim_support(&self) -> bool {
        self.preserves(PreservedInvariantV1::ClaimSupport)
    }

    pub fn allows_receipt_compaction(&self) -> bool {
        self.max_compacted_receipts > 0
            && (!self.requires_receipt_retention
                || (self.receipt_retention_policy_id.is_some()
                    && !self.approval_receipt_ids.is_empty()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SupportCoreV1 {
    pub support_core_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub accepted_claim_ids: Vec<ArtifactId>,
    pub support_claim_ids: Vec<ArtifactId>,
    pub support_evidence_ids: Vec<ArtifactId>,
    pub support_receipt_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub superseded_claim_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quarantined_claim_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub computed_at: DateTime<Utc>,
}

impl SupportCoreV1 {
    pub fn new(
        accepted_claim_ids: Vec<ArtifactId>,
        support_claim_ids: Vec<ArtifactId>,
        support_evidence_ids: Vec<ArtifactId>,
        support_receipt_ids: Vec<ArtifactId>,
        superseded_claim_ids: Vec<ArtifactId>,
        quarantined_claim_ids: Vec<ArtifactId>,
    ) -> Self {
        let accepted_claim_ids = sorted_unique_artifact_ids(accepted_claim_ids);
        let mut all_support_claim_ids = support_claim_ids;
        all_support_claim_ids.extend(accepted_claim_ids.clone());
        Self {
            support_core_id: display_only_unstable_id("support-core"),
            kind: ArtifactKindV1::SupportCore,
            accepted_claim_ids,
            support_claim_ids: sorted_unique_artifact_ids(all_support_claim_ids),
            support_evidence_ids: sorted_unique_artifact_ids(support_evidence_ids),
            support_receipt_ids: sorted_unique_artifact_ids(support_receipt_ids),
            superseded_claim_ids: sorted_unique_artifact_ids(superseded_claim_ids),
            quarantined_claim_ids: sorted_unique_artifact_ids(quarantined_claim_ids),
            reason_codes: vec!["accepted-claim-support-core".into()],
            canonical_backpointers: canonical_owner_backpointer(
                "semantic-memory-forge",
                "SupportSetV1",
                "canonical-support-expression-owner",
            ),
            computed_at: Utc::now(),
        }
    }

    pub fn claim_is_support(&self, claim_id: &ArtifactId) -> bool {
        self.support_claim_ids.contains(claim_id)
    }

    pub fn claim_is_superseded_or_quarantined(&self, claim_id: &ArtifactId) -> bool {
        self.superseded_claim_ids.contains(claim_id)
            || self.quarantined_claim_ids.contains(claim_id)
    }

    pub fn claim_can_be_removed_without_support_loss(&self, claim_id: &ArtifactId) -> bool {
        !self.claim_is_support(claim_id) || self.claim_is_superseded_or_quarantined(claim_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RemovalFrontierV1 {
    pub frontier_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub support_core_id: ArtifactId,
    pub candidate_claim_ids: Vec<ArtifactId>,
    pub candidate_evidence_ids: Vec<ArtifactId>,
    pub candidate_receipt_ids: Vec<ArtifactId>,
    pub removable_claim_ids: Vec<ArtifactId>,
    pub blocked_claim_ids: Vec<ArtifactId>,
    pub removable_evidence_ids: Vec<ArtifactId>,
    pub blocked_evidence_ids: Vec<ArtifactId>,
    pub removable_receipt_ids: Vec<ArtifactId>,
    pub blocked_receipt_ids: Vec<ArtifactId>,
    pub invariant_checks: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub computed_at: DateTime<Utc>,
}

impl RemovalFrontierV1 {
    pub fn new(
        support_core: &SupportCoreV1,
        candidate_claim_ids: Vec<ArtifactId>,
        candidate_evidence_ids: Vec<ArtifactId>,
        candidate_receipt_ids: Vec<ArtifactId>,
        budget: &InvariantBudgetV1,
    ) -> Self {
        let candidate_claim_ids = sorted_unique_artifact_ids(candidate_claim_ids);
        let candidate_evidence_ids = sorted_unique_artifact_ids(candidate_evidence_ids);
        let candidate_receipt_ids = sorted_unique_artifact_ids(candidate_receipt_ids);

        let mut removable_claim_ids = Vec::new();
        let mut blocked_claim_ids = Vec::new();
        for claim_id in &candidate_claim_ids {
            if support_core.claim_can_be_removed_without_support_loss(claim_id) {
                removable_claim_ids.push(claim_id.clone());
            } else {
                blocked_claim_ids.push(claim_id.clone());
            }
        }

        let mut removable_evidence_ids = Vec::new();
        let mut blocked_evidence_ids = Vec::new();
        for evidence_id in &candidate_evidence_ids {
            if budget.preserves_claim_support()
                && support_core.support_evidence_ids.contains(evidence_id)
            {
                blocked_evidence_ids.push(evidence_id.clone());
            } else {
                removable_evidence_ids.push(evidence_id.clone());
            }
        }

        let mut removable_receipt_ids = Vec::new();
        let mut blocked_receipt_ids = Vec::new();
        for receipt_id in &candidate_receipt_ids {
            let receipt_lineage_blocks = budget.preserves(PreservedInvariantV1::ReceiptLineage)
                && support_core.support_receipt_ids.contains(receipt_id)
                && !budget.allows_receipt_compaction();
            let retention_policy_blocks =
                budget.requires_receipt_retention && !budget.allows_receipt_compaction();
            if receipt_lineage_blocks || retention_policy_blocks {
                blocked_receipt_ids.push(receipt_id.clone());
            } else {
                removable_receipt_ids.push(receipt_id.clone());
            }
        }

        let mut invariant_checks = Vec::new();
        if budget.preserves_claim_support() {
            invariant_checks.push("claim-support-preserved".into());
        }
        if budget.preserves_as_of_queries() {
            invariant_checks.push("as-of-query-preserved".into());
        }
        if budget.preserves(PreservedInvariantV1::ReceiptLineage) {
            invariant_checks.push("receipt-lineage-preserved".into());
        }
        if budget.preserves(PreservedInvariantV1::LegalRetention) {
            invariant_checks.push("legal-retention-preserved".into());
        }
        let blocked = !blocked_claim_ids.is_empty()
            || !blocked_evidence_ids.is_empty()
            || !blocked_receipt_ids.is_empty();

        Self {
            frontier_id: display_only_unstable_id("removal-frontier"),
            kind: ArtifactKindV1::RemovalFrontier,
            support_core_id: support_core.support_core_id.clone(),
            candidate_claim_ids,
            candidate_evidence_ids,
            candidate_receipt_ids,
            removable_claim_ids: sorted_unique_artifact_ids(removable_claim_ids),
            blocked_claim_ids: sorted_unique_artifact_ids(blocked_claim_ids),
            removable_evidence_ids: sorted_unique_artifact_ids(removable_evidence_ids),
            blocked_evidence_ids: sorted_unique_artifact_ids(blocked_evidence_ids),
            removable_receipt_ids: sorted_unique_artifact_ids(removable_receipt_ids),
            blocked_receipt_ids: sorted_unique_artifact_ids(blocked_receipt_ids),
            invariant_checks,
            reason_codes: if blocked {
                vec!["removal-frontier-blocked-by-invariant".into()]
            } else {
                vec!["removal-frontier-lawful".into()]
            },
            canonical_backpointers: canonical_owner_backpointer(
                "semantic-memory-forge",
                "SupportSetV1",
                "canonical-removal-support-owner",
            ),
            computed_at: Utc::now(),
        }
    }

    pub fn is_blocked(&self) -> bool {
        !self.blocked_claim_ids.is_empty()
            || !self.blocked_evidence_ids.is_empty()
            || !self.blocked_receipt_ids.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SubtractionPlanV1 {
    pub plan_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub operator: SubtractionOperatorV1,
    pub support_core_id: ArtifactId,
    pub removal_frontier_id: ArtifactId,
    pub invariant_budget_id: ArtifactId,
    pub target_claim_ids: Vec<ArtifactId>,
    pub target_evidence_ids: Vec<ArtifactId>,
    pub target_receipt_ids: Vec<ArtifactId>,
    pub dry_run: bool,
    pub approved_for_mutation: bool,
    pub destructive_deletion: bool,
    pub blocked: bool,
    #[serde(default = "v11_activation_reserved_draft")]
    pub activation_level: V11ActivationLevelV1,
    #[serde(default = "bool_true")]
    pub advisory_only: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub planned_at: DateTime<Utc>,
}

impl SubtractionPlanV1 {
    pub fn dry_run(
        operator: SubtractionOperatorV1,
        support_core: &SupportCoreV1,
        frontier: &RemovalFrontierV1,
        budget: &InvariantBudgetV1,
    ) -> Self {
        let mut reason_codes = vec![
            "subtraction-plan-dry-run-before-mutation".into(),
            "v11b-subtraction-plan-advisory-only".into(),
        ];
        if frontier.is_blocked() {
            reason_codes.push("subtraction-plan-blocked-by-frontier".into());
        }
        if !frontier.candidate_receipt_ids.is_empty() && !budget.allows_receipt_compaction() {
            reason_codes.push("receipt-compaction-requires-retention-policy-and-approval".into());
        }
        let blocked = frontier.is_blocked()
            || (!frontier.candidate_receipt_ids.is_empty() && !budget.allows_receipt_compaction());
        Self {
            plan_id: display_only_unstable_id("subtraction-plan"),
            kind: ArtifactKindV1::SubtractionPlan,
            operator,
            support_core_id: support_core.support_core_id.clone(),
            removal_frontier_id: frontier.frontier_id.clone(),
            invariant_budget_id: budget.budget_id.clone(),
            target_claim_ids: frontier.candidate_claim_ids.clone(),
            target_evidence_ids: frontier.candidate_evidence_ids.clone(),
            target_receipt_ids: frontier.candidate_receipt_ids.clone(),
            dry_run: true,
            approved_for_mutation: false,
            destructive_deletion: false,
            blocked,
            activation_level: V11ActivationLevelV1::AdvisoryOnly,
            advisory_only: true,
            reason_codes,
            canonical_backpointers: canonical_owner_backpointer(
                "semantic-memory-forge",
                "RetractionRecordV1",
                "canonical-subtraction-retraction-owner",
            ),
            planned_at: Utc::now(),
        }
    }

    pub fn is_lawful_append_only_reduction(&self) -> bool {
        self.dry_run && !self.blocked && !self.destructive_deletion
    }

    pub fn can_mutate_runtime_state(&self) -> bool {
        self.activation_level.is_active()
            && !self.advisory_only
            && self.approved_for_mutation
            && !self.dry_run
            && !self.blocked
            && !self.destructive_deletion
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct HistoryPreservationReportV1 {
    pub report_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub plan_id: ArtifactId,
    pub invariant_budget_id: ArtifactId,
    pub before_digest: DisplayDigestV1,
    pub after_digest: DisplayDigestV1,
    pub checked_query_receipt_ids: Vec<ArtifactId>,
    pub preserved_invariants: Vec<PreservedInvariantV1>,
    pub as_of_queries_preserved: bool,
    pub claim_support_preserved: bool,
    pub receipt_lineage_preserved: bool,
    pub legal_retention_preserved: bool,
    pub degraded: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

impl HistoryPreservationReportV1 {
    pub fn new(
        plan: &SubtractionPlanV1,
        budget: &InvariantBudgetV1,
        before_digest: DisplayDigestV1,
        after_digest: DisplayDigestV1,
        checked_query_receipt_ids: Vec<ArtifactId>,
    ) -> Self {
        let digest_stable = before_digest == after_digest;
        let as_of_queries_preserved = !budget.preserves_as_of_queries()
            || digest_stable
            || !checked_query_receipt_ids.is_empty();
        let claim_support_preserved = !budget.preserves_claim_support() || !plan.blocked;
        let receipt_lineage_preserved = plan.target_receipt_ids.is_empty()
            || !budget.preserves(PreservedInvariantV1::ReceiptLineage)
            || budget.allows_receipt_compaction();
        let legal_retention_preserved = plan.target_receipt_ids.is_empty()
            || !budget.preserves(PreservedInvariantV1::LegalRetention)
            || budget.allows_receipt_compaction();
        let degraded = !(as_of_queries_preserved
            && claim_support_preserved
            && receipt_lineage_preserved
            && legal_retention_preserved);
        let mut reason_codes = if degraded {
            vec!["history-preservation-degraded".into()]
        } else {
            vec!["history-preservation-passed".into()]
        };
        if digest_stable {
            reason_codes.push("before-after-digests-stable".into());
        }
        Self {
            report_id: display_only_unstable_id("history-preservation-report"),
            kind: ArtifactKindV1::HistoryPreservationReport,
            plan_id: plan.plan_id.clone(),
            invariant_budget_id: budget.budget_id.clone(),
            before_digest,
            after_digest,
            checked_query_receipt_ids: sorted_unique_artifact_ids(checked_query_receipt_ids),
            preserved_invariants: sorted_unique_invariants(budget.preserved_invariants.clone()),
            as_of_queries_preserved,
            claim_support_preserved,
            receipt_lineage_preserved,
            legal_retention_preserved,
            degraded,
            reason_codes,
            recorded_at: Utc::now(),
        }
    }

    pub fn preserves_declared_history(&self) -> bool {
        !self.degraded
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompactionReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub plan_id: ArtifactId,
    pub support_core_id: ArtifactId,
    pub removal_frontier_id: ArtifactId,
    pub invariant_budget_id: ArtifactId,
    pub history_report_id: ArtifactId,
    pub operator: SubtractionOperatorV1,
    pub before_digest: DisplayDigestV1,
    pub after_digest: DisplayDigestV1,
    pub compacted: bool,
    pub destructive_deletion: bool,
    pub receipt_compaction_approved: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_receipt_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub compacted_at: DateTime<Utc>,
}

impl CompactionReportV1 {
    pub fn append_only(
        plan: &SubtractionPlanV1,
        report: &HistoryPreservationReportV1,
        source_receipt_ids: Vec<ArtifactId>,
    ) -> Self {
        let compacted =
            plan.is_lawful_append_only_reduction() && report.preserves_declared_history();
        Self {
            receipt_id: display_only_unstable_id("compaction-report"),
            kind: ArtifactKindV1::CompactionReport,
            plan_id: plan.plan_id.clone(),
            support_core_id: plan.support_core_id.clone(),
            removal_frontier_id: plan.removal_frontier_id.clone(),
            invariant_budget_id: plan.invariant_budget_id.clone(),
            history_report_id: report.report_id.clone(),
            operator: plan.operator,
            before_digest: report.before_digest.clone(),
            after_digest: report.after_digest.clone(),
            compacted,
            destructive_deletion: false,
            receipt_compaction_approved: plan.target_receipt_ids.is_empty()
                || report.receipt_lineage_preserved,
            run_id: None,
            attempt_id: None,
            source_receipt_ids: sorted_unique_artifact_ids(source_receipt_ids),
            reason_codes: if compacted {
                vec!["append-only-compaction-recorded".into()]
            } else {
                vec!["compaction-blocked-by-history-preservation".into()]
            },
            compacted_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityStateV1 {
    Configured,
    Available,
    Healthy,
    Registered,
    ExposedThisTurn,
    ExecutableThisTurn,
    Attempted,
    Succeeded,
    Failed,
    Declared,
    Invoked,
    Hidden,
    Degraded,
    FallbackOnly,
    Unavailable,
    Deferred,
    Disabled,
    BlockedByPolicy,
    RequiresApproval,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeCapabilityTruthV1 {
    pub capability_id: String,
    pub states: Vec<CapabilityStateV1>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolSchemaV1 {
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub parser_fallback_hint: String,
}

impl ToolSchemaV1 {
    pub fn canonical_backpointer(&self) -> CanonicalBackpointerV1 {
        CanonicalBackpointerV1::owner_type(
            "llm-tool-runtime",
            "ToolDescriptor.input_schema",
            "canonical-tool-schema-owner",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolDescriptorV1 {
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub risk_class: CanonicalToolSideEffectClass,
    pub read_only: bool,
    pub hidden: bool,
    #[serde(default)]
    pub requires_native_tool_loop: bool,
    pub schema: ToolSchemaV1,
}

impl ToolDescriptorV1 {
    pub fn tool_id(&self) -> String {
        format!("{}:{}:{}", self.namespace, self.name, self.version)
    }

    pub fn canonical_backpointer(&self) -> CanonicalBackpointerV1 {
        CanonicalBackpointerV1::external(
            "llm-tool-runtime",
            "ToolDescriptor",
            "canonical-tool-descriptor-owner",
            self.tool_id(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolProviderSchemaV1 {
    pub tool_id: String,
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub parser_fallback_hint: String,
}

impl ToolProviderSchemaV1 {
    pub fn from_descriptor(descriptor: &ToolDescriptorV1) -> Self {
        Self {
            tool_id: descriptor.tool_id(),
            name: descriptor.name.clone(),
            description: descriptor.description.clone(),
            input_schema: descriptor.schema.input_schema.clone(),
            output_schema: descriptor.schema.output_schema.clone(),
            parser_fallback_hint: descriptor.schema.parser_fallback_hint.clone(),
        }
    }

    pub fn canonical_backpointer(&self) -> CanonicalBackpointerV1 {
        CanonicalBackpointerV1::external(
            "llm-tool-runtime",
            "ToolDescriptor",
            "canonical-provider-schema-source",
            self.tool_id.clone(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolExposurePlanV1 {
    pub exposure_id: ArtifactId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub declared_tool_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub registered_tool_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub executable_tool_ids: Vec<String>,
    pub exposed_tool_ids: Vec<String>,
    pub hidden_tool_ids: Vec<String>,
    pub blocked_tool_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decisions: Vec<CapabilityGateDecisionV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub approval_requests: Vec<ApprovalRequestV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permit_use_receipts: Vec<PermitUseReportV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provider_tool_schemas: Vec<ToolProviderSchemaV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sandbox_root: Option<String>,
    #[serde(default)]
    pub degraded: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub reason: Option<String>,
}

pub type ToolExposurePlanV2 = ToolExposurePlanV1;
pub type ToolExposureSetV1 = ToolExposurePlanV1;
