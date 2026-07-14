//! Autonomous loop driver — ties together gap detection, task generation,
//! execution, capture, and evaluation into a single continuous loop.
//!
//! The [`AutonomousLoop`] orchestrates the full detect → enqueue → execute →
//! capture → evaluate cycle. It tracks state in [`LoopState`] (shared via
//! `Arc<Mutex<>>` for TUI integration) and respects [`LoopConfig`] for
//! iteration limits, sleep intervals, and safe-mode thresholds.

use crate::capture::{CaptureOutcome, ClaimCandidateV1, ResultCapture};
use crate::entropy_search::EntropyGradientSearcher;
use crate::evaluation::{
    ClaimEvaluationInputV1, EvaluationGate, EvaluationReportV1, FactDisposition,
    SourceSpanQualityV1,
};
use crate::executor::{ExecutionResult, LoopExecutor};
use crate::gap_detector::GapDetector;
use crate::hostile_audit::HostileAuditGate;
use crate::missions::{Mission, MissionScheduler};
use crate::proof_debt::{PaymentMethod, ProofDebtBudget, RiskClass};
use crate::receipt::{
    CommittedLoopStateV1, CycleReceiptInputV1, CycleReceiptV1, LoopMode as CycleMode, ReceiptLedger,
};
use crate::task_generator::TaskGenerator;
use crate::viscosity::ViscosityController;
use aidens_contracts::{ArtifactId, QueueLeaseV1};
use aidens_daemon_kit::DaemonControllerV1;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tokio::time::Duration;

// ---------------------------------------------------------------------------
// Config & State
// ---------------------------------------------------------------------------

/// Safety mode controlling whether evaluated candidates can affect canonical
/// knowledge.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoopMode {
    /// Offline candidate-only learning. Canonical writes are forbidden.
    #[default]
    Shadow,
    /// Candidate-only learning awaiting an external reviewer decision.
    Reviewed,
    /// Evidence-gated learning with canonical promotion enabled.
    Autonomous,
}

/// Configuration for the autonomous loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopConfig {
    /// Safety mode. Defaults to write-isolated shadow learning.
    #[serde(default)]
    pub loop_mode: LoopMode,
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
            loop_mode: LoopMode::Shadow,
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
            auditor_url: String::new(), // empty = no hostile audit
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
    pub mode: CycleMode,
    /// Outstanding proof-debt count.
    pub proof_debt_outstanding: usize,
    /// Domains explored this cycle.
    pub domains_explored: Vec<String>,
    /// Saturated domains.
    pub saturated_domains: Vec<String>,
}

/// Terminal outcome of a loop run initiated with an explicit stop signal.
///
/// A stop request is observed only at a cycle boundary, after the loop has
/// emitted its receipt and completed or cancelled any leased job transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopTermination {
    /// The configured finite iteration limit was reached.
    IterationLimitReached,
    /// The caller requested a stop at a safe cycle boundary.
    StopRequested,
}

/// Candidate-only artifact emitted by shadow/reviewed learning cycles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateBundleV1 {
    pub job_id: String,
    pub mode: LoopMode,
    pub candidates: Vec<ClaimCandidateV1>,
    pub evaluation_reports: Vec<EvaluationReportV1>,
}

/// Fixed replay input for deterministic offline evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayCaseV1 {
    pub candidate: ClaimCandidateV1,
    #[serde(default)]
    pub contradicting_fact_ids: Vec<String>,
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
    /// Explicit process-owned in-memory ledger for cycle receipts.
    pub receipts: Arc<Mutex<ReceiptLedger>>,
    /// Mission scheduler for structured high-ROI objectives.
    pub mission_scheduler: Arc<Mutex<MissionScheduler>>,
    /// Inspectable candidate/evaluation artifacts produced by the loop.
    pub candidate_bundles: Arc<Mutex<Vec<CandidateBundleV1>>>,
}

/// Material metrics for a single autonomous-loop cycle receipt.
#[derive(Debug, Default)]
struct CycleMetrics {
    iteration: usize,
    gaps: usize,
    tasks: usize,
    captured: usize,
    rejected: usize,
    quarantined: usize,
    domains: Vec<String>,
    errors: Vec<String>,
    mode: CycleMode,
}

fn evaluate_candidate(
    gate: &EvaluationGate,
    candidate: &ClaimCandidateV1,
    execution_success: bool,
    contradicting_fact_ids: &[String],
) -> EvaluationReportV1 {
    let source_span = candidate
        .source_spans
        .first()
        .map(|span| SourceSpanQualityV1 {
            start: span.output_byte_range.start,
            end: span.output_byte_range.end,
            source_len: span.output_byte_len,
            output_digest_present: !span.output_digest.is_empty(),
            model_name_present: !span.model_name.is_empty(),
            prompt_config_digest_present: !span.prompt_config_digest.is_empty(),
        });
    gate.evaluate_claim(&ClaimEvaluationInputV1 {
        content: &candidate.claim,
        execution_success,
        retrieval_evidence: candidate.retrieval_evidence.clone(),
        contradicting_fact_ids: if contradicting_fact_ids.is_empty() {
            candidate.contradicting_fact_ids.clone()
        } else {
            contradicting_fact_ids.to_vec()
        },
        source_span,
    })
}

fn recover_loop_state(receipts: &[CycleReceiptV1]) -> LoopState {
    if let Some(committed) = receipts
        .last()
        .and_then(|receipt| receipt.committed_state.as_ref())
    {
        return LoopState {
            iteration: committed.iteration,
            gaps_detected: committed.gaps_detected,
            tasks_generated: committed.tasks_generated,
            tasks_completed: committed.tasks_completed,
            tasks_failed: committed.tasks_failed,
            facts_captured: committed.facts_captured,
            facts_rejected: committed.facts_rejected,
            consecutive_failures: committed.consecutive_failures,
            current_job: committed.current_job.clone(),
            last_error: committed.last_error.clone(),
            safe_mode: committed.safe_mode,
            strictness: committed.strictness.clone(),
            mode: committed.cycle_mode,
            proof_debt_outstanding: committed.proof_debt_outstanding,
            domains_explored: committed.domains_explored.clone(),
            saturated_domains: committed.saturated_domains.clone(),
        };
    }
    let mut state = LoopState::default();
    for receipt in receipts {
        state.iteration = receipt.iteration;
        state.gaps_detected += receipt.gaps_detected;
        state.facts_captured += receipt.facts_captured;
        state.facts_rejected += receipt.facts_rejected;
        state.strictness = receipt.strictness.clone();
        state.mode = receipt.mode;
        state.proof_debt_outstanding = receipt.proof_debt_outstanding;
        state.domains_explored = receipt.domains_explored.clone();
        state.saturated_domains = receipt.saturated_domains.clone();
        state.last_error = receipt.errors.last().cloned();
    }
    state
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
        let runtime_config = aidens_memory_kit::runtime_config_for_namespace("autonomous");
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
        let queue = DaemonControllerV1::open(&config.queue_dir, namespace, "aidens-autonomous")?;

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
        let prompt_config_digest = format!(
            "sha256:{:x}",
            Sha256::digest(
                format!(
                    "{}\0{}\0{}",
                    crate::executor::SYSTEM_PROMPT,
                    config.chosen_model,
                    config.model_url
                )
                .as_bytes()
            )
        );
        let capture = ResultCapture::new(memory, &config.http_base_url)
            .with_source_config(&config.chosen_model, prompt_config_digest);
        let evaluation = EvaluationGate::new();

        // Build new control pieces.
        let viscosity = ViscosityController::with_defaults();
        let proof_debt = ProofDebtBudget::new();
        let entropy_search = EntropyGradientSearcher::new(&config.http_base_url);
        let hostile_audit = if !config.auditor_url.is_empty() && !config.auditor_model.is_empty() {
            Some(HostileAuditGate::new(
                &config.auditor_url,
                &config.auditor_model,
            ))
        } else {
            None
        };
        let receipt_path = config.memory_dir.join("autonomous-cycle-receipts.jsonl");
        let receipts = ReceiptLedger::open(&receipt_path)?;
        let recovered_state = recover_loop_state(receipts.history());
        let mission_scheduler = MissionScheduler::new();

        Ok(Self {
            detector,
            generator,
            executor,
            capture,
            evaluation,
            queue,
            config,
            state: Arc::new(Mutex::new(recovered_state)),
            attempted_gaps: Arc::new(Mutex::new(HashSet::new())),
            viscosity: Arc::new(Mutex::new(viscosity)),
            proof_debt: Arc::new(Mutex::new(proof_debt)),
            entropy_search: Arc::new(Mutex::new(entropy_search)),
            hostile_audit,
            receipts: Arc::new(Mutex::new(receipts)),
            mission_scheduler: Arc::new(Mutex::new(mission_scheduler)),
            candidate_bundles: Arc::new(Mutex::new(Vec::new())),
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
            Some(HostileAuditGate::new(
                &config.auditor_url,
                &config.auditor_model,
            ))
        } else {
            None
        };
        let receipt_path = config.memory_dir.join("autonomous-cycle-receipts.jsonl");
        let receipts = match ReceiptLedger::open(&receipt_path) {
            Ok(ledger) => ledger,
            Err(error) => ReceiptLedger::unavailable(error.to_string()),
        };
        let recovered_state = recover_loop_state(receipts.history());
        Self {
            detector,
            generator,
            executor,
            capture,
            evaluation,
            queue,
            config,
            state: Arc::new(Mutex::new(recovered_state)),
            attempted_gaps: Arc::new(Mutex::new(HashSet::new())),
            viscosity: Arc::new(Mutex::new(ViscosityController::with_defaults())),
            proof_debt: Arc::new(Mutex::new(ProofDebtBudget::new())),
            entropy_search: Arc::new(Mutex::new(EntropyGradientSearcher::new(
                "http://127.0.0.1:1738",
            ))),
            hostile_audit,
            receipts: Arc::new(Mutex::new(receipts)),
            mission_scheduler: Arc::new(Mutex::new(MissionScheduler::new())),
            candidate_bundles: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get a snapshot of the current loop state.
    pub fn state_snapshot(&self) -> LoopState {
        self.state.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// Return a read-only snapshot of the process-owned receipt history.
    pub fn receipt_history(&self) -> Result<Vec<CycleReceiptV1>> {
        self.receipts
            .lock()
            .map(|ledger| ledger.history().to_vec())
            .map_err(|e| anyhow::anyhow!("receipt ledger lock: {e}"))
    }

    /// Snapshot candidate bundles and their evaluation reports.
    pub fn candidate_bundle_history(&self) -> Result<Vec<CandidateBundleV1>> {
        self.candidate_bundles
            .lock()
            .map(|bundles| bundles.clone())
            .map_err(|e| anyhow::anyhow!("candidate bundle lock: {e}"))
    }

    /// Replay a fixed candidate dataset without performing any memory writes.
    pub fn replay_fixed_dataset(&self, dataset: &[ReplayCaseV1]) -> Vec<EvaluationReportV1> {
        dataset
            .iter()
            .map(|case| {
                evaluate_candidate(
                    &self.evaluation,
                    &case.candidate,
                    true,
                    &case.contradicting_fact_ids,
                )
            })
            .collect()
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
    /// Run the autonomous loop until its configured iteration limit is reached.
    ///
    /// This compatibility entry point retains the historical error on a finite
    /// limit. UI callers that need a safe stop signal should use
    /// [`Self::run_until_stopped`] instead.
    pub async fn run(&self) -> Result<()> {
        match self.run_internal(None).await? {
            LoopTermination::IterationLimitReached => {
                self.check_max_iterations(self.config.max_iterations)
            }
            LoopTermination::StopRequested => Ok(()),
        }
    }

    /// Run until the configured finite limit or a caller-owned stop signal.
    ///
    /// A requested stop is deliberately observed only between cycles, after a
    /// cycle has emitted its receipt and reached a durable queue boundary.
    pub async fn run_until_stopped(&self, stop_requested: &AtomicBool) -> Result<LoopTermination> {
        self.run_internal(Some(stop_requested)).await
    }

    async fn run_internal(&self, stop_requested: Option<&AtomicBool>) -> Result<LoopTermination> {
        loop {
            if Self::stop_requested(stop_requested) {
                return Ok(LoopTermination::StopRequested);
            }
            // Snapshot state for this iteration.
            let iteration = {
                let mut state = self
                    .state
                    .lock()
                    .map_err(|e| anyhow::anyhow!("state lock: {e}"))?;
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
                mode == CycleMode::Subtractive
            };

            if in_subtractive || self.proof_debt_should_shift() {
                let mut errors = Vec::new();
                self.update_state(|s| {
                    s.mode = CycleMode::Subtractive;
                });
                if let Err(e) = self.run_subtractive_cycle().await {
                    let error = format!("subtractive cycle failed: {e}");
                    errors.push(error.clone());
                    self.update_state(|s| {
                        s.last_error = Some(error);
                    });
                }
                // Check if we can return to additive mode.
                if !self.proof_debt_should_shift() {
                    self.update_state(|s| {
                        s.mode = CycleMode::Additive;
                    });
                }
                // Emit receipt for subtractive cycle.
                self.emit_cycle_receipt(CycleMetrics {
                    iteration,
                    errors,
                    mode: CycleMode::Subtractive,
                    ..CycleMetrics::default()
                })?;
                if self.sleep_iteration_or_stopped(stop_requested).await {
                    return Ok(LoopTermination::StopRequested);
                }
                if self.max_iterations_reached(iteration) {
                    return Ok(LoopTermination::IterationLimitReached);
                }
                continue;
            }

            let mut cycle_gaps = 0usize;
            let mut cycle_errors = Vec::new();

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
                    match self.run_mission_detection(&mission, iteration).await {
                        Ok(gaps) => cycle_gaps += gaps,
                        Err(e) => {
                            let error = format!("mission detection failed: {e}");
                            cycle_errors.push(error.clone());
                            self.update_state(|s| {
                                s.last_error = Some(error);
                            });
                        }
                    }
                } else {
                    // Fall back to entropy-guided gap detection.
                    let should_detect = self.config.gap_detection_interval > 0
                        && (iteration - 1) % self.config.gap_detection_interval == 0;
                    if should_detect {
                        match self.run_gap_detection().await {
                            Ok(gaps) => cycle_gaps += gaps,
                            Err(e) => {
                                let error = format!("gap detection failed: {e}");
                                cycle_errors.push(error.clone());
                                self.update_state(|s| {
                                    s.last_error = Some(error);
                                });
                            }
                        }
                    }
                }
            }

            // 4. Acquire next job.
            let lease_outcome = match self.queue.acquire_next("autonomous-loop", 300) {
                Ok(Some(outcome)) => outcome,
                Ok(None) => {
                    // No job available — emit idle receipt and sleep.
                    self.emit_cycle_receipt(CycleMetrics {
                        iteration,
                        gaps: cycle_gaps,
                        errors: cycle_errors,
                        mode: CycleMode::Additive,
                        ..CycleMetrics::default()
                    })?;
                    if self.sleep_iteration_or_stopped(stop_requested).await {
                        return Ok(LoopTermination::StopRequested);
                    }
                    if self.max_iterations_reached(iteration) {
                        return Ok(LoopTermination::IterationLimitReached);
                    }
                    continue;
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    cycle_errors.push(error_msg.clone());
                    self.viscosity_record(false, false, FactDisposition::Reject, 0, 0);
                    self.update_state(|s| {
                        s.last_error = Some(format!("acquire_next failed: {error_msg}"));
                        s.consecutive_failures += 1;
                    });
                    self.check_safe_mode();
                    self.emit_cycle_receipt(CycleMetrics {
                        iteration,
                        gaps: cycle_gaps,
                        errors: cycle_errors,
                        mode: CycleMode::Additive,
                        ..CycleMetrics::default()
                    })?;
                    if self.sleep_iteration_or_stopped(stop_requested).await {
                        return Ok(LoopTermination::StopRequested);
                    }
                    if self.max_iterations_reached(iteration) {
                        return Ok(LoopTermination::IterationLimitReached);
                    }
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
                    cycle_errors.push(error_msg.clone());
                    self.viscosity_record(false, false, FactDisposition::Reject, 0, 0);
                    // The durable queue transition is the authority for terminal job state.
                    // Do not clear in-memory state or suppress the gap if cancellation fails.
                    let cancellation_error =
                        self.cancel_failed_job(&job_id, "execution-error").err();
                    if let Some(error) = &cancellation_error {
                        cycle_errors.push(error.to_string());
                    }
                    let err_gap_key = format!(
                        "{}|{}",
                        payload
                            .get("fact_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                        payload
                            .get("gap_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                    );
                    if cancellation_error.is_none() && !err_gap_key.is_empty() && err_gap_key != "|"
                    {
                        let _ = self
                            .attempted_gaps
                            .lock()
                            .map(|mut g| g.insert(err_gap_key));
                    }
                    self.check_safe_mode();
                    self.emit_cycle_receipt(CycleMetrics {
                        iteration,
                        gaps: cycle_gaps,
                        tasks: 1,
                        errors: cycle_errors,
                        mode: CycleMode::Additive,
                        ..CycleMetrics::default()
                    })?;
                    if let Some(error) = cancellation_error {
                        return Err(error);
                    }
                    if self.sleep_iteration_or_stopped(stop_requested).await {
                        return Ok(LoopTermination::StopRequested);
                    }
                    if self.max_iterations_reached(iteration) {
                        return Ok(LoopTermination::IterationLimitReached);
                    }
                    continue;
                }
            };

            // 6. Capture results.
            let capture_outcome: CaptureOutcome = match self.capture.capture(&exec_result).await {
                Ok(outcome) => outcome,
                Err(e) => {
                    let error = format!("capture failed: {e}");
                    cycle_errors.push(error.clone());
                    self.update_state(|s| {
                        s.last_error = Some(error);
                    });
                    CaptureOutcome {
                        facts_added: 0,
                        facts_skipped_duplicates: 0,
                        fact_ids: Vec::new(),
                        candidates: Vec::new(),
                    }
                }
            };

            // 7. Evaluate captured facts (with viscosity-adjusted threshold).
            let promotion_threshold = self.viscosity_promotion_threshold();
            let mut facts_promoted = 0usize;
            let mut facts_quarantined = 0usize;
            let mut facts_rejected_this = 0usize;

            let mut evaluation_reports = Vec::new();
            for candidate in &capture_outcome.candidates {
                let fact_id = &candidate.candidate_fact_id;
                let report =
                    evaluate_candidate(&self.evaluation, candidate, exec_result.success, &[]);
                let disposition = report.disposition;
                let score = report.score;
                evaluation_reports.push(report);

                // Apply viscosity-adjusted promotion threshold.
                let evidence_allows_promotion = disposition == FactDisposition::Promote;
                let effective_disposition =
                    if evidence_allows_promotion && score >= promotion_threshold {
                        FactDisposition::Promote
                    } else if disposition != FactDisposition::Reject {
                        FactDisposition::Quarantine
                    } else {
                        disposition
                    };

                // 8. Hostile audit (if strict/frozen and fact would be promoted).
                let final_disposition =
                    if should_audit && effective_disposition == FactDisposition::Promote {
                        if let Some(audit_gate) = &self.hostile_audit {
                            match audit_gate.audit(&exec_result.output, fact_id).await {
                                Ok(audit_result) if !audit_result.survived => {
                                    FactDisposition::Quarantine
                                }
                                // AUTO-003 fix: audit errors must quarantine,
                                // not promote. The old `_ =>` catch-all let
                                // both Ok(survived=true) and Err promote the fact.
                                Ok(_) => effective_disposition,
                                Err(e) => {
                                    eprintln!(
                                        "WARN: hostile audit failed; quarantining candidate: {e}"
                                    );
                                    FactDisposition::Quarantine
                                }
                            }
                        } else {
                            effective_disposition
                        }
                    } else {
                        effective_disposition
                    };

                let final_disposition =
                    match self.apply_learning_mode(candidate, final_disposition).await {
                        Ok(disposition) => disposition,
                        Err(error) => {
                            cycle_errors.push(format!("canonical promotion failed: {error}"));
                            FactDisposition::Quarantine
                        }
                    };

                match final_disposition {
                    FactDisposition::Promote => {
                        facts_promoted += 1;
                        self.update_state(|s| {
                            s.facts_captured += 1;
                        });

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
                        self.update_state(|s| {
                            s.facts_captured += 1;
                        });
                    }
                    FactDisposition::Reject => {
                        facts_rejected_this += 1;
                        self.update_state(|s| {
                            s.facts_rejected += 1;
                        });
                    }
                }
            }

            if !capture_outcome.candidates.is_empty() {
                let bundle = CandidateBundleV1 {
                    job_id: exec_result.job_id.clone(),
                    mode: self.config.loop_mode,
                    candidates: capture_outcome.candidates.clone(),
                    evaluation_reports,
                };
                self.candidate_bundles
                    .lock()
                    .map_err(|e| anyhow::anyhow!("candidate bundle lock: {e}"))?
                    .push(bundle);
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
                payload
                    .get("fact_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                payload
                    .get("gap_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
            );

            let mut cancellation_error = None;
            if exec_result.success {
                self.complete_successful_job(&job_id, &lease)?;
            } else {
                cancellation_error = self.cancel_failed_job(&job_id, "execution-failed").err();
                if let Some(error) = &cancellation_error {
                    cycle_errors.push(error.to_string());
                }
            }

            // Mark this gap as attempted only after its durable queue transition.
            if cancellation_error.is_none() && !gap_key.is_empty() && gap_key != "|" {
                let _ = self.attempted_gaps.lock().map(|mut g| g.insert(gap_key));
            }

            // 11. Record cycle outcome in viscosity controller.
            let was_duplicate = capture_outcome.facts_skipped_duplicates > 0;
            // Use the majority disposition for viscosity recording.
            let cycle_disposition =
                if facts_promoted > facts_quarantined && facts_promoted > facts_rejected_this {
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
            let domains: Vec<String> = self
                .state
                .lock()
                .map(|s| s.domains_explored.clone())
                .unwrap_or_default();
            if !exec_result.success {
                cycle_errors.push(
                    exec_result
                        .error
                        .clone()
                        .unwrap_or_else(|| "unknown error".to_string()),
                );
            }
            self.emit_cycle_receipt(CycleMetrics {
                iteration,
                gaps: cycle_gaps,
                tasks: 1,
                captured: facts_promoted + facts_quarantined,
                rejected: facts_rejected_this,
                quarantined: facts_quarantined,
                domains,
                errors: cycle_errors,
                mode: CycleMode::Additive,
            })?;

            // 14. Check consecutive failures → safe mode.
            self.check_safe_mode();

            if let Some(error) = cancellation_error {
                return Err(error);
            }

            // 15. Check max iterations.
            if self.max_iterations_reached(iteration) {
                return Ok(LoopTermination::IterationLimitReached);
            }

            // 16. Sleep between iterations (viscosity-adjusted).
            if self.sleep_iteration_or_stopped(stop_requested).await {
                return Ok(LoopTermination::StopRequested);
            }
        }
    }

    /// Emit a cycle receipt and update the receipt chain.
    fn emit_cycle_receipt(&self, metrics: CycleMetrics) -> Result<CycleReceiptV1> {
        let strictness = self.viscosity_strictness_name();
        let debt_outstanding = self
            .proof_debt
            .lock()
            .map(|d| d.total_outstanding())
            .unwrap_or(0);
        let total_incurred = self
            .proof_debt
            .lock()
            .map(|d| d.debt_receipt().total_incurred)
            .unwrap_or(0);
        let saturated = self.entropy_saturated_domains();
        let state = self
            .state
            .lock()
            .map_err(|e| anyhow::anyhow!("state lock for receipt commit: {e}"))?
            .clone();
        let committed_state = CommittedLoopStateV1 {
            iteration: metrics.iteration,
            gaps_detected: state.gaps_detected,
            tasks_generated: state.tasks_generated,
            tasks_completed: state.tasks_completed,
            tasks_failed: state.tasks_failed,
            facts_captured: state.facts_captured,
            facts_rejected: state.facts_rejected,
            consecutive_failures: state.consecutive_failures,
            current_job: state.current_job,
            last_error: state.last_error,
            safe_mode: state.safe_mode,
            strictness: strictness.clone(),
            cycle_mode: metrics.mode,
            proof_debt_outstanding: debt_outstanding,
            domains_explored: metrics.domains.clone(),
            saturated_domains: saturated.clone(),
        };

        let mut ledger = self
            .receipts
            .lock()
            .map_err(|e| anyhow::anyhow!("receipt ledger lock: {e}"))?;
        ledger.emit_durable_with_state(
            CycleReceiptInputV1 {
                iteration: metrics.iteration,
                gaps_detected: metrics.gaps,
                tasks_executed: metrics.tasks,
                facts_captured: metrics.captured,
                facts_rejected: metrics.rejected,
                facts_quarantined: metrics.quarantined,
                viscosity_signal: None, // TODO: wire full signal snapshot.
                strictness,
                proof_debt_outstanding: debt_outstanding,
                proof_debt_total_incurred: total_incurred,
                mode: metrics.mode,
                domains_explored: metrics.domains,
                saturated_domains: saturated,
                errors: metrics.errors,
            },
            Some(committed_state),
        )
    }

    async fn apply_learning_mode(
        &self,
        candidate: &ClaimCandidateV1,
        disposition: FactDisposition,
    ) -> Result<FactDisposition> {
        if disposition != FactDisposition::Promote {
            return Ok(disposition);
        }
        match self.config.loop_mode {
            LoopMode::Shadow | LoopMode::Reviewed => Ok(FactDisposition::Quarantine),
            LoopMode::Autonomous => {
                self.capture.promote_candidate(candidate).await?;
                Ok(FactDisposition::Promote)
            }
        }
    }

    /// Complete the durable queue transition before exposing completion in memory.
    fn complete_successful_job(&self, job_id: &ArtifactId, lease: &QueueLeaseV1) -> Result<()> {
        if let Err(error) = self.queue.complete(job_id, lease) {
            let error = format!("durable queue completion failed: {error}");
            let mut state = self
                .state
                .lock()
                .map_err(|e| anyhow::anyhow!("state lock after completion failure: {e}"))?;
            state.consecutive_failures += 1;
            state.last_error = Some(error.clone());
            return Err(anyhow::anyhow!(error));
        }

        let mut state = self
            .state
            .lock()
            .map_err(|e| anyhow::anyhow!("state lock after durable completion: {e}"))?;
        state.tasks_completed += 1;
        state.consecutive_failures = 0;
        state.current_job = None;
        state.last_error = None;
        Ok(())
    }

    /// Persist a failed-job cancellation before exposing its terminal state in memory.
    ///
    /// A failed append leaves the durable job leased, so the in-memory loop must
    /// preserve that fact and avoid suppressing the gap as already attempted.
    fn cancel_failed_job(&self, job_id: &ArtifactId, reason: &str) -> Result<()> {
        if let Err(error) = self.queue.cancel(job_id, reason) {
            let error = format!("durable queue cancellation failed: {error}");
            let mut state = self.state.lock().map_err(|lock_error| {
                anyhow::anyhow!("state lock after cancellation failure: {lock_error}")
            })?;
            state.consecutive_failures += 1;
            state.last_error = Some(error.clone());
            return Err(anyhow::anyhow!(error));
        }

        let mut state = self.state.lock().map_err(|lock_error| {
            anyhow::anyhow!("state lock after durable cancellation: {lock_error}")
        })?;
        state.tasks_failed += 1;
        state.consecutive_failures += 1;
        state.current_job = None;
        state.last_error = Some(reason.to_string());
        Ok(())
    }

    /// Run mission-based gap detection for a specific mission.
    ///
    /// This calls the mission's `detect_issues` method, generates tasks from
    /// the detected gaps, and records the result in the scheduler for
    /// adaptive priority adjustment.
    async fn run_mission_detection(&self, mission: &Mission, iteration: usize) -> Result<usize> {
        let attempted = self.attempted_gaps.lock().unwrap().clone();
        let gaps = mission
            .detect_issues(&self.config.http_base_url, &attempted)
            .await?;

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

        Ok(issue_count)
    }

    /// Run gap detection using entropy-gradient-guided domain selection.
    ///
    /// Instead of scanning all priority namespaces randomly, this queries
    /// the entropy-gradient searcher for the top domains to explore, then
    /// runs namespace-targeted gap detection on each.
    async fn run_gap_detection(&self) -> Result<usize> {
        let attempted = self.attempted_gaps.lock().unwrap().clone();

        // Get top domains to explore from entropy-gradient searcher.
        // Must not hold std::sync::MutexGuard across .await (not Send).
        let search_url = self
            .entropy_search
            .lock()
            .ok()
            .map(|searcher| searcher.http_base_url.clone());
        let targets = if let Some(url) = search_url {
            EntropyGradientSearcher::new(&url)
                .next_targets(5)
                .await
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        if targets.is_empty() {
            // Fall back to the original broad detection if entropy search
            // fails (e.g., SM server not running or no stats available).
            let gaps = self.detector.detect_gaps(30, &attempted).await?;
            let gap_count = gaps.len();
            self.update_state(|s| {
                s.gaps_detected += gap_count;
            });
            if !gaps.is_empty() {
                let job_ids = self.generator.generate_tasks(&gaps).await?;
                self.update_state(|s| {
                    s.tasks_generated += job_ids.len();
                });
            }
            return Ok(gap_count);
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

        let gap_count = all_gaps.len();
        self.update_state(|s| {
            s.gaps_detected += gap_count;
            s.domains_explored = domains_explored;
            s.saturated_domains = self.entropy_saturated_domains();
        });

        if all_gaps.is_empty() {
            return Ok(0);
        }

        let job_ids = self.generator.generate_tasks(&all_gaps).await?;
        self.update_state(|s| {
            s.tasks_generated += job_ids.len();
        });

        Ok(gap_count)
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
            let has_contradiction = self.check_claim_contradictions(claim_id).await?;
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
                CycleMode::Subtractive
            } else {
                CycleMode::Additive
            };
        });

        // 4. If debt is low enough, return to additive mode.
        if !self.proof_debt_should_shift() {
            self.update_state(|s| {
                s.mode = CycleMode::Additive;
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
        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("discord request for claim {claim_id}: {e}"))?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "discord request for claim {claim_id} returned {}",
                response.status()
            ));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("discord response for claim {claim_id}: {e}"))?;
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
            vc.record(
                success,
                was_duplicate,
                disposition,
                contradictions,
                facts_added,
            );
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
            state
                .map(|s| {
                    s.consecutive_failures >= self.config.max_consecutive_failures && !s.safe_mode
                })
                .unwrap_or(false)
        };

        if should_enter_safe {
            self.update_state(|s| {
                s.safe_mode = true;
                s.last_error = Some(format!(
                    "safe mode activated after {} consecutive failures",
                    s.consecutive_failures
                ));
            });
            let _ = self
                .queue
                .set_safe_mode(true, "consecutive-failure-threshold");
        }
    }

    /// Check whether a finite loop has reached its configured limit.
    fn max_iterations_reached(&self, iteration: usize) -> bool {
        self.config.max_iterations > 0 && iteration >= self.config.max_iterations
    }

    /// Compatibility helper retained for callers/tests expecting the historical
    /// limit error from [`Self::run`].
    fn check_max_iterations(&self, iteration: usize) -> Result<()> {
        if self.max_iterations_reached(iteration) {
            return Err(anyhow::anyhow!(
                "max_iterations ({}) reached — loop terminating",
                self.config.max_iterations
            ));
        }
        Ok(())
    }

    fn stop_requested(stop_requested: Option<&AtomicBool>) -> bool {
        stop_requested.is_some_and(|signal| signal.load(Ordering::Acquire))
    }

    /// Sleep without interrupting an active cycle, then observe a requested
    /// stop at the next safe boundary.
    async fn sleep_iteration_or_stopped(&self, stop_requested: Option<&AtomicBool>) -> bool {
        if Self::stop_requested(stop_requested) {
            return true;
        }
        self.sleep_iteration().await;
        Self::stop_requested(stop_requested)
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
    use aidens_contracts::{CanonicalToolSideEffectClass, JobStateV1, JobV1};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "aidens-autonomous-loop-{name}-{id}-{}",
            std::process::id()
        ));
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
        let state = LoopState {
            iteration: 5,
            gaps_detected: 3,
            tasks_generated: 3,
            tasks_completed: 2,
            tasks_failed: 1,
            facts_captured: 2,
            current_job: Some("job:123".to_string()),
            ..Default::default()
        };

        assert_eq!(state.iteration, 5);
        assert_eq!(state.gaps_detected, 3);
        assert_eq!(state.tasks_completed, 2);
        assert_eq!(state.tasks_failed, 1);
        assert_eq!(state.current_job, Some("job:123".to_string()));
    }

    #[test]
    fn loop_state_enters_safe_mode() {
        let state = LoopState {
            consecutive_failures: 5,
            safe_mode: true,
            last_error: Some("safe mode activated".to_string()),
            ..Default::default()
        };

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
            memory_dir,
            queue_dir,
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

    #[tokio::test]
    async fn explicit_stop_prevents_a_new_cycle_from_starting() {
        let autonomous_loop = AutonomousLoop::from_config(LoopConfig {
            memory_dir: temp_dir("stop-before-cycle-memory"),
            queue_dir: temp_dir("stop-before-cycle-queue"),
            ..Default::default()
        })
        .unwrap();
        let stop = AtomicBool::new(true);

        let termination = autonomous_loop.run_until_stopped(&stop).await.unwrap();

        assert_eq!(termination, LoopTermination::StopRequested);
        assert_eq!(autonomous_loop.state_snapshot().iteration, 0);
        assert!(autonomous_loop.receipt_history().unwrap().is_empty());
    }

    #[test]
    fn durable_completion_failure_does_not_advance_in_memory_completion() {
        let memory_dir = temp_dir("completion-failure-memory");
        let queue_dir = temp_dir("completion-failure-queue");
        let config = LoopConfig {
            memory_dir,
            queue_dir,
            ..Default::default()
        };
        let autonomous_loop = AutonomousLoop::from_config(config).unwrap();
        let job = JobV1::new(
            autonomous_loop.queue.namespace_id(),
            "completion-failure-job",
            "test",
            serde_json::json!({"prompt": "test"}),
            CanonicalToolSideEffectClass::ReadOnly,
            None,
            None,
        );
        let job_id = job.job_id.clone();
        let job_id_string = job_id.to_string();
        autonomous_loop.queue.enqueue_job(job).unwrap();
        let acquired = autonomous_loop
            .queue
            .acquire_next("autonomous-loop", 300)
            .unwrap()
            .unwrap();
        let mut invalid_lease = acquired.lease;
        invalid_lease.lease_id = aidens_contracts::ArtifactId::new("lease:wrong");
        autonomous_loop.update_state(|state| {
            state.current_job = Some(job_id.to_string());
        });

        let result = autonomous_loop.complete_successful_job(&job_id, &invalid_lease);

        assert!(result.is_err());
        let state = autonomous_loop.state_snapshot();
        assert_eq!(state.tasks_completed, 0);
        assert_eq!(state.current_job.as_deref(), Some(job_id_string.as_str()));
        let queue_snapshot = autonomous_loop.queue.snapshot().unwrap();
        assert_eq!(queue_snapshot.jobs[0].state, JobStateV1::Leased);
    }

    #[test]
    fn cancellation_failure_keeps_the_leased_job_and_does_not_mark_the_gap_attempted() {
        let memory_dir = temp_dir("cancel-failure-memory");
        let queue_dir = temp_dir("cancel-failure-queue");
        let autonomous_loop = AutonomousLoop::from_config(LoopConfig {
            memory_dir,
            queue_dir: queue_dir.clone(),
            ..Default::default()
        })
        .unwrap();
        let job = JobV1::new(
            autonomous_loop.queue.namespace_id(),
            "cancel-failure-job",
            "test",
            serde_json::json!({"fact_id": "fact:cancel", "gap_type": "missing-context"}),
            CanonicalToolSideEffectClass::ReadOnly,
            None,
            None,
        );
        let job_id = job.job_id.clone();
        let job_id_string = job_id.to_string();
        autonomous_loop.queue.enqueue_job(job).unwrap();
        autonomous_loop
            .queue
            .acquire_next("autonomous-loop", 300)
            .unwrap()
            .unwrap();
        autonomous_loop.update_state(|state| {
            state.current_job = Some(job_id_string.clone());
        });
        std::fs::write(queue_dir.join("queue.ndjson"), "not-json\n").unwrap();

        let result = autonomous_loop.cancel_failed_job(&job_id, "execution-error");

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("durable queue cancellation failed"));
        let state = autonomous_loop.state_snapshot();
        assert_eq!(state.tasks_failed, 0);
        assert_eq!(state.current_job.as_deref(), Some(job_id_string.as_str()));
        assert!(state
            .last_error
            .as_deref()
            .is_some_and(|error| error.contains("durable queue cancellation failed")));
        assert!(autonomous_loop.attempted_gaps.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn cycle_receipt_uses_immutable_subtractive_error_snapshot() {
        let memory_dir = temp_dir("subtractive-receipt-memory");
        let queue_dir = temp_dir("subtractive-receipt-queue");
        let autonomous_loop = AutonomousLoop::from_config(LoopConfig {
            max_iterations: 1,
            sleep_between_iterations_ms: 0,
            memory_dir,
            queue_dir,
            http_base_url: "http://127.0.0.1:9".into(),
            ..Default::default()
        })
        .unwrap();
        {
            let mut debt = autonomous_loop.proof_debt.lock().unwrap();
            for index in 0..4 {
                debt.incur(&format!("claim-low-{index}"), "autonomous", RiskClass::Low);
            }
            debt.pay_all_low_risk(PaymentMethod::NoContradictions);
            debt.incur("claim-medium", "autonomous", RiskClass::Medium);
        }
        autonomous_loop.update_state(|state| state.mode = CycleMode::Subtractive);

        let result = autonomous_loop.run().await;

        assert!(result.is_err());
        assert_eq!(autonomous_loop.state_snapshot().mode, CycleMode::Additive);
        let history = autonomous_loop.receipt_history().unwrap();
        let receipt = history.last().unwrap();
        assert_eq!(receipt.mode, CycleMode::Subtractive);
        assert_eq!(receipt.errors.len(), 1);
        assert!(receipt.errors[0].contains("subtractive cycle failed: discord request"));
    }

    #[test]
    fn cycle_receipt_records_nonzero_per_cycle_gap_count() {
        let memory_dir = temp_dir("gap-receipt-memory");
        let queue_dir = temp_dir("gap-receipt-queue");
        let autonomous_loop = AutonomousLoop::from_config(LoopConfig {
            memory_dir,
            queue_dir,
            ..Default::default()
        })
        .unwrap();
        autonomous_loop.update_state(|state| state.gaps_detected = 41);

        autonomous_loop
            .emit_cycle_receipt(CycleMetrics {
                iteration: 3,
                gaps: 4,
                mode: CycleMode::Additive,
                ..CycleMetrics::default()
            })
            .unwrap();

        let history = autonomous_loop.receipt_history().unwrap();
        assert_eq!(history.last().unwrap().gaps_detected, 4);
    }

    #[test]
    fn restart_recovers_last_committed_loop_state() {
        let memory_dir = temp_dir("restart-recovery-memory");
        let queue_dir = temp_dir("restart-recovery-queue");
        {
            let autonomous_loop = AutonomousLoop::from_config(LoopConfig {
                memory_dir: memory_dir.clone(),
                queue_dir: queue_dir.clone(),
                ..Default::default()
            })
            .unwrap();
            autonomous_loop.update_state(|state| {
                state.iteration = 3;
                state.gaps_detected = 2;
                state.tasks_completed = 1;
                state.facts_captured = 1;
            });
            autonomous_loop
                .emit_cycle_receipt(CycleMetrics {
                    iteration: 3,
                    gaps: 2,
                    tasks: 1,
                    captured: 1,
                    mode: CycleMode::Additive,
                    ..CycleMetrics::default()
                })
                .unwrap();
        }

        let restarted = AutonomousLoop::from_config(LoopConfig {
            memory_dir,
            queue_dir,
            ..Default::default()
        })
        .unwrap();
        let state = restarted.state_snapshot();
        assert_eq!(state.iteration, 3);
        assert_eq!(state.gaps_detected, 2);
        assert_eq!(state.tasks_completed, 1);
        assert_eq!(state.facts_captured, 1);
        restarted.receipts.lock().unwrap().verify_chain().unwrap();
    }

    #[test]
    fn loop_mode_defaults_to_shadow() {
        assert_eq!(LoopConfig::default().loop_mode, LoopMode::Shadow);
    }

    #[tokio::test]
    async fn shadow_is_write_isolated_and_autonomous_can_promote() {
        let claim = "The durable queue appends committed entries with explicit receipt evidence.";
        let make_result = || ExecutionResult {
            job_id: "job:mode-test".into(),
            output: claim.into(),
            success: true,
            error: None,
            gap_type: "missing-context".into(),
            source_fact_id: "fact:source".into(),
            source_valid_time: "2026-01-01T00:00:00Z".into(),
        };

        let shadow = AutonomousLoop::from_config(LoopConfig {
            loop_mode: LoopMode::Shadow,
            memory_dir: temp_dir("shadow-isolation-memory"),
            queue_dir: temp_dir("shadow-isolation-queue"),
            ..Default::default()
        })
        .unwrap();
        let shadow_outcome = shadow.capture.capture(&make_result()).await.unwrap();
        let shadow_candidate = shadow_outcome.candidates.first().unwrap();
        let disposition = shadow
            .apply_learning_mode(shadow_candidate, FactDisposition::Promote)
            .await
            .unwrap();
        assert_eq!(disposition, FactDisposition::Quarantine);
        let canonical = shadow
            .capture
            .memory
            .search(claim, Some(&["autonomous".into()]), Some(5))
            .await
            .unwrap();
        assert!(canonical.is_empty());

        let autonomous = AutonomousLoop::from_config(LoopConfig {
            loop_mode: LoopMode::Autonomous,
            memory_dir: temp_dir("autonomous-promotion-memory"),
            queue_dir: temp_dir("autonomous-promotion-queue"),
            ..Default::default()
        })
        .unwrap();
        let autonomous_outcome = autonomous.capture.capture(&make_result()).await.unwrap();
        let autonomous_candidate = autonomous_outcome.candidates.first().unwrap();
        let disposition = autonomous
            .apply_learning_mode(autonomous_candidate, FactDisposition::Promote)
            .await
            .unwrap();
        assert_eq!(disposition, FactDisposition::Promote);
        let canonical = autonomous
            .capture
            .memory
            .search(claim, Some(&["autonomous".into()]), Some(5))
            .await
            .unwrap();
        assert!(canonical.iter().any(|result| result.content == claim));
    }
}
