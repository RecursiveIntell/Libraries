#![cfg_attr(test, allow(clippy::expect_used))]
use chrono::{DateTime, FixedOffset};
use constraint_compiler::{compile_batch, CompileOutput, CompilerPolicy};
use forge_memory_bridge::ProjectionImportBatchV3;
use kernel_execution::{execute_delta_propagation, execute_message_passing_baseline};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{ContentDigest, OracleSliceId, RegionId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OracleMode {
    ExactBounded,
    ConservativeFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OracleSliceSelectionPolicy {
    pub policy_name: String,
    pub prefers_bounded_regions: bool,
    pub max_exact_nodes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OracleAssessment {
    pub slice_id: OracleSliceId,
    pub mode: OracleMode,
    pub supported: bool,
    pub satisfied_constraint_count: usize,
    pub selected_region_ids: Vec<RegionId>,
    pub selection_policy: OracleSliceSelectionPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OracleParityCertificate {
    pub slice_id: OracleSliceId,
    pub region_ids: Vec<RegionId>,
    pub parity_match: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DeltaParityAssessment {
    pub changed_node_ids: Vec<String>,
    pub recomputed_node_ids: Vec<String>,
    pub recomputed_region_ids: Vec<RegionId>,
    pub parity_match: bool,
    pub certificate: OracleParityCertificate,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TemporalReplaySnapshot {
    pub recorded_at: String,
    pub batch: ProjectionImportBatchV3,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TemporalReplayAssessment {
    pub cutoff_recorded_at: String,
    pub selected_recorded_at: String,
    pub source_envelope_id: String,
    pub graph_hash: ContentDigest,
    pub matched_expected_hash: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RefutationMode {
    CausalRefuter,
    MinimalPerturbation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum OracleRefutationOutcome {
    FlipWitness { removed_node_ids: Vec<String> },
    NoFlipFound { searched_budget: usize },
    NotApplicable { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OracleRefutationResult {
    pub mode: RefutationMode,
    pub target_node_id: String,
    pub baseline_supported: bool,
    pub searched_budget: usize,
    pub outcome: OracleRefutationOutcome,
}

const MAX_REMOVAL_SET_SIZE: usize = 4;
const MAX_REMOVAL_COMBINATION_BUDGET: usize = 256;

/// Returns an exact bounded-oracle assessment when the compiled graph supports one.
pub fn evaluate_exact_bounded(compiled: &CompileOutput) -> Option<OracleAssessment> {
    let slice = compiled.oracle_candidates.first()?;
    let selected_region_ids = compiled
        .regions
        .iter()
        .filter(|region| {
            region
                .node_ids
                .iter()
                .any(|node_id| slice.node_ids.contains(node_id))
        })
        .map(|region| region.region_id.clone())
        .collect::<Vec<_>>();
    Some(OracleAssessment {
        slice_id: slice.oracle_slice_id.clone(),
        mode: OracleMode::ExactBounded,
        supported: compiled.degradations.is_empty() && compiled.nodes.len() <= 8,
        satisfied_constraint_count: compiled.constraints.len(),
        selected_region_ids,
        selection_policy: OracleSliceSelectionPolicy {
            policy_name: "bounded_region_exact_first".into(),
            prefers_bounded_regions: true,
            max_exact_nodes: 8,
        },
    })
}

/// Returns the conservative fallback oracle assessment for a compiled graph.
pub fn evaluate_conservative(compiled: &CompileOutput) -> OracleAssessment {
    let slice_id = compiled
        .oracle_candidates
        .first()
        .map(|slice| slice.oracle_slice_id.clone())
        .unwrap_or_else(|| OracleSliceId::new("oracle:unsupported"));
    OracleAssessment {
        slice_id,
        mode: OracleMode::ConservativeFallback,
        supported: true,
        satisfied_constraint_count: compiled.constraints.len(),
        selected_region_ids: compiled
            .regions
            .iter()
            .map(|region| region.region_id.clone())
            .collect(),
        selection_policy: OracleSliceSelectionPolicy {
            policy_name: "conservative_fallback".into(),
            prefers_bounded_regions: true,
            max_exact_nodes: 8,
        },
    }
}

/// Compares delta recomputation against a full bounded rerun over the affected focus set.
pub fn evaluate_delta_parity(
    compiled: &CompileOutput,
    changed_node_ids: &[String],
    max_iterations: u32,
) -> DeltaParityAssessment {
    let delta = execute_delta_propagation(compiled, changed_node_ids, max_iterations);
    let bounded = execute_message_passing_baseline(compiled, max_iterations);
    let recomputed_nodes = delta.recomputed_node_ids.clone();
    let recomputed_region_ids = delta.recomputed_region_ids.clone();
    let delta_focus = delta
        .execution
        .node_beliefs
        .iter()
        .filter(|belief| recomputed_nodes.contains(&belief.node_id))
        .map(|belief| (belief.node_id.clone(), belief.belief_micros))
        .collect::<Vec<_>>();
    let bounded_focus = bounded
        .node_beliefs
        .iter()
        .filter(|belief| recomputed_nodes.contains(&belief.node_id))
        .map(|belief| (belief.node_id.clone(), belief.belief_micros))
        .collect::<Vec<_>>();

    DeltaParityAssessment {
        changed_node_ids: changed_node_ids.to_vec(),
        recomputed_node_ids: recomputed_nodes.clone(),
        recomputed_region_ids: recomputed_region_ids.clone(),
        parity_match: delta_focus == bounded_focus,
        certificate: OracleParityCertificate {
            slice_id: compiled
                .oracle_candidates
                .first()
                .map(|slice| slice.oracle_slice_id.clone())
                .unwrap_or_else(|| OracleSliceId::new("oracle:unsupported")),
            region_ids: recomputed_region_ids,
            parity_match: delta_focus == bounded_focus,
        },
    }
}

/// Recompiles the newest snapshot at or before the supplied cutoff time.
pub fn replay_as_of(
    snapshots: &[TemporalReplaySnapshot],
    cutoff_recorded_at: &str,
    policy: &CompilerPolicy,
) -> Option<CompileOutput> {
    let cutoff = parsed_utc_timestamp(cutoff_recorded_at)?;
    let snapshot = snapshots
        .iter()
        .filter_map(|snapshot| {
            Some((
                parsed_utc_timestamp(&snapshot.recorded_at)?,
                snapshot.recorded_at.as_str(),
                snapshot,
            ))
        })
        .filter(|(recorded_at, _, _)| *recorded_at <= cutoff)
        .max_by_key(|(recorded_at, ..)| *recorded_at)
        .map(|(_, _, snapshot)| snapshot)?;
    Some(compile_batch(&snapshot.batch, policy))
}

/// Evaluates a temporal replay slice against an expected compiled graph hash.
pub fn evaluate_temporal_replay(
    snapshots: &[TemporalReplaySnapshot],
    cutoff_recorded_at: &str,
    policy: &CompilerPolicy,
    expected_hash: &ContentDigest,
) -> Option<TemporalReplayAssessment> {
    let cutoff = parsed_utc_timestamp(cutoff_recorded_at)?;
    let snapshot = snapshots
        .iter()
        .filter_map(|snapshot| {
            Some((
                parsed_utc_timestamp(&snapshot.recorded_at)?,
                snapshot.recorded_at.as_str(),
                snapshot,
            ))
        })
        .filter(|(recorded_at, _, _)| *recorded_at <= cutoff)
        .max_by_key(|(recorded_at, ..)| *recorded_at)
        .map(|(_, _, snapshot)| snapshot)?;
    let compiled = compile_batch(&snapshot.batch, policy);
    Some(TemporalReplayAssessment {
        cutoff_recorded_at: cutoff_recorded_at.to_string(),
        selected_recorded_at: snapshot.recorded_at.clone(),
        source_envelope_id: snapshot.batch.source_envelope_id.to_string(),
        matched_expected_hash: &compiled.graph_hash == expected_hash,
        graph_hash: compiled.graph_hash,
    })
}

/// Searches for a causal refutation witness that flips support for the target node.
pub fn evaluate_causal_refuter(
    compiled: &CompileOutput,
    target_node_id: &str,
    max_removed_nodes: usize,
) -> OracleRefutationResult {
    refute_internal(
        RefutationMode::CausalRefuter,
        compiled,
        target_node_id,
        max_removed_nodes,
    )
}

/// Searches for the smallest perturbation witness that flips support for the target node.
pub fn minimal_perturbation_witness(
    compiled: &CompileOutput,
    target_node_id: &str,
    max_removed_nodes: usize,
) -> OracleRefutationResult {
    refute_internal(
        RefutationMode::MinimalPerturbation,
        compiled,
        target_node_id,
        max_removed_nodes,
    )
}

/// Evaluates the minimal-perturbation refuter mode for a target node.
pub fn evaluate_minimal_perturbation(
    compiled: &CompileOutput,
    target_node_id: &str,
    max_removed_nodes: usize,
) -> OracleRefutationResult {
    minimal_perturbation_witness(compiled, target_node_id, max_removed_nodes)
}

fn refute_internal(
    mode: RefutationMode,
    compiled: &CompileOutput,
    target_node_id: &str,
    max_removed_nodes: usize,
) -> OracleRefutationResult {
    let baseline_supported = node_is_supported(compiled, target_node_id, &[]);
    if !baseline_supported {
        return OracleRefutationResult {
            mode,
            target_node_id: target_node_id.to_string(),
            baseline_supported,
            searched_budget: 0,
            outcome: OracleRefutationOutcome::NotApplicable {
                reason: "target node is not supported in the compiled graph".into(),
            },
        };
    }

    let candidates = perturbation_candidates(compiled, target_node_id);
    if candidates.is_empty() {
        return OracleRefutationResult {
            mode,
            target_node_id: target_node_id.to_string(),
            baseline_supported,
            searched_budget: 0,
            outcome: OracleRefutationOutcome::NotApplicable {
                reason: "no admissible perturbation candidates are connected to the target".into(),
            },
        };
    }

    let mut searched_budget = 0usize;
    let max_size = max_removed_nodes
        .max(1)
        .min(candidates.len())
        .min(MAX_REMOVAL_SET_SIZE);

    for size in 1..=max_size {
        let (witness, budget_exhausted) = find_witness_by_size(
            &candidates,
            compiled,
            target_node_id,
            size,
            MAX_REMOVAL_COMBINATION_BUDGET,
            &mut searched_budget,
        );
        if let Some(removed_node_ids) = witness {
            return OracleRefutationResult {
                mode,
                target_node_id: target_node_id.to_string(),
                baseline_supported,
                searched_budget,
                outcome: OracleRefutationOutcome::FlipWitness { removed_node_ids },
            };
        }
        if budget_exhausted {
            return OracleRefutationResult {
                mode,
                target_node_id: target_node_id.to_string(),
                baseline_supported,
                searched_budget,
                outcome: OracleRefutationOutcome::NoFlipFound { searched_budget },
            };
        }
    }

    OracleRefutationResult {
        mode,
        target_node_id: target_node_id.to_string(),
        baseline_supported,
        searched_budget,
        outcome: OracleRefutationOutcome::NoFlipFound { searched_budget },
    }
}

fn perturbation_candidates(compiled: &CompileOutput, target_node_id: &str) -> Vec<String> {
    let mut candidates = compiled
        .hyperedges
        .iter()
        .filter(|edge| {
            edge.member_node_ids
                .iter()
                .any(|node| node == target_node_id)
        })
        .flat_map(|edge| edge.member_node_ids.iter())
        .filter(|node_id| node_id.as_str() != target_node_id)
        .filter(|node_id| !node_id.starts_with("nuisance:"))
        .cloned()
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.dedup();
    candidates
}

fn parsed_utc_timestamp(value: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(value).ok()
}

fn node_is_supported(compiled: &CompileOutput, target_node_id: &str, removed: &[String]) -> bool {
    compiled.hyperedges.iter().any(|edge| {
        if !edge
            .member_node_ids
            .iter()
            .any(|node| node == target_node_id)
        {
            return false;
        }
        edge.member_node_ids
            .iter()
            .filter(|node_id| node_id.as_str() != target_node_id)
            .filter(|node_id| !removed.contains(node_id))
            .any(|node_id| !node_id.starts_with("nuisance:"))
    })
}

fn find_witness_by_size(
    candidates: &[String],
    compiled: &CompileOutput,
    target_node_id: &str,
    size: usize,
    max_budget: usize,
    searched_budget: &mut usize,
) -> (Option<Vec<String>>, bool) {
    let mut current = Vec::with_capacity(size);
    search_combinations_for_witness(
        candidates,
        size,
        0,
        &mut current,
        compiled,
        target_node_id,
        max_budget,
        searched_budget,
    )
}

#[allow(clippy::too_many_arguments)]
fn search_combinations_for_witness(
    items: &[String],
    size: usize,
    index: usize,
    current: &mut Vec<String>,
    compiled: &CompileOutput,
    target_node_id: &str,
    max_budget: usize,
    searched_budget: &mut usize,
) -> (Option<Vec<String>>, bool) {
    if current.len() == size {
        if *searched_budget >= max_budget {
            return (None, true);
        }

        *searched_budget += 1;
        if !node_is_supported(compiled, target_node_id, current) {
            return (Some(current.clone()), false);
        }
        return (None, false);
    }
    if index >= items.len() {
        return (None, false);
    }

    let mut idx = index;
    while idx < items.len() {
        current.push(items[idx].clone());
        let (witness, budget_exhausted) = search_combinations_for_witness(
            items,
            size,
            idx + 1,
            current,
            compiled,
            target_node_id,
            max_budget,
            searched_budget,
        );
        current.pop();

        if let Some(witness) = witness {
            return (Some(witness), false);
        }
        if budget_exhausted {
            return (None, true);
        }

        idx += 1;
    }
    (None, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use constraint_compiler::{
        CompilationBoundary, CompiledRegion, GraphGeometryManifest, GraphSurfaceKind,
        InferenceHyperedge, InferenceNode,
    };
    use forge_memory_bridge::transform_envelope_v3;
    use proptest::prelude::*;
    use recursive_kernel_core::ConstraintUnit;
    use semantic_memory_forge::{
        ConstraintSeedKind, ExportAuthority, ExportClaim, ExportConfidenceClass, ExportEnvelopeV3,
        ExportRecord, ExportRecordSemanticsV3, ExportRecordV3, ForgeExportMeta,
        ProjectionVisibilityClass, EXPORT_ENVELOPE_V3_SCHEMA,
    };
    use stack_ids::{
        AssertionGroupId, ClaimFamilyId, ConstraintId, EntityId, EnvelopeId, OperatorId,
        RegionDigestId, RegionId, ScopeKey,
    };

    fn rich_batch(envelope_id: &str, claim_suffix: &str) -> ProjectionImportBatchV3 {
        let scope = ScopeKey::namespace_only("oracle");
        let records = vec![
            ExportRecordV3 {
                record: ExportRecord::Claim(ExportClaim {
                    claim_id: None,
                    claim_version_id: Some(format!("claim-version-{claim_suffix}-1").into()),
                    subject_entity_id: EntityId::new(format!("entity-{claim_suffix}-1")),
                    predicate: "supports".into(),
                    object_anchor: serde_json::json!("result"),
                    valid_from: None,
                    valid_to: None,
                    confidence: 1.0,
                    content: "entity supports result".into(),
                    projection_family: "forge_verification".into(),
                    supersedes_claim_id: None,
                    supersedes_claim_version_id: None,
                    metadata: None,
                }),
                semantics: Some(ExportRecordSemanticsV3 {
                    claim_family_id: Some(ClaimFamilyId::new(format!("family-{claim_suffix}"))),
                    assertion_group_id: Some(AssertionGroupId::new(format!(
                        "group-{claim_suffix}"
                    ))),
                    relation_group_id: None,
                    joint_evidence_group_id: None,
                    constraint_seed_kind: Some(ConstraintSeedKind::Hyperedge),
                    treatment_hint: None,
                    outcome_hint: None,
                    confounder_hint: None,
                    instrument_hint: None,
                    effect_modifier_hint: None,
                    contradiction_candidate_group_id: None,
                    mutual_exclusion_group_id: None,
                    comparability_snapshot_version: None,
                    nuisance_snapshot: None,
                    projection_visibility_class: ProjectionVisibilityClass::Standard,
                    export_confidence_class: ExportConfidenceClass::Verified,
                    derivation_seed_ids: vec![],
                    review_priority_hint: None,
                }),
            },
            ExportRecordV3 {
                record: ExportRecord::Claim(ExportClaim {
                    claim_id: None,
                    claim_version_id: Some(format!("claim-version-{claim_suffix}-2").into()),
                    subject_entity_id: EntityId::new(format!("entity-{claim_suffix}-2")),
                    predicate: "supports".into(),
                    object_anchor: serde_json::json!("result"),
                    valid_from: None,
                    valid_to: None,
                    confidence: 0.95,
                    content: "entity 2 supports result".into(),
                    projection_family: "forge_verification".into(),
                    supersedes_claim_id: None,
                    supersedes_claim_version_id: None,
                    metadata: None,
                }),
                semantics: Some(ExportRecordSemanticsV3 {
                    claim_family_id: Some(ClaimFamilyId::new(format!("family-{claim_suffix}"))),
                    assertion_group_id: Some(AssertionGroupId::new(format!(
                        "group-{claim_suffix}"
                    ))),
                    relation_group_id: None,
                    joint_evidence_group_id: None,
                    constraint_seed_kind: Some(ConstraintSeedKind::Hyperedge),
                    treatment_hint: None,
                    outcome_hint: None,
                    confounder_hint: None,
                    instrument_hint: None,
                    effect_modifier_hint: None,
                    contradiction_candidate_group_id: None,
                    mutual_exclusion_group_id: None,
                    comparability_snapshot_version: None,
                    nuisance_snapshot: None,
                    projection_visibility_class: ProjectionVisibilityClass::Standard,
                    export_confidence_class: ExportConfidenceClass::Verified,
                    derivation_seed_ids: vec![],
                    review_priority_hint: None,
                }),
            },
        ];
        let export_meta = ForgeExportMeta {
            authority: ExportAuthority::Forge,
            run_id: Some("run-1".into()),
            direct_write: false,
            comparability_snapshot_version: None,
            exported_at: "2026-03-10T00:00:00Z".into(),
        };
        let digest =
            ExportEnvelopeV3::compute_digest("forge", &scope, &records, Some(&export_meta), None)
                .unwrap();
        let envelope = ExportEnvelopeV3 {
            envelope_id: EnvelopeId::new(envelope_id),
            schema_version: EXPORT_ENVELOPE_V3_SCHEMA.into(),
            content_digest: digest,
            source_authority: "forge".into(),
            scope_key: scope,
            trace_ctx: None,
            exported_at: "2026-03-10T00:00:00Z".into(),
            export_meta: Some(export_meta),
            evidence_bundle: None,
            support_sets: vec![],
            contradiction_witnesses: vec![],
            retraction_records: vec![],
            claim_states_v13: vec![],
            intervention_bundles_v14: vec![],
            outcome_schemas_v14: vec![],
            cohort_contracts_v14: vec![],
            counterfactual_slices_v14: vec![],
            experiment_cases_v14: vec![],
            comparability_matrices_v14: vec![],
            decision_traces_v14: vec![],
            refuter_suites_v14: vec![],
            refuter_results_v14: vec![],
            experiment_budgets_v14: vec![],
            rollout_decisions_v14: vec![],
            rollback_decisions_v14: vec![],
            attestation_envelopes_v15: vec![],
            trust_root_sets_v15: vec![],
            artifact_admission_policies_v15: vec![],
            transparency_receipts_v15: vec![],
            attestation_revocations_v15: vec![],
            attestation_supersessions_v15: vec![],
            remote_oracle_leases_v15: vec![],
            remote_slice_requests_v15: vec![],
            remote_slice_results_v15: vec![],
            cross_runtime_replay_tickets_v15: vec![],
            dispute_bundles_v15: vec![],
            disclosure_policies_v15: vec![],
            disclosure_budgets_v15: vec![],
            records,
        };
        transform_envelope_v3(&envelope).unwrap()
    }

    fn explicit_refute_fixture() -> CompileOutput {
        CompileOutput {
            graph_hash: ContentDigest::compute(b"oracle-causal-cost-fixture"),
            scope_key: ScopeKey::namespace_only("oracle"),
            geometry_manifest: GraphGeometryManifest {
                surfaces: vec![
                    GraphSurfaceKind::Storage,
                    GraphSurfaceKind::Retrieval,
                    GraphSurfaceKind::Inference,
                    GraphSurfaceKind::Repair,
                    GraphSurfaceKind::Control,
                ],
                compilation_boundaries: vec![CompilationBoundary {
                    from_surface: GraphSurfaceKind::Inference,
                    to_surface: GraphSurfaceKind::Repair,
                    artifact_families: vec!["syndrome".into()],
                    deterministic: true,
                }],
                no_silent_collapse: true,
            },
            nodes: vec![
                InferenceNode {
                    node_id: "target".into(),
                    kind: "claim_version".into(),
                },
                InferenceNode {
                    node_id: "peer-b".into(),
                    kind: "claim_version".into(),
                },
                InferenceNode {
                    node_id: "peer-c".into(),
                    kind: "claim_version".into(),
                },
            ],
            hyperedges: vec![InferenceHyperedge {
                edge_id: "edge:triad".into(),
                member_node_ids: vec!["target".into(), "peer-b".into(), "peer-c".into()],
            }],
            constraints: vec![ConstraintUnit {
                constraint_id: ConstraintId::new("constraint:edge:triad"),
                kind: "hyperedge".into(),
                variable_ids: vec!["target".into(), "peer-b".into(), "peer-c".into()],
                operator_id: OperatorId::new("kernel-oracles:tests"),
            }],
            regions: vec![CompiledRegion {
                region_id: RegionId::new("region:oracle-triad"),
                region_digest_id: RegionDigestId::new("region-digest:oracle-triad"),
                node_ids: vec!["target".into(), "peer-b".into(), "peer-c".into()],
                hyperedge_ids: vec!["edge:triad".into()],
                constraint_ids: vec![ConstraintId::new("constraint:edge:triad")],
                bounded_default_unit_of_work: true,
            }],
            invalidation_cones: vec![],
            degradations: vec![],
            oracle_candidates: vec![],
        }
    }

    fn compiled_fixture() -> CompileOutput {
        compile_batch(
            &rich_batch("env-1", "alpha"),
            &CompilerPolicy {
                policy_version: "policy-1".into(),
                include_hyperedges: true,
            },
        )
    }

    fn compile_many_claim_fixture(claim_count: usize) -> (CompileOutput, Vec<String>) {
        let scope = ScopeKey::namespace_only("oracle");
        let claims: Vec<String> = (0..claim_count)
            .map(|index| format!("claim-large-{index}"))
            .collect();
        let records: Vec<ExportRecordV3> = claims
            .iter()
            .map(|claim_id| ExportRecordV3 {
                record: ExportRecord::Claim(ExportClaim {
                    claim_id: None,
                    claim_version_id: Some(claim_id.clone().into()),
                    subject_entity_id: EntityId::new(format!("entity-{claim_id}")),
                    predicate: "supports".into(),
                    object_anchor: serde_json::json!("result"),
                    valid_from: None,
                    valid_to: None,
                    confidence: 1.0,
                    content: format!("{claim_id} supports result"),
                    projection_family: "forge_verification".into(),
                    supersedes_claim_id: None,
                    supersedes_claim_version_id: None,
                    metadata: None,
                }),
                semantics: Some(ExportRecordSemanticsV3 {
                    claim_family_id: Some(ClaimFamilyId::new("family-large")),
                    assertion_group_id: Some(AssertionGroupId::new("group-large")),
                    relation_group_id: None,
                    joint_evidence_group_id: None,
                    constraint_seed_kind: Some(ConstraintSeedKind::Hyperedge),
                    treatment_hint: None,
                    outcome_hint: None,
                    confounder_hint: None,
                    instrument_hint: None,
                    effect_modifier_hint: None,
                    contradiction_candidate_group_id: None,
                    mutual_exclusion_group_id: None,
                    comparability_snapshot_version: None,
                    nuisance_snapshot: None,
                    projection_visibility_class: ProjectionVisibilityClass::Standard,
                    export_confidence_class: ExportConfidenceClass::Verified,
                    derivation_seed_ids: vec![],
                    review_priority_hint: None,
                }),
            })
            .collect();
        let export_meta = ForgeExportMeta {
            authority: ExportAuthority::Forge,
            run_id: Some("run-large".into()),
            direct_write: false,
            comparability_snapshot_version: None,
            exported_at: "2026-03-10T00:00:00Z".into(),
        };
        let digest =
            ExportEnvelopeV3::compute_digest("forge", &scope, &records, Some(&export_meta), None)
                .unwrap();
        let envelope = ExportEnvelopeV3 {
            envelope_id: EnvelopeId::new("env-large"),
            schema_version: EXPORT_ENVELOPE_V3_SCHEMA.into(),
            content_digest: digest,
            source_authority: "forge".into(),
            scope_key: scope,
            trace_ctx: None,
            exported_at: "2026-03-10T00:00:00Z".into(),
            export_meta: Some(export_meta),
            evidence_bundle: None,
            support_sets: vec![],
            contradiction_witnesses: vec![],
            retraction_records: vec![],
            claim_states_v13: vec![],
            intervention_bundles_v14: vec![],
            outcome_schemas_v14: vec![],
            cohort_contracts_v14: vec![],
            counterfactual_slices_v14: vec![],
            experiment_cases_v14: vec![],
            comparability_matrices_v14: vec![],
            decision_traces_v14: vec![],
            refuter_suites_v14: vec![],
            refuter_results_v14: vec![],
            experiment_budgets_v14: vec![],
            rollout_decisions_v14: vec![],
            rollback_decisions_v14: vec![],
            attestation_envelopes_v15: vec![],
            trust_root_sets_v15: vec![],
            artifact_admission_policies_v15: vec![],
            transparency_receipts_v15: vec![],
            attestation_revocations_v15: vec![],
            attestation_supersessions_v15: vec![],
            remote_oracle_leases_v15: vec![],
            remote_slice_requests_v15: vec![],
            remote_slice_results_v15: vec![],
            cross_runtime_replay_tickets_v15: vec![],
            dispute_bundles_v15: vec![],
            disclosure_policies_v15: vec![],
            disclosure_budgets_v15: vec![],
            records,
        };
        let batch = transform_envelope_v3(&envelope).unwrap();
        let compiled = compile_batch(
            &batch,
            &CompilerPolicy {
                policy_version: "policy-1".into(),
                include_hyperedges: true,
            },
        );
        (compiled, claims)
    }

    #[test]
    fn exact_bounded_path_exists_for_small_supported_slice() {
        let compiled = compiled_fixture();
        let exact = evaluate_exact_bounded(&compiled).expect("expected exact oracle slice");
        let conservative = evaluate_conservative(&compiled);

        assert!(exact.supported);
        assert_eq!(
            exact.satisfied_constraint_count,
            conservative.satisfied_constraint_count
        );
        assert!(!compiled.oracle_candidates[0]
            .node_ids
            .iter()
            .any(|node_id| node_id.starts_with("nuisance:")));
        assert!(exact.selection_policy.prefers_bounded_regions);
        assert!(!exact.selected_region_ids.is_empty());
        assert_eq!(exact.selected_region_ids, conservative.selected_region_ids);
    }

    #[test]
    fn temporal_replay_selects_latest_snapshot_before_cutoff() {
        let policy = CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        };
        let older = rich_batch("env-old", "old");
        let newer = rich_batch("env-new", "new");
        let expected = compile_batch(&older, &policy);
        let replay = evaluate_temporal_replay(
            &[
                TemporalReplaySnapshot {
                    recorded_at: "2026-03-08T00:00:00Z".into(),
                    batch: older,
                },
                TemporalReplaySnapshot {
                    recorded_at: "2026-03-10T00:00:00Z".into(),
                    batch: newer,
                },
            ],
            "2026-03-09T00:00:00Z",
            &policy,
            &expected.graph_hash,
        )
        .expect("replay result expected");

        assert!(replay.matched_expected_hash);
        assert_eq!(replay.selected_recorded_at, "2026-03-08T00:00:00Z");
        assert_eq!(replay.source_envelope_id, "env-old");
    }

    #[test]
    fn temporal_replay_handles_timezone_offsets_without_string_ordering_bugs() {
        let policy = CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        };
        let earlier = rich_batch("env-offset-earlier", "older");
        let newer = rich_batch("env-offset-newer", "newer");
        let expected = compile_batch(&newer, &policy);
        let replay = evaluate_temporal_replay(
            &[
                TemporalReplaySnapshot {
                    recorded_at: "2026-03-08T15:00:00-05:00".into(),
                    batch: earlier,
                },
                TemporalReplaySnapshot {
                    recorded_at: "2026-03-08T21:00:00+02:00".into(),
                    batch: newer,
                },
            ],
            "2026-03-08T19:30:00+00:00",
            &policy,
            &expected.graph_hash,
        )
        .expect("replay result expected");

        assert_eq!(replay.selected_recorded_at, "2026-03-08T21:00:00+02:00");
    }

    #[test]
    fn temporal_replay_returns_none_for_malformed_cutoff_timestamp() {
        let policy = CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        };
        let replay = replay_as_of(
            &[TemporalReplaySnapshot {
                recorded_at: "2026-03-08T00:00:00Z".into(),
                batch: rich_batch("env-old", "old"),
            }],
            "not-a-real-timestamp",
            &policy,
        );

        assert!(replay.is_none());
    }

    #[test]
    fn temporal_replay_returns_none_for_malformed_snapshot_timestamp() {
        let policy = CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        };
        let replay = replay_as_of(
            &[TemporalReplaySnapshot {
                recorded_at: "not-a-real-timestamp".into(),
                batch: rich_batch("env-old", "old"),
            }],
            "2026-03-09T00:00:00Z",
            &policy,
        );

        assert!(replay.is_none());
    }

    #[test]
    fn causal_refuter_emits_flip_witness_for_single_peer_removal() {
        let compiled = compiled_fixture();
        let result = evaluate_causal_refuter(&compiled, "claim-version-alpha-1", 1);

        assert_eq!(result.mode, RefutationMode::CausalRefuter);
        assert!(matches!(
            result.outcome,
            OracleRefutationOutcome::FlipWitness { .. }
        ));
    }

    #[test]
    fn minimal_perturbation_search_defaults_to_one_step_budget() {
        let compiled = compiled_fixture();
        let result = minimal_perturbation_witness(&compiled, "claim-version-alpha-1", 0);

        assert!(matches!(
            result.outcome,
            OracleRefutationOutcome::FlipWitness { .. }
        ));
        assert_eq!(result.searched_budget, 1);
    }

    #[test]
    fn causal_refuter_budget_is_exhausted_before_flip_is_possible() {
        let (compiled, claims) = compile_many_claim_fixture(11);
        let target = &claims[0];
        let result = evaluate_causal_refuter(&compiled, target, 10);

        match result.outcome {
            OracleRefutationOutcome::NoFlipFound { searched_budget } => {
                assert_eq!(
                    searched_budget, MAX_REMOVAL_COMBINATION_BUDGET,
                    "combinatorial search should stop at bounded budget"
                );
            }
            other => panic!("expected NoFlipFound, got {other:?}"),
        }
        assert_eq!(
            result.mode,
            RefutationMode::CausalRefuter,
            "mode should remain causal refuter"
        );
        assert!(result.searched_budget >= MAX_REMOVAL_COMBINATION_BUDGET);
    }

    #[test]
    fn causal_refuter_reports_actual_witness_search_budget() {
        let compiled = explicit_refute_fixture();
        let result = evaluate_causal_refuter(&compiled, "target", 2);

        match result.outcome {
            OracleRefutationOutcome::FlipWitness { removed_node_ids } => {
                assert_eq!(
                    removed_node_ids.len(),
                    2,
                    "expected two-node witness to break support"
                );
            }
            other => panic!("expected flip witness, got {other:?}"),
        }
        assert_eq!(result.searched_budget, 3);
    }

    #[test]
    fn minimal_perturbation_preserves_small_graph_flip_witness() {
        let compiled = compiled_fixture();
        let result = minimal_perturbation_witness(&compiled, "claim-version-alpha-1", 4);

        assert!(matches!(
            result.outcome,
            OracleRefutationOutcome::FlipWitness { .. }
        ));
        assert_eq!(result.searched_budget, 1);
    }

    #[test]
    fn delta_parity_matches_full_recompute_on_recomputed_nodes() {
        let compiled = compiled_fixture();
        let parity = evaluate_delta_parity(&compiled, &["claim-version-alpha-2".into()], 4);

        assert!(parity.parity_match);
        assert!(!parity.recomputed_node_ids.is_empty());
        assert!(!parity.recomputed_region_ids.is_empty());
        assert!(parity.certificate.parity_match);
    }

    proptest! {
        #[test]
        fn malformed_rfc3339_timestamps_are_rejected(value in ".{1,64}") {
            prop_assume!(chrono::DateTime::parse_from_rfc3339(&value).is_err());
            prop_assert!(parsed_utc_timestamp(&value).is_none());
        }
    }
}
