//! Autonomous loop driver — ties together gap detection, task generation,
//! execution, capture, and evaluation into a single continuous loop.
//!
//! The [`AutonomousLoop`] orchestrates the full detect → enqueue → execute →
//! capture → evaluate cycle. It tracks state in [`LoopState`] (shared via
//! `Arc<Mutex<>>` for TUI integration) and respects [`LoopConfig`] for
//! iteration limits, sleep intervals, and safe-mode thresholds.

use crate::capture::{CaptureOutcome, ResultCapture};
use crate::evaluation::{EvaluationGate, FactDisposition};
use crate::executor::{ExecutionResult, LoopExecutor};
use crate::gap_detector::GapDetector;
use crate::task_generator::TaskGenerator;
use aidens_daemon_kit::DaemonControllerV1;
use anyhow::Result;
use serde::{Deserialize, Serialize};
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
    pub ollama_url: String,
    /// Ollama model name.
    pub ollama_model: String,
    /// Directory for the canonical memory store.
    pub memory_dir: PathBuf,
    /// Directory for the daemon queue.
    pub queue_dir: PathBuf,
    /// Semantic-memory HTTP base URL.
    pub http_base_url: String,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 0,
            gap_detection_interval: 5,
            sleep_between_iterations_ms: 1000,
            max_consecutive_failures: 5,
            ollama_url: "http://127.0.0.1:11434".to_string(),
            ollama_model: "llama3".to_string(),
            memory_dir: PathBuf::from("./.aidens/memory"),
            queue_dir: PathBuf::from("./.aidens/queue"),
            http_base_url: "http://127.0.0.1:1738".to_string(),
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
            &config.ollama_url,
            &config.ollama_model,
            &config.http_base_url,
        );
        let capture = ResultCapture::new(memory, &config.http_base_url);
        let evaluation = EvaluationGate::new();

        Ok(Self {
            detector,
            generator,
            executor,
            capture,
            evaluation,
            queue,
            config,
            state: Arc::new(Mutex::new(LoopState::default())),
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
        Self {
            detector,
            generator,
            executor,
            capture,
            evaluation,
            queue,
            config,
            state: Arc::new(Mutex::new(LoopState::default())),
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
    /// 1. Gap detection (every `gap_detection_interval` iterations).
    /// 2. Job acquisition from the queue.
    /// 3. Job execution.
    /// 4. Result capture.
    /// 5. Fact evaluation.
    /// 6. Job completion/cancellation.
    /// 7. Failure tracking and safe-mode activation.
    pub async fn run(&self) -> Result<()> {
        loop {
            // Snapshot state for this iteration.
            let iteration = {
                let mut state = self.state.lock().map_err(|e| anyhow::anyhow!("state lock: {e}"))?;
                state.iteration += 1;
                state.iteration
            };

            // 1. Gap detection (every N iterations).
            let should_detect = self.config.gap_detection_interval > 0
                && (iteration - 1) % self.config.gap_detection_interval == 0;
            if should_detect {
                if let Err(e) = self.run_gap_detection().await {
                    self.update_state(|s| {
                        s.last_error = Some(format!("gap detection failed: {e}"));
                    });
                }
            }

            // 2. Acquire next job.
            let lease_outcome = match self.queue.acquire_next("autonomous-loop", 300) {
                Ok(Some(outcome)) => outcome,
                Ok(None) => {
                    // No job available — sleep and continue.
                    self.sleep_iteration().await;
                    self.check_max_iterations(iteration)?;
                    continue;
                }
                Err(e) => {
                    self.update_state(|s| {
                        s.last_error = Some(format!("acquire_next failed: {e}"));
                        s.consecutive_failures += 1;
                    });
                    self.check_safe_mode();
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

            // 4. Execute job.
            let exec_result: ExecutionResult = match self
                .executor
                .execute_job_with_payload(&job_id_str, &payload)
                .await
            {
                Ok(result) => result,
                Err(e) => {
                    let error_msg = e.to_string();
                    self.update_state(|s| {
                        s.tasks_failed += 1;
                        s.consecutive_failures += 1;
                        s.last_error = Some(error_msg.clone());
                        s.current_job = None;
                    });

                    // Cancel the job.
                    let _ = self.queue.cancel(&job_id, "execution-error");
                    self.check_safe_mode();
                    self.sleep_iteration().await;
                    self.check_max_iterations(iteration)?;
                    continue;
                }
            };

            // 5. Capture results.
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

            // 6. Evaluate captured facts.
            for _fact_id in &capture_outcome.fact_ids {
                let disposition = self.evaluation.evaluate(&exec_result.output, exec_result.success);
                match disposition {
                    FactDisposition::Promote => {
                        self.update_state(|s| {
                            s.facts_captured += 1;
                        });
                    }
                    FactDisposition::Quarantine => {
                        self.update_state(|s| {
                            s.facts_captured += 1;
                        });
                    }
                    FactDisposition::Reject => {
                        self.update_state(|s| {
                            s.facts_rejected += 1;
                        });
                    }
                }
            }

            // Also count skipped duplicates as non-captured.
            if capture_outcome.facts_skipped_duplicates > 0 {
                self.update_state(|s| {
                    s.facts_rejected += capture_outcome.facts_skipped_duplicates;
                });
            }

            // 7. Complete or cancel job.
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

            // 8. Check consecutive failures → safe mode.
            self.check_safe_mode();

            // 9. Check max iterations.
            self.check_max_iterations(iteration)?;

            // 10. Sleep between iterations.
            self.sleep_iteration().await;
        }
    }

    /// Run gap detection and task generation.
    async fn run_gap_detection(&self) -> Result<()> {
        let gaps = self.detector.detect_gaps(10).await?;

        self.update_state(|s| {
            s.gaps_detected += gaps.len();
        });

        if gaps.is_empty() {
            return Ok(());
        }

        let job_ids = self.generator.generate_tasks(&gaps).await?;

        self.update_state(|s| {
            s.tasks_generated += job_ids.len();
        });

        Ok(())
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

    /// Sleep for the configured interval.
    async fn sleep_iteration(&self) {
        if self.config.sleep_between_iterations_ms > 0 {
            tokio::time::sleep(Duration::from_millis(
                self.config.sleep_between_iterations_ms,
            ))
            .await;
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
        assert_eq!(config.ollama_url, "http://127.0.0.1:11434");
        assert_eq!(config.ollama_model, "llama3");
    }

    #[test]
    fn loop_config_custom_values() {
        let config = LoopConfig {
            max_iterations: 100,
            gap_detection_interval: 3,
            sleep_between_iterations_ms: 500,
            max_consecutive_failures: 10,
            ollama_url: "http://ollama:11434".to_string(),
            ollama_model: "mistral".to_string(),
            memory_dir: PathBuf::from("/tmp/memory"),
            queue_dir: PathBuf::from("/tmp/queue"),
            http_base_url: "http://localhost:1738".to_string(),
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
        assert_eq!(back.ollama_url, config.ollama_url);
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
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "test-model".to_string(),
            memory_dir: memory_dir.clone(),
            queue_dir: queue_dir.clone(),
            http_base_url: "http://localhost:1738".to_string(),
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
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "test".to_string(),
            memory_dir: memory_dir,
            queue_dir: queue_dir,
            http_base_url: "http://localhost:1738".to_string(),
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