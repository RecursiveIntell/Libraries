//! pilot-bridge — JSON stdin/stdout bridge for forge-pilot evaluation.
//!
//! Exposes forge-pilot's closed-loop capabilities (observe, bootstrap,
//! evaluate, receipt verification) as a simple CLI, mirroring the
//! cea-bridge / knowledge-router pattern. This bridge runs REAL forge-engine
//! evidence flows — it is the "real causal lane", distinct from cea-bridge's
//! synthetic telemetry.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use forge_engine::ForgeStore;
use forge_pilot::{
    bootstrap_source_workspace, observe_scope, LoopConfig, LoopRunner, LoopRunnerResources,
};
use knowledge_runtime::{RuntimeConfig, Scope};
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder};
use serde::{Deserialize, Serialize};

const EVIDENCE_KIND: &str = "forge_pilot";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BridgeRequest {
    #[serde(default)]
    workspace_path: Option<String>,
    #[serde(default)]
    memory_dir: Option<String>,
    #[serde(default)]
    forge_db: Option<String>,
    #[serde(default)]
    namespace: Option<String>,
    #[serde(default)]
    max_iterations: Option<u32>,
    #[serde(default)]
    time_budget_secs: Option<u64>,
    /// Hard cap on source files scanned per run. Refuses to run when the
    /// workspace exceeds this count — prevents memory blowups on large
    /// monorepos (a 8k-file scan previously OOM-killed the host).
    #[serde(default)]
    max_source_files: Option<usize>,
    /// Receipt payload for receipt-verify. Carried in the request because
    /// main() consumes stdin once before dispatching.
    #[serde(default)]
    receipt: Option<serde_json::Value>,
}

impl Default for BridgeRequest {
    fn default() -> Self {
        Self {
            workspace_path: None,
            memory_dir: None,
            forge_db: None,
            namespace: None,
            max_iterations: None,
            time_budget_secs: None,
            max_source_files: None,
            receipt: None,
        }
    }
}

/// Default workspace scale guard: refuse to scan more than this many source
/// files in one run. forge-pilot's chunking + symbol extraction is memory-
/// hungry; ~2k files is a safe ceiling for a single bounded run.
const DEFAULT_MAX_SOURCE_FILES: usize = 2_000;

/// Count source files under a workspace WITHOUT loading them, so the guard
/// can reject oversized workspaces before any memory-heavy work begins.
fn count_source_files(root: &Path) -> usize {
    let mut count = 0usize;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name == ".git" || name == "target" || name == "node_modules" {
                    continue;
                }
                stack.push(path);
            } else if path.extension().is_some_and(|ext| {
                matches!(
                    ext.to_str(),
                    Some("rs" | "py" | "ts" | "tsx" | "js" | "jsx" | "go" | "c" | "h" | "cpp"
                        | "hpp" | "toml" | "json" | "yaml" | "yml" | "md")
                )
            }) {
                count += 1;
            }
            if count > DEFAULT_MAX_SOURCE_FILES {
                return count;
            }
        }
    }
    count
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: pilot-bridge <command> [--workspace <path>] [--memory-dir <path>] [--forge-db <path>] [--namespace <ns>]");
        eprintln!("commands: status, observe, bootstrap, evaluate, receipt-verify");
        std::process::exit(1);
    }
    let command = args[1].clone();
    let request: BridgeRequest = match read_stdin_json() {
        Ok(serde_json::Value::Null) => BridgeRequest::default(),
        Ok(value) => match serde_json::from_value(value) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("error: invalid request JSON: {e}");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let result = rt.block_on(async { dispatch(&command, &request).await });

    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

async fn dispatch(command: &str, request: &BridgeRequest) -> Result<(), String> {
    match command {
        "status" => cmd_status(request),
        "observe" => cmd_observe(request).await,
        "bootstrap" => cmd_bootstrap(request).await,
        "evaluate" => cmd_evaluate(request).await,
        "receipt-verify" => cmd_receipt_verify(request),
        _ => Err(format!("unknown command: {command}")),
    }
}

fn default_paths(request: &BridgeRequest) -> (String, String, String, String) {
    let workspace = request
        .workspace_path
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).display().to_string());
    let memory_dir = request
        .memory_dir
        .clone()
        .unwrap_or_else(|| default_memory_dir());
    let forge_db = request
        .forge_db
        .clone()
        .unwrap_or_else(|| default_forge_db());
    let namespace = request.namespace.clone().unwrap_or_else(|| "default".to_string());
    (workspace, memory_dir, forge_db, namespace)
}

fn default_memory_dir() -> String {
    std::env::var("PILOT_MEMORY_DIR")
        .unwrap_or_else(|_| format!("{}/.recall/pilot-memory", std::env::var("HOME").unwrap_or_else(|_| ".".into())))
}

fn default_forge_db() -> String {
    std::env::var("PILOT_FORGE_DB")
        .unwrap_or_else(|_| format!("{}/.recall/forge/forge.db", std::env::var("HOME").unwrap_or_else(|_| ".".into())))
}

fn build_config(request: &BridgeRequest) -> LoopConfig {
    let (workspace, memory_dir, forge_db, namespace) = default_paths(request);
    let scope = Scope::new(namespace);
    let mut config = LoopConfig::default_for_scope(scope.clone(), workspace);
    config.memory_dir = memory_dir;
    config.forge_db_path = forge_db;
    config.max_iterations = request.max_iterations.unwrap_or(1);
    if let Some(budget) = request.time_budget_secs {
        config.time_budget_secs = budget;
    }
    config.runtime_config = RuntimeConfig {
        default_scope: scope,
        ..config.runtime_config.clone()
    };
    config
}

fn open_resources(config: &LoopConfig) -> Result<LoopRunnerResources, String> {
    let memory_dir = Path::new(&config.memory_dir);
    std::fs::create_dir_all(memory_dir)
        .map_err(|e| format!("memory dir {} could not be created: {e}", memory_dir.display()))?;
    let memory_config = MemoryConfig {
        base_dir: memory_dir.to_path_buf(),
        ..Default::default()
    };
    let embedder = Box::new(MockEmbedder::new(memory_config.embedding.dimensions));
    let memory_store = MemoryStore::open_with_embedder(memory_config, embedder)
        .map_err(|e| format!("memory store could not be opened: {e}"))?;

    let forge_db = Path::new(&config.forge_db_path);
    if let Some(parent) = forge_db.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let forge_store = ForgeStore::open(forge_db).map_err(|e| format!("forge db could not be opened: {e}"))?;

    LoopRunnerResources::from_memory_store(memory_store, forge_store, config.runtime_config.clone())
        .map_err(|e| e.to_string())
}

fn cmd_status(request: &BridgeRequest) -> Result<(), String> {
    let config = build_config(request);
    let paths = forge_pilot::inspect_observation_paths(&config);
    let (workspace, memory_dir, forge_db, namespace) = default_paths(request);

    let forge_state = if Path::new(&forge_db).exists() { "present" } else { "missing" };
    let memory_state = if Path::new(&memory_dir).exists() { "present" } else { "missing" };

    write_stdout_json(&serde_json::json!({
        "status": "ok",
        "workspace_path": workspace,
        "memory_dir": memory_dir,
        "memory_dir_state": memory_state,
        "forge_db": forge_db,
        "forge_db_state": forge_state,
        "namespace": namespace,
        "observation_paths": {
            "memory_dir_state": format!("{:?}", paths.memory_dir_state),
            "forge_db_state": format!("{:?}", paths.forge_db_state),
        },
        "evidence_kind": EVIDENCE_KIND,
        "causal_claim": true,
    }))
}

/// Scale guard shared by scan-heavy commands (bootstrap, observe).
///
/// Returns `Ok(())` when the workspace is within the configured file cap,
/// or a descriptive refusal before any memory-heavy work begins.
fn guard_workspace_scale(request: &BridgeRequest, workspace: &str) -> Result<(), String> {
    let cap = request.max_source_files.unwrap_or(DEFAULT_MAX_SOURCE_FILES);
    let root = Path::new(workspace);
    if !root.is_dir() {
        return Err(format!("workspace path is not a directory: {workspace}"));
    }
    let count = count_source_files(root);
    if count > cap {
        return Err(format!(
            "workspace scale guard refused: {count} source files exceeds the configured cap of {cap}. \
             pilot-bridge is bounded-run tooling; point it at a smaller workspace (e.g. one crate) \
             or raise max_source_files explicitly. Large monorepo scans can exhaust host memory."
        ));
    }
    Ok(())
}

async fn cmd_observe(request: &BridgeRequest) -> Result<(), String> {
    let (workspace, ..) = default_paths(request);
    guard_workspace_scale(request, &workspace)?;
    let config = build_config(request);
    let resources = open_resources(&config)?;
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .map_err(|e| e.to_string())?;
    write_stdout_json(&serde_json::json!({
        "status": "ok",
        "observation": observation,
        "evidence_kind": EVIDENCE_KIND,
        "causal_claim": true,
    }))
}

async fn cmd_bootstrap(request: &BridgeRequest) -> Result<(), String> {
    let (workspace, ..) = default_paths(request);
    guard_workspace_scale(request, &workspace)?;
    let config = build_config(request);
    let resources = open_resources(&config)?;
    let report = bootstrap_source_workspace(&resources.memory_store, &config)
        .await
        .map_err(|e| e.to_string())?;
    write_stdout_json(&serde_json::json!({
        "status": "ok",
        "report": report,
        "evidence_kind": EVIDENCE_KIND,
        "causal_claim": true,
    }))
}

async fn cmd_evaluate(request: &BridgeRequest) -> Result<(), String> {
    let config = build_config(request);
    let resources = open_resources(&config)?;
    let mut runner = LoopRunner::new(config, resources);
    let report = runner.run().await.map_err(|e| e.to_string())?;
    write_stdout_json(&serde_json::json!({
        "status": "ok",
        "report": report,
        "evidence_kind": EVIDENCE_KIND,
        "causal_claim": true,
    }))
}

/// Verify a receipt.
///
/// Verification mode depends on the receipt's id shape:
/// - 64-hex BLAKE3 digest id → recompute the canonical digest over the JSON
///   and compare (content-integrity verification).
/// - UUID id (forge-pilot loop receipts) → structural validation: required
///   fields present, schema_version non-empty, timestamps parseable.
/// The verdict never overclaims: a structurally valid UUID receipt is
/// reported as `valid: true` with `verification_mode: "structural"`, not as
/// a digest match.
fn cmd_receipt_verify(request: &BridgeRequest) -> Result<(), String> {
    let receipt = request
        .receipt
        .clone()
        .ok_or_else(|| "receipt missing (pass {\"receipt\": {...}} on stdin)".to_string())?;

    let declared_id = receipt
        .get("receipt_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let is_digest = declared_id.len() == 64 && declared_id.bytes().all(|b| b.is_ascii_hexdigit());
    let schema_version = receipt
        .get("schema_version")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if is_digest {
        // Content-integrity mode: recompute canonical digest and compare.
        let canonical = serde_json::to_string(&receipt).map_err(|e| e.to_string())?;
        let digest = blake3::hash(canonical.as_bytes()).to_hex().to_string();
        write_stdout_json(&serde_json::json!({
            "status": "ok",
            "valid": declared_id == digest,
            "verification_mode": "digest",
            "declared_id": declared_id,
            "recomputed_digest": digest,
            "note": "digest comparison over canonical JSON; mismatch means the receipt content changed after issue",
            "evidence_kind": EVIDENCE_KIND,
            "causal_claim": true,
        }))
    } else {
        // Structural mode: required fields + timestamp parse.
        let mut problems: Vec<String> = Vec::new();
        if declared_id.is_empty() {
            problems.push("receipt_id missing".to_string());
        }
        if schema_version.is_empty() {
            problems.push("schema_version missing".to_string());
        }
        for field in ["started_at", "finished_at"] {
            if let Some(value) = receipt.get(field).and_then(|v| v.as_str()) {
                if chrono::DateTime::parse_from_rfc3339(value).is_err() {
                    problems.push(format!("{field} is not a valid RFC3339 timestamp"));
                }
            }
        }
        if receipt.get("workspace_path").is_none() {
            problems.push("workspace_path missing".to_string());
        }
        write_stdout_json(&serde_json::json!({
            "status": "ok",
            "valid": problems.is_empty(),
            "verification_mode": "structural",
            "declared_id": declared_id,
            "schema_version": schema_version,
            "problems": problems,
            "note": "UUID-bound receipt verified structurally (required fields + timestamps); no content digest is bound to this receipt id",
            "evidence_kind": EVIDENCE_KIND,
            "causal_claim": true,
        }))
    }
}

fn read_stdin_json() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    if input.trim().is_empty() {
        Ok(serde_json::Value::Null)
    } else {
        Ok(serde_json::from_str(&input)?)
    }
}

fn write_stdout_json(value: &serde_json::Value) -> Result<(), String> {
    let mut stdout = io::stdout();
    serde_json::to_writer_pretty(&mut stdout, value).map_err(|e| e.to_string())?;
    stdout.write_all(b"\n").map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_paths_resolve_without_panicking() {
        let req = BridgeRequest::default();
        let (workspace, memory_dir, forge_db, namespace) = default_paths(&req);
        assert!(!workspace.is_empty());
        assert!(memory_dir.contains("pilot-memory"));
        assert!(forge_db.contains("forge.db"));
        assert_eq!(namespace, "default");
    }

    #[test]
    fn request_parses_fields() {
        let json = r#"{"workspace_path":"/tmp/w","namespace":"coding","max_iterations":3}"#;
        let req: BridgeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.workspace_path.as_deref(), Some("/tmp/w"));
        assert_eq!(req.namespace.as_deref(), Some("coding"));
        assert_eq!(req.max_iterations, Some(3));
    }
}
