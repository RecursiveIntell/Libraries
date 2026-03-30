#![allow(deprecated)]

//! Bundle export seam for the canonical Forge -> bridge -> memory lane.
//!
//! Forge is not a durable knowledge store. It emits Forge-owned export
//! envelopes, persists export receipts for idempotency, and leaves projection
//! import semantics to `forge-memory-bridge` and `semantic-memory`.

use semantic_memory_forge::{
    CausalRoleHint, ConstraintSeedKind, ExportAuthority, ExportClaim, ExportEntityAlias,
    ExportEnvelopeV1, ExportEnvelopeV2, ExportEnvelopeV3, ExportEpisode, ExportEvidenceRef,
    ExportRecord, ExportRecordSemanticsV3, ExportRecordV3, ExportRelation, ForgeExportMeta,
    EXPORT_ENVELOPE_V1_SCHEMA, EXPORT_ENVELOPE_V2_SCHEMA, EXPORT_ENVELOPE_V3_SCHEMA,
};
use serde::{Deserialize, Serialize};
use stack_ids::{
    AssertionGroupId, ClaimFamilyId, ClaimId, ClaimVersionId, DigestBuilder, EntityId, EnvelopeId,
    EpisodeId, RelationGroupId, RelationVersionId, ScopeKey, TraceCtx,
};

use crate::error::ForgeResult;
use crate::lab::evidence::{EffectRelationLineageSource, ExperimentEvidenceBundle};
use crate::store::ForgeStore;

/// Current rendering version for bundle exports.
pub const RENDERING_VERSION: u32 = 3;

/// Deterministic rendered representation of an `ExperimentEvidenceBundle`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeExport {
    /// Deterministic key for this export receipt and envelope identity.
    pub export_key: String,
    /// The bundle ID this export was derived from.
    pub bundle_id: String,
    /// Rendering version used.
    pub rendering_version: u32,
    /// Legacy namespace routing input.
    pub namespace: String,
    /// Bundle metadata (JSON-encodable).
    pub meta: serde_json::Value,
    /// Bundle content text for embedding or audit trails.
    pub content: String,
}

struct CanonicalExportEnvelopeMaterial {
    envelope_id: EnvelopeId,
    source_authority: String,
    scope_key: ScopeKey,
    trace_ctx: Option<TraceCtx>,
    exported_at: String,
    records: Vec<ExportRecord>,
    export_meta: ForgeExportMeta,
    evidence_bundle: semantic_memory_forge::EvidenceBundle,
}

impl EpisodeExport {
    /// Create an export rendering from an `ExperimentEvidenceBundle`.
    ///
    /// The `export_key` is deterministic: blake3(bundle_id + rendering_version + namespace).
    pub fn from_bundle(bundle: &ExperimentEvidenceBundle, namespace: &str) -> Self {
        let export_key = compute_export_key(&bundle.bundle_id, RENDERING_VERSION, namespace);
        let meta = bundle.to_episode_meta();
        let content = bundle.to_episode_content();

        Self {
            export_key,
            bundle_id: bundle.bundle_id.clone(),
            rendering_version: RENDERING_VERSION,
            namespace: namespace.to_string(),
            meta,
            content,
        }
    }

    /// Persist an export receipt to the store.
    ///
    /// This is idempotent: exporting the same bundle twice produces no change.
    pub fn persist_receipt(
        &self,
        store: &ForgeStore,
        write_through_ok: Option<bool>,
    ) -> ForgeResult<bool> {
        store.insert_export_receipt(
            &self.export_key,
            &self.bundle_id,
            self.rendering_version,
            &self.namespace,
            write_through_ok,
        )
    }

    /// Check if this export has already been receipted.
    pub fn already_exported(&self, store: &ForgeStore) -> ForgeResult<bool> {
        store.has_export_receipt(&self.export_key)
    }

    /// Convert this rendering into Forge's compatibility export envelope.
    ///
    /// The compatibility path is:
    /// `ExportEnvelopeV1 -> forge-memory-bridge -> ProjectionImportBatchV1
    /// -> semantic-memory import transaction`.
    ///
    /// Phase status: migration-only
    /// Removal condition: remove when all consumers have migrated to `to_export_envelope_v3()`
    #[deprecated(
        since = "0.1.0",
        note = "to_export_envelope_v1() is compatibility-only. Use to_export_envelope_v3() for the canonical export lane."
    )]
    pub fn to_export_envelope_v1(
        &self,
        bundle: &ExperimentEvidenceBundle,
    ) -> ForgeResult<ExportEnvelopeV1> {
        let material = self.build_canonical_export_material(bundle)?;
        let content_digest = ExportEnvelopeV1::compute_digest(
            &material.source_authority,
            &material.scope_key,
            &material.records,
        )?;

        Ok(ExportEnvelopeV1 {
            envelope_id: material.envelope_id,
            schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
            content_digest,
            source_authority: material.source_authority,
            scope_key: material.scope_key,
            trace_ctx: material.trace_ctx,
            exported_at: material.exported_at,
            records: material.records,
        })
    }

    /// Convert this rendering into Forge's compatibility-only V2 export envelope.
    ///
    /// This lane is retained for legacy consumers and tests only. New users
    /// should call [`to_export_envelope_v3`](Self::to_export_envelope_v3) to
    /// get canonical kernel-export semantics.
    ///
    /// Phase status: migration-only
    /// Removal condition: remove when all consumers have migrated to `to_export_envelope_v3()`
    #[allow(deprecated)]
    #[deprecated(
        since = "0.2.0",
        note = "to_export_envelope_v2() is compatibility-only. Use to_export_envelope_v3() for the canonical export lane."
    )]
    pub fn to_export_envelope_v2(
        &self,
        bundle: &ExperimentEvidenceBundle,
    ) -> ForgeResult<ExportEnvelopeV2> {
        let material = self.build_canonical_export_material(bundle)?;
        let content_digest = ExportEnvelopeV2::compute_digest(
            &material.source_authority,
            &material.scope_key,
            &material.records,
            Some(&material.export_meta),
            Some(&material.evidence_bundle),
        )?;

        Ok(ExportEnvelopeV2 {
            envelope_id: material.envelope_id,
            schema_version: EXPORT_ENVELOPE_V2_SCHEMA.into(),
            content_digest,
            source_authority: material.source_authority,
            scope_key: material.scope_key,
            trace_ctx: material.trace_ctx,
            exported_at: material.exported_at,
            export_meta: Some(material.export_meta),
            evidence_bundle: Some(material.evidence_bundle),
            records: material.records,
        })
    }

    /// Convert this rendering into Forge's canonical V3 export envelope.
    ///
    /// This is the production lane. It derives `ExportEnvelopeV3` directly
    /// from the local bundle model and enriches records with concrete
    /// semantics where the source bundle supplies them.
    pub fn to_export_envelope_v3(
        &self,
        bundle: &ExperimentEvidenceBundle,
    ) -> ForgeResult<ExportEnvelopeV3> {
        let material = self.build_canonical_export_material(bundle)?;
        let records = material
            .records
            .into_iter()
            .map(|record| {
                let semantics = record_semantics_v3(bundle, &record);
                ExportRecordV3 { record, semantics }
            })
            .collect::<Vec<_>>();

        let content_digest = ExportEnvelopeV3::compute_digest(
            &material.source_authority,
            &material.scope_key,
            &records,
            Some(&material.export_meta),
            Some(&material.evidence_bundle),
        )?;

        Ok(ExportEnvelopeV3 {
            envelope_id: material.envelope_id,
            schema_version: EXPORT_ENVELOPE_V3_SCHEMA.into(),
            content_digest,
            source_authority: material.source_authority,
            scope_key: material.scope_key,
            trace_ctx: material.trace_ctx,
            exported_at: material.exported_at,
            export_meta: Some(material.export_meta),
            evidence_bundle: Some(material.evidence_bundle),
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
        })
    }

    fn build_canonical_export_material(
        &self,
        bundle: &ExperimentEvidenceBundle,
    ) -> ForgeResult<CanonicalExportEnvelopeMaterial> {
        let mut canonical_bundle = bundle.to_canonical_evidence_bundle();
        let scope_key = ScopeKey::from_legacy_namespace(&self.namespace);
        let claim_id = forge_bundle_claim_id(&self.export_key);
        let claim_version_id = forge_bundle_claim_version_id(&self.export_key, bundle, &claim_id);
        let episode_id = EpisodeId::new(format!("forge-bundle-episode:{}", self.export_key));
        let trace_ctx = bundle
            .trace_id
            .as_ref()
            .map(|trace_id| TraceCtx::from_trace_id(trace_id.as_str()));
        let exported_at = chrono::Utc::now().to_rfc3339();
        let mut records = Vec::new();

        records.push(ExportRecord::Claim(ExportClaim {
            claim_id: Some(claim_id.clone()),
            claim_version_id: Some(claim_version_id.clone()),
            subject_entity_id: canonical_subject_entity_id(bundle),
            predicate: "forge_evidence_bundle".into(),
            object_anchor: serde_json::json!({
                "bundle_id": self.bundle_id,
                "candidate_id": bundle.candidate_id,
                "eval_id": bundle.eval_id,
                "rendering_version": self.rendering_version,
            }),
            valid_from: bundle_valid_from(bundle),
            valid_to: None,
            confidence: cast_confidence(bundle.scores.weighted_total),
            content: self.content.clone(),
            projection_family: "forge_verification".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: bundle.supersedes_claim_version_id.clone(),
            metadata: Some(bundle_claim_metadata(self, bundle, &canonical_bundle)),
        }));

        records.extend(
            bundle_relation_records(bundle, &claim_id, Some(&episode_id), &scope_key)
                .into_iter()
                .map(ExportRecord::Relation),
        );

        records.extend(
            bundle_verification_trial_records(bundle, &claim_id, Some(&episode_id), &scope_key)
                .into_iter()
                .map(ExportRecord::Relation),
        );

        records.extend(
            bundle_refutation_artifact_records(bundle, &claim_id, Some(&episode_id), &scope_key)
                .into_iter()
                .map(ExportRecord::Relation),
        );

        records.extend(
            bundle_alias_records(bundle, &scope_key)
                .into_iter()
                .map(ExportRecord::EntityAlias),
        );

        records.push(ExportRecord::Episode(ExportEpisode {
            episode_id: Some(episode_id),
            document_id: bundle
                .run_id
                .clone()
                .unwrap_or_else(|| format!("forge_bundle:{}", self.bundle_id)),
            cause_ids: bundle_cause_ids(bundle),
            effect_type: bundle_effect_type(bundle),
            outcome: bundle
                .outcome
                .clone()
                .unwrap_or_else(|| default_outcome(bundle)),
            confidence: cast_confidence(bundle.scores.weighted_total),
            experiment_id: bundle.run_id.clone(),
            metadata: Some(serde_json::json!({
                "bundle_id": self.bundle_id,
                "candidate_id": bundle.candidate_id,
                "eval_id": bundle.eval_id,
                "version_id": bundle.version_id,
                "attempt_id": bundle.attempt_id,
                "claim_strength": format!("{}", bundle.claim_strength),
                "assessment": bundle.assessment,
                "bundle_scope": bundle.bundle_scope,
                "treatment": bundle.treatment,
                "covariates": bundle.covariates,
                "known_threats": bundle.known_threats,
                "primary_effect": bundle.primary_effect,
                "warning_count": bundle.warnings.len(),
                "receipt_count": bundle.receipts.len(),
                "verification_trial_count": bundle.verification_trials.len(),
                "refutation_artifact_count": bundle.refutation_artifacts.len(),
                "verification_trials": bundle
                    .verification_trials
                    .iter()
                    .map(|trial| serde_json::json!({
                        "attempt_id": trial.attempt_id,
                        "trial_id": trial.trial_id,
                        "baseline_or_patch": trial.baseline_or_patch,
                        "completed": trial.completed,
                    }))
                    .collect::<Vec<_>>(),
                "refutation_artifacts": bundle
                    .refutation_artifacts
                    .iter()
                    .map(|artifact| serde_json::json!({
                        "artifact_id": artifact.artifact_id,
                        "artifact_type": format!("{:?}", artifact.artifact_type),
                        "outcome": format!("{:?}", artifact.outcome),
                    }))
                    .collect::<Vec<_>>(),
                "experiment_diff_counts": bundle.experiment_diff.as_ref().map(|diff| serde_json::json!({
                    "effects": diff.effects.len(),
                    "regressions": diff.regressions,
                    "improvements": diff.improvements,
                    "stable_failures": diff.stable_failures,
                    "stable_passes": diff.stable_passes,
                    "statistically_meaningful": diff.statistically_meaningful,
                    "sample_warning": diff.sample_warning,
                })),
            })),
        }));

        records.extend(
            bundle_evidence_refs(bundle, &claim_id, &claim_version_id)
                .into_iter()
                .map(ExportRecord::EvidenceRef),
        );

        canonical_bundle.claim_ids = vec![claim_id];

        Ok(CanonicalExportEnvelopeMaterial {
            envelope_id: EnvelopeId::new(self.export_key.clone()),
            source_authority: ExportAuthority::Forge.as_str().into(),
            scope_key,
            trace_ctx,
            exported_at: exported_at.clone(),
            records,
            export_meta: ForgeExportMeta {
                authority: ExportAuthority::Forge,
                run_id: bundle.run_id.clone(),
                direct_write: false,
                comparability_snapshot_version: bundle.bundle_scope.as_ref().map(|scope| {
                    format!(
                        "{}:{}:{}",
                        scope.workload_id, scope.backend_family, scope.timeout_class
                    )
                }),
                exported_at,
            },
            evidence_bundle: canonical_bundle,
        })
    }
}

fn relation_group_id_for_record(
    bundle: &ExperimentEvidenceBundle,
    relation: &ExportRelation,
) -> RelationGroupId {
    if let Some(claim_id) = relation.source_claim_id.as_ref() {
        RelationGroupId::new(format!(
            "rel-bundle:{}:claim:{}:predicate:{}",
            bundle.bundle_id, claim_id, relation.predicate
        ))
    } else {
        RelationGroupId::new(format!(
            "rel-bundle:{}:subject:{}:predicate:{}",
            bundle.bundle_id, relation.subject_entity_id, relation.predicate
        ))
    }
}

fn constraint_seed_kind_for_predicate(predicate: &str) -> ConstraintSeedKind {
    if predicate.contains("verification_refutation") || predicate.contains("verification_trial") {
        ConstraintSeedKind::CausalRefutation
    } else if predicate.contains("hypothesis_edge") {
        ConstraintSeedKind::MutualExclusion
    } else if predicate.contains("temporal") {
        ConstraintSeedKind::TemporalCoherence
    } else {
        ConstraintSeedKind::Hyperedge
    }
}

fn record_semantics_v3(
    bundle: &ExperimentEvidenceBundle,
    record: &ExportRecord,
) -> Option<ExportRecordSemanticsV3> {
    let projection_visibility_class = semantic_memory_forge::ProjectionVisibilityClass::Standard;
    let export_confidence_class = if bundle.assessment.is_some() {
        semantic_memory_forge::ExportConfidenceClass::Reviewed
    } else {
        semantic_memory_forge::ExportConfidenceClass::ThinExport
    };

    match record {
        ExportRecord::Claim(claim) => {
            let mut derivation_seed_ids = Vec::new();
            if let Some(claim_version_id) = claim.claim_version_id.as_ref() {
                derivation_seed_ids.push(format!("claim_version:{claim_version_id}"));
            }
            if let Some(supersedes) = claim.supersedes_claim_version_id.as_ref() {
                derivation_seed_ids.push(format!("supersedes_claim_version:{supersedes}"));
            }
            let mut semantics = ExportRecordSemanticsV3 {
                claim_family_id: Some(ClaimFamilyId::new(bundle.candidate_id.clone())),
                assertion_group_id: Some(AssertionGroupId::new(format!(
                    "claim:{}",
                    claim.subject_entity_id
                ))),
                relation_group_id: None,
                joint_evidence_group_id: None,
                // Claims are grouped through claim/assertion identities. Constraint
                // seeds are emitted by relation/episode records, so absence here is
                // deliberate rather than accidental.
                constraint_seed_kind: None,
                treatment_hint: None,
                outcome_hint: None,
                confounder_hint: None,
                instrument_hint: None,
                effect_modifier_hint: None,
                contradiction_candidate_group_id: None,
                mutual_exclusion_group_id: None,
                comparability_snapshot_version: bundle.bundle_scope.as_ref().map(|scope| {
                    format!(
                        "{}:{}:{}",
                        scope.workload_id, scope.backend_family, scope.timeout_class
                    )
                }),
                nuisance_snapshot: None,
                projection_visibility_class,
                export_confidence_class,
                derivation_seed_ids,
                review_priority_hint: None,
            };

            if claim.supersedes_claim_version_id.is_some() {
                semantics.review_priority_hint = Some("supersedes_known_claim_version".into());
            }

            Some(semantics)
        }
        ExportRecord::Relation(relation) => Some(ExportRecordSemanticsV3 {
            claim_family_id: None,
            assertion_group_id: None,
            relation_group_id: Some(relation_group_id_for_record(bundle, relation)),
            joint_evidence_group_id: None,
            constraint_seed_kind: Some(constraint_seed_kind_for_predicate(&relation.predicate)),
            treatment_hint: None,
            outcome_hint: None,
            confounder_hint: None,
            instrument_hint: None,
            effect_modifier_hint: None,
            contradiction_candidate_group_id: None,
            mutual_exclusion_group_id: None,
            comparability_snapshot_version: None,
            nuisance_snapshot: None,
            projection_visibility_class,
            export_confidence_class,
            derivation_seed_ids: {
                let mut derivation_seed_ids = Vec::new();
                if let Some(relation_version_id) = relation.relation_version_id.as_ref() {
                    derivation_seed_ids.push(format!("relation_version:{relation_version_id}"));
                }
                if let Some(supersedes) = relation.supersedes_relation_version_id.as_ref() {
                    derivation_seed_ids.push(format!("supersedes_relation_version:{supersedes}"));
                }
                if derivation_seed_ids.is_empty() {
                    derivation_seed_ids.push(relation.predicate.clone());
                }
                derivation_seed_ids
            },
            review_priority_hint: None,
        }),
        ExportRecord::Episode(_) => Some(ExportRecordSemanticsV3 {
            claim_family_id: None,
            assertion_group_id: None,
            relation_group_id: None,
            joint_evidence_group_id: None,
            constraint_seed_kind: Some(ConstraintSeedKind::TemporalCoherence),
            treatment_hint: Some(CausalRoleHint::Treatment),
            outcome_hint: Some(CausalRoleHint::Outcome),
            confounder_hint: None,
            instrument_hint: None,
            effect_modifier_hint: None,
            contradiction_candidate_group_id: None,
            mutual_exclusion_group_id: None,
            comparability_snapshot_version: None,
            nuisance_snapshot: None,
            projection_visibility_class,
            export_confidence_class,
            derivation_seed_ids: vec![bundle.bundle_id.clone()],
            review_priority_hint: None,
        }),
        ExportRecord::EntityAlias(_) | ExportRecord::EvidenceRef(_) => None,
    }
}

fn bundle_claim_metadata(
    export: &EpisodeExport,
    bundle: &ExperimentEvidenceBundle,
    canonical_bundle: &semantic_memory_forge::EvidenceBundle,
) -> serde_json::Value {
    let promotion_state = canonical_bundle
        .verification_summary
        .as_ref()
        .map(|summary| summary.promotion_state.clone())
        .or_else(|| bundle.promotion_state.clone());
    serde_json::json!({
        "forge_bundle_id": export.bundle_id,
        "forge_rendering_version": export.rendering_version,
        "forge_bundle_meta": export.meta.clone(),
        "evidence_bundle_id": canonical_bundle.id.as_str(),
        "estimator_meta": canonical_bundle.estimator_meta,
        "comparability_snapshot": canonical_bundle.comparability_snapshot,
        "comparability_snapshot_version": canonical_bundle.comparability_snapshot_version,
        "verification_summary": canonical_bundle.verification_summary,
        "refutation_artifacts": canonical_bundle.refutation_artifacts,
        "verification_trial_family": canonical_bundle.verification_trials,
        "promotion_state": promotion_state,
    })
}

fn canonical_subject_entity_id(bundle: &ExperimentEvidenceBundle) -> EntityId {
    EntityId::new(format!("bundle:{}", bundle.candidate_id))
}

fn forge_bundle_claim_id(export_key: &str) -> ClaimId {
    ClaimId::new(format!("forge-bundle-claim:{export_key}"))
}

fn forge_bundle_claim_version_id(
    export_key: &str,
    bundle: &ExperimentEvidenceBundle,
    claim_id: &ClaimId,
) -> ClaimVersionId {
    let mut builder = DigestBuilder::new();
    builder
        .update_str("forge_bundle_claim_version")
        .separator()
        .update_str(export_key)
        .separator()
        .update_str(claim_id.as_str())
        .separator()
        .update_str(&bundle.version_id)
        .separator()
        .update_str(&bundle.created_at);
    ClaimVersionId::new(format!(
        "forge-bundle-claim-version:{}",
        builder.finalize().hex()
    ))
}

fn deterministic_relation_version_id(
    bundle: &ExperimentEvidenceBundle,
    scope_key: &ScopeKey,
    relation_kind: &str,
    predicate: &str,
    object_anchor: &serde_json::Value,
) -> RelationVersionId {
    let mut builder = DigestBuilder::new();
    builder
        .update_str("forge_bundle_relation_version")
        .separator()
        .update_str(&bundle.bundle_id)
        .separator()
        .update_str(&bundle.version_id)
        .separator()
        .update_str(&bundle.created_at)
        .separator()
        .update_str(relation_kind)
        .separator()
        .update_str(predicate)
        .separator();
    let scope_json = serde_json::to_string(scope_key).unwrap_or_else(|_| "{}".into());
    builder.update_str(&scope_json).separator();
    let object_json = serde_json::to_string(object_anchor).unwrap_or_else(|_| "null".into());
    builder.update_str(&object_json);
    RelationVersionId::new(format!(
        "forge-bundle-relation-version:{}",
        builder.finalize().hex()
    ))
}

fn bundle_valid_from(bundle: &ExperimentEvidenceBundle) -> Option<String> {
    if bundle.created_at.is_empty() {
        None
    } else {
        Some(bundle.created_at.clone())
    }
}

fn bundle_relation_records(
    bundle: &ExperimentEvidenceBundle,
    claim_id: &ClaimId,
    episode_id: Option<&EpisodeId>,
    scope_key: &ScopeKey,
) -> Vec<ExportRelation> {
    let mut records = Vec::new();

    if let Some(effect) = bundle.primary_effect.as_ref() {
        records.push(export_relation_from_effect(
            effect,
            EffectRelationLineageSource::PrimaryEffect,
            claim_id,
            episode_id,
            bundle,
            scope_key,
        ));
    }

    for effect in &bundle.all_effects {
        if bundle.primary_effect.as_ref().is_some_and(|primary| {
            primary.kind == effect.kind
                && primary.message == effect.message
                && primary.in_baseline == effect.in_baseline
                && primary.in_patched == effect.in_patched
        }) {
            continue;
        }
        records.push(export_relation_from_effect(
            effect,
            EffectRelationLineageSource::AllEffect,
            claim_id,
            episode_id,
            bundle,
            scope_key,
        ));
    }

    for edge in &bundle.hypothesis_edges {
        let predicate = format!(
            "hypothesis_edge_{}",
            debug_name_to_snake_case(&format!("{:?}", edge.kind))
        );
        let object_anchor = serde_json::json!({
            "edge_id": edge.edge_id,
            "source_edit": edge.source_edit,
            "target_effect": edge.target_effect,
            "source": "hypothesis_edge",
            "status": format!("{:?}", edge.status),
            "evidence_ids": edge.evidence_ids,
            "contradiction_ids": edge.contradiction_ids,
            "verification_status": format!("{:?}", edge.verification_status),
        });
        records.push(ExportRelation {
            relation_version_id: Some(deterministic_relation_version_id(
                bundle,
                scope_key,
                "hypothesis_edge",
                &predicate,
                &object_anchor,
            )),
            subject_entity_id: canonical_subject_entity_id(bundle),
            predicate,
            object_anchor,
            valid_from: bundle_valid_from(bundle),
            valid_to: None,
            confidence: cast_confidence(edge.confidence),
            projection_family: "forge_verification".into(),
            source_claim_id: Some(claim_id.clone()),
            source_episode_id: episode_id.cloned(),
            supersedes_relation_version_id: bundle.superseded_hypothesis_relation_version_id(edge),
            metadata: Some(serde_json::json!({
                "bundle_id": bundle.bundle_id,
                "source": "hypothesis_edge",
                "edge_signature": bundle_signature(bundle, "hypothesis_edge", &edge.edge_id),
            })),
        });
    }

    if let Some(diff) = &bundle.experiment_diff {
        for effect in &diff.effects {
            records.push(export_relation_from_effect(
                effect,
                EffectRelationLineageSource::ExperimentDiff,
                claim_id,
                episode_id,
                bundle,
                scope_key,
            ));
        }
    }

    records
}

fn bundle_verification_trial_records(
    bundle: &ExperimentEvidenceBundle,
    claim_id: &ClaimId,
    episode_id: Option<&EpisodeId>,
    scope_key: &ScopeKey,
) -> Vec<ExportRelation> {
    let mut records = Vec::new();

    for trial in &bundle.verification_trials {
        let predicate = match trial.baseline_or_patch {
            crate::lab::evidence::BaselineOrPatch::Baseline => "verification_trial_baseline",
            crate::lab::evidence::BaselineOrPatch::Patched => "verification_trial_patched",
        };

        let object_anchor = serde_json::json!({
            "trial_id": trial.trial_id,
            "attempt_id": trial.attempt_id,
            "baseline_or_patch": format!("{:?}", trial.baseline_or_patch),
            "completed": trial.completed,
            "receipts": trial.receipts,
            "bundle_id": bundle.bundle_id,
            "candidate_id": bundle.candidate_id,
        });

        records.push(ExportRelation {
            relation_version_id: Some(deterministic_relation_version_id(
                bundle,
                scope_key,
                predicate,
                predicate,
                &object_anchor,
            )),
            subject_entity_id: canonical_subject_entity_id(bundle),
            predicate: predicate.to_string(),
            object_anchor,
            valid_from: bundle_valid_from(bundle),
            valid_to: None,
            confidence: cast_confidence(bundle.scores.weighted_total),
            projection_family: "forge_verification".into(),
            source_claim_id: Some(claim_id.clone()),
            source_episode_id: episode_id.cloned(),
            supersedes_relation_version_id: bundle
                .superseded_verification_trial_relation_version_id(trial),
            metadata: Some(serde_json::json!({
                "bundle_id": bundle.bundle_id,
                "attempt_id": trial.attempt_id,
                "trial_id": trial.trial_id,
                "baseline_or_patch": format!("{:?}", trial.baseline_or_patch),
            })),
        });
    }

    records
}

fn bundle_refutation_artifact_records(
    bundle: &ExperimentEvidenceBundle,
    claim_id: &ClaimId,
    episode_id: Option<&EpisodeId>,
    scope_key: &ScopeKey,
) -> Vec<ExportRelation> {
    let mut records = Vec::new();

    for artifact in &bundle.refutation_artifacts {
        let predicate = format!(
            "verification_refutation_{}",
            debug_name_to_snake_case(&format!("{:?}", artifact.artifact_type))
        );

        let outcome = match &artifact.outcome {
            crate::lab::evidence::RefutationArtifactOutcome::Passed => "passed",
            crate::lab::evidence::RefutationArtifactOutcome::Failed { .. } => "failed",
            crate::lab::evidence::RefutationArtifactOutcome::Inconclusive { .. } => "inconclusive",
            crate::lab::evidence::RefutationArtifactOutcome::Skipped { .. } => "skipped",
        };

        let object_anchor = serde_json::json!({
            "artifact_id": artifact.artifact_id,
            "artifact_type": format!("{:?}", artifact.artifact_type),
            "attempt_id": artifact.attempt_id,
            "trial_id": artifact.trial_id,
            "outcome": outcome,
            "estimate_delta": artifact.estimate_delta,
            "details": artifact.details,
        });

        records.push(ExportRelation {
            relation_version_id: Some(deterministic_relation_version_id(
                bundle,
                scope_key,
                "refutation_artifact",
                &predicate,
                &object_anchor,
            )),
            subject_entity_id: canonical_subject_entity_id(bundle),
            predicate,
            object_anchor,
            valid_from: bundle_valid_from(bundle),
            valid_to: None,
            confidence: cast_confidence(bundle.scores.weighted_total),
            projection_family: "forge_verification".into(),
            source_claim_id: Some(claim_id.clone()),
            source_episode_id: episode_id.cloned(),
            supersedes_relation_version_id: bundle
                .superseded_refutation_relation_version_id(artifact),
            metadata: Some(serde_json::json!({
                "bundle_id": bundle.bundle_id,
                "artifact_type": format!("{:?}", artifact.artifact_type),
                "artifact_id": artifact.artifact_id,
                "outcome": outcome,
            })),
        });
    }

    records
}

fn export_relation_from_effect(
    effect: &crate::experiment::TypedLocatedEffect,
    source: EffectRelationLineageSource,
    claim_id: &ClaimId,
    episode_id: Option<&EpisodeId>,
    bundle: &ExperimentEvidenceBundle,
    scope_key: &ScopeKey,
) -> ExportRelation {
    let confidence = if effect.in_baseline == effect.in_patched {
        0.0
    } else {
        cast_confidence(bundle.scores.weighted_total)
    };

    let predicate = format!(
        "{}_{}",
        source.export_source_key(),
        debug_name_to_snake_case(&format!("{:?}", effect.kind))
    );
    let object_anchor = serde_json::json!({
        "message": effect.message,
        "kind": format!("{:?}", effect.kind),
        "in_baseline": effect.in_baseline,
        "in_patched": effect.in_patched,
        "file": effect.file,
        "line": effect.line,
        "source": source.export_source_key(),
    });

    ExportRelation {
        relation_version_id: Some(deterministic_relation_version_id(
            bundle,
            scope_key,
            source.export_source_key(),
            &predicate,
            &object_anchor,
        )),
        subject_entity_id: canonical_subject_entity_id(bundle),
        predicate,
        object_anchor,
        valid_from: bundle_valid_from(bundle),
        valid_to: None,
        confidence,
        projection_family: "forge_verification".into(),
        source_claim_id: Some(claim_id.clone()),
        source_episode_id: episode_id.cloned(),
        supersedes_relation_version_id: bundle
            .superseded_effect_relation_version_id(source, effect),
        metadata: Some(serde_json::json!({
            "bundle_id": bundle.bundle_id,
            "source": source.export_source_key(),
            "effect_signature": bundle_signature(bundle, source.export_source_key(), &effect.message),
        })),
    }
}

fn bundle_signature(bundle: &ExperimentEvidenceBundle, source: &str, detail: &str) -> String {
    let payload = serde_json::json!({
        "source": source,
        "bundle_id": bundle.bundle_id,
        "candidate_id": bundle.candidate_id,
        "detail": detail,
        "attempt_id": bundle.attempt_id,
    });

    blake3::hash(payload.to_string().as_bytes())
        .to_hex()
        .to_string()
}

fn bundle_alias_records(
    bundle: &ExperimentEvidenceBundle,
    scope: &ScopeKey,
) -> Vec<ExportEntityAlias> {
    let mut aliases = Vec::new();

    if bundle.candidate_id.is_empty() {
        return aliases;
    }

    let canonical = canonical_subject_entity_id(bundle);
    aliases.push(ExportEntityAlias {
        canonical_entity_id: canonical.clone(),
        alias_text: bundle.candidate_id.clone(),
        alias_source: "forge_bundle_candidate".into(),
        match_evidence: Some(serde_json::json!({
            "bundle_id": bundle.bundle_id,
            "eval_id": bundle.eval_id,
            "candidate_id": bundle.candidate_id,
            "attempt_id": bundle.attempt_id,
        })),
        confidence: cast_confidence(bundle.scores.weighted_total),
        scope: Some(scope.clone()),
        superseded_by_entity_id: None,
        split_from_entity_id: None,
    });

    if let Some(patch_hash) = &bundle.patch_hash {
        aliases.push(ExportEntityAlias {
            canonical_entity_id: canonical.clone(),
            alias_text: patch_hash.clone(),
            alias_source: "forge_patch_hash".into(),
            match_evidence: Some(serde_json::json!({
                "bundle_id": bundle.bundle_id,
                "patch_hash_source": "bundle.patch_hash",
            })),
            confidence: cast_confidence(bundle.scores.weighted_total),
            scope: Some(scope.clone()),
            superseded_by_entity_id: None,
            split_from_entity_id: None,
        });
    }

    aliases
}

fn bundle_evidence_refs(
    bundle: &ExperimentEvidenceBundle,
    claim_id: &ClaimId,
    claim_version_id: &ClaimVersionId,
) -> Vec<ExportEvidenceRef> {
    let mut refs = Vec::new();

    refs.push(ExportEvidenceRef {
        claim_id: claim_id.clone(),
        claim_version_id: Some(claim_version_id.clone()),
        fetch_handle: format!("forge:bundle:{}", bundle.bundle_id),
        source_authority: ExportAuthority::Forge.as_str().into(),
        metadata: Some(serde_json::json!({
            "bundle_id": bundle.bundle_id,
            "candidate_id": bundle.candidate_id,
            "eval_id": bundle.eval_id,
            "version_id": bundle.version_id,
        })),
    });

    if let Some(attempt_id) = &bundle.attempt_id {
        refs.push(ExportEvidenceRef {
            claim_id: claim_id.clone(),
            claim_version_id: Some(claim_version_id.clone()),
            fetch_handle: format!("forge:attempt:{}", attempt_id),
            source_authority: ExportAuthority::Forge.as_str().into(),
            metadata: Some(serde_json::json!({
                "bundle_id": bundle.bundle_id,
                "attempt_id": attempt_id,
                "evidence_kind": "attempt",
            })),
        });
    }

    if let Some(run_id) = &bundle.run_id {
        refs.push(ExportEvidenceRef {
            claim_id: claim_id.clone(),
            claim_version_id: Some(claim_version_id.clone()),
            fetch_handle: format!("forge:run:{}", run_id),
            source_authority: ExportAuthority::Forge.as_str().into(),
            metadata: Some(serde_json::json!({
                "bundle_id": bundle.bundle_id,
                "run_id": run_id,
                "evidence_kind": "run",
            })),
        });
    }

    for receipt in &bundle.receipts {
        refs.push(ExportEvidenceRef {
            claim_id: claim_id.clone(),
            claim_version_id: Some(claim_version_id.clone()),
            fetch_handle: format!("forge:receipt:{}", receipt.receipt_id),
            source_authority: ExportAuthority::Forge.as_str().into(),
            metadata: Some(serde_json::json!({
                "receipt_id": receipt.receipt_id,
                "receipt_kind": format!("{:?}", receipt.kind),
                "storage": format!("{:?}", receipt.storage),
            })),
        });
    }

    refs
}

fn cast_confidence(source: f64) -> f32 {
    source.clamp(0.0, 1.0) as f32
}

fn bundle_cause_ids(bundle: &ExperimentEvidenceBundle) -> Vec<String> {
    [
        bundle
            .attempt_id
            .as_ref()
            .map(|attempt_id| format!("attempt:{attempt_id}")),
        Some(format!("candidate:{}", bundle.candidate_id)),
        Some(format!("eval:{}", bundle.eval_id)),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn bundle_effect_type(bundle: &ExperimentEvidenceBundle) -> String {
    if let Some(effect) = &bundle.primary_effect {
        return debug_name_to_snake_case(&format!("{:?}", effect.kind));
    }

    if let Some(effect) = bundle.all_effects.first() {
        return debug_name_to_snake_case(&format!("{:?}", effect.kind));
    }

    bundle
        .experiment_diff
        .as_ref()
        .and_then(|diff| diff.effects.first())
        .map(|effect| debug_name_to_snake_case(&format!("{:?}", effect.kind)))
        .unwrap_or_else(|| "verification_bundle".into())
}

fn default_outcome(bundle: &ExperimentEvidenceBundle) -> String {
    if let Some(diff) = &bundle.experiment_diff {
        return format!(
            "diff regressions={} improvements={} stable_failures={} stable_passes={}",
            diff.regressions, diff.improvements, diff.stable_failures, diff.stable_passes
        );
    }
    "verification_bundle".into()
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

/// Compute a deterministic export key.
pub fn compute_export_key(bundle_id: &str, rendering_version: u32, namespace: &str) -> String {
    let input = format!("{bundle_id}:{rendering_version}:{namespace}");
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

/// Export an `ExperimentEvidenceBundle` through Forge's canonical export schema.
///
/// This is the normative path for the stack. It emits `ExportEnvelopeV3`,
/// persists an export receipt, and is the canonical path into the bridge.
pub async fn export_bundle(
    bundle: &ExperimentEvidenceBundle,
    namespace: &str,
    store: &ForgeStore,
) -> ForgeResult<ExportEnvelopeV3> {
    let export_bundle = bundle_with_store_promotion_state(bundle, store)?;
    let export = EpisodeExport::from_bundle(&export_bundle, namespace);
    let envelope = export.to_export_envelope_v3(&export_bundle)?;

    if export.already_exported(store)? {
        tracing::info!(
            export_key = %export.export_key,
            bundle_id = %export.bundle_id,
            "export already receipted, skipping"
        );
        return Ok(envelope);
    }

    export.persist_receipt(store, None)?;
    Ok(envelope)
}

/// Compatibility-only escape hatch: export and immediately import into memory.
///
/// The normal path is still caller-orchestrated:
/// `ExportEnvelopeV3 -> forge-memory-bridge -> semantic-memory::import_projection_batch()`.
#[cfg(feature = "danger-sm-write")]
pub async fn export_bundle_with_memory_write_through_compat(
    bundle: &ExperimentEvidenceBundle,
    namespace: &str,
    store: &ForgeStore,
    memory: &semantic_memory::MemoryStore,
) -> ForgeResult<ExportEnvelopeV3> {
    let export_bundle = bundle_with_store_promotion_state(bundle, store)?;
    let export = EpisodeExport::from_bundle(&export_bundle, namespace);
    let envelope = export.to_export_envelope_v3(&export_bundle)?;

    if export.already_exported(store)? {
        tracing::info!(
            export_key = %export.export_key,
            bundle_id = %export.bundle_id,
            "compat write-through export already receipted, skipping"
        );
        return Ok(envelope);
    }

    let write_through_ok = match forge_memory_bridge::transform_envelope_v3(&envelope) {
        Ok(batch) => match memory.import_projection_batch(&batch).await {
            Ok(_) => {
                tracing::warn!(
                    export_key = %export.export_key,
                    "compat direct memory import executed via canonical bridge batch"
                );
                Some(true)
            }
            Err(err) => {
                tracing::warn!(
                    export_key = %export.export_key,
                    error = %err,
                    "compat direct memory import failed"
                );
                Some(false)
            }
        },
        Err(err) => {
            tracing::warn!(
                export_key = %export.export_key,
                error = %err,
                "compat direct memory import failed before memory write"
            );
            Some(false)
        }
    };

    export.persist_receipt(store, write_through_ok)?;
    Ok(envelope)
}

/// Compatibility-only escape hatch blocked unless `danger-sm-write` is enabled.
#[cfg(not(feature = "danger-sm-write"))]
pub async fn export_bundle_with_memory_write_through_compat(
    _bundle: &ExperimentEvidenceBundle,
    _namespace: &str,
    _store: &ForgeStore,
    _memory: &semantic_memory::MemoryStore,
) -> ForgeResult<ExportEnvelopeV3> {
    Err(crate::error::ForgeError::WriteThroughBlocked)
}

fn bundle_with_store_promotion_state(
    bundle: &ExperimentEvidenceBundle,
    store: &ForgeStore,
) -> ForgeResult<ExperimentEvidenceBundle> {
    let mut enriched = bundle.clone();
    if enriched.promotion_state.is_none() {
        if let Some(promotion) = store.get_latest_promotion()? {
            if promotion.candidate_id == enriched.candidate_id {
                enriched.promotion_state = Some(semantic_memory_forge::PromotionState::Promoted {
                    version_id: Some(promotion.version_id),
                    promoted_at: Some(promotion.promoted_at),
                });
            }
        }
    }
    Ok(enriched)
}
