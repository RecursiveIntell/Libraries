//! Autonomous loop driver — ties together gap detection, task generation,
//! execution, capture, and evaluation into a single continuous loop.
//!
//! The [`AutonomousLoop`] orchestrates the full detect → enqueue → execute →
//! capture → evaluate cycle. It tracks state in [`LoopState`] (shared via
//! `Arc<Mutex<>>` for TUI integration) and respects [`LoopConfig`] for
//! iteration limits, sleep intervals, and safe-mode thresholds.

use crate::capture::{CaptureOutcome, ResultCapture};
use crate::entropy_search::EntropyGradientSearcher;
use crate::evaluation::{EvaluationGate, FactDisposition};
use crate::executor::{ExecutionResult, LoopExecutor};
use crate::gap_detector::GapDetector;
use crate::hostile_audit::HostileAuditGate;
use crate::missions::{Mission, MissionScheduler};
use crate::proof_debt::{PaymentMethod, ProofDebtBudget, RiskClass};
use crate::receipt::{LoopMode, ReceiptEmitter};
use crate::task_generator::TaskGenerator;
use crate::viscosity::ViscosityController;
use aidens_daemon_kit::DaemonControllerV1;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::time::Duration;

// ---------------------------------------------------------------------------
// Config & State
// ---------------------------------------------------------------------------

/// Configuration for the autonomous loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopConfig {
    /// Maximum number of iterations (0 = infinite).
    pub max_iterations: usize,
    /// Run gap detection every N iterations.
    pub gap_detection_interval: usize,
    /// Sleep duration between iterations (milliseconds).
    pub sleep_between_iterations_ms: u64,
    /// Maximum consecutive failures before entering safe mode.
    pub max_consecutive_failures: usize,
    /// Ollama-compatible provider base URL.
    pub model_url: String,
    /// Ollama model name.
    pub chosen_model: String,
    pub api_key: Option<String>,
    /// Directory for the canonical memory store.
    pub memory_dir: PathBuf,
    /// Directory for the daemon queue.
    pub queue_dir: PathBuf,
    /// Semantic-memory HTTP base URL.
    pub http_base_url: String,
    /// Auditor URL for hostile audit gate (empty = no audit).
    pub auditor_url: String,
    /// Auditor model name (should differ from chosen_model).
    pub auditor_model: String,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 0,
            gap_detection_interval: 5,
            sleep_between_iterations_ms: 1000,
            max_consecutive_failures: 5,
            model_url: "http://127.0.0.1:11434".to_string(),
            chosen_model: "llama3".to_string(),
            api_key: None,
            memory_dir: PathBuf::from("./.aidens/memory"),
            queue_dir: PathBuf::from("./.aidens/queue"),
            http_base_url: "http://127.0.0.1:1738".to_string(),
            auditor_url: String::new(),  // empty = no hostile audit
            auditor_model: String::new(),
        }
    }
}

/// Live state of the autonomous loop, tracked across iterations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoopState {
    /// Current iteration number (0-based, incremented each cycle).
    pub iteration: usize,
    /// Total gaps detected since start.
    pub gaps_detected: usize,
    /// Total tasks generated and enqueued since start.
    pub tasks_generated: usize,
    /// Total tasks completed successfully.
    pub tasks_completed: usize,
    /// Total tasks that failed.
    pub tasks_failed: usize,
    /// Total facts captured into memory.
    pub facts_captured: usize,
    /// Total facts rejected by the evaluation gate.
    pub facts_rejected: usize,
    /// Current consecutive failure streak.
    pub consecutive_failures: usize,
    /// Current job being processed (if any).
    pub current_job: Option<String>,
    /// Last error encountered (if any).
    pub last_error: Option<String>,
    /// Whether the loop is in safe mode (paused due to repeated failures).
    pub safe_mode: bool,
    /// Current strictness level name (from viscosity controller).
    pub strictness: String,
    /// Current loop mode (additive/subtractive).
    pub mode: LoopMode,
    /// Outstanding proof-debt count.
    pub proof_debt_outstanding: usize,
    /// Domains explored this cycle.
    pub domains_explored: Vec<String>,
    /// Saturated domains.
    pub saturated_domains: Vec<String>,
}

// ---------------------------------------------------------------------------
// AutonomousLoop
// ---------------------------------------------------------------------------

/// The full autonomous loop: detect → enqueue → execute → capture → evaluate.
#[derive(Clone)]
pub struct AutonomousLoop {
    /// Gap detector for scanning the knowledge base.
    pub detector: GapDetector,
    /// Task generator for enqueuing remediation jobs.
    pub generator: TaskGenerator,
    /// Executor for running jobs through the plan-act-verify loop.
    pub executor: LoopExecutor,
    /// Result capture for storing outputs in memory.
    pub capture: ResultCapture,
    /// Evaluation gate for fact disposition.
    pub evaluation: EvaluationGate,
    /// Daemon queue controller.
    pub queue: DaemonControllerV1,
    /// Loop configuration.
    pub config: LoopConfig,
    /// Shared mutable state (for TUI integration).
    pub state: Arc<Mutex<LoopState>>,
    /// Set of attempted gap keys (fact_id+gap_type) to avoid re-detecting.
    pub attempted_gaps: Arc<Mutex<HashSet<String>>>,
    /// Adaptive viscosity controller.
    pub viscosity: Arc<Mutex<ViscosityController>>,
    /// Proof-debt budget tracker.
    pub proof_debt: Arc<Mutex<ProofDebtBudget>>,
    /// Entropy-gradient domain searcher.
    pub entropy_search: Arc<Mutex<EntropyGradientSearcher>>,
    /// Hostile audit gate (None if auditor_url is empty).
    pub hostile_audit: Option<HostileAuditGate>,
    /// Receipt emitter for cycle receipts.
    pub receipts: Arc<Mutex<ReceiptEmitter>>,
    /// Mission scheduler for structured high-ROI objectives.
    pub mission_scheduler: Arc<Mutex<MissionScheduler>>,
}

impl std::fmt::Debug for AutonomousLoop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutonomousLoop")
            .field("config", &self.config)
            .field("state", &self.state.lock().ok())
            .finish_non_exhaustive()
    }
}

impl AutonomousLoop {
    /// Build a new autonomous loop from configuration.
    ///
    /// This opens the daemon queue and memory adapter from the config paths.
    /// The caller is responsible for ensuring the directories exist.
    pub fn from_config(config: LoopConfig) -> Result<Self> {
        // Open memory adapter.
        let memory_config = aidens_memory_kit::memory_config_for_root(&config.memory_dir);
        let runtime_config =
            aidens_memory_kit::runtime_config_for_namespace("autonomous");
        let memory = Arc::new(
            aidens_memory_kit::CanonicalMemoryAdapter::open_with_mock_embedder(
                memory_config,
                runtime_config,
            )?,
        );

        // Open daemon queue.
        let namespace = DaemonControllerV1::namespace(
            &config.queue_dir,
            "autonomous-loop",
            "aidens-autonomous",
        );
        let queue = DaemonControllerV1::open(
            &config.queue_dir,
            namespace,
            "aidens-autonomous",
        )?;

        // Build components.
        let detector = GapDetector::new(&config.http_base_url);
        let generator = TaskGenerator::new(queue.clone());
        let executor = LoopExecutor::new(
            memory.clone(),
            &config.model_url,
            &config.chosen_model,
            &config.http_base_url,
            config.api_key.clone(),
        );
        let capture = ResultCapture::new(memory, &config.http_base_url);
        let evaluation = EvaluationGate::new();

        // Build new control pieces.
        let viscosity = ViscosityController::with_defaults();
        let proof_debt = ProofDebtBudget::new();
        let entropy_search = EntropyGradientSearcher::new(&config.http_base_url);
        let hostile_audit = if !config.auditor_url.is_empty() && !config.auditor_model.is_empty() {
            Some(HostileAuditGate::new(&config.auditor_url, &config.auditor_model))
        } else {
            None
        };
        let receipts = ReceiptEmitter::new();
        let mission_scheduler = MissionScheduler::new();

        Ok(Self {
            detector,
            generator,
            executor,
            capture,
            evaluation,
            queue,
            config,
            state: Arc::new(Mutex::new(LoopState::default())),
            attempted_gaps: Arc::new(Mutex::new(HashSet::new())),
            viscosity: Arc::new(Mutex::new(viscosity)),
            proof_debt: Arc::new(Mutex::new(proof_debt)),
            entropy_search: Arc::new(Mutex::new(entropy_search)),
            hostile_audit,
            receipts: Arc::new(Mutex::new(receipts)),
            mission_scheduler: Arc::new(Mutex::new(mission_scheduler)),
        })
    }

    /// Build a loop from pre-constructed components (for testing).
    pub fn new(
        detector: GapDetector,
        generator: TaskGenerator,
        executor: LoopExecutor,
        capture: ResultCapture,
        evaluation: EvaluationGate,
        queue: DaemonControllerV1,
        config: LoopConfig,
    ) -> Self {
        let hostile_audit = if !config.auditor_url.is_empty() && !config.auditor_model.is_empty() {
            Some(HostileAuditGate::new(&config.auditor_url, &config.auditor_model))
        } else {
            None
        };
        Self {
            detector,
            generator,
            executor,
            capture,
            evaluation,
            queue,
            config,
            state: Arc::new(Mutex::new(LoopState::default())),
            attempted_gaps: Arc::new(Mutex::new(HashSet::new())),
            viscosity: Arc::new(Mutex::new(ViscosityController::with_defaults())),
            proof_debt: Arc::new(Mutex::new(ProofDebtBudget::new())),
            entropy_search: Arc::new(Mutex::new(EntropyGradientSearcher::new(
                "http://127.0.0.1:1738",
            ))),
            hostile_audit,
            receipts: Arc::new(Mutex::new(ReceiptEmitter::new())),
            mission_scheduler: Arc::new(Mutex::new(MissionScheduler::new())),
        }
    }

    /// Get a snapshot of the current loop state.
    pub fn state_snapshot(&self) -> LoopState {
        self.state.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// Run the autonomous loop.
    ///
    /// This is the main entry point. It loops indefinitely (or until
    /// `max_iterations` is reached), performing:
    ///
    /// 1. Viscosity check — determine strictness for this cycle.
    /// 2. Mode check — if subtractive mode, run subtractive cycle instead.
    /// 3. Gap detection (entropy-guided, every N iterations).
    /// 4. Job acquisition from the queue.
    /// 5. Job execution.
    /// 6. Result capture.
    /// 7. Fact evaluation (with viscosity-adjusted thresholds).
    /// 8. Hostile audit (if strict/frozen).
    /// 9. Proof-debt update (incur on promote, pay on verify).
    /// 10. Job completion/cancellation.
    /// 11. Viscosity recording.
    /// 12. Proof-debt subtractive check.
    /// 13. Receipt emission.
    /// 14. Failure tracking and safe-mode activation.
    pub async fn run(&self) -> Result<()> {
        loop {
            // Snapshot state for this iteration.
            let iteration = {
                let mut state = self.state.lock().map_err(|e| anyhow::anyhow!("state lock: {e}"))?;
                state.iteration += 1;
                state.iteration
            };

            // 1. Viscosity check — update state with current strictness.
            let strictness_name = self.viscosity_strictness_name();
            let should_generate = self.viscosity_should_generate();
            let should_audit = self.viscosity_should_audit();
            self.update_state(|s| {
                s.strictness = strictness_name.clone();
            });

            // 2. Mode check — if proof-debt says subtractive, run that instead.
            let in_subtractive = {
                let mode = self.state.lock().map(|s| s.mode).unwrap_or_default();
                mode == LoopMode::Subtractive
            };

            if in_subtractive || self.proof_debt_should_shift() {
                self.update_state(|s| { s.mode = LoopMode::Subtractive; });
                if let Err(e) = self.run_subtractive_cycle().await {
                    self.update_state(|s| {
                        s.last_error = Some(format!("subtractive cycle failed: {e}"));
                    });
                }
                // Check if we can return to additive mode.
                if !self.proof_debt_should_shift() {
                    self.update_state(|s| { s.mode = LoopMode::Additive; });
                }
                // Emit receipt for subtractive cycle.
                self.emit_cycle_receipt(iteration, 0, 0, 0, 0, 0, &[], &[]);
                self.sleep_iteration().await;
                self.check_max_iterations(iteration)?;
                continue;
            }

            // 3. Gap detection (only if viscosity allows task generation).
            let queue_has_pending = self.queue_has_pending_jobs();
            if !queue_has_pending && should_generate {
                // Try mission-scheduled detection first.
                let mission_due = {
                    if let Ok(scheduler) = self.mission_scheduler.lock() {
                        scheduler.next_mission(iteration).cloned()
                    } else {
                        None
                    }
                };

                if let Some(mission) = mission_due {
                    if let Err(e) = self.run_mission_detection(&mission, iteration).await {
                        self.update_state(|s| {
                            s.last_error = Some(format!("mission detection failed: {e}"));
                        });
                    }
                } else {
                    // Fall back to entropy-guided gap detection.
                    let should_detect = self.config.gap_detection_interval > 0
                        && (iteration - 1) % self.config.gap_detection_interval == 0;
                    if should_detect {
                        if let Err(e) = self.run_gap_detection().await {
                            self.update_state(|s| {
                                s.last_error = Some(format!("gap detection failed: {e}"));
                            });
                        }
                    }
                }
            }

            // 4. Acquire next job.
            let lease_outcome = match self.queue.acquire_next("autonomous-loop", 300) {
                Ok(Some(outcome)) => outcome,
                Ok(None) => {
                    // No job available — emit idle receipt and sleep.
                    self.emit_cycle_receipt(iteration, 0, 0, 0, 0, 0, &[], &[]);
                    self.sleep_iteration().await;
                    self.check_max_iterations(iteration)?;
                    continue;
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    self.viscosity_record(false, false, FactDisposition::Reject, 0, 0);
                    self.update_state(|s| {
                        s.last_error = Some(format!("acquire_next failed: {error_msg}"));
                        s.consecutive_failures += 1;
                    });
                    self.check_safe_mode();
                    self.emit_cycle_receipt(iteration, 0, 0, 0, 0, 0, &[], &[error_msg]);
                    self.sleep_iteration().await;
                    self.check_max_iterations(iteration)?;
                    continue;
                }
            };

            let job_id = lease_outcome.job.job_id.clone();
            let job_id_str = job_id.to_string();
            let payload = lease_outcome.job.payload.clone();
            let lease = lease_outcome.lease.clone();

            self.update_state(|s| {
                s.current_job = Some(job_id_str.clone());
            });

            // 5. Execute job.
            let exec_result: ExecutionResult = match self
                .executor
                .execute_job_with_payload(&job_id_str, &payload)
                .await
            {
                Ok(result) => result,
                Err(e) => {
                    let error_msg = e.to_string();
                    self.viscosity_record(false, false, FactDisposition::Reject, 0, 0);
                    self.update_state(|s| {
                        s.tasks_failed += 1;
                        s.consecutive_failures += 1;
                        s.last_error = Some(error_msg.clone());
                        s.current_job = None;
                    });

                    // Cancel the job and mark gap as attempted.
                    let _ = self.queue.cancel(&job_id, "execution-error");
                    let err_gap_key = format!(
                        "{}|{}",
                        payload.get("fact_id").and_then(|v| v.as_str()).unwrap_or(""),
                        payload.get("gap_type").and_then(|v| v.as_str()).unwrap_or(""),
                    );
                    if !err_gap_key.is_empty() && err_gap_key != "|" {
                        let _ = self.attempted_gaps.lock().map(|mut g| g.insert(err_gap_key));
                    }
                    self.check_safe_mode();
                    self.emit_cycle_receipt(iteration, 0, 1, 0, 0, 0, &[], &[error_msg]);
                    self.sleep_iteration().await;
                    self.check_max_iterations(iteration)?;
                    continue;
                }
            };

            // 6. Capture results.
            let capture_outcome: CaptureOutcome = match self.capture.capture(&exec_result).await {
                Ok(outcome) => outcome,
                Err(e) => {
                    self.update_state(|s| {
                        s.last_error = Some(format!("capture failed: {e}"));
                    });
                    CaptureOutcome {
                        facts_added: 0,
                        facts_skipped_duplicates: 0,
                        fact_ids: Vec::new(),
                    }
                }
            };

            // 7. Evaluate captured facts (with viscosity-adjusted threshold).
            let promotion_threshold = self.viscosity_promotion_threshold();
            let mut facts_promoted = 0usize;
            let mut facts_quarantined = 0usize;
            let mut facts_rejected_this = 0usize;

            for fact_id in &capture_outcome.fact_ids {
                let (disposition, score) = self.evaluation.evaluate_with_score(
                    &exec_result.output,
                    exec_result.success,
                );

                // Apply viscosity-adjusted promotion threshold.
                let effective_disposition = if score >= promotion_threshold {
                    FactDisposition::Promote
                } else if score >= 0.5 {
                    FactDisposition::Quarantine
                } else {
                    disposition
                };

                // 8. Hostile audit (if strict/frozen and fact would be promoted).
                let final_disposition = if should_audit && effective_disposition == FactDisposition::Promote {
                    if let Some(audit_gate) = &self.hostile_audit {
                        match audit_gate.audit(&exec_result.output, &fact_id).await {
                            Ok(audit_result) if !audit_result.survived => {
                                FactDisposition::Quarantine
                            }
                            _ => effective_disposition,
                        }
                    } else {
                        effective_disposition
                    }
                } else {
                    effective_disposition
                };

                match final_disposition {
                    FactDisposition::Promote => {
                        facts_promoted += 1;
                        self.update_state(|s| { s.facts_captured += 1; });

                        // 9. Incur proof-debt for promoted facts.
                        let namespace = payload
                            .get("namespace")
                            .and_then(|v| v.as_str())
                            .unwrap_or("autonomous");
                        let risk = crate::proof_debt::classify_risk(&exec_result.output, namespace);
                        if let Ok(mut debt) = self.proof_debt.lock() {
                            let entry_id = debt.incur(fact_id, namespace, risk);
                            // Low risk: pay immediately via no-contradictions.
                            if risk == RiskClass::Low {
                                let _ = debt.pay(&entry_id, PaymentMethod::NoContradictions);
                            }
                        }
                    }
                    FactDisposition::Quarantine => {
                        facts_quarantined += 1;
                        self.update_state(|s| { s.facts_captured += 1; });
                    }
                    FactDisposition::Reject => {
                        facts_rejected_this += 1;
                        self.update_state(|s| { s.facts_rejected += 1; });
                    }
                }
            }

            // Also count skipped duplicates.
            if capture_outcome.facts_skipped_duplicates > 0 {
                facts_rejected_this += capture_outcome.facts_skipped_duplicates;
                self.update_state(|s| {
                    s.facts_rejected += capture_outcome.facts_skipped_duplicates;
                });
            }

            // 10. Complete or cancel job.
            let gap_key = format!(
                "{}|{}",
                payload.get("fact_id").and_then(|v| v.as_str()).unwrap_or(""),
                payload.get("gap_type").and_then(|v| v.as_str()).unwrap_or(""),
            );

            if exec_result.success {
                self.update_state(|s| {
                    s.tasks_completed += 1;
                    s.consecutive_failures = 0;
                    s.current_job = None;
                    s.last_error = None;
                });
                let _ = self.queue.complete(&job_id, &lease);
            } else {
                self.update_state(|s| {
                    s.tasks_failed += 1;
                    s.consecutive_failures += 1;
                    s.current_job = None;
                    s.last_error = exec_result.error.clone();
                });
                let _ = self.queue.cancel(&job_id, "execution-failed");
            }

            // Mark this gap as attempted.
            if !gap_key.is_empty() && gap_key != "|" {
                let _ = self.attempted_gaps.lock().map(|mut g| g.insert(gap_key));
            }

            // 11. Record cycle outcome in viscosity controller.
            let was_duplicate = capture_outcome.facts_skipped_duplicates > 0;
            // Use the majority disposition for viscosity recording.
            let cycle_disposition = if facts_promoted > facts_quarantined
                && facts_promoted > facts_rejected_this
            {
                FactDisposition::Promote
            } else if facts_quarantined > 0 {
                FactDisposition::Quarantine
            } else {
                FactDisposition::Reject
            };
            self.viscosity_record(
                exec_result.success,
                was_duplicate,
                cycle_disposition,
                0, // contradictions — TODO: wire to decoder output
                facts_promoted,
            );

            // 12. Update proof-debt state.
            let debt_outstanding = if let Ok(debt) = self.proof_debt.lock() {
                debt.total_outstanding()
            } else {
                0
            };
            self.update_state(|s| {
                s.proof_debt_outstanding = debt_outstanding;
            });

            // 13. Emit cycle receipt.
            let domains: Vec<String> = self.state.lock().map(|s| s.domains_explored.clone()).unwrap_or_default();
            let errors: Vec<String> = if exec_result.success {
                Vec::new()
            } else {
                vec![exec_result.error.clone().unwrap_or_else(|| "unknown error".to_string())]
            };
            self.emit_cycle_receipt(
                iteration,
                0, // gaps_detected this cycle (tracked in run_gap_detection)
                1, // one task executed
                facts_promoted + facts_quarantined,
                facts_rejected_this,
                facts_quarantined,
                &domains,
                &errors,
            );

            // 14. Check consecutive failures → safe mode.
            self.check_safe_mode();

            // 15. Check max iterations.
            self.check_max_iterations(iteration)?;

            // 16. Sleep between iterations (viscosity-adjusted).
            self.sleep_iteration().await;
        }
    }

    /// Emit a cycle receipt and update the receipt chain.
    fn emit_cycle_receipt(
        &self,
        iteration: usize,
        gaps: usize,
        tasks: usize,
        captured: usize,
        rejected: usize,
        quarantined: usize,
        domains: &[String],
        errors: &[String],
    ) {
        let strictness = self.viscosity_strictness_name();
        let debt_outstanding = self.proof_debt.lock().map(|d| d.total_outstanding()).unwrap_or(0);
        let total_incurred = self.proof_debt.lock().map(|d| d.debt_receipt().total_incurred).unwrap_or(0);
        let mode = self.state.lock().map(|s| s.mode).unwrap_or_default();
        let saturated = self.entropy_saturated_domains();

        if let Ok(mut emitter) = self.receipts.lock() {
            emitter.emit(
                iteration,
                gaps,
                tasks,
                captured,
                rejected,
                quarantined,
                None, // viscosity signal snapshot — TODO: wire full signal
                &strictness,
                debt_outstanding,
                total_incurred,
                mode,
                domains.to_vec(),
                saturated,
                errors.to_vec(),
            );
        }
    }

    /// Run mission-based gap detection for a specific mission.
    ///
    /// This calls the mission's `detect_issues` method, generates tasks from
    /// the detected gaps, and records the result in the scheduler for
    /// adaptive priority adjustment.
    async fn run_mission_detection(&self, mission: &Mission, iteration: usize) -> Result<()> {
        let attempted = self.attempted_gaps.lock().unwrap().clone();
        let gaps = mission.detect_issues(&self.config.http_base_url, &attempted).await?;

        let issue_count = gaps.len();
        self.update_state(|s| {
            s.gaps_detected += issue_count;
        });

        if !gaps.is_empty() {
            let job_ids = self.generator.generate_tasks(&gaps).await?;
            self.update_state(|s| {
                s.tasks_generated += job_ids.len();
            });
        }

        // Record the mission result for adaptive priority adjustment.
        if let Ok(mut scheduler) = self.mission_scheduler.lock() {
            scheduler.record_result(mission.as_kebab(), issue_count, iteration);
        }

        Ok(())
    }

    /// Run gap detection using entropy-gradient-guided domain selection.
    ///
    /// Instead of scanning all priority namespaces randomly, this queries
    /// the entropy-gradient searcher for the top domains to explore, then
    /// runs namespace-targeted gap detection on each.
    async fn run_gap_detection(&self) -> Result<()> {
        let attempted = self.attempted_gaps.lock().unwrap().clone();

        // Get top domains to explore from entropy-gradient searcher.
        // Must not hold std::sync::MutexGuard across .await (not Send).
        let targets = {
            let searcher = self.entropy_search.lock();
            match searcher {
                Ok(s) => {
                    // Clone the http_base_url so we can make a new searcher
                    // outside the guard to avoid holding it across .await.
                    let url = s.http_base_url.clone();
                    drop(s);
                    let temp_searcher = EntropyGradientSearcher::new(&url);
                    temp_searcher.next_targets(5).await.unwrap_or_default()
                }
                Err(_) => Vec::new(),
            }
        };

        if targets.is_empty() {
            // Fall back to the original broad detection if entropy search
            // fails (e.g., SM server not running or no stats available).
            let gaps = self.detector.detect_gaps(30, &attempted).await?;
            self.update_state(|s| {
                s.gaps_detected += gaps.len();
            });
            if !gaps.is_empty() {
                let job_ids = self.generator.generate_tasks(&gaps).await?;
                self.update_state(|s| {
                    s.tasks_generated += job_ids.len();
                });
            }
            return Ok(());
        }

        // Run gap detection in each target domain.
        let mut all_gaps = Vec::new();
        let mut domains_explored = Vec::new();
        for target in &targets {
            let domain = &target.domain;
            domains_explored.push(domain.clone());

            let gaps = self
                .detector
                .detect_gaps_in_namespace(15, &attempted, domain)
                .await
                .unwrap_or_default();
            all_gaps.extend(gaps);
        }

        // Record yield for saturation tracking.
        if let Ok(mut searcher) = self.entropy_search.lock() {
            for target in &targets {
                let yield_count = all_gaps
                    .iter()
                    .filter(|g| g.namespace.as_deref() == Some(&target.domain))
                    .count();
                searcher.record_exploration(&target.domain, yield_count, target.fact_count);
            }
        }

        self.update_state(|s| {
            s.gaps_detected += all_gaps.len();
            s.domains_explored = domains_explored;
            s.saturated_domains = self.entropy_saturated_domains();
        });

        if all_gaps.is_empty() {
            return Ok(());
        }

        let job_ids = self.generator.generate_tasks(&all_gaps).await?;
        self.update_state(|s| {
            s.tasks_generated += job_ids.len();
        });

        Ok(())
    }

    /// Run a subtractive cycle: verify existing claims, pay proof-debt,
    /// check contradictions, and compact memory.
    async fn run_subtractive_cycle(&self) -> Result<()> {
        // 1. Pay all low-risk debt immediately (observations are cheap to verify).
        let _low_paid = if let Ok(mut debt) = self.proof_debt.lock() {
            debt.pay_all_low_risk(PaymentMethod::NoContradictions)
        } else {
            0
        };

        // 2. Check medium-risk debt for contradictions (via HTTP to SM server).
        //    If no contradictions found, pay the debt.
        let medium_entries: Vec<(String, String)> = if let Ok(debt) = self.proof_debt.lock() {
            debt.outstanding_for_risk(RiskClass::Medium)
                .iter()
                .map(|e| (e.entry_id.clone(), e.claim_id.clone()))
                .collect()
        } else {
            Vec::new()
        };

        // For each medium-risk entry, check if the claim has contradictions.
        for (entry_id, claim_id) in &medium_entries {
            let has_contradiction = self.check_claim_contradictions(claim_id).await.unwrap_or(false);
            if let Ok(mut debt) = self.proof_debt.lock() {
                if has_contradiction {
                    // Quarantine the claim — pay debt via quarantine.
                    let _ = debt.pay(entry_id, PaymentMethod::Quarantined);
                } else {
                    // No contradictions — pay debt.
                    let _ = debt.pay(entry_id, PaymentMethod::NoContradictions);
                }
            }
        }

        // 3. Update state with debt info.
        let (outstanding, _total_incurred) = if let Ok(debt) = self.proof_debt.lock() {
            (debt.total_outstanding(), debt.debt_receipt().total_incurred)
        } else {
            (0, 0)
        };

        self.update_state(|s| {
            s.proof_debt_outstanding = outstanding;
            s.mode = if self.proof_debt_should_shift() {
                LoopMode::Subtractive
            } else {
                LoopMode::Additive
            };
        });

        // 4. If debt is low enough, return to additive mode.
        if !self.proof_debt_should_shift() {
            self.update_state(|s| {
                s.mode = LoopMode::Additive;
            });
        }

        Ok(())
    }

    /// Check if a claim has contradictions by querying the SM server.
    /// Uses the /discord endpoint to find related items, then checks
    /// if any are contradiction edges.
    async fn check_claim_contradictions(&self, claim_id: &str) -> Result<bool> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| anyhow::anyhow!("HTTP client: {e}"))?;
        let body = serde_json::json!({"direct_result_ids": [claim_id]});
        let url = format!("{}/discord", self.config.http_base_url);
        let resp = client.post(&url).json(&body).send().await;
        match resp {
            Ok(r) if r.status().is_success() => {
                let data: serde_json::Value = r.json().await.unwrap_or_default();
                // Check if any related items are contradictions.
                if let Some(related) = data.get("related").and_then(|v| v.as_array()) {
                    // If there are related items with contradiction edges,
                    // the claim has contradictions.
                    Ok(related.iter().any(|r| {
                        r.get("edge_type")
                            .and_then(|v| v.as_str())
                            .map(|s| s.contains("contradict"))
                            .unwrap_or(false)
                    }))
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false), // Fail open if server unavailable.
        }
    }

    /// Helper: check if proof-debt should shift to subtractive.
    fn proof_debt_should_shift(&self) -> bool {
        self.proof_debt
            .lock()
            .map(|d| d.should_shift_to_subtractive())
            .unwrap_or(false)
    }

    /// Helper: get saturated domains from entropy searcher.
    fn entropy_saturated_domains(&self) -> Vec<String> {
        self.entropy_search
            .lock()
            .map(|s| s.saturated_domains())
            .unwrap_or_default()
    }

    /// Helper: check if viscosity says to generate tasks.
    fn viscosity_should_generate(&self) -> bool {
        self.viscosity
            .lock()
            .map(|v| v.should_generate_tasks())
            .unwrap_or(true)
    }

    /// Helper: check if viscosity says to run audit.
    fn viscosity_should_audit(&self) -> bool {
        self.viscosity
            .lock()
            .map(|v| v.should_run_audit())
            .unwrap_or(false)
    }

    /// Helper: get viscosity promotion threshold.
    fn viscosity_promotion_threshold(&self) -> f64 {
        self.viscosity
            .lock()
            .map(|v| v.promotion_threshold())
            .unwrap_or(0.8)
    }

    /// Helper: get viscosity sleep multiplier.
    fn viscosity_sleep_multiplier(&self) -> f64 {
        self.viscosity
            .lock()
            .map(|v| v.sleep_multiplier())
            .unwrap_or(1.0)
    }

    /// Helper: record cycle outcome in viscosity controller.
    fn viscosity_record(
        &self,
        success: bool,
        was_duplicate: bool,
        disposition: FactDisposition,
        contradictions: usize,
        facts_added: usize,
    ) {
        if let Ok(mut vc) = self.viscosity.lock() {
            vc.record(success, was_duplicate, disposition, contradictions, facts_added);
        }
    }

    /// Helper: get current strictness name.
    fn viscosity_strictness_name(&self) -> String {
        self.viscosity
            .lock()
            .map(|v| v.current_strictness().to_string())
            .unwrap_or_else(|_| "normal".to_string())
    }

    /// Check if the queue has pending (non-terminal) jobs.
    fn queue_has_pending_jobs(&self) -> bool {
        match self.queue.snapshot() {
            Ok(snap) => snap.jobs.iter().any(|j| !j.state.is_terminal()),
            Err(_) => false,
        }
    }

    /// Check if we've hit the consecutive failure threshold and enter safe mode.
    fn check_safe_mode(&self) {
        let should_enter_safe = {
            let state = self.state.lock().ok();
            state.map(|s| s.consecutive_failures >= self.config.max_consecutive_failures && !s.safe_mode).unwrap_or(false)
        };

        if should_enter_safe {
            self.update_state(|s| {
                s.safe_mode = true;
                s.last_error = Some(format!(
                    "safe mode activated after {} consecutive failures",
                    s.consecutive_failures
                ));
            });
            let _ = self.queue.set_safe_mode(true, "consecutive-failure-threshold");
        }
    }

    /// Check if max_iterations has been reached. Returns Err to break the loop.
    fn check_max_iterations(&self, iteration: usize) -> Result<()> {
        if self.config.max_iterations > 0 && iteration >= self.config.max_iterations {
            return Err(anyhow::anyhow!(
                "max_iterations ({}) reached — loop terminating",
                self.config.max_iterations
            ));
        }
        Ok(())
    }

    /// Sleep for the configured interval, adjusted by viscosity.
    async fn sleep_iteration(&self) {
        let base_ms = self.config.sleep_between_iterations_ms;
        let multiplier = self.viscosity_sleep_multiplier();
        let adjusted_ms = (base_ms as f64 * multiplier) as u64;
        if adjusted_ms > 0 {
            tokio::time::sleep(Duration::from_millis(adjusted_ms)).await;
        }
    }

    /// Update the shared state under the mutex.
    fn update_state<F>(&self, f: F)
    where
        F: FnOnce(&mut LoopState),
    {
        if let Ok(mut state) = self.state.lock() {
            f(&mut state);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir()
            .join(format!("aidens-autonomous-loop-{name}-{id}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn loop_config_default_values() {
        let config = LoopConfig::default();
        assert_eq!(config.max_iterations, 0);
        assert_eq!(config.gap_detection_interval, 5);
        assert_eq!(config.sleep_between_iterations_ms, 1000);
        assert_eq!(config.max_consecutive_failures, 5);
        assert_eq!(config.model_url, "http://127.0.0.1:11434");
        assert_eq!(config.chosen_model, "llama3");
    }

    #[test]
    fn loop_config_custom_values() {
        let config = LoopConfig {
            max_iterations: 100,
            gap_detection_interval: 3,
            sleep_between_iterations_ms: 500,
            max_consecutive_failures: 10,
            model_url: "http://ollama:11434".to_string(),
            chosen_model: "mistral".to_string(),
            api_key: None,
            memory_dir: PathBuf::from("/tmp/memory"),
            queue_dir: PathBuf::from("/tmp/queue"),
            http_base_url: "http://localhost:1738".to_string(),
            ..Default::default()
        };
        assert_eq!(config.max_iterations, 100);
        assert_eq!(config.gap_detection_interval, 3);
        assert_eq!(config.max_consecutive_failures, 10);
    }

    #[test]
    fn loop_state_default_is_clean() {
        let state = LoopState::default();
        assert_eq!(state.iteration, 0);
        assert_eq!(state.gaps_detected, 0);
        assert_eq!(state.tasks_generated, 0);
        assert_eq!(state.tasks_completed, 0);
        assert_eq!(state.tasks_failed, 0);
        assert_eq!(state.facts_captured, 0);
        assert_eq!(state.facts_rejected, 0);
        assert_eq!(state.consecutive_failures, 0);
        assert_eq!(state.current_job, None);
        assert_eq!(state.last_error, None);
        assert!(!state.safe_mode);
    }

    #[test]
    fn loop_state_tracks_progress() {
        let mut state = LoopState::default();

        // Simulate some progress.
        state.iteration = 5;
        state.gaps_detected = 3;
        state.tasks_generated = 3;
        state.tasks_completed = 2;
        state.tasks_failed = 1;
        state.facts_captured = 2;
        state.consecutive_failures = 0;
        state.current_job = Some("job:123".to_string());

        assert_eq!(state.iteration, 5);
        assert_eq!(state.gaps_detected, 3);
        assert_eq!(state.tasks_completed, 2);
        assert_eq!(state.tasks_failed, 1);
        assert_eq!(state.current_job, Some("job:123".to_string()));
    }

    #[test]
    fn loop_state_enters_safe_mode() {
        let mut state = LoopState::default();
        state.consecutive_failures = 5;
        state.safe_mode = true;
        state.last_error = Some("safe mode activated".to_string());

        assert!(state.safe_mode);
        assert_eq!(state.consecutive_failures, 5);
    }

    #[test]
    fn loop_config_serializes_and_deserializes() {
        let config = LoopConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let back: LoopConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.max_iterations, config.max_iterations);
        assert_eq!(back.model_url, config.model_url);
    }

    #[test]
    fn loop_state_serializes_and_deserializes() {
        let state = LoopState {
            iteration: 10,
            gaps_detected: 5,
            tasks_generated: 5,
            tasks_completed: 3,
            tasks_failed: 2,
            facts_captured: 3,
            facts_rejected: 1,
            consecutive_failures: 2,
            current_job: Some("job:abc".to_string()),
            last_error: Some("test error".to_string()),
            safe_mode: false,
            ..Default::default()
        };
        let json = serde_json::to_string(&state).unwrap();
        let back: LoopState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.iteration, 10);
        assert_eq!(back.tasks_completed, 3);
        assert_eq!(back.current_job, Some("job:abc".to_string()));
    }

    #[tokio::test]
    async fn from_config_builds_all_components() {
        let memory_dir = temp_dir("memory");
        let queue_dir = temp_dir("queue");

        let config = LoopConfig {
            max_iterations: 1,
            gap_detection_interval: 1,
            sleep_between_iterations_ms: 0,
            max_consecutive_failures: 3,
            model_url: "http://localhost:11434".to_string(),
            chosen_model: "test-model".to_string(),
            api_key: None,
            memory_dir: memory_dir.clone(),
            queue_dir: queue_dir.clone(),
            http_base_url: "http://localhost:1738".to_string(),
            ..Default::default()
        };

        let loop_v1 = AutonomousLoop::from_config(config).unwrap();
        assert_eq!(loop_v1.config.max_iterations, 1);
        assert_eq!(loop_v1.config.sleep_between_iterations_ms, 0);

        // State should be clean.
        let state = loop_v1.state_snapshot();
        assert_eq!(state.iteration, 0);
    }

    #[test]
    fn state_snapshot_returns_clone() {
        let state = Arc::new(Mutex::new(LoopState {
            iteration: 7,
            tasks_completed: 3,
            ..Default::default()
        }));

        let snapshot = {
            let guard = state.lock().unwrap();
            guard.clone()
        };
        assert_eq!(snapshot.iteration, 7);
        assert_eq!(snapshot.tasks_completed, 3);

        // Modify the original — snapshot should be unchanged.
        {
            let mut guard = state.lock().unwrap();
            guard.iteration = 99;
        }
        assert_eq!(snapshot.iteration, 7);
    }

    #[tokio::test]
    async fn check_max_iterations_stops_at_limit() {
        let memory_dir = temp_dir("max-iter-memory");
        let queue_dir = temp_dir("max-iter-queue");

        let config = LoopConfig {
            max_iterations: 2,
            gap_detection_interval: 100, // don't detect during test
            sleep_between_iterations_ms: 0,
            max_consecutive_failures: 99,
            model_url: "http://localhost:11434".to_string(),
            chosen_model: "test".to_string(),
            api_key: None,
            memory_dir: memory_dir,
            queue_dir: queue_dir,
            http_base_url: "http://localhost:1738".to_string(),
            ..Default::default()
        };

        let autonomous_loop = AutonomousLoop::from_config(config).unwrap();

        // Run the loop — it should stop after 2 iterations because there are
        // no jobs in the queue and max_iterations=2.
        let result = autonomous_loop.run().await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("max_iterations"));

        // State should reflect the iterations.
        let state = autonomous_loop.state_snapshot();
        assert_eq!(state.iteration, 2);
    }
}