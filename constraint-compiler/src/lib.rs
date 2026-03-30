use forge_memory_bridge::{ImportProjectionRecord, ProjectionImportBatchV3};
use recursive_kernel_core::{constraint_compiler_operator, ConstraintUnit, OperatorMetadata};
use schemars::JsonSchema;
use semantic_memory_forge::{ConstraintSeedKind, ExportRecordSemanticsV3};
use serde::{Deserialize, Serialize};
use stack_ids::{ConstraintId, ContentDigest, OracleSliceId, RegionDigestId, RegionId, ScopeKey};
use std::collections::{BTreeMap, BTreeSet};

fn compiler_operator() -> OperatorMetadata {
    constraint_compiler_operator()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompilerPolicy {
    pub policy_version: String,
    pub include_hyperedges: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintDegradation {
    MissingClaimFamily,
    MissingAssertionGroup,
    MissingRelationGroup,
    ThinExport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InferenceNode {
    pub node_id: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InferenceHyperedge {
    pub edge_id: String,
    pub member_node_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InvalidationCone {
    pub source_node_id: String,
    pub affected_node_ids: Vec<String>,
    pub affected_hyperedge_ids: Vec<String>,
    pub affected_constraint_ids: Vec<ConstraintId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OracleSliceCandidate {
    pub oracle_slice_id: OracleSliceId,
    pub node_ids: Vec<String>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum GraphSurfaceKind {
    Storage,
    Retrieval,
    Inference,
    Repair,
    Control,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompilationBoundary {
    pub from_surface: GraphSurfaceKind,
    pub to_surface: GraphSurfaceKind,
    pub artifact_families: Vec<String>,
    pub deterministic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GraphGeometryManifest {
    pub surfaces: Vec<GraphSurfaceKind>,
    pub compilation_boundaries: Vec<CompilationBoundary>,
    pub no_silent_collapse: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompiledRegion {
    pub region_id: RegionId,
    pub region_digest_id: RegionDigestId,
    pub node_ids: Vec<String>,
    pub hyperedge_ids: Vec<String>,
    pub constraint_ids: Vec<ConstraintId>,
    pub bounded_default_unit_of_work: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompileOutput {
    pub graph_hash: ContentDigest,
    pub scope_key: ScopeKey,
    pub geometry_manifest: GraphGeometryManifest,
    pub nodes: Vec<InferenceNode>,
    pub hyperedges: Vec<InferenceHyperedge>,
    pub constraints: Vec<ConstraintUnit>,
    pub regions: Vec<CompiledRegion>,
    pub invalidation_cones: Vec<InvalidationCone>,
    pub degradations: Vec<ConstraintDegradation>,
    pub oracle_candidates: Vec<OracleSliceCandidate>,
}

/// Compiles an importer batch into the bounded inference graph consumed by the kernel lane.
pub fn compile_batch(batch: &ProjectionImportBatchV3, policy: &CompilerPolicy) -> CompileOutput {
    let compiler = compiler_operator();
    let mut nodes = BTreeMap::<String, InferenceNode>::new();
    let mut hyperedge_members = BTreeMap::<String, BTreeSet<String>>::new();
    let mut hyperedge_kinds = BTreeMap::<String, String>::new();
    let mut degradations = Vec::new();
    let mut nuisance_constraints = Vec::new();
    let auxiliary_records_stay_thin = batch.records.iter().any(|record| {
        record.semantics.as_ref().is_some_and(|semantics| {
            semantics.export_confidence_class
                == semantic_memory_forge::ExportConfidenceClass::ThinExport
        })
    });

    for rich_record in &batch.records {
        if !matches!(
            rich_record.record,
            ImportProjectionRecord::ClaimVersion(_) | ImportProjectionRecord::RelationVersion(_)
        ) {
            if rich_record.semantics.is_none()
                && (matches!(rich_record.record, ImportProjectionRecord::Episode(_))
                    || (auxiliary_records_stay_thin
                        && matches!(
                            rich_record.record,
                            ImportProjectionRecord::EntityAlias(_)
                                | ImportProjectionRecord::EvidenceRef(_)
                        )))
            {
                degradations.push(ConstraintDegradation::ThinExport);
            }
            continue;
        }

        let node_id = stable_node_id(&rich_record.record);
        let kind = record_kind(&rich_record.record).to_string();
        nodes.insert(
            node_id.clone(),
            InferenceNode {
                node_id: node_id.clone(),
                kind,
            },
        );

        match &rich_record.semantics {
            Some(semantics) => {
                materialize_nuisance_state(
                    &mut nodes,
                    &mut hyperedge_members,
                    &mut hyperedge_kinds,
                    &mut nuisance_constraints,
                    &node_id,
                    semantics,
                    policy.include_hyperedges,
                );
                match rich_record.record {
                    ImportProjectionRecord::ClaimVersion(_) => {
                        if policy.include_hyperedges {
                            if let Some(group_id) = &semantics.assertion_group_id {
                                let edge_id = format!("assertion_group:{group_id}");
                                hyperedge_members
                                    .entry(edge_id.clone())
                                    .or_default()
                                    .insert(node_id.clone());
                                hyperedge_kinds.entry(edge_id).or_insert_with(|| {
                                    constraint_seed_name(semantics.constraint_seed_kind.as_ref())
                                });
                            } else {
                                degradations.push(ConstraintDegradation::MissingAssertionGroup);
                            }
                        }
                    }
                    ImportProjectionRecord::RelationVersion(_) => {
                        if policy.include_hyperedges {
                            if let Some(group_id) = &semantics.relation_group_id {
                                let edge_id = format!("relation_group:{group_id}");
                                hyperedge_members
                                    .entry(edge_id.clone())
                                    .or_default()
                                    .insert(node_id.clone());
                                hyperedge_kinds.entry(edge_id).or_insert_with(|| {
                                    constraint_seed_name(semantics.constraint_seed_kind.as_ref())
                                });
                            } else {
                                degradations.push(ConstraintDegradation::MissingRelationGroup);
                            }
                        }
                    }
                    ImportProjectionRecord::Episode(_)
                    | ImportProjectionRecord::EntityAlias(_)
                    | ImportProjectionRecord::EvidenceRef(_) => {}
                }
            }
            None => {
                degradations.push(ConstraintDegradation::ThinExport);
            }
        }

        if let Some(semantics) = &rich_record.semantics {
            if semantics.claim_family_id.is_none()
                && matches!(rich_record.record, ImportProjectionRecord::ClaimVersion(_))
            {
                degradations.push(ConstraintDegradation::MissingClaimFamily);
            }
            if semantics.relation_group_id.is_none()
                && matches!(
                    rich_record.record,
                    ImportProjectionRecord::RelationVersion(_)
                )
            {
                degradations.push(ConstraintDegradation::MissingRelationGroup);
            }
        }
    }

    let mut nodes: Vec<_> = nodes.into_values().collect();
    nodes.sort_by(|a, b| a.node_id.cmp(&b.node_id));

    let mut hyperedges = Vec::new();
    let mut constraints = Vec::new();
    for (edge_id, members) in hyperedge_members {
        let mut member_node_ids: Vec<_> = members.into_iter().collect();
        member_node_ids.sort();

        hyperedges.push(InferenceHyperedge {
            edge_id: edge_id.clone(),
            member_node_ids: member_node_ids.clone(),
        });
        constraints.push(ConstraintUnit {
            constraint_id: ConstraintId::new(format!("constraint:{edge_id}")),
            kind: hyperedge_kinds
                .get(&edge_id)
                .cloned()
                .unwrap_or_else(|| "group".into()),
            variable_ids: member_node_ids,
            operator_id: compiler.operator_id.clone(),
        });
    }
    constraints.extend(nuisance_constraints);

    hyperedges.sort_by(|a, b| a.edge_id.cmp(&b.edge_id));
    constraints.sort_by(|a, b| a.constraint_id.as_str().cmp(b.constraint_id.as_str()));
    let geometry_manifest = graph_geometry_manifest();
    let regions = compile_regions(&nodes, &hyperedges, &constraints);
    let invalidation_cones = build_invalidation_cones(&hyperedges, &constraints);
    degradations.sort();
    degradations.dedup();

    let oracle_candidate_node_ids = nodes
        .iter()
        .filter(|node| node.kind != "nuisance_state")
        .map(|node| node.node_id.clone())
        .collect::<Vec<_>>();
    let oracle_candidates = if degradations.is_empty()
        && !oracle_candidate_node_ids.is_empty()
        && oracle_candidate_node_ids.len() <= 8
    {
        vec![OracleSliceCandidate {
            oracle_slice_id: OracleSliceId::new(format!("oracle:{}", batch.source_envelope_id)),
            node_ids: oracle_candidate_node_ids,
        }]
    } else {
        vec![]
    };

    let graph_hash = ContentDigest::compute_json(&(
        policy,
        &batch.scope_key,
        &geometry_manifest,
        &nodes,
        &hyperedges,
        &constraints,
        &regions,
        &invalidation_cones,
        &degradations,
    ))
    .expect("graph hash serialization is infallible for known types");

    CompileOutput {
        graph_hash,
        scope_key: batch.scope_key.clone(),
        geometry_manifest,
        nodes,
        hyperedges,
        constraints,
        regions,
        invalidation_cones,
        degradations,
        oracle_candidates,
    }
}

fn graph_geometry_manifest() -> GraphGeometryManifest {
    GraphGeometryManifest {
        surfaces: vec![
            GraphSurfaceKind::Storage,
            GraphSurfaceKind::Retrieval,
            GraphSurfaceKind::Inference,
            GraphSurfaceKind::Repair,
            GraphSurfaceKind::Control,
        ],
        compilation_boundaries: vec![
            CompilationBoundary {
                from_surface: GraphSurfaceKind::Storage,
                to_surface: GraphSurfaceKind::Retrieval,
                artifact_families: vec!["candidate_expansion".into(), "query_scope".into()],
                deterministic: true,
            },
            CompilationBoundary {
                from_surface: GraphSurfaceKind::Retrieval,
                to_surface: GraphSurfaceKind::Inference,
                artifact_families: vec!["candidate_claims".into(), "evidence_refs".into()],
                deterministic: true,
            },
            CompilationBoundary {
                from_surface: GraphSurfaceKind::Inference,
                to_surface: GraphSurfaceKind::Repair,
                artifact_families: vec![
                    "syndrome".into(),
                    "witness".into(),
                    "invalidation_manifest".into(),
                ],
                deterministic: true,
            },
            CompilationBoundary {
                from_surface: GraphSurfaceKind::Control,
                to_surface: GraphSurfaceKind::Inference,
                artifact_families: vec!["region_budget".into(), "execution_budget".into()],
                deterministic: true,
            },
            CompilationBoundary {
                from_surface: GraphSurfaceKind::Inference,
                to_surface: GraphSurfaceKind::Control,
                artifact_families: vec!["control_receipt".into(), "residual".into()],
                deterministic: true,
            },
        ],
        no_silent_collapse: true,
    }
}

fn compile_regions(
    nodes: &[InferenceNode],
    hyperedges: &[InferenceHyperedge],
    constraints: &[ConstraintUnit],
) -> Vec<CompiledRegion> {
    let mut node_neighbors = BTreeMap::<String, BTreeSet<String>>::new();
    let mut edges_by_node = BTreeMap::<String, BTreeSet<String>>::new();

    for node in nodes {
        node_neighbors.entry(node.node_id.clone()).or_default();
        edges_by_node.entry(node.node_id.clone()).or_default();
    }

    for hyperedge in hyperedges {
        for node_id in &hyperedge.member_node_ids {
            edges_by_node
                .entry(node_id.clone())
                .or_default()
                .insert(hyperedge.edge_id.clone());
            for peer in &hyperedge.member_node_ids {
                if peer != node_id {
                    node_neighbors
                        .entry(node_id.clone())
                        .or_default()
                        .insert(peer.clone());
                }
            }
        }
    }

    let mut visited = BTreeSet::new();
    let constraint_lookup = constraints
        .iter()
        .map(|constraint| {
            (
                constraint
                    .constraint_id
                    .as_str()
                    .strip_prefix("constraint:")
                    .unwrap_or(constraint.constraint_id.as_str())
                    .to_string(),
                constraint.constraint_id.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut regions = Vec::new();

    for node in nodes {
        if !visited.insert(node.node_id.clone()) {
            continue;
        }

        let mut queue = vec![node.node_id.clone()];
        let mut node_ids = BTreeSet::new();
        let mut hyperedge_ids = BTreeSet::new();
        let mut constraint_ids = BTreeSet::new();

        while let Some(current) = queue.pop() {
            node_ids.insert(current.clone());

            if let Some(edge_ids) = edges_by_node.get(&current) {
                for edge_id in edge_ids {
                    hyperedge_ids.insert(edge_id.clone());
                    if let Some(constraint_id) = constraint_lookup.get(edge_id) {
                        constraint_ids.insert(constraint_id.clone());
                    }
                }
            }

            if let Some(neighbors) = node_neighbors.get(&current) {
                for neighbor in neighbors {
                    if visited.insert(neighbor.clone()) {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }

        let node_ids = node_ids.into_iter().collect::<Vec<_>>();
        let hyperedge_ids = hyperedge_ids.into_iter().collect::<Vec<_>>();
        let constraint_ids = constraint_ids.into_iter().collect::<Vec<_>>();
        let region_digest = ContentDigest::compute_json(&(&node_ids, &hyperedge_ids, &constraint_ids))
            .expect("region digest serialization is infallible for known types");
        regions.push(CompiledRegion {
            region_id: RegionId::new(format!("region:{}", region_digest.hex())),
            region_digest_id: RegionDigestId::new(format!("region-digest:{}", region_digest.hex())),
            node_ids,
            hyperedge_ids,
            constraint_ids,
            bounded_default_unit_of_work: true,
        });
    }

    regions.sort_by(|a, b| a.region_id.as_str().cmp(b.region_id.as_str()));
    regions
}

fn constraint_seed_name(kind: Option<&ConstraintSeedKind>) -> String {
    match kind {
        Some(ConstraintSeedKind::Hyperedge) => "hyperedge".into(),
        Some(ConstraintSeedKind::MutualExclusion) => "mutual_exclusion".into(),
        Some(ConstraintSeedKind::TemporalCoherence) => "temporal_coherence".into(),
        Some(ConstraintSeedKind::CausalRefutation) => "causal_refutation".into(),
        Some(ConstraintSeedKind::NuisanceDisclosure) => "nuisance_disclosure".into(),
        None => "group".into(),
    }
}

fn materialize_nuisance_state(
    nodes: &mut BTreeMap<String, InferenceNode>,
    hyperedge_members: &mut BTreeMap<String, BTreeSet<String>>,
    hyperedge_kinds: &mut BTreeMap<String, String>,
    nuisance_constraints: &mut Vec<ConstraintUnit>,
    source_node_id: &str,
    semantics: &ExportRecordSemanticsV3,
    include_hyperedges: bool,
) {
    let Some(nuisance_key) = nuisance_node_id(semantics) else {
        return;
    };

    nodes
        .entry(nuisance_key.clone())
        .or_insert_with(|| InferenceNode {
            node_id: nuisance_key.clone(),
            kind: "nuisance_state".into(),
        });

    let edge_id = format!("nuisance_edge:{source_node_id}:{nuisance_key}");
    if include_hyperedges {
        hyperedge_members
            .entry(edge_id.clone())
            .or_default()
            .extend([source_node_id.to_string(), nuisance_key.clone()]);
        hyperedge_kinds
            .entry(edge_id.clone())
            .or_insert_with(|| "nuisance_disclosure".into());
        return;
    }

    nuisance_constraints.push(ConstraintUnit {
        constraint_id: ConstraintId::new(format!("constraint:{edge_id}")),
        kind: "nuisance_disclosure".into(),
        variable_ids: vec![source_node_id.to_string(), nuisance_key],
        operator_id: compiler_operator().operator_id,
    });
}

fn nuisance_node_id(semantics: &ExportRecordSemanticsV3) -> Option<String> {
    fn hash_sorted_values(builder: &mut blake3::Hasher, values: &[String]) {
        let mut normalized = values.to_vec();
        normalized.sort();
        normalized.dedup();
        for value in normalized {
            builder.update(value.as_bytes());
        }
    }

    if let Some(version) = semantics.comparability_snapshot_version.as_ref() {
        return Some(format!("nuisance:comparability:{version}"));
    }

    let snapshot = semantics.nuisance_snapshot.as_ref()?;
    let mut builder = blake3::Hasher::new();
    if let Some(value) = snapshot.environment_fingerprint.as_deref() {
        builder.update(value.as_bytes());
    }
    if let Some(value) = snapshot.toolchain_version.as_deref() {
        builder.update(value.as_bytes());
    }
    if let Some(value) = snapshot.dependency_set_hash.as_deref() {
        builder.update(value.as_bytes());
    }
    hash_sorted_values(&mut builder, &snapshot.scope_mismatch_markers);
    hash_sorted_values(&mut builder, &snapshot.measurement_notes);
    hash_sorted_values(&mut builder, &snapshot.selection_bias_markers);
    Some(format!("nuisance:snapshot:{}", builder.finalize().to_hex()))
}

fn build_invalidation_cones(
    hyperedges: &[InferenceHyperedge],
    constraints: &[ConstraintUnit],
) -> Vec<InvalidationCone> {
    let mut edges_by_node = BTreeMap::<String, BTreeSet<String>>::new();
    for hyperedge in hyperedges {
        for node_id in &hyperedge.member_node_ids {
            edges_by_node
                .entry(node_id.clone())
                .or_default()
                .insert(hyperedge.edge_id.clone());
        }
    }

    let constraints_by_edge: BTreeMap<_, _> = constraints
        .iter()
        .map(|constraint| {
            (
                constraint
                    .constraint_id
                    .as_str()
                    .strip_prefix("constraint:")
                    .unwrap_or(constraint.constraint_id.as_str())
                    .to_string(),
                constraint.constraint_id.clone(),
            )
        })
        .collect();

    let mut cones = Vec::new();
    for (source_node_id, edge_ids) in edges_by_node {
        let mut affected_node_ids = BTreeSet::new();
        let mut affected_constraint_ids = BTreeSet::new();

        for edge_id in &edge_ids {
            if let Some(hyperedge) = hyperedges.iter().find(|edge| edge.edge_id == *edge_id) {
                affected_node_ids.extend(hyperedge.member_node_ids.iter().cloned());
            }
            if let Some(constraint_id) = constraints_by_edge.get(edge_id) {
                affected_constraint_ids.insert(constraint_id.clone());
            }
        }

        cones.push(InvalidationCone {
            source_node_id,
            affected_node_ids: affected_node_ids.into_iter().collect(),
            affected_hyperedge_ids: edge_ids.into_iter().collect(),
            affected_constraint_ids: affected_constraint_ids.into_iter().collect(),
        });
    }
    cones.sort_by(|a, b| a.source_node_id.cmp(&b.source_node_id));
    cones
}

fn stable_node_id(record: &ImportProjectionRecord) -> String {
    match record {
        ImportProjectionRecord::ClaimVersion(claim) => claim.claim_version_id.to_string(),
        ImportProjectionRecord::RelationVersion(relation) => {
            relation.relation_version_id.to_string()
        }
        ImportProjectionRecord::Episode(episode) => episode.episode_id.to_string(),
        ImportProjectionRecord::EntityAlias(alias) => {
            format!("alias:{}:{}", alias.canonical_entity_id, alias.alias_text)
        }
        ImportProjectionRecord::EvidenceRef(evidence) => {
            format!("evidence:{}:{}", evidence.claim_id, evidence.fetch_handle)
        }
    }
}

fn record_kind(record: &ImportProjectionRecord) -> &'static str {
    match record {
        ImportProjectionRecord::ClaimVersion(_) => "claim_version",
        ImportProjectionRecord::RelationVersion(_) => "relation_version",
        ImportProjectionRecord::Episode(_) => "episode",
        ImportProjectionRecord::EntityAlias(_) => "entity_alias",
        ImportProjectionRecord::EvidenceRef(_) => "evidence_ref",
    }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
