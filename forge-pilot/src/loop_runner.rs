//! OODA loop runner that drives repeated observe-orient-decide-act cycles.
//!
//! `LoopRunner` owns the runtime resources and pilot history, runs up to
//! `max_iterations` cycles, and produces a `LoopReport` summarizing all
//! iterations, halt reason, and receipts.

use crate::act::execute_plan;
use crate::config::LoopConfig;
use crate::decide::select_candidate;
use crate::error::PilotError;
use crate::export::canonical_roundtrip;
use crate::history::PilotHistory;
use crate::observe::{observe_scope, Observation, ObservationDisposition};
use crate::orient::score_targets;
use crate::receipts::{
    build_execution_context, build_export_trace, build_lineage_receipt, build_repair_record,
    build_stop_rule_evaluation, build_verification_plan_artifact,
};
use crate::types::RepairClassV1;
use crate::ActionFamily;
use forge_engine::ForgeStore;
use knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter;
use knowledge_runtime::{KnowledgeRuntime, RuntimeConfig};
use semantic_memory::MemoryStore;
use serde_json::json;
use stack_ids::{AttemptId, TraceCtx, TrialId};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::time::{sleep, Duration, Instant};
use verification_adjudication::adjudicate_case;
use verification_calibration::CalibrationSnapshot;
use verification_control::{
    replay_case, schedule_check_plan, BudgetLineage, CaseRegion, CheckPlan, ControlReceipt,
    DegradationMarker, LedgerEntry, LedgerEvent, QueueHop, VerificationAttempt,
    VerificationAttemptState, VerificationCase,
};
use verification_policy::{approval_matches, evaluate_policy, policy_as_of};

#[path = "loop_runner_helpers.rs"]
mod loop_runner_helpers;
#[path = "loop_runner_report.rs"]
mod loop_runner_report;

use loop_runner_helpers::{
    currently_promoted, execute_rollback_invalidation, map_check_method, plan_input,
    promotion_class_for_plan, reversibility_class_for_plan,
};
use loop_runner_report::apply_loop_report_boundary_repairs;
pub use loop_runner_report::{
    parse_loop_report_boundary, HaltReason, LoopBudgetReceipt, LoopIterationReport, LoopReceipt,
    LoopReport, LOOP_ITERATION_REPORT_V1_SCHEMA, LOOP_REPORT_V1_SCHEMA,
    PILOT_LOOP_RECEIPT_V1_SCHEMA,
};

/// Runtime resources required by the loop runner: knowledge runtime, memory store, and Forge store.
pub struct LoopRunnerResources {
    pub runtime: KnowledgeRuntime,
    pub memory_store: MemoryStore,
    pub forge_store: ForgeStore,
}

impl LoopRunnerResources {
    /// Builds loop-runner resources from stores plus runtime configuration.
    pub fn from_memory_store(
        memory_store: MemoryStore,
        forge_store: ForgeStore,
        runtime_config: RuntimeConfig,
    ) -> Result<Self, PilotError> {
        let runtime = KnowledgeRuntime::new(
            runtime_config,
            SemanticMemoryAdapter::new(memory_store.clone()),
        )?;
        Ok(Self {
            runtime,
            memory_store,
            forge_store,
        })
    }
}

/// Thread-safe flag for requesting an early halt of the running loop.
#[derive(Debug, Clone, Default)]
pub struct ExternalHaltFlag {
    flag: Arc<AtomicBool>,
}

impl ExternalHaltFlag {
    /// Requests that the active loop stop at the next safe halt point.
    pub fn halt(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    fn is_halted(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

/// Stateful OODA loop runner that executes repeated verification cycles.
pub struct LoopRunner {
    config: LoopConfig,
    resources: LoopRunnerResources,
    history: PilotHistory,
    external_halt: ExternalHaltFlag,
}

struct LoopTotals {
    targets_investigated: Vec<String>,
    actions_executed: u32,
    exports_completed: u32,
    imports_completed: u32,
    degraded_iterations: u32,
    advisory_only_steps: bool,
}

impl LoopRunner {
    /// Creates a loop runner with fresh history and halt state.
    pub fn new(config: LoopConfig, resources: LoopRunnerResources) -> Self {
        Self {
            config,
            resources,
            history: PilotHistory::default(),
            external_halt: ExternalHaltFlag::default(),
        }
    }

    /// Returns a cloneable external halt flag for this runner.
    pub fn halt_flag(&self) -> ExternalHaltFlag {
        self.external_halt.clone()
    }

    /// Borrows the accumulated pilot history.
    pub fn history(&self) -> &PilotHistory {
        &self.history
    }

    /// Runs a single observation against the configured scope and stores.
    pub async fn observe(&self) -> Result<Observation, PilotError> {
        observe_scope(
            &self.resources.runtime,
            &self.resources.memory_store,
            &self.config,
        )
        .await
    }

    /// Runs the full OODA loop until a halt condition is reached.
    pub async fn run(&mut self) -> Result<LoopReport, PilotError> {
        let started = Instant::now();
        let started_at = chrono::Utc::now();
        let trace_ctx = TraceCtx::generate();
        let attempt_id = AttemptId::generate();
        let mut iterations = Vec::new();
        let mut totals = LoopTotals {
            targets_investigated: Vec::new(),
            actions_executed: 0,
            exports_completed: 0,
            imports_completed: 0,
            degraded_iterations: 0,
            advisory_only_steps: false,
        };
        let mut last_observation: Option<Arc<Observation>> = None;

        for iteration_index in 0..self.config.max_iterations {
            if self.external_halt.is_halted() {
                return Ok(self.finish_report(
                    started_at,
                    trace_ctx.clone(),
                    attempt_id.clone(),
                    started,
                    iterations,
                    totals,
                    HaltReason::ExternalHalt,
                    last_observation.as_deref(),
                ));
            }
            if started.elapsed().as_secs() >= self.config.time_budget_secs {
                return Ok(self.finish_report(
                    started_at,
                    trace_ctx.clone(),
                    attempt_id.clone(),
                    started,
                    iterations,
                    totals,
                    HaltReason::TimeBudgetExhausted,
                    last_observation.as_deref(),
                ));
            }

            let iteration_timestamp = chrono::Utc::now();
            let observation = self.observe().await?;
            let observation = Arc::new(observation);
            last_observation = Some(Arc::clone(&observation));
            let degraded = !observation.degradations.is_empty();
            if degraded {
                totals.degraded_iterations += 1;
            }
            if matches!(
                observation.status.disposition,
                ObservationDisposition::NamespaceMismatch
            ) {
                return Ok(self.finish_report(
                    started_at,
                    trace_ctx.clone(),
                    attempt_id.clone(),
                    started,
                    iterations,
                    totals,
                    HaltReason::NamespaceMismatch,
                    Some(&*observation),
                ));
            }
            if matches!(
                observation.status.disposition,
                ObservationDisposition::StorageUnavailable
            ) {
                return Ok(self.finish_report(
                    started_at,
                    trace_ctx.clone(),
                    attempt_id.clone(),
                    started,
                    iterations,
                    totals,
                    HaltReason::StorageUnavailable,
                    Some(&*observation),
                ));
            }
            if matches!(
                observation.status.disposition,
                ObservationDisposition::StorageCorrupt
            ) {
                return Ok(self.finish_report(
                    started_at,
                    trace_ctx.clone(),
                    attempt_id.clone(),
                    started,
                    iterations,
                    totals,
                    HaltReason::StorageCorrupt,
                    Some(&*observation),
                ));
            }
            if observation.missing_kernel_payload() && observation.claim_versions.is_empty() {
                return Ok(self.finish_report(
                    started_at,
                    trace_ctx.clone(),
                    attempt_id.clone(),
                    started,
                    iterations,
                    totals,
                    HaltReason::MissingKernelPayload,
                    Some(&*observation),
                ));
            }

            // --- Governance gate evaluation ---
            #[cfg(feature = "governance")]
            let governance_gate_result = observation.governance.as_ref().map(|gov_obs| {
                let gate = crate::governance_gate::gate_execution(gov_obs);
                let receipt = crate::governance_gate::build_governance_receipt(gov_obs, &gate);
                (gate, receipt)
            });

            #[cfg(feature = "governance")]
            if let Some((ref gate, ref receipt)) = governance_gate_result {
                match gate {
                    crate::governance_gate::GovernanceGateResult::Blocked { reason } => {
                        tracing::warn!(reason = %reason, "governance gate blocked execution");
                        let governance_receipt = Some(receipt.clone());
                        iterations.push(LoopIterationReport {
                            schema_version: LOOP_ITERATION_REPORT_V1_SCHEMA.into(),
                            iteration: iteration_index + 1,
                            selected_target_key: None,
                            action_family: None,
                            export_completed: false,
                            import_completed: false,
                            degraded,
                            advisory_only: false,
                            outcome_signature: None,
                            verification_case: None,
                            check_plan: None,
                            verification_attempt: None,
                            scheduler_decision: None,
                            policy_decision: None,
                            approval_records: Vec::new(),
                            calibration_snapshot: None,
                            adjudication: None,
                            control_receipt: None,
                            target_normalization: None,
                            decision_audit: None,
                            lineage_receipt: None,
                            export_trace: None,
                            stop_rule_evaluation: None,
                            verification_plan_artifact: None,
                            boundary_repairs: Vec::new(),
                            repair_records: Vec::new(),
                            rollback_invalidation_count: None,
                            #[cfg(feature = "governance")]
                            governance_receipt,
                        });
                        let halt_reason = if reason.contains("incident") {
                            HaltReason::GovernanceIncidentHalt
                        } else {
                            HaltReason::GovernanceAuthorityInsufficient
                        };
                        return Ok(self.finish_report(
                            started_at,
                            trace_ctx.clone(),
                            attempt_id.clone(),
                            started,
                            iterations,
                            totals,
                            halt_reason,
                            Some(&*observation),
                        ));
                    }
                    crate::governance_gate::GovernanceGateResult::AdvisoryOnly { reason } => {
                        tracing::warn!(reason = %reason, "governance gate advisory-only mode; halting execution");
                        totals.advisory_only_steps = true;
                        // AUTH-003 fix: AdvisoryOnly must NOT proceed to execution.
                        // Only an Authorized result permits effect invocation.
                        return Ok(self.finish_report(
                            started_at,
                            trace_ctx.clone(),
                            attempt_id.clone(),
                            started,
                            iterations,
                            totals,
                            HaltReason::AdvisoryOnlyFallback,
                            Some(&*observation),
                        ));
                    }
                    crate::governance_gate::GovernanceGateResult::Allow => {}
                }
            }

            let candidates = score_targets(&observation, &self.history, &self.config);
            let decision = select_candidate(candidates, &self.history, &self.config);
            let Some(selected) = decision.selected else {
                let halt_reason = if totals.advisory_only_steps {
                    HaltReason::AdvisoryOnlyFallback
                } else if decision.exhausted_candidate_count > 0 {
                    if totals.advisory_only_steps {
                        HaltReason::AdvisoryOnlyFallback
                    } else {
                        HaltReason::AllTargetsExhausted
                    }
                } else if observation.thin_export_active() {
                    HaltReason::ThinExportBlockedExecution
                } else if matches!(
                    observation.status.disposition,
                    ObservationDisposition::ImportRequired
                ) {
                    HaltReason::ImportRequired
                } else {
                    HaltReason::NothingWorthInvestigating
                };
                return Ok(self.finish_report(
                    started_at,
                    trace_ctx.clone(),
                    attempt_id.clone(),
                    started,
                    iterations,
                    totals,
                    halt_reason,
                    Some(&*observation),
                ));
            };

            self.history.mark_selected(&selected.stable_key);
            totals
                .targets_investigated
                .push(selected.stable_key.clone());
            let iteration_started_at = iteration_timestamp.to_rfc3339();
            let trial_id = TrialId::generate();
            let verification_case = VerificationCase::new(
                selected.target.case_class(),
                CaseRegion {
                    namespace: self.config.scope.namespace.clone(),
                    scope_key: Some(self.config.scope.key()),
                    target_key: selected.stable_key.clone(),
                    region_id: None,
                    region_digest_id: None,
                    claim_version_id: selected.target.primary_claim_version_id(),
                    as_of_recorded_at: observation
                        .import_log
                        .as_ref()
                        .map(|row| row.imported_at.clone()),
                },
                trace_ctx.clone(),
                attempt_id.clone(),
                iteration_started_at.clone(),
                degraded,
                matches!(
                    selected.plan,
                    crate::act::PlanKind::AdvisoryOnlyVerificationPlan(_)
                ),
            );
            let check_plan = CheckPlan::new(
                verification_case.case_id.clone(),
                map_check_method(&selected.plan),
                selected.plan.check_names(),
                promotion_class_for_plan(&selected.plan, degraded),
                reversibility_class_for_plan(&selected.plan),
                !matches!(
                    selected.plan,
                    crate::act::PlanKind::AdvisoryOnlyVerificationPlan(_)
                ) && !degraded,
                matches!(
                    selected.plan,
                    crate::act::PlanKind::AdvisoryOnlyVerificationPlan(_)
                ),
                degraded,
                selected.rationale.clone(),
                plan_input(&selected.plan),
            );
            let mut check_plan = check_plan;
            check_plan.target_exactness = if degraded {
                semantic_memory_forge::ExactnessLevelV1::Conservative
            } else {
                semantic_memory_forge::ExactnessLevelV1::Exact
            };
            check_plan.evidence_admissibility = if degraded {
                semantic_memory_forge::EvidenceAdmissibilityV1::Restricted
            } else {
                semantic_memory_forge::EvidenceAdmissibilityV1::Admissible
            };
            check_plan.cheap_check_ladder = decision
                .audit
                .as_ref()
                .map(|audit| audit.cheap_check_ladder.clone())
                .unwrap_or_default();
            check_plan.proof_obligations_remaining = if degraded
                || matches!(
                    selected.plan,
                    crate::act::PlanKind::AdvisoryOnlyVerificationPlan(_)
                ) {
                vec!["exact proof path unavailable".into()]
            } else {
                Vec::new()
            };
            check_plan.proof_profile = Some(verification_control::ProofProfileV1 {
                profile_name: "forge-pilot.loop".into(),
                evidence_admissibility: check_plan.evidence_admissibility.clone(),
                cheap_checks: check_plan.cheap_check_ladder.clone(),
                proof_obligations_remaining: check_plan.proof_obligations_remaining.clone(),
                admissible_evidence: if degraded {
                    vec!["control_receipt".into(), "advisory_oracle_slice".into()]
                } else {
                    vec![
                        "control_receipt".into(),
                        "replay_lineage".into(),
                        "oracle_slice".into(),
                    ]
                },
            });
            let policy_snapshot =
                policy_as_of(&self.config.policy_snapshots, &iteration_started_at)
                    .cloned()
                    .unwrap_or_else(|| {
                        self.config
                            .policy_snapshots
                            .first()
                            .cloned()
                            .unwrap_or_else(|| {
                                // AUTH-001 fix: fail-closed when no policy is configured.
                                // Use deny-default instead of permissive to prevent
                                // missing governance state from increasing authority.
                                verification_policy::PolicySnapshot::deny(
                                    "forge-pilot.fallback-deny",
                                    iteration_started_at.clone(),
                                )
                            })
                    });
            let budget_exhausted = started.elapsed().as_secs() >= self.config.time_budget_secs;
            let policy_decision = evaluate_policy(
                &policy_snapshot,
                &verification_case,
                &check_plan,
                &self.config.approval_records,
                degraded,
                budget_exhausted,
            );
            let approval_records = self
                .config
                .approval_records
                .iter()
                .filter(|approval| {
                    approval.policy_version == policy_snapshot.policy_version
                        && approval_matches(
                            approval,
                            &verification_case,
                            &check_plan,
                            iteration_started_at.as_str(),
                        )
                })
                .cloned()
                .collect::<Vec<_>>();
            let execution_permit = policy_decision.issue_execution_permit(
                &verification_case,
                &check_plan,
                &approval_records,
            );
            let scheduler_decision = schedule_check_plan(
                &verification_case,
                &check_plan,
                BudgetLineage {
                    budget_family: "forge-pilot.orchestration".into(),
                    retry_family: attempt_id.clone(),
                    queue_hop_count: 1,
                    max_time_budget_ms: Some(self.config.time_budget_secs * 1000),
                    remaining_time_budget_ms: Some(
                        self.config
                            .time_budget_secs
                            .saturating_sub(started.elapsed().as_secs())
                            * 1000,
                    ),
                    max_cost_budget_units: None,
                    remaining_cost_budget_units: None,
                    exhausted: budget_exhausted,
                },
                observation
                    .degradations
                    .iter()
                    .map(|degradation| DegradationMarker {
                        kind: degradation.kind.clone(),
                        reason: degradation.detail.clone(),
                        blocks_promotion: true,
                    })
                    .collect(),
                vec![QueueHop {
                    hop_index: 0,
                    from_queue: "pilot.observe".into(),
                    to_queue: "pilot.verify".into(),
                    enqueued_at: iteration_started_at.clone(),
                    dequeued_at: Some(iteration_timestamp.to_rfc3339()),
                }],
            );

            let mut action_family = None;
            let mut export_completed = false;
            let mut import_completed = false;
            let mut advisory_only = check_plan.advisory_only;
            let mut outcome_signature = None::<String>;
            let mut refuted = false;
            let mut produced_artifact_refs = Vec::<String>::new();

            if let Ok(permit) = execution_permit.as_ref() {
                let action = execute_plan(
                    &observation,
                    &selected.stable_key,
                    &selected.plan,
                    permit,
                    &self.config,
                    &self.resources.forge_store,
                )
                .await?;
                action_family = Some(action.family);
                advisory_only = action.advisory_only;
                outcome_signature = Some(action.outcome_signature.clone());
                totals.actions_executed += u32::from(!action.advisory_only);
                totals.advisory_only_steps |= action.advisory_only;

                if let Some(bundle) = &action.bundle {
                    produced_artifact_refs.push(bundle.bundle_id.clone());
                    let roundtrip = canonical_roundtrip(
                        bundle,
                        &self.config.scope.namespace,
                        &self.resources.forge_store,
                        &self.resources.memory_store,
                    )
                    .await?;
                    totals.exports_completed += 1;
                    totals.imports_completed +=
                        u32::from(roundtrip.import_result.status == "complete");
                    export_completed = true;
                    import_completed = roundtrip.import_result.status == "complete";
                }

                self.history.record_outcome(
                    &selected.stable_key,
                    action
                        .bundle
                        .as_ref()
                        .map(|bundle| bundle.bundle_id.clone()),
                    action.outcome_signature.clone(),
                    self.config.max_retries_per_target,
                );

                refuted = action
                    .oracle_execution
                    .as_ref()
                    .and_then(|oracle| oracle.refutation.as_ref())
                    .is_some_and(|refutation| {
                        matches!(
                            refutation.outcome,
                            kernel_oracles::OracleRefutationOutcome::FlipWitness { .. }
                        )
                    });
            } else {
                if execution_permit
                    .as_ref()
                    .err()
                    .is_some_and(permit_error_forces_advisory_only)
                {
                    action_family = Some(ActionFamily::AdvisoryOnly);
                    advisory_only = true;
                    totals.advisory_only_steps |= true;
                }
                self.history.record_outcome(
                    &selected.stable_key,
                    None,
                    if advisory_only {
                        "advisory_only_by_policy".into()
                    } else {
                        "blocked_by_policy".into()
                    },
                    self.config.max_retries_per_target,
                );
            }

            let calibration_snapshot = CalibrationSnapshot::evaluate(
                verification_case.case_id.clone(),
                iteration_timestamp.to_rfc3339(),
                observation
                    .recent_imports
                    .iter()
                    .filter_map(|row| row.comparability_snapshot_version.clone())
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    <= 1,
                observation
                    .explanation
                    .as_ref()
                    .map_or(true, |explanation| {
                        explanation.calibration_caveats.is_empty()
                    }),
                self.config.calibration_abstention_threshold_micros,
                if degraded {
                    self.config.calibration_abstention_threshold_micros
                } else {
                    100_000
                },
                observation
                    .degradations
                    .iter()
                    .map(|degradation| format!("{}:{}", degradation.kind, degradation.detail))
                    .chain(
                        observation
                            .explanation
                            .as_ref()
                            .into_iter()
                            .flat_map(|explanation| explanation.calibration_caveats.clone()),
                    )
                    .collect(),
            );
            if calibration_snapshot.forces_advisory_only {
                advisory_only = true;
                totals.advisory_only_steps |= true;
            }
            if advisory_only {
                apply_advisory_plan_constraints(&mut check_plan);
            }
            let effective_degraded = degraded || advisory_only;

            let verification_attempt = VerificationAttempt::completed(
                verification_case.case_id.clone(),
                check_plan.plan_id.clone(),
                attempt_id.clone(),
                Some(trial_id),
                if execution_permit.is_err() && !advisory_only {
                    VerificationAttemptState::Blocked
                } else if advisory_only {
                    VerificationAttemptState::AdvisoryOnly
                } else {
                    VerificationAttemptState::Succeeded
                },
                advisory_only,
                effective_degraded,
                iteration_started_at.clone(),
                iteration_timestamp.to_rfc3339(),
                outcome_signature.clone(),
            );
            let control_receipt = ControlReceipt::new_case_execution(
                &verification_case,
                &check_plan,
                &verification_attempt,
                false,
                json!({
                    "target_key": selected.stable_key.clone(),
                    "action_family": action_family.as_ref().map(|family| format!("{:?}", family)),
                    "export_completed": export_completed,
                    "import_completed": import_completed,
                    "policy_allowed": execution_permit.is_ok(),
                    "scheduler_promotion_blocked": scheduler_decision.promotion_blocked,
                }),
            );
            let adjudication = adjudicate_case(
                &verification_case,
                &check_plan,
                &verification_attempt,
                &control_receipt,
                &policy_decision,
                &calibration_snapshot,
                refuted,
                scheduler_decision.budget_lineage.exhausted,
                currently_promoted(&observation, &selected),
            );
            let rollback_invalidation_count = if adjudication.rollback_plan.required {
                execute_rollback_invalidation(
                    &self.resources.memory_store,
                    &verification_case,
                    &adjudication,
                )
                .await?
            } else {
                None
            };
            let replayed = replay_case(&[
                LedgerEntry::new(
                    verification_case.case_id.clone(),
                    1,
                    LedgerEvent::CaseOpened {
                        case: verification_case.clone(),
                    },
                ),
                LedgerEntry::new(
                    verification_case.case_id.clone(),
                    2,
                    LedgerEvent::PlanAdopted {
                        plan: check_plan.clone(),
                    },
                ),
                LedgerEntry::new(
                    verification_case.case_id.clone(),
                    3,
                    LedgerEvent::AttemptRecorded {
                        attempt: verification_attempt.clone(),
                    },
                ),
                LedgerEntry::new(
                    verification_case.case_id.clone(),
                    4,
                    LedgerEvent::ReceiptAppended {
                        receipt: Box::new(control_receipt.clone()),
                    },
                ),
                LedgerEntry::new(
                    verification_case.case_id.clone(),
                    5,
                    LedgerEvent::CaseClosed {
                        case_id: verification_case.case_id.clone(),
                        disposition: adjudication.terminal_disposition.clone(),
                    },
                ),
            ])
            .map_err(PilotError::ControlPlaneReplay)?;

            let lineage_receipt = build_lineage_receipt(
                replayed.case.as_ref().unwrap_or(&verification_case),
                &check_plan,
                &verification_attempt,
                &selected.stable_key,
                outcome_signature
                    .clone()
                    .unwrap_or_else(|| "no_action".into()),
                vec![
                    selected.stable_key.clone(),
                    selected.normalization.bounded_region_digest.clone(),
                ],
                produced_artifact_refs.clone(),
                observation
                    .degradations
                    .iter()
                    .map(|degradation| format!("{}:{}", degradation.kind, degradation.detail))
                    .collect(),
            );
            let export_trace = build_export_trace(
                replayed.case.as_ref().unwrap_or(&verification_case),
                &check_plan,
                &verification_attempt,
                action_family,
                export_completed,
                import_completed,
                produced_artifact_refs,
            );
            let retry_cap_reached = self.history.retry_count(&selected.stable_key)
                >= self.config.max_retries_per_target;
            let stop_rule_evaluation = build_stop_rule_evaluation(
                if retry_cap_reached {
                    "retry_cap_exceeded"
                } else if advisory_only {
                    "advisory_only"
                } else if effective_degraded {
                    "degraded_continue"
                } else {
                    "continue_or_complete"
                },
                retry_cap_reached,
                self.config.cooldown_secs > 0 && iteration_index + 1 < self.config.max_iterations,
                self.history.is_exhausted(&selected.stable_key),
                effective_degraded,
                advisory_only,
            );
            let execution_context = build_execution_context(
                replayed.case.as_ref().unwrap_or(&verification_case),
                &check_plan,
                &verification_attempt,
                &scheduler_decision,
                effective_degraded,
                advisory_only,
            );
            let verification_plan_artifact = build_verification_plan_artifact(
                &check_plan,
                {
                    let mut blocked_checks: Vec<String> = decision
                        .audit
                        .as_ref()
                        .map(|audit| audit.blocked_steps.as_slice())
                        .unwrap_or(&[])
                        .iter()
                        .map(|blocked| format!("{:?}: {}", blocked.step_kind, blocked.reason))
                        .collect();
                    if check_plan.degraded {
                        blocked_checks.push("plan degraded".into());
                    }
                    if advisory_only {
                        blocked_checks
                            .push("advisory-only path blocks promotion-bearing execution".into());
                    }
                    if calibration_snapshot.forces_advisory_only {
                        blocked_checks.push("calibration forced advisory-only disposition".into());
                    }
                    blocked_checks
                },
                if execution_permit.is_ok() {
                    Vec::new()
                } else {
                    let mut reasons = policy_decision.reasons.clone();
                    if let Err(error) = &execution_permit {
                        reasons.push(error.to_string());
                    }
                    reasons
                },
                if effective_degraded {
                    vec!["runtime degraded".into()]
                } else {
                    Vec::new()
                },
            );
            let mut repair_records = Vec::new();
            if adjudication.rollback_plan.required || effective_degraded || advisory_only {
                repair_records.push(build_repair_record(
                    format!("repair:{}", verification_case.case_id),
                    vec![selected.stable_key.clone()],
                    if adjudication.rollback_plan.required {
                        RepairClassV1::RollbackRepair
                    } else {
                        RepairClassV1::VerificationStateRepair
                    },
                    vec![control_receipt.receipt_id.to_string()],
                    if adjudication.rollback_plan.required {
                        "projection_scope".into()
                    } else {
                        "verification_state".into()
                    },
                    if adjudication.rollback_plan.required {
                        "requires_recompute".into()
                    } else {
                        "replayable".into()
                    },
                    if adjudication.rollback_plan.required {
                        format!(
                            "rollback invalidated {} derived rows",
                            rollback_invalidation_count.unwrap_or(0)
                        )
                    } else if advisory_only {
                        "kept output advisory-only pending stronger verification".into()
                    } else {
                        "recorded degraded execution without mutating truth".into()
                    },
                    execution_context.clone(),
                ));
            }
            if retry_cap_reached {
                iterations.push(LoopIterationReport {
                    schema_version: LOOP_ITERATION_REPORT_V1_SCHEMA.into(),
                    iteration: iteration_index + 1,
                    selected_target_key: Some(selected.stable_key),
                    action_family,
                    export_completed,
                    import_completed,
                    degraded,
                    advisory_only,
                    outcome_signature,
                    verification_case: replayed.case,
                    check_plan: Some(check_plan),
                    verification_attempt: Some(verification_attempt),
                    scheduler_decision: Some(scheduler_decision),
                    policy_decision: Some(policy_decision),
                    approval_records,
                    calibration_snapshot: Some(calibration_snapshot),
                    adjudication: Some(adjudication),
                    control_receipt: Some(control_receipt),
                    target_normalization: Some(selected.normalization),
                    decision_audit: decision.audit,
                    lineage_receipt: Some(lineage_receipt),
                    export_trace: Some(export_trace),
                    stop_rule_evaluation: Some(stop_rule_evaluation),
                    verification_plan_artifact: Some(verification_plan_artifact),
                    boundary_repairs: Vec::new(),
                    repair_records,
                    rollback_invalidation_count,
                    #[cfg(feature = "governance")]
                    governance_receipt: governance_gate_result.as_ref().map(|(_, r)| r.clone()),
                });
                return Ok(self.finish_report(
                    started_at,
                    trace_ctx.clone(),
                    attempt_id.clone(),
                    started,
                    iterations,
                    totals,
                    HaltReason::RetryCapExceeded,
                    Some(&*observation),
                ));
            }

            iterations.push(LoopIterationReport {
                schema_version: LOOP_ITERATION_REPORT_V1_SCHEMA.into(),
                iteration: iteration_index + 1,
                selected_target_key: Some(selected.stable_key),
                action_family,
                export_completed,
                import_completed,
                degraded: effective_degraded,
                advisory_only,
                outcome_signature,
                verification_case: replayed.case,
                check_plan: Some(check_plan),
                verification_attempt: Some(verification_attempt),
                scheduler_decision: Some(scheduler_decision),
                policy_decision: Some(policy_decision),
                approval_records,
                calibration_snapshot: Some(calibration_snapshot),
                adjudication: Some(adjudication),
                control_receipt: Some(control_receipt),
                target_normalization: Some(selected.normalization),
                decision_audit: decision.audit,
                lineage_receipt: Some(lineage_receipt),
                export_trace: Some(export_trace),
                stop_rule_evaluation: Some(stop_rule_evaluation),
                verification_plan_artifact: Some(verification_plan_artifact),
                boundary_repairs: Vec::new(),
                repair_records,
                rollback_invalidation_count,
                #[cfg(feature = "governance")]
                governance_receipt: governance_gate_result.as_ref().map(|(_, r)| r.clone()),
            });

            if advisory_only
                && matches!(
                    observation.status.disposition,
                    ObservationDisposition::ImportRequired
                )
                && matches!(
                    selected.plan,
                    crate::act::PlanKind::AdvisoryOnlyVerificationPlan(_)
                )
            {
                return Ok(self.finish_report(
                    started_at,
                    trace_ctx.clone(),
                    attempt_id.clone(),
                    started,
                    iterations,
                    totals,
                    HaltReason::AdvisoryOnlyFallback,
                    Some(&*observation),
                ));
            }

            if self.config.cooldown_secs > 0 && iteration_index + 1 < self.config.max_iterations {
                sleep(Duration::from_secs(self.config.cooldown_secs)).await;
            }
        }

        Ok(self.finish_report(
            started_at,
            trace_ctx,
            attempt_id,
            started,
            iterations,
            totals,
            HaltReason::MaxIterationsReached,
            last_observation.as_deref(),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_report(
        &self,
        started_at: chrono::DateTime<chrono::Utc>,
        trace_ctx: TraceCtx,
        attempt_id: AttemptId,
        started: Instant,
        iterations: Vec<LoopIterationReport>,
        totals: LoopTotals,
        halt_reason: HaltReason,
        observation: Option<&Observation>,
    ) -> LoopReport {
        let elapsed_seconds = started.elapsed().as_secs_f64();
        let finished_at = chrono::Utc::now().to_rfc3339();
        let receipt = LoopReceipt {
            schema_version: PILOT_LOOP_RECEIPT_V1_SCHEMA.into(),
            receipt_id: uuid::Uuid::new_v4().to_string(),
            trace_ctx,
            attempt_id,
            namespace: self.config.scope.namespace.clone(),
            workspace_path: self.config.workspace_path.clone(),
            observation_paths: observation.map(|observation| observation.paths.clone()),
            observation_status: observation.map(|observation| observation.status.clone()),
            non_authoritative: true,
            advisory_only: totals.advisory_only_steps,
            consumer_only_truth_access: true,
            budget: LoopBudgetReceipt {
                workload_class: "orchestration".into(),
                time_budget_secs: self.config.time_budget_secs,
                max_iterations: self.config.max_iterations,
                cooldown_secs: self.config.cooldown_secs,
                max_retries_per_target: self.config.max_retries_per_target,
            },
            halt_reason: halt_reason.clone(),
            iterations_completed: iterations.len() as u32,
            actions_executed: totals.actions_executed,
            exports_completed: totals.exports_completed,
            imports_completed: totals.imports_completed,
            degraded_iterations: totals.degraded_iterations,
            targets_investigated: totals.targets_investigated.clone(),
            started_at: started_at.to_rfc3339(),
            finished_at,
            elapsed_seconds,
        };
        let mut report = LoopReport {
            schema_version: LOOP_REPORT_V1_SCHEMA.into(),
            receipt,
            iterations_completed: iterations.len() as u32,
            actions_executed: totals.actions_executed,
            exports_completed: totals.exports_completed,
            imports_completed: totals.imports_completed,
            halt_reason,
            targets_investigated: totals.targets_investigated,
            degraded_iterations: totals.degraded_iterations,
            elapsed_seconds,
            advisory_only_steps: totals.advisory_only_steps,
            iterations,
        };
        apply_loop_report_boundary_repairs(&mut report);
        report
    }
}

fn permit_error_forces_advisory_only(error: &verification_policy::PermitIssuanceError) -> bool {
    matches!(
        error,
        verification_policy::PermitIssuanceError::PlanNotPromotionCapable
            | verification_policy::PermitIssuanceError::PromotionBlocked
            | verification_policy::PermitIssuanceError::CitationIncomplete
            | verification_policy::PermitIssuanceError::ObligationRefsIncomplete
    )
}

fn apply_advisory_plan_constraints(plan: &mut CheckPlan) {
    plan.advisory_only = true;
    plan.promotable_if_completed = false;
    plan.target_exactness = semantic_memory_forge::ExactnessLevelV1::Conservative;
    plan.evidence_admissibility = semantic_memory_forge::EvidenceAdmissibilityV1::Restricted;
    if plan.proof_obligations_remaining.is_empty() {
        plan.proof_obligations_remaining
            .push("promotion requires a non-advisory proof path".into());
    }
    if let Some(profile) = plan.proof_profile.as_mut() {
        profile.evidence_admissibility = plan.evidence_admissibility.clone();
        profile.proof_obligations_remaining = plan.proof_obligations_remaining.clone();
        profile.admissible_evidence =
            vec!["control_receipt".into(), "advisory_oracle_slice".into()];
    }
}
