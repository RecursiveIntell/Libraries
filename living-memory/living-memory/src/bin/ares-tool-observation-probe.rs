//! OFFLINE, DISPOSABLE canary for the tool-receipt -> observation-memory path.
//!
//! This is NOT a production transport, adapter, or integration, and it makes no
//! production-capability claim. It is a default-off CLI that must be invoked
//! explicitly by an operator, with a mandatory opt-in flag.
//!
//! Safety shape:
//! * Disabled unless `--enable-offline-canary` is passed. Wrong/missing scope or
//!   any argument problem is denied BEFORE any filesystem access.
//! * The tool (`ares_offline_available_parallelism`) and namespace
//!   (`ares-tool-observation-offline-canary`) are compile-time constants; the
//!   caller cannot choose a tool, command, or path to execute.
//! * `--out-dir` must name a directory that does not yet exist (its parent must
//!   exist). It is created atomically with `create_dir`; an existing path of any
//!   kind (empty, non-empty, file, symlink) is denied and nothing is executed.
//!   Both the Forge DB and the memory store live only under that directory.
//! * The tool calls `std::thread::available_parallelism()`: no subprocess,
//!   caller-selected files, or network. `MockEmbedder` is used because this canary is
//!   explicitly offline and disposable; it must not be used for real memory.
//! * Exactly one execution, one projection attempt, no automatic retry. To try
//!   again the operator must supply a different NEW directory.
//! * Output is a single JSON metadata line (status, receipt ID, readback
//!   booleans). The tool's backend payload and the raw receipt are never printed.

use async_trait::async_trait;
use forge_engine::{ForgeStore, ForgeToolReceiptSink};
use llm_tool_runtime::semantic_memory::ToolObservationMemory;
use llm_tool_runtime::{
    McpSurfaceKind, Tool, ToolApprovalKind, ToolBackendKind, ToolCall, ToolCtx, ToolDescriptor,
    ToolError, ToolErrorClass, ToolExposureMode, ToolExposurePolicy, ToolIdempotencyClass,
    ToolOriginKind, ToolOutputMode, ToolPlannerStage, ToolReceiptPersistence, ToolRegistry,
    ToolResult, ToolRetryOwner, ToolRuntime, ToolSideEffectClass,
};
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder};
use stack_ids::{AttemptId, TraceCtx, TrialId};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

/// Fixed tool: not selectable by the caller or by any model.
const TOOL_NAME: &str = "ares_offline_available_parallelism";
const TOOL_VERSION: &str = "1.0.0";
/// Fixed observation namespace; `--scope` must equal this exactly.
const NAMESPACE: &str = "ares-tool-observation-offline-canary";
const ENABLE_FLAG: &str = "--enable-offline-canary";
const FORGE_DB_FILE: &str = "forge.db";
const MEMORY_DIR: &str = "mem";
const CALLER: &str = "ares-tool-observation-probe";

const USAGE: &str = "usage: ares-tool-observation-probe --enable-offline-canary \
--scope ares-tool-observation-offline-canary --out-dir <NEW_DIRECTORY>";

struct Args {
    out_dir: PathBuf,
}

/// Why the run stopped before touching the filesystem.
enum Refusal {
    Disabled,
    Denied(&'static str),
}

fn parse_args(args: Vec<OsString>) -> Result<Args, Refusal> {
    // Default-off: without the explicit flag nothing else is even interpreted.
    if !args.iter().any(|arg| arg.to_str() == Some(ENABLE_FLAG)) {
        return Err(Refusal::Disabled);
    }
    let mut enabled = false;
    let mut scope: Option<String> = None;
    let mut out_dir: Option<PathBuf> = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.to_str() {
            Some(ENABLE_FLAG) => {
                if enabled {
                    return Err(Refusal::Denied("duplicate argument"));
                }
                enabled = true;
            }
            Some("--scope") => {
                if scope.is_some() {
                    return Err(Refusal::Denied("duplicate argument"));
                }
                let value = iter
                    .next()
                    .and_then(|value| value.into_string().ok())
                    .ok_or(Refusal::Denied("missing or invalid --scope value"))?;
                scope = Some(value);
            }
            Some("--out-dir") => {
                if out_dir.is_some() {
                    return Err(Refusal::Denied("duplicate argument"));
                }
                let value = iter
                    .next()
                    .ok_or(Refusal::Denied("missing --out-dir value"))?;
                out_dir = Some(PathBuf::from(value));
            }
            _ => return Err(Refusal::Denied("unrecognized argument")),
        }
    }
    if scope.as_deref() != Some(NAMESPACE) {
        return Err(Refusal::Denied("scope not admitted for this canary"));
    }
    let out_dir = out_dir.ok_or(Refusal::Denied("missing --out-dir"))?;
    if out_dir.as_os_str().is_empty() {
        return Err(Refusal::Denied("empty --out-dir"));
    }
    Ok(Args { out_dir })
}

/// Atomically create the single NEW sandbox directory. Never creates parents and
/// never reuses an existing path.
fn create_new_sandbox_dir(path: &Path) -> Result<(), &'static str> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    match builder.create(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            Err("out-dir already exists; a new directory is required")
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err("out-dir parent does not exist")
        }
        Err(_) => Err("could not create out-dir"),
    }
}

/// The only tool this canary can run. Reads one OS value, with no caller-selected
/// files, subprocesses, or network.
struct AvailableParallelismTool {
    descriptor: ToolDescriptor,
}

impl AvailableParallelismTool {
    fn new() -> Self {
        Self {
            descriptor: ToolDescriptor {
                name: TOOL_NAME.into(),
                version: TOOL_VERSION.into(),
                description: Some("Offline canary: read std::thread::available_parallelism".into()),
                backend_kind: ToolBackendKind::LocalFunction,
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }),
                output_mode: ToolOutputMode::StructuredJson,
                read_only: true,
                side_effect_class: ToolSideEffectClass::ReadOnly,
                idempotency_class: ToolIdempotencyClass::Idempotent,
                approval_kind: ToolApprovalKind::None,
                timeout_ms: 5_000,
                concurrency_key: None,
                cache_ttl_ms: None,
                exposure_mode: ToolExposureMode::Hidden,
                mcp_surface_kind: McpSurfaceKind::None,
                exposure_policy: ToolExposurePolicy::default(),
                receipt_persistence: ToolReceiptPersistence::ForgeRaw,
                output_size_limit_bytes: Some(1024),
                provider_payload: None,
            },
        }
    }
}

#[async_trait]
impl Tool for AvailableParallelismTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn invoke(&self, _ctx: &ToolCtx, _call: &ToolCall) -> Result<ToolResult, ToolError> {
        let parallelism = std::thread::available_parallelism().map_err(|_| {
            ToolError::new(ToolErrorClass::Execution, "available_parallelism failed")
        })?;
        Ok(ToolResult::json(
            serde_json::json!({ "available_parallelism": parallelism.get() }),
        ))
    }
}

fn tool_ctx() -> ToolCtx {
    ToolCtx {
        trace_ctx: TraceCtx::generate(),
        attempt_id: AttemptId::generate(),
        trial_id: TrialId::generate(),
        deadline: None,
        workload_class: Some("offline_canary".into()),
        budget_context: None,
        scope: None,
        dry_run: false,
        approval_grant: None,
        execution_permit: None,
        idempotency_key: None,
        caller: CALLER.into(),
        planner_stage: ToolPlannerStage::Execution,
        parent_receipt_id: None,
        family_receipt_id: None,
        replay_parent_receipt_id: None,
        remote_oracle_lease_id: None,
        remote_slice_result_id: None,
        attestation_envelope_id: None,
        cross_runtime_replay_ticket_id: None,
        retry_owner: Some(ToolRetryOwner::External),
    }
}

/// A stage failure reported as metadata only (stage + error class).
struct Failure {
    stage: &'static str,
    class: String,
}

impl Failure {
    fn at(stage: &'static str, class: impl std::fmt::Debug) -> Self {
        Self {
            stage,
            class: format!("{class:?}"),
        }
    }
}

fn memory_config(sandbox: &Path) -> MemoryConfig {
    MemoryConfig {
        base_dir: sandbox.join(MEMORY_DIR),
        ..Default::default()
    }
}

/// OFFLINE ONLY: deterministic mock embedder, no network or model.
fn open_memory(sandbox: &Path) -> Result<MemoryStore, Failure> {
    let config = memory_config(sandbox);
    let embedder = MockEmbedder::new(config.embedding.dimensions);
    MemoryStore::open_with_embedder(config, Box::new(embedder))
        .map_err(|_| Failure::at("memory_open", "Open"))
}

/// Phase 1: execute once, commit the Forge receipt, project by committed ID.
/// Everything opened here is dropped on return so phase 2 truly reopens.
async fn execute_and_project(sandbox: &Path) -> Result<String, Failure> {
    let store = Arc::new(
        ForgeStore::open(&sandbox.join(FORGE_DB_FILE))
            .map_err(|_| Failure::at("forge_open", "Open"))?,
    );
    let memory = open_memory(sandbox)?;
    let sink = Arc::new(
        ForgeToolReceiptSink::with_observation_scope(store, TOOL_NAME, NAMESPACE)
            .map_err(|error| Failure::at("sink_scope", error.class))?,
    );

    let mut registry = ToolRegistry::new();
    registry.register(AvailableParallelismTool::new());
    let runtime = ToolRuntime::new(registry).with_receipt_sink(sink.clone());

    let call = ToolCall::new(
        TOOL_NAME,
        TOOL_VERSION,
        serde_json::json!({}),
        ToolOriginKind::Local,
    );
    // Single execution. The runtime returns only after the ForgeRaw sink
    // acknowledged the receipt; a persistence failure surfaces as `result: Err`.
    let execution = runtime.execute(&tool_ctx(), &call, None, None).await;
    if let Err(error) = execution.result {
        return Err(Failure::at("execute", error.class));
    }
    let receipt_id = execution.receipt.receipt_id;

    // Project strictly by committed receipt ID. The payload in `execution.result`
    // is dropped without being printed or projected.
    sink.project_observation(&memory, &receipt_id)
        .await
        .map_err(|error| Failure::at("project", error.class))?;
    Ok(receipt_id)
}

struct Readback {
    forge_found: bool,
    forge_scope_matches: bool,
    forge_status_succeeded: bool,
    projection_found: bool,
    projection_status_succeeded: bool,
}

/// Phase 2: reopen both stores from disk and read the receipt + projection back.
async fn reopen_and_read_back(sandbox: &Path, receipt_id: &str) -> Result<Readback, Failure> {
    let store = ForgeStore::open(&sandbox.join(FORGE_DB_FILE))
        .map_err(|_| Failure::at("forge_reopen", "Open"))?;
    let row = store
        .get_tool_receipt(receipt_id)
        .map_err(|_| Failure::at("forge_readback", "Read"))?;
    let (forge_found, forge_scope_matches, forge_status_succeeded) = match &row {
        Some(row) => (
            row.receipt_id == receipt_id && row.tool_name == TOOL_NAME,
            row.observation_scope.as_deref() == Some(NAMESPACE),
            row.error_class.is_none(),
        ),
        None => (false, false, false),
    };

    let memory = open_memory(sandbox)?;
    let found = memory
        .find_similar_tool_observations(TOOL_NAME, NAMESPACE, 10)
        .await
        .map_err(|error| Failure::at("projection_readback", error.class))?;
    let matching = found.observations.iter().find(|record| {
        record.receipt_id.as_deref() == Some(receipt_id)
            && record.tool_name == TOOL_NAME
            && record.scope == NAMESPACE
    });
    Ok(Readback {
        forge_found,
        forge_scope_matches,
        forge_status_succeeded,
        projection_found: matching.is_some(),
        projection_status_succeeded: matching.is_some_and(|record| record.status == "succeeded"),
    })
}

fn emit(value: serde_json::Value) {
    println!("{value}");
}

fn refuse(refusal: Refusal) -> ExitCode {
    let (status, reason) = match refusal {
        Refusal::Disabled => (
            "disabled",
            "offline canary is off unless explicitly enabled",
        ),
        Refusal::Denied(reason) => ("denied", reason),
    };
    emit(serde_json::json!({
        "canary": "offline_disposable",
        "status": status,
        "reason": reason,
        "executed": false,
    }));
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args_os().skip(1).collect()) {
        Ok(args) => args,
        Err(refusal) => return refuse(refusal),
    };
    if let Err(reason) = create_new_sandbox_dir(&args.out_dir) {
        return refuse(Refusal::Denied(reason));
    }

    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(_) => {
            emit(serde_json::json!({
                "canary": "offline_disposable",
                "status": "failed",
                "stage": "async_runtime",
                "executed": false,
            }));
            return ExitCode::from(1);
        }
    };

    // No retry anywhere below: one execution, one projection, one readback.
    let outcome = runtime.block_on(async {
        let receipt_id = execute_and_project(&args.out_dir).await?;
        let readback = reopen_and_read_back(&args.out_dir, &receipt_id).await?;
        Ok::<_, Failure>((receipt_id, readback))
    });

    match outcome {
        Ok((receipt_id, readback)) => {
            let ok = readback.forge_found
                && readback.forge_scope_matches
                && readback.forge_status_succeeded
                && readback.projection_found
                && readback.projection_status_succeeded;
            emit(serde_json::json!({
                "canary": "offline_disposable",
                "status": if ok { "succeeded" } else { "readback_mismatch" },
                "executed": true,
                "tool_name": TOOL_NAME,
                "namespace": NAMESPACE,
                "receipt_id": receipt_id,
                "reopened": true,
                "retries": 0,
                "embedder": "mock_offline_only",
                "forge_readback": {
                    "found": readback.forge_found,
                    "scope_matches": readback.forge_scope_matches,
                    "status": if readback.forge_status_succeeded { "succeeded" } else { "failed" },
                },
                "projection_readback": {
                    "found": readback.projection_found,
                    "status": if readback.projection_status_succeeded { "succeeded" } else { "failed" },
                },
                "backend_payload": "withheld",
            }));
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(failure) => {
            emit(serde_json::json!({
                "canary": "offline_disposable",
                "status": "failed",
                "stage": failure.stage,
                "error_class": failure.class,
                "retries": 0,
            }));
            ExitCode::from(1)
        }
    }
}
