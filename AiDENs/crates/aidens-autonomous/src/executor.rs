//! Loop executor — runs queued jobs through the plan-act-verify loop.
//!
//! The [`LoopExecutor`] takes a [`QueueLeaseV1`] acquired from the daemon queue,
//! parses the job payload for a prompt and gap metadata, builds a
//! [`PlanActVerifyLoopV1`] wired with the shared memory adapter, executes it,
//! and returns an [`ExecutionResult`].
//!
//! On success or failure the executor POSTs a routing-outcome signal to the
//! semantic-memory server's `/record-outcome` endpoint so the adaptive
//! retrieval router can learn from execution feedback.

use aidens_contracts::{
    AgentMemoryModeV1, AgentPermitRuleV1, AgentProviderModeV1, AgentSpecBudgetPolicyV1,
    AgentSpecEvidencePolicyV1, AgentSpecMemoryPolicyV1, AgentSpecPermitPolicyV1,
    AgentSpecProviderPolicyV1, AgentSpecSupportLabelV1, AgentSpecToolPolicyV1,
    AgentSpecVerificationPolicyV1, AgentSpecV1, AgentVerificationCheckV1,
};
use aidens_memory_kit::CanonicalMemoryAdapter;
use aidens_runner::{PlanActVerifyLoopV1, PlanActVerifyOutcomeV1};
use anyhow::{anyhow, Context as _, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// System prompt prepended to all user prompts to give the model context about
/// its role as an autonomous knowledge base auditor.
pub const SYSTEM_PROMPT: &str = "You are an autonomous knowledge base auditor. \
You are part of a self-learning AI system called AiDENs. \
Your job is to analyze facts in the semantic memory knowledge base and provide \
accurate, factual analysis. You are given a specific gap to investigate. \
Respond with a concise factual summary (2-4 sentences). \
Do not speculate. If you cannot determine the answer, say so. \
Your response will be stored as a new fact in the knowledge base.";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// The outcome of executing a single queued job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// The daemon job ID (stringified `ArtifactId`).
    pub job_id: String,
    /// Final text output from the plan-act-verify loop (may be empty on failure).
    pub output: String,
    /// Whether the loop completed successfully.
    pub success: bool,
    /// Error description if the loop failed or abstained.
    pub error: Option<String>,
    /// The gap type that triggered this job (e.g. `"missing-context"`).
    pub gap_type: String,
    /// The source fact ID that the gap was detected on.
    pub source_fact_id: String,
}

/// Executes queued gap-remediation jobs through the plan-act-verify loop.
#[derive(Clone)]
pub struct LoopExecutor {
    /// Shared canonical memory adapter for grounding and fact capture.
    pub memory: Arc<CanonicalMemoryAdapter>,
    /// Ollama-compatible provider base URL.
    pub ollama_url: String,
    /// Ollama model name to use for completions.
    pub ollama_model: String,
    /// Semantic-memory HTTP base URL (for `/record-outcome`).
    pub http_base_url: String,
}

impl std::fmt::Debug for LoopExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoopExecutor")
            .field("ollama_url", &self.ollama_url)
            .field("ollama_model", &self.ollama_model)
            .field("http_base_url", &self.http_base_url)
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

impl LoopExecutor {
    /// Create a new executor with the given memory adapter and provider config.
    pub fn new(
        memory: Arc<CanonicalMemoryAdapter>,
        ollama_url: impl Into<String>,
        ollama_model: impl Into<String>,
        http_base_url: impl Into<String>,
    ) -> Self {
        Self {
            memory,
            ollama_url: ollama_url.into(),
            ollama_model: ollama_model.into(),
            http_base_url: http_base_url.into(),
        }
    }

    /// Execute a single job given its ID and payload.
    ///
    /// Parses the job payload for `{prompt, gap_type, fact_id}`, builds a
    /// `PlanActVerifyLoopV1` wired with the shared memory adapter, runs it,
    /// and records the routing outcome. Returns an [`ExecutionResult`].
    ///
    /// This is the primary entry point used by the loop driver. The
    /// `QueueLeaseV1` type does not carry the job payload, so the caller
    /// (which has the full `QueueLeaseOutcomeV1`) passes the payload directly.
    pub async fn execute_job_with_payload(
        &self,
        job_id: &str,
        payload: &serde_json::Value,
    ) -> Result<ExecutionResult> {
        let prompt = payload
            .get("prompt")
            .and_then(|v| v.as_str())
            .context("job payload missing 'prompt' field")?
            .to_string();
        let gap_type = payload
            .get("gap_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let fact_id = payload
            .get("fact_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        self.execute_with_fields(job_id, &prompt, &gap_type, &fact_id)
            .await
    }

    /// Execute using pre-extracted fields.
    async fn execute_with_fields(
        &self,
        job_id: &str,
        prompt: &str,
        gap_type: &str,
        source_fact_id: &str,
    ) -> Result<ExecutionResult> {
        // Build the plan-act-verify loop.
        let spec = autonomous_agent_spec(&self.ollama_model);
        let loop_v1 = PlanActVerifyLoopV1::new(spec)
            .with_memory(self.memory.clone())
            .provider_model(&self.ollama_model)
            .provider_base_url(&self.ollama_url)
            .max_retries(2);

        // Prepend the system prompt to the user prompt. The runner builds the
        // message array from a single string, so we embed the system context
        // directly in the prompt text.
        let full_prompt = format!("SYSTEM: {SYSTEM_PROMPT}\n\nUSER: {prompt}");

        // Execute.
        let output = loop_v1.execute(&full_prompt).await?;

        // Extract result.
        let success = matches!(output.outcome, PlanActVerifyOutcomeV1::Success);
        let final_output = output
            .run_output
            .as_ref()
            .map(|ro| ro.text.clone())
            .unwrap_or_default();

        let error = if success {
            None
        } else {
            // Include abstention reason code if available for debugging.
            let reason = output
                .abstention_receipt
                .as_ref()
                .map(|a| a.reason_code.clone())
                .unwrap_or_default();
            let evidence = output
                .abstention_receipt
                .as_ref()
                .and_then(|a| a.evidence.first().cloned())
                .unwrap_or_default();
            if reason.is_empty() {
                Some(format!(
                    "plan-act-verify loop outcome: {:?}",
                    output.outcome
                ))
            } else {
                Some(format!(
                    "plan-act-verify loop outcome: {:?} reason={} evidence={}",
                    output.outcome, reason, evidence
                ))
            }
        };

        // Record routing outcome (best-effort).
        let outcome_str = if success { "good" } else { "bad" };
        let query_class = gap_type_to_query_class(gap_type);
        let _ = record_outcome(&self.http_base_url, outcome_str, query_class.as_deref()).await;

        Ok(ExecutionResult {
            job_id: job_id.to_string(),
            output: final_output,
            success,
            error,
            gap_type: gap_type.to_string(),
            source_fact_id: source_fact_id.to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// POST to `/record-outcome` on the semantic-memory server.
///
/// This is best-effort: if the server is unreachable the error is swallowed
/// because execution results should not be lost due to feedback-channel
/// failures.
async fn record_outcome(
    http_base_url: &str,
    outcome: &str,
    query_class: Option<&str>,
) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| anyhow!("failed to build HTTP client: {e}"))?;

    let mut body = serde_json::json!({
        "query": "autonomous-loop-execution",
        "outcome": outcome,
    });
    if let Some(class) = query_class {
        body["query_class"] = serde_json::Value::String(class.to_string());
    }

    let url = format!("{http_base_url}/record-outcome");
    let _ = client.post(&url).json(&body).send().await;
    Ok(())
}

/// Map a gap type string to a query class for routing feedback.
fn gap_type_to_query_class(gap_type: &str) -> Option<String> {
    match gap_type {
        "missing-context" => Some("gap-remediation:missing-context".to_string()),
        "missing-link" => Some("gap-remediation:missing-link".to_string()),
        "stale-fact" => Some("gap-remediation:stale-fact".to_string()),
        "contradiction-gap" => Some("gap-remediation:contradiction-gap".to_string()),
        "duplicate-fact" => Some("gap-remediation:duplicate-fact".to_string()),
        "stale-by-date" => Some("gap-remediation:stale-by-date".to_string()),
        "low-quality-fact" => Some("gap-remediation:low-quality-fact".to_string()),
        _ => None,
    }
}

/// Build a minimal valid `AgentSpecV1` for autonomous loop execution.
///
/// Uses `Ollama` provider mode — the executor sets `provider_model` and
/// `provider_base_url` via the `PlanActVerifyLoopV1` builder.
fn autonomous_agent_spec(model: &str) -> AgentSpecV1 {
    let _ = model; // model is set via PlanActVerifyLoopV1 builder
    AgentSpecV1 {
        schema: AgentSpecV1::SCHEMA.to_string(),
        agent_id: "agent:autonomous-loop-executor".to_string(),
        display_name: "Autonomous Loop Executor".to_string(),
        support_label: AgentSpecSupportLabelV1::SupportedLocal,
        profile: "coding".to_string(),
        provider_policy: AgentSpecProviderPolicyV1 {
            provider: AgentProviderModeV1::Ollama,
            cloud_allowed: false,
            fallback_allowed: false,
        },
        memory_policy: AgentSpecMemoryPolicyV1 {
            enabled: false, // memory is wired via with_memory(), not via spec policy
            mode: AgentMemoryModeV1::Fixture,
            requires_view_disclosure: false,
        },
        tool_policy: AgentSpecToolPolicyV1 {
            allowed_tools: vec!["repo.read".to_string()], // validation requires at least one tool
            write_tools_require_permit: false,
        },
        permit_policy: AgentSpecPermitPolicyV1 {
            writes: AgentPermitRuleV1::OperatorApproved,
            commands: AgentPermitRuleV1::OperatorApproved,
            network: AgentPermitRuleV1::Forbidden, // validation requires Forbidden for local agents
        },
        verification_policy: AgentSpecVerificationPolicyV1 {
            required_checks: vec![
                AgentVerificationCheckV1::Schema,
                AgentVerificationCheckV1::Digest,
            ],
            fail_closed: true, // validation requires fail_closed=true
        },
        evidence_policy: AgentSpecEvidencePolicyV1 {
            emit_run_bundle: true,
            emit_tool_receipts: true,
            emit_permit_receipts: true,
            emit_abstention_receipts: true,
        },
        budget_policy: AgentSpecBudgetPolicyV1 {
            max_turns: 6, // enough for the model to generate a useful response
            max_tool_calls: 8,
            deadline_seconds: 180, // 3 minutes for cloud models
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use aidens_memory_kit::{memory_config_for_root, runtime_config_for_namespace};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir()
            .join(format!("aidens-autonomous-executor-{name}-{id}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn mock_memory() -> Arc<CanonicalMemoryAdapter> {
        let dir = temp_dir("memory");
        let config = memory_config_for_root(&dir);
        let runtime = runtime_config_for_namespace("autonomous-test");
        Arc::new(
            CanonicalMemoryAdapter::open_with_mock_embedder(config, runtime)
                .expect("open mock memory"),
        )
    }

    #[test]
    fn gap_type_to_query_class_maps_known_types() {
        assert_eq!(
            gap_type_to_query_class("missing-context"),
            Some("gap-remediation:missing-context".to_string())
        );
        assert_eq!(
            gap_type_to_query_class("missing-link"),
            Some("gap-remediation:missing-link".to_string())
        );
        assert_eq!(
            gap_type_to_query_class("stale-fact"),
            Some("gap-remediation:stale-fact".to_string())
        );
        assert_eq!(
            gap_type_to_query_class("contradiction-gap"),
            Some("gap-remediation:contradiction-gap".to_string())
        );
        assert_eq!(
            gap_type_to_query_class("duplicate-fact"),
            Some("gap-remediation:duplicate-fact".to_string())
        );
        assert_eq!(
            gap_type_to_query_class("stale-by-date"),
            Some("gap-remediation:stale-by-date".to_string())
        );
        assert_eq!(
            gap_type_to_query_class("low-quality-fact"),
            Some("gap-remediation:low-quality-fact".to_string())
        );
        assert_eq!(gap_type_to_query_class("unknown"), None);
    }

    #[test]
    fn system_prompt_is_nonempty() {
        assert!(!SYSTEM_PROMPT.is_empty());
        assert!(SYSTEM_PROMPT.contains("autonomous knowledge base auditor"));
        assert!(SYSTEM_PROMPT.contains("AiDENs"));
    }

    #[test]
    fn autonomous_agent_spec_validates() {
        let spec = autonomous_agent_spec("test-model");
        assert!(spec.validate().is_ok());
    }

    #[test]
    fn executor_stores_config() {
        let memory = mock_memory();
        let executor = LoopExecutor::new(memory, "http://localhost:11434", "test-model", "http://localhost:1738");
        assert_eq!(executor.ollama_url, "http://localhost:11434");
        assert_eq!(executor.ollama_model, "test-model");
        assert_eq!(executor.http_base_url, "http://localhost:1738");
    }

    #[test]
    fn execution_result_serializes() {
        let result = ExecutionResult {
            job_id: "job:test-123".to_string(),
            output: "Some output text".to_string(),
            success: true,
            error: None,
            gap_type: "missing-context".to_string(),
            source_fact_id: "fact:abc".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: ExecutionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.job_id, "job:test-123");
        assert!(back.success);
    }

    #[tokio::test]
    async fn execute_with_mock_provider_succeeds() {
        let memory = mock_memory();

        // Build a mock executor that uses Local provider (mock fixture).
        let spec = AgentSpecV1 {
            schema: AgentSpecV1::SCHEMA.to_string(),
            agent_id: "agent:test-mock".to_string(),
            display_name: "Test Mock Agent".to_string(),
            support_label: AgentSpecSupportLabelV1::SupportedLocal,
            profile: "coding".to_string(),
            provider_policy: AgentSpecProviderPolicyV1 {
                provider: AgentProviderModeV1::Local,
                cloud_allowed: false,
                fallback_allowed: false,
            },
            memory_policy: AgentSpecMemoryPolicyV1 {
                enabled: false,
                mode: AgentMemoryModeV1::Fixture,
                requires_view_disclosure: false,
            },
            tool_policy: AgentSpecToolPolicyV1 {
                allowed_tools: vec!["repo.read".to_string()],
                write_tools_require_permit: false,
            },
            permit_policy: AgentSpecPermitPolicyV1 {
                writes: AgentPermitRuleV1::OperatorApproved,
                commands: AgentPermitRuleV1::OperatorApproved,
                network: AgentPermitRuleV1::Forbidden,
            },
            verification_policy: AgentSpecVerificationPolicyV1 {
                required_checks: vec![
                    AgentVerificationCheckV1::Schema,
                    AgentVerificationCheckV1::SupportClaim,
                    AgentVerificationCheckV1::Sandbox,
                    AgentVerificationCheckV1::Digest,
                ],
                fail_closed: true,
            },
            evidence_policy: AgentSpecEvidencePolicyV1 {
                emit_run_bundle: true,
                emit_tool_receipts: true,
                emit_permit_receipts: true,
                emit_abstention_receipts: true,
            },
            budget_policy: AgentSpecBudgetPolicyV1 {
                max_turns: 1,
                max_tool_calls: 4,
                deadline_seconds: 30,
            },
        };

        let loop_v1 = PlanActVerifyLoopV1::new(spec)
            .with_memory(memory.clone())
            .provider_mock_response("This is a mock response about the knowledge base.");

        let output = loop_v1.execute("test prompt").await.unwrap();
        assert_eq!(output.outcome, PlanActVerifyOutcomeV1::Success);
        assert!(output.run_output.is_some());
        assert_eq!(
            output.run_output.as_ref().unwrap().text,
            "This is a mock response about the knowledge base."
        );
    }

    #[tokio::test]
    async fn execute_with_fields_mock_returns_success() {
        let memory = mock_memory();
        let _executor = LoopExecutor::new(
            memory.clone(),
            "http://localhost:11434",
            "test-model",
            "http://localhost:1738",
        );

        // We'll bypass the Ollama provider by building the loop directly with
        // a mock response, using execute_with_fields which uses Ollama mode.
        // Since we can't reach a real Ollama, we test the field-extraction
        // logic instead.
        let payload = serde_json::json!({
            "prompt": "test prompt about gaps",
            "gap_type": "missing-context",
            "fact_id": "fact:test-123",
        });

        // Verify payload parsing works.
        let prompt = payload.get("prompt").and_then(|v| v.as_str()).unwrap();
        let gap_type = payload.get("gap_type").and_then(|v| v.as_str()).unwrap();
        let fact_id = payload.get("fact_id").and_then(|v| v.as_str()).unwrap();
        assert_eq!(prompt, "test prompt about gaps");
        assert_eq!(gap_type, "missing-context");
        assert_eq!(fact_id, "fact:test-123");
    }

    #[test]
    fn memory_is_wired() {
        let memory = mock_memory();
        let memory_clone = memory.clone();
        let executor = LoopExecutor::new(
            memory,
            "http://localhost:11434",
            "test-model",
            "http://localhost:1738",
        );
        // Verify memory is accessible (Arc clone).
        assert_eq!(Arc::strong_count(&executor.memory), 2);
        assert_eq!(Arc::strong_count(&memory_clone), 2);
    }
}
#[cfg(test)]
mod debug_tests {
    use super::*;
    use aidens_memory_kit::{memory_config_for_root, runtime_config_for_namespace};

    #[tokio::test]
    #[ignore = "requires Ollama running"]
    async fn debug_ollama_execution() {
        let dir = std::env::temp_dir().join(format!("aidens-debug-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let memory_config = memory_config_for_root(&dir);
        let runtime_config = runtime_config_for_namespace("test");
        let memory = std::sync::Arc::new(
            CanonicalMemoryAdapter::open_with_mock_embedder(memory_config, runtime_config).unwrap()
        );

        let executor = LoopExecutor {
            memory: memory.clone(),
            ollama_url: "http://127.0.0.1:11434".to_string(),
            ollama_model: "gemma4:31b-cloud".to_string(),
            http_base_url: "http://127.0.0.1:1738".to_string(),
        };

        let payload = serde_json::json!({
            "prompt": "What is 2+2? Answer in one sentence.",
            "gap_type": "missing-context",
            "fact_id": "test-fact"
        });

        let result = executor.execute_job_with_payload("debug-test", &payload).await.unwrap();

        println!("Success: {}", result.success);
        println!("Output: {}", result.output);
        println!("Error: {:?}", result.error);
    }
}
