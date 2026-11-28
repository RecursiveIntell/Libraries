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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
                    Some(
                        "rs" | "py"
                            | "ts"
                            | "tsx"
                            | "js"
                            | "jsx"
                            | "go"
                            | "c"
                            | "h"
                            | "cpp"
                            | "hpp"
                            | "toml"
                            | "json"
                            | "yaml"
                            | "yml"
                            | "md"
                    )
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
    let workspace = request.workspace_path.clone().unwrap_or_else(|| {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .display()
            .to_string()
    });
    let memory_dir = request
        .memory_dir
        .clone()
        .unwrap_or_else(default_memory_dir);
    let forge_db = request.forge_db.clone().unwrap_or_else(default_forge_db);
    let namespace = request
        .namespace
        .clone()
        .unwrap_or_else(|| "default".to_string());
    (workspace, memory_dir, forge_db, namespace)
}

fn default_memory_dir() -> String {
    std::env::var("PILOT_MEMORY_DIR").unwrap_or_else(|_| {
        format!(
            "{}/.recall/pilot-memory",
            std::env::var("HOME").unwrap_or_else(|_| ".".into())
        )
    })
}

fn default_forge_db() -> String {
    std::env::var("PILOT_FORGE_DB").unwrap_or_else(|_| {
        format!(
            "{}/.recall/forge/forge.db",
            std::env::var("HOME").unwrap_or_else(|_| ".".into())
        )
    })
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
    std::fs::create_dir_all(memory_dir).map_err(|e| {
        format!(
            "memory dir {} could not be created: {e}",
            memory_dir.display()
        )
    })?;
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
    let forge_store =
        ForgeStore::open(forge_db).map_err(|e| format!("forge db could not be opened: {e}"))?;

    LoopRunnerResources::from_memory_store(memory_store, forge_store, config.runtime_config.clone())
        .map_err(|e| e.to_string())
}

fn cmd_status(request: &BridgeRequest) -> Result<(), String> {
    let config = build_config(request);
    let paths = forge_pilot::inspect_observation_paths(&config);
    let (workspace, memory_dir, forge_db, namespace) = default_paths(request);

    let forge_state = if Path::new(&forge_db).exists() {
        "present"
    } else {
        "missing"
    };
    let memory_state = if Path::new(&memory_dir).exists() {
        "present"
    } else {
        "missing"
    };

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

/// Name of the digest contract implemented by [`receipt_content_digest`].
const RECEIPT_DIGEST_CONTRACT: &str = "pilot-bridge.receipt-digest.v1";

/// Deterministic JSON text: object keys sorted, no insignificant whitespace.
///
/// Sorting is done here so the result does not depend on whether
/// `serde_json`'s `preserve_order` feature is enabled somewhere in the build.
fn canonical_json(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut out = String::from("{");
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::Value::String((*key).clone()).to_string());
                out.push(':');
                if let Some(child) = map.get(key.as_str()) {
                    out.push_str(&canonical_json(child));
                }
            }
            out.push('}');
            out
        }
        serde_json::Value::Array(items) => {
            let mut out = String::from("[");
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(&canonical_json(item));
            }
            out.push(']');
            out
        }
        other => other.to_string(),
    }
}

/// Content digest of a receipt under [`RECEIPT_DIGEST_CONTRACT`].
///
/// BLAKE3 over: the contract name, one newline, then the canonical JSON of the
/// receipt with its `receipt_id` field removed. The id is excluded because it
/// is the value being checked; hashing it would make a match impossible.
fn receipt_content_digest(receipt: &serde_json::Value) -> String {
    let mut content = receipt.clone();
    if let Some(map) = content.as_object_mut() {
        map.remove("receipt_id");
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(RECEIPT_DIGEST_CONTRACT.as_bytes());
    hasher.update(b"\n");
    hasher.update(canonical_json(&content).as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// Build the verdict for a receipt. Pure: performs no I/O.
///
/// Both modes run the structural checks (required fields present, timestamps
/// parseable) and report them as `well_formed` and `problems`.
///
/// - 64-hex id: content-integrity mode. `integrity_verified` is true only when
///   the id equals the recomputed content digest; `valid` requires that and
///   `well_formed`.
/// - any other id (for example a UUID): no digest is bound to the receipt, so
///   `integrity_verified` is false and `valid` is the structural result.
///
/// `receipt-verify` checks integrity, not causality, so it never sets
/// `causal_claim`.
fn receipt_verdict(receipt: &serde_json::Value) -> serde_json::Value {
    let declared_id = receipt
        .get("receipt_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let schema_version = receipt
        .get("schema_version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let is_digest = declared_id.len() == 64 && declared_id.bytes().all(|b| b.is_ascii_hexdigit());

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
    let well_formed = problems.is_empty();

    if is_digest {
        let digest = receipt_content_digest(receipt);
        let matches = declared_id.eq_ignore_ascii_case(&digest);
        serde_json::json!({
            "status": "ok",
            "valid": matches && well_formed,
            "integrity_verified": matches,
            "well_formed": well_formed,
            "verification_mode": "digest",
            "digest_contract": RECEIPT_DIGEST_CONTRACT,
            "declared_id": declared_id,
            "recomputed_digest": digest,
            "schema_version": schema_version,
            "problems": problems,
            "note": "integrity_verified means receipt_id equals the unkeyed digest of this content (canonical JSON without receipt_id); it does not show who issued the receipt; valid also requires well_formed",
            "evidence_kind": EVIDENCE_KIND,
            "causal_claim": false,
        })
    } else {
        serde_json::json!({
            "status": "ok",
            "valid": well_formed,
            "integrity_verified": false,
            "well_formed": well_formed,
            "verification_mode": "structural",
            "declared_id": declared_id,
            "schema_version": schema_version,
            "problems": problems,
            "note": "receipt id is not a content digest, so content integrity cannot be verified; well_formed covers required fields and timestamps only",
            "evidence_kind": EVIDENCE_KIND,
            "causal_claim": false,
        })
    }
}

/// Verify a receipt read from the request and print the verdict.
fn cmd_receipt_verify(request: &BridgeRequest) -> Result<(), String> {
    let receipt = request
        .receipt
        .clone()
        .ok_or_else(|| "receipt missing (pass {\"receipt\": {...}} on stdin)".to_string())?;
    write_stdout_json(&receipt_verdict(&receipt))
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

    fn with_digest_id(mut receipt: serde_json::Value) -> serde_json::Value {
        let digest = receipt_content_digest(&receipt);
        receipt["receipt_id"] = serde_json::Value::String(digest);
        receipt
    }

    #[test]
    fn receipt_digest_matches_the_pinned_v1_vector() {
        // Independently computed: BLAKE3("pilot-bridge.receipt-digest.v1\n" +
        // "{\"schema_version\":\"v1\",\"workspace_path\":\"/x\"}").
        let receipt = serde_json::json!({"workspace_path": "/x", "schema_version": "v1"});
        assert_eq!(
            receipt_content_digest(&receipt),
            "268349e976a6bf1a8e40c063846bc6c72555d8bd9accf8270c3284a7a624dbcd"
        );
    }

    #[test]
    fn receipt_digest_ignores_key_order_and_the_id_itself() {
        let a: serde_json::Value =
            serde_json::from_str(r#"{"b":{"y":1,"x":2},"a":[1,2],"receipt_id":"ignored"}"#)
                .unwrap();
        let b: serde_json::Value =
            serde_json::from_str(r#"{"a":[1,2],"b":{"x":2,"y":1}}"#).unwrap();
        assert_eq!(receipt_content_digest(&a), receipt_content_digest(&b));
        assert_eq!(canonical_json(&b), r#"{"a":[1,2],"b":{"x":2,"y":1}}"#);
    }

    #[test]
    fn digest_mode_accepts_a_receipt_whose_id_is_its_content_digest() {
        let receipt = with_digest_id(serde_json::json!({
            "schema_version": "v1", "workspace_path": "/x", "payload": "hello"
        }));
        let verdict = receipt_verdict(&receipt);
        assert_eq!(verdict["verification_mode"], "digest");
        assert_eq!(verdict["valid"], true);
        assert_eq!(verdict["integrity_verified"], true);
        assert_eq!(verdict["digest_contract"], RECEIPT_DIGEST_CONTRACT);
    }

    #[test]
    fn digest_mode_requires_a_well_formed_receipt_for_valid() {
        let receipt = with_digest_id(serde_json::json!({"payload": "hello"}));
        let verdict = receipt_verdict(&receipt);
        assert_eq!(verdict["verification_mode"], "digest");
        assert_eq!(verdict["integrity_verified"], true);
        assert_eq!(verdict["well_formed"], false);
        assert_eq!(verdict["valid"], false);
        assert_eq!(
            verdict["problems"],
            serde_json::json!(["schema_version missing", "workspace_path missing"])
        );
    }

    #[test]
    fn digest_mode_rejects_a_receipt_whose_content_changed() {
        let mut receipt = with_digest_id(serde_json::json!({
            "schema_version": "v1", "workspace_path": "/x", "payload": "hello"
        }));
        receipt["payload"] = serde_json::Value::String("tampered".to_string());
        let verdict = receipt_verdict(&receipt);
        assert_eq!(verdict["verification_mode"], "digest");
        assert_eq!(verdict["valid"], false);
        assert_eq!(verdict["integrity_verified"], false);
    }

    #[test]
    fn structural_mode_never_reports_integrity_or_a_causal_claim() {
        let fabricated = serde_json::json!({
            "receipt_id": "not-a-digest", "schema_version": "v1", "workspace_path": "/x"
        });
        let verdict = receipt_verdict(&fabricated);
        assert_eq!(verdict["verification_mode"], "structural");
        assert_eq!(verdict["well_formed"], true);
        assert_eq!(verdict["valid"], true);
        assert_eq!(verdict["integrity_verified"], false);
        assert_eq!(verdict["causal_claim"], false);
        let incomplete = serde_json::json!({"receipt_id": "not-a-digest", "schema_version": "v1"});
        let verdict = receipt_verdict(&incomplete);
        assert_eq!(verdict["well_formed"], false);
        assert_eq!(verdict["valid"], false);
        assert_eq!(verdict["integrity_verified"], false);
    }

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
