use constitutional_memory::{
    evaluate_amendment, evaluate_archive_compaction, AmendmentDecisionV1, AmendmentProposalV1,
    ArchiveManifestV1, CharterBundleV1, CompactionReceiptV1, HistoricalQueryGuaranteeV1,
};
use constraint_compiler::{compile_batch, CompileOutput, CompilerPolicy};
use discovery_portfolio::{
    evaluate_portfolio_plan, CampaignDecisionTraceV1, DiscoveryProgramV1, ExperimentCampaignV1,
    InformationValueEstimateV1, PortfolioPlanV1, ProgramHypothesisSetV1, VerificationLoadBudgetV1,
};
use federated_settlement::{
    evaluate_divergence_or_suspension, evaluate_settlement, evaluate_shared_replay,
    SettlementCaseV1, SettlementReceiptV1, SharedDivergenceReportV1, SharedReplaySliceV1,
    TreatySuspensionV1,
};
#[allow(deprecated)]
use forge_memory_bridge::{transform_envelope_v2, transform_envelope_v3, ProjectionImportBatchV3};
use kernel_execution::{schedule_execution, ExecutionBudget, ScheduledExecution};
use kernel_oracles::{
    evaluate_causal_refuter, evaluate_conservative, evaluate_exact_bounded,
    evaluate_temporal_replay, minimal_perturbation_witness, OracleRefutationResult,
    TemporalReplayAssessment, TemporalReplaySnapshot,
};
use mechanism_runtime::{
    evaluate_fit_run, FitRunV1, MechanismBundleV1, RolloutStabilityReportV1, SimulationContractV1,
    TheoryRefuterSuiteV1, TheoryVersionV1,
};
use semantic_memory_forge::{ExportEnvelopeV2, ExportEnvelopeV3};
use spec_execution::{
    establish_veto_challenge_baseline, generate_companion_bundles, generate_schema_bundle,
    GeneratedCompanionBundles, GeneratedSchemaBundleV1, HumanVetoBundleV1, MetaChallengeBundleV1,
    NormativeASTV1, ProofEvaluationReceiptV1, ProofObligationSetV1, SelfHostingBuildReceiptV1,
    SpecBundleV1,
};
use verification_control::{replay_case, LedgerEntry, ReplayedCaseState};

#[derive(Debug, Clone)]
pub struct BridgeReferenceInterpretation {
    pub source_schema_version: String,
    pub record_count: usize,
    pub preserved_execution_context: bool,
    pub preserved_evidence_bundle: bool,
}

impl BridgeReferenceInterpretation {
    /// Interprets a V3 Forge export envelope through the reference bridge path.
    pub fn from_v3(envelope: &ExportEnvelopeV3) -> Result<Self, forge_memory_bridge::BridgeError> {
        let batch = transform_envelope_v3(envelope)?;
        Ok(Self {
            source_schema_version: envelope.schema_version.clone(),
            preserved_execution_context: batch.execution_context.is_some(),
            preserved_evidence_bundle: batch.evidence_bundle.is_some(),
            record_count: batch.records.len(),
        })
    }

    #[allow(deprecated)]
    /// Interprets a deprecated V2 Forge export envelope through the reference bridge path.
    pub fn from_v2(envelope: &ExportEnvelopeV2) -> Result<Self, forge_memory_bridge::BridgeError> {
        let batch = transform_envelope_v2(envelope)?;
        Ok(Self {
            source_schema_version: envelope.schema_version.clone(),
            preserved_execution_context: false,
            preserved_evidence_bundle: false,
            record_count: batch.records.len(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct BitemporalReferenceInterpretation {
    pub assessment: TemporalReplayAssessment,
    pub compiled: CompileOutput,
}

/// Replays the newest snapshot before a cutoff and returns both assessment and compiled graph.
pub fn interpret_bitemporal_replay(
    snapshots: &[TemporalReplaySnapshot],
    cutoff_recorded_at: &str,
    policy: &CompilerPolicy,
    expected_hash: &stack_ids::ContentDigest,
) -> Option<BitemporalReferenceInterpretation> {
    let assessment =
        evaluate_temporal_replay(snapshots, cutoff_recorded_at, policy, expected_hash)?;
    let selected = snapshots
        .iter()
        .find(|snapshot| snapshot.recorded_at == assessment.selected_recorded_at)?;
    let compiled = compile_batch(&selected.batch, policy);
    Some(BitemporalReferenceInterpretation {
        assessment,
        compiled,
    })
}

#[derive(Debug, Clone)]
pub struct BoundedKernelSliceInterpretation {
    pub compiled: CompileOutput,
    pub scheduled: ScheduledExecution,
    pub exact_supported: bool,
}

/// Interprets a batch through compilation, oracle support, and scheduler selection.
pub fn interpret_bounded_kernel_slice(
    batch: &ProjectionImportBatchV3,
    policy: &CompilerPolicy,
    budget: &ExecutionBudget,
) -> BoundedKernelSliceInterpretation {
    let compiled = compile_batch(batch, policy);
    let exact_supported = evaluate_exact_bounded(&compiled).is_some();
    let scheduled = schedule_execution(&compiled, budget);
    BoundedKernelSliceInterpretation {
        compiled,
        scheduled,
        exact_supported,
    }
}

#[derive(Debug, Clone)]
pub struct RepairReplayInterpretation {
    pub replayed: ReplayedCaseState,
}

/// Replays a repair ledger into the reference replay state.
pub fn interpret_repair_replay(
    entries: &[LedgerEntry],
) -> Result<RepairReplayInterpretation, String> {
    replay_case(entries).map(|replayed| RepairReplayInterpretation { replayed })
}

/// Interprets a settlement case into its bounded admissibility receipt.
pub fn interpret_settlement_admissibility(
    case: &SettlementCaseV1,
    generated_at: &str,
) -> SettlementReceiptV1 {
    evaluate_settlement(case, generated_at)
}

/// Interprets the shared replay slice for a settlement case.
pub fn interpret_shared_replay(case: &SettlementCaseV1, generated_at: &str) -> SharedReplaySliceV1 {
    evaluate_shared_replay(case, generated_at)
}

/// Interprets divergence reporting and optional suspension for a settlement case.
pub fn interpret_divergence_or_suspension(
    case: &SettlementCaseV1,
    replay: &SharedReplaySliceV1,
    divergence_summary: &str,
    generated_at: &str,
) -> (SharedDivergenceReportV1, Option<TreatySuspensionV1>) {
    evaluate_divergence_or_suspension(case, replay, divergence_summary, generated_at)
}

/// Interprets the bounded mechanism-fit decision for a theory and contract.
pub fn interpret_mechanism_fit(
    mechanism: &MechanismBundleV1,
    theory: &TheoryVersionV1,
    contract: &SimulationContractV1,
    refuter_suite: &TheoryRefuterSuiteV1,
    stability_report: &RolloutStabilityReportV1,
    fit_score: f64,
    generated_at: &str,
) -> FitRunV1 {
    evaluate_fit_run(
        mechanism,
        theory,
        contract,
        refuter_suite,
        stability_report,
        fit_score,
        generated_at,
    )
}

/// Interprets portfolio budgeting and campaign-selection pressure for a program.
pub fn interpret_portfolio_budget(
    program: &DiscoveryProgramV1,
    hypotheses: &ProgramHypothesisSetV1,
    plan: &PortfolioPlanV1,
    campaigns: &[ExperimentCampaignV1],
    value_estimates: &[InformationValueEstimateV1],
    budget: &VerificationLoadBudgetV1,
    generated_at: &str,
) -> CampaignDecisionTraceV1 {
    evaluate_portfolio_plan(
        program,
        hypotheses,
        plan,
        campaigns,
        value_estimates,
        budget,
        generated_at,
    )
}

/// Interprets a constitutional amendment proposal into its bounded decision artifact.
pub fn interpret_constitutional_change(
    proposal: &AmendmentProposalV1,
    migration_obligations_satisfied: bool,
    archive_manifest: Option<&ArchiveManifestV1>,
    historical_query_guarantee: Option<&HistoricalQueryGuaranteeV1>,
    decided_at: &str,
) -> AmendmentDecisionV1 {
    evaluate_amendment(
        proposal,
        migration_obligations_satisfied,
        archive_manifest,
        historical_query_guarantee,
        decided_at,
    )
}

/// Interprets archive compaction into manifest, guarantee, and receipt artifacts.
pub fn interpret_archive_compaction(
    charter: &CharterBundleV1,
    preserved_refs: Vec<String>,
    dropped_detail_refs: Vec<String>,
    guaranteed_query_modes: Vec<String>,
) -> (
    ArchiveManifestV1,
    HistoricalQueryGuaranteeV1,
    CompactionReceiptV1,
) {
    evaluate_archive_compaction(
        charter.charter_bundle_id.clone(),
        preserved_refs,
        dropped_detail_refs,
        guaranteed_query_modes,
    )
}

/// Interprets generated schema integrity for a spec surface.
pub fn interpret_generated_artifact_integrity(
    spec: &SpecBundleV1,
    ast: &NormativeASTV1,
    obligations: &ProofObligationSetV1,
    schema_family: &str,
    generated_at: &str,
) -> (GeneratedSchemaBundleV1, ProofEvaluationReceiptV1) {
    generate_schema_bundle(spec, ast, obligations, schema_family, generated_at)
}

/// Interprets the generated companion bundles for a spec surface.
pub fn interpret_generated_companion_bundles(
    spec: &SpecBundleV1,
    ast: &NormativeASTV1,
    obligations: &ProofObligationSetV1,
    schema_family: &str,
    generated_at: &str,
) -> GeneratedCompanionBundles {
    generate_companion_bundles(spec, ast, obligations, schema_family, generated_at)
}

/// Interprets veto/challenge governance for generated surface artifacts.
pub fn interpret_generated_surface_governance(
    spec: &SpecBundleV1,
    bundles: &GeneratedCompanionBundles,
    human_veto: Option<&HumanVetoBundleV1>,
    meta_challenge: Option<&MetaChallengeBundleV1>,
    generated_at: &str,
) -> SelfHostingBuildReceiptV1 {
    establish_veto_challenge_baseline(
        spec,
        &bundles.schema_bundle,
        &bundles.interpreter_bundle,
        &bundles.conformance_corpus,
        &bundles.migration_plan,
        &bundles.proof_evaluation_receipt,
        human_veto,
        meta_challenge,
        generated_at,
    )
}

#[derive(Debug, Clone)]
pub struct CausalWorkflowInterpretation {
    pub compiled: CompileOutput,
    pub exact_supported: bool,
    pub conservative_supported: bool,
    pub causal_refutation: OracleRefutationResult,
    pub minimal_perturbation: OracleRefutationResult,
}

/// Interprets a causal workflow through oracle and refutation reference paths.
pub fn interpret_causal_workflow(
    batch: &ProjectionImportBatchV3,
    policy: &CompilerPolicy,
    target_node_id: &str,
    search_budget: usize,
) -> CausalWorkflowInterpretation {
    let compiled = compile_batch(batch, policy);
    let exact_supported = evaluate_exact_bounded(&compiled).is_some();
    let conservative_supported = evaluate_conservative(&compiled).supported;
    let causal_refutation = evaluate_causal_refuter(&compiled, target_node_id, search_budget);
    let minimal_perturbation =
        minimal_perturbation_witness(&compiled, target_node_id, search_budget);
    CausalWorkflowInterpretation {
        compiled,
        exact_supported,
        conservative_supported,
        causal_refutation,
        minimal_perturbation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semantic_memory_forge::{
        ConstraintSeedKind, ExportAuthority, ExportClaim, ExportConfidenceClass, ExportEnvelopeV2,
        ExportEnvelopeV3, ExportRecord, ExportRecordSemanticsV3, ExportRecordV3, ForgeExportMeta,
        ProjectionVisibilityClass,
    };
    use stack_ids::{
        AssertionGroupId, AttemptId, ClaimFamilyId, ClaimId, ClaimVersionId, EntityId, EnvelopeId,
        ScopeKey, TraceCtx,
    };
    use verification_control::{
        CaseRegion, CheckMethod, CheckPlan, ControlReceipt, LedgerEntry, LedgerEvent,
        PromotionClass, ReversibilityClass, TerminalDisposition, VerificationAttempt,
        VerificationAttemptState, VerificationCase, VerificationCaseClass,
    };

    fn sample_v3_envelope() -> ExportEnvelopeV3 {
        let scope = ScopeKey::namespace_only("interp");
        let export_meta = ForgeExportMeta {
            authority: ExportAuthority::Forge,
            run_id: Some("run-1".into()),
            direct_write: false,
            comparability_snapshot_version: None,
            exported_at: "2026-03-12T00:00:00Z".into(),
        };
        let records = vec![ExportRecordV3 {
            record: ExportRecord::Claim(ExportClaim {
                claim_id: None,
                claim_version_id: Some(ClaimVersionId::new("claim-v1")),
                subject_entity_id: EntityId::new("entity-1"),
                predicate: "supports".into(),
                object_anchor: serde_json::json!("result"),
                valid_from: None,
                valid_to: None,
                confidence: 1.0,
                content: "claim-v1 supports result".into(),
                projection_family: "forge_verification".into(),
                supersedes_claim_id: None,
                supersedes_claim_version_id: None,
                metadata: None,
            }),
            semantics: Some(ExportRecordSemanticsV3 {
                claim_family_id: Some(ClaimFamilyId::new("family-1")),
                assertion_group_id: Some(AssertionGroupId::new("group-1")),
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
        }];
        let digest =
            ExportEnvelopeV3::compute_digest("forge", &scope, &records, Some(&export_meta), None)
                .unwrap();
        ExportEnvelopeV3 {
            envelope_id: EnvelopeId::new("env-v3"),
            schema_version: semantic_memory_forge::EXPORT_ENVELOPE_V3_SCHEMA.into(),
            content_digest: digest,
            source_authority: "forge".into(),
            scope_key: scope,
            trace_ctx: Some(TraceCtx::generate()),
            exported_at: "2026-03-12T00:00:00Z".into(),
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
        }
    }

    fn sample_v2_envelope() -> ExportEnvelopeV2 {
        let scope = ScopeKey::namespace_only("interp");
        let export_meta = ForgeExportMeta {
            authority: ExportAuthority::Forge,
            run_id: Some("run-2".into()),
            direct_write: false,
            comparability_snapshot_version: None,
            exported_at: "2026-03-12T00:00:00Z".into(),
        };
        let records = vec![ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new("claim-1")),
            claim_version_id: Some(ClaimVersionId::new("claim-v1")),
            subject_entity_id: EntityId::new("entity-1"),
            predicate: "supports".into(),
            object_anchor: serde_json::json!("result"),
            valid_from: None,
            valid_to: None,
            confidence: 1.0,
            content: "claim-v1 supports result".into(),
            projection_family: "forge_verification".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: None,
        })];
        let digest =
            ExportEnvelopeV2::compute_digest("forge", &scope, &records, Some(&export_meta), None)
                .unwrap();
        ExportEnvelopeV2 {
            envelope_id: EnvelopeId::new("env-v2"),
            schema_version: semantic_memory_forge::EXPORT_ENVELOPE_V2_SCHEMA.into(),
            content_digest: digest,
            source_authority: "forge".into(),
            scope_key: scope,
            trace_ctx: Some(TraceCtx::generate()),
            exported_at: "2026-03-12T00:00:00Z".into(),
            export_meta: Some(export_meta),
            evidence_bundle: None,
            records,
        }
    }

    #[test]
    fn bridge_reference_interpreter_handles_v2_and_v3() {
        let v3 = BridgeReferenceInterpretation::from_v3(&sample_v3_envelope()).unwrap();
        let v2 = BridgeReferenceInterpretation::from_v2(&sample_v2_envelope()).unwrap();
        assert_eq!(v3.record_count, 1);
        assert_eq!(v2.record_count, 1);
        assert_eq!(
            v3.source_schema_version,
            semantic_memory_forge::EXPORT_ENVELOPE_V3_SCHEMA
        );
        assert_eq!(
            v2.source_schema_version,
            semantic_memory_forge::EXPORT_ENVELOPE_V2_SCHEMA
        );
    }

    #[test]
    fn bounded_kernel_slice_reference_interpreter_is_deterministic() {
        let batch = transform_envelope_v3(&sample_v3_envelope()).unwrap();
        let policy = CompilerPolicy {
            policy_version: "interp".into(),
            include_hyperedges: true,
        };
        let a = interpret_bounded_kernel_slice(&batch, &policy, &ExecutionBudget::default());
        let b = interpret_bounded_kernel_slice(&batch, &policy, &ExecutionBudget::default());
        assert_eq!(a.compiled.graph_hash, b.compiled.graph_hash);
        assert_eq!(
            a.scheduled.execution.execution_mode,
            b.scheduled.execution.execution_mode
        );
    }

    #[test]
    fn bitemporal_reference_interpreter_selects_latest_snapshot_before_cutoff() {
        let policy = CompilerPolicy {
            policy_version: "interp".into(),
            include_hyperedges: true,
        };
        let older = transform_envelope_v3(&sample_v3_envelope()).unwrap();
        let mut newer = older.clone();
        newer.source_envelope_id = "env-v3-new".into();
        let expected = compile_batch(&older, &policy).graph_hash;
        let snapshots = vec![
            TemporalReplaySnapshot {
                recorded_at: "2026-03-11T00:00:00Z".into(),
                batch: older,
            },
            TemporalReplaySnapshot {
                recorded_at: "2026-03-13T00:00:00Z".into(),
                batch: newer,
            },
        ];
        let replay =
            interpret_bitemporal_replay(&snapshots, "2026-03-12T00:00:00Z", &policy, &expected)
                .unwrap();
        assert_eq!(
            replay.assessment.selected_recorded_at,
            "2026-03-11T00:00:00Z"
        );
        assert!(replay.assessment.matched_expected_hash);
    }

    #[test]
    fn repair_replay_reference_interpreter_rebuilds_case_state() {
        let case = VerificationCase::new(
            VerificationCaseClass::UnverifiedClaimVersion,
            CaseRegion {
                namespace: "interp".into(),
                scope_key: Some(ScopeKey::namespace_only("interp")),
                target_key: "claim-v1".into(),
                region_id: None,
                region_digest_id: None,
                claim_version_id: Some(ClaimVersionId::new("claim-v1")),
                as_of_recorded_at: None,
            },
            TraceCtx::generate(),
            AttemptId::new("attempt-1"),
            "2026-03-12T00:00:00Z",
            false,
            false,
        );
        let plan = CheckPlan::new(
            case.case_id.clone(),
            CheckMethod::ExactBoundedOracle,
            vec!["oracle".into()],
            PromotionClass::P1,
            ReversibilityClass::ReversibleScoped,
            true,
            false,
            false,
            "interp",
            serde_json::json!({}),
        );
        let attempt = VerificationAttempt::completed(
            case.case_id.clone(),
            plan.plan_id.clone(),
            case.attempt_id.clone(),
            Some(stack_ids::TrialId::new("trial-1")),
            VerificationAttemptState::Succeeded,
            false,
            false,
            "2026-03-12T00:00:00Z",
            "2026-03-12T00:00:01Z",
            Some("supported".into()),
        );
        let receipt =
            ControlReceipt::new_case_execution(&case, &plan, &attempt, true, serde_json::json!({}));
        let replayed = interpret_repair_replay(&[
            LedgerEntry::new(
                case.case_id.clone(),
                1,
                LedgerEvent::CaseOpened { case: case.clone() },
            ),
            LedgerEntry::new(
                case.case_id.clone(),
                2,
                LedgerEvent::PlanAdopted { plan: plan.clone() },
            ),
            LedgerEntry::new(
                case.case_id.clone(),
                3,
                LedgerEvent::AttemptRecorded { attempt },
            ),
            LedgerEntry::new(
                case.case_id.clone(),
                4,
                LedgerEvent::ReceiptAppended {
                    receipt: Box::new(receipt),
                },
            ),
            LedgerEntry::new(
                case.case_id.clone(),
                5,
                LedgerEvent::CaseClosed {
                    case_id: case.case_id.clone(),
                    disposition: TerminalDisposition::EligibleForPromotion,
                },
            ),
        ])
        .unwrap();
        assert_eq!(replayed.replayed.plan_count, 1);
        assert_eq!(
            replayed.replayed.terminal_disposition,
            Some(TerminalDisposition::EligibleForPromotion)
        );
    }

    #[test]
    fn causal_workflow_reference_interpreter_is_advisory_only() {
        let batch = transform_envelope_v3(&sample_v3_envelope()).unwrap();
        let policy = CompilerPolicy {
            policy_version: "interp".into(),
            include_hyperedges: true,
        };
        let interpretation = interpret_causal_workflow(&batch, &policy, "claim-v1", 1);
        assert!(interpretation.conservative_supported);
        assert_eq!(interpretation.causal_refutation.target_node_id, "claim-v1");
    }
}
