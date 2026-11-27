//! Tests for the OFFLINE, DISPOSABLE `ares-tool-observation-probe` canary.
//!
//! These drive the real binary as a subprocess. They make no claim about any
//! production transport or capability.

use forge_engine::ForgeStore;
use llm_tool_runtime::semantic_memory::ToolObservationMemory;
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder};
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_ares-tool-observation-probe");
const TOOL_NAME: &str = "ares_offline_available_parallelism";
const NAMESPACE: &str = "ares-tool-observation-offline-canary";

fn run(args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .env_remove("RUST_LOG")
        .output()
        .unwrap_or_else(|error| panic!("spawn probe binary: {error}"))
}

fn run_enabled(scope: &str, out_dir: &Path) -> Output {
    run(&[
        "--enable-offline-canary",
        "--scope",
        scope,
        "--out-dir",
        out_dir.to_str().unwrap(),
    ])
}

fn stdout_json(output: &Output) -> serde_json::Value {
    let text = String::from_utf8(output.stdout.clone()).unwrap();
    assert_eq!(text.lines().count(), 1, "exactly one metadata line: {text}");
    serde_json::from_str(text.trim()).unwrap()
}

fn assert_not_executed(output: &Output, expected_status: &str) {
    assert!(!output.status.success());
    let json = stdout_json(output);
    assert_eq!(json["status"], expected_status);
    assert_eq!(json["executed"], false);
    assert!(json.get("receipt_id").is_none());
}

#[test]
fn disabled_by_default_creates_no_directory() {
    let tmp = TempDir::new().unwrap();
    let sandbox = tmp.path().join("sandbox");

    // No opt-in flag at all.
    let output = run(&["--scope", NAMESPACE, "--out-dir", sandbox.to_str().unwrap()]);
    assert_not_executed(&output, "disabled");
    assert!(!sandbox.exists());

    // No arguments at all.
    let output = run(&[]);
    assert_not_executed(&output, "disabled");

    // Near-miss flag spellings are not opt-in.
    for flag in [
        "--enable",
        "--enable-offline-canary=1",
        "enable-offline-canary",
    ] {
        let output = run(&[
            flag,
            "--scope",
            NAMESPACE,
            "--out-dir",
            sandbox.to_str().unwrap(),
        ]);
        assert_not_executed(&output, "disabled");
        assert!(!sandbox.exists());
    }
    assert_eq!(std::fs::read_dir(tmp.path()).unwrap().count(), 0);
}

#[test]
fn wrong_or_missing_scope_creates_no_directory() {
    let tmp = TempDir::new().unwrap();
    let sandbox = tmp.path().join("sandbox");

    for scope in ["pilot", "", "ares-tool-observation-offline-canary ", "Ares"] {
        let output = run_enabled(scope, &sandbox);
        assert_not_executed(&output, "denied");
        assert!(!sandbox.exists(), "scope {scope:?} must not create a dir");
    }

    // Missing --scope.
    let output = run(&[
        "--enable-offline-canary",
        "--out-dir",
        sandbox.to_str().unwrap(),
    ]);
    assert_not_executed(&output, "denied");
    assert!(!sandbox.exists());

    // Missing --out-dir, unknown argument, and caller-supplied tool/command.
    let output = run(&["--enable-offline-canary", "--scope", NAMESPACE]);
    assert_not_executed(&output, "denied");
    let output = run(&[
        "--enable-offline-canary",
        "--scope",
        NAMESPACE,
        "--out-dir",
        sandbox.to_str().unwrap(),
        "--tool",
        "anything",
    ]);
    assert_not_executed(&output, "denied");
    assert!(!sandbox.exists());
    assert_eq!(std::fs::read_dir(tmp.path()).unwrap().count(), 0);
}

#[test]
fn missing_parent_is_denied_without_creating_parents() {
    let tmp = TempDir::new().unwrap();
    let sandbox = tmp.path().join("no-such-parent").join("sandbox");
    let output = run_enabled(NAMESPACE, &sandbox);
    assert_not_executed(&output, "denied");
    assert!(!tmp.path().join("no-such-parent").exists());
}

#[test]
fn existing_directory_or_file_is_denied_and_left_untouched() {
    let tmp = TempDir::new().unwrap();

    // Existing empty directory.
    let empty = tmp.path().join("empty");
    std::fs::create_dir(&empty).unwrap();
    let output = run_enabled(NAMESPACE, &empty);
    assert_not_executed(&output, "denied");
    assert_eq!(std::fs::read_dir(&empty).unwrap().count(), 0);

    // Existing non-empty directory.
    let nonempty = tmp.path().join("nonempty");
    std::fs::create_dir(&nonempty).unwrap();
    std::fs::write(nonempty.join("keep.txt"), b"keep").unwrap();
    let output = run_enabled(NAMESPACE, &nonempty);
    assert_not_executed(&output, "denied");
    let names: Vec<_> = std::fs::read_dir(&nonempty)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, vec![std::ffi::OsString::from("keep.txt")]);
    assert_eq!(std::fs::read(nonempty.join("keep.txt")).unwrap(), b"keep");

    // Existing regular file.
    let file = tmp.path().join("file");
    std::fs::write(&file, b"x").unwrap();
    let output = run_enabled(NAMESPACE, &file);
    assert_not_executed(&output, "denied");
    assert_eq!(std::fs::read(&file).unwrap(), b"x");
}

#[tokio::test]
async fn happy_path_reads_back_forge_receipt_and_projection_without_payload() {
    let tmp = TempDir::new().unwrap();
    let sandbox = tmp.path().join("sandbox");

    let output = run_enabled(NAMESPACE, &sandbox);
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = stdout_json(&output);
    assert_eq!(json["status"], "succeeded");
    assert_eq!(json["executed"], true);
    assert_eq!(json["tool_name"], TOOL_NAME);
    assert_eq!(json["namespace"], NAMESPACE);
    assert_eq!(json["reopened"], true);
    assert_eq!(json["retries"], 0);
    assert_eq!(json["forge_readback"]["found"], true);
    assert_eq!(json["forge_readback"]["scope_matches"], true);
    assert_eq!(json["projection_readback"]["found"], true);
    assert_eq!(json["projection_readback"]["status"], "succeeded");
    let receipt_id = json["receipt_id"].as_str().unwrap().to_owned();
    assert!(!receipt_id.is_empty());

    // Independently reopen the Forge store and read the committed receipt.
    let forge = ForgeStore::open(&sandbox.join("forge.db")).unwrap();
    let row = forge.get_tool_receipt(&receipt_id).unwrap().unwrap();
    assert_eq!(row.tool_name, TOOL_NAME);
    assert_eq!(row.observation_scope.as_deref(), Some(NAMESPACE));
    assert!(row.error_class.is_none());
    assert!(row.raw_payload_json.contains(TOOL_NAME));

    // The backend payload (value and receipt body) is never printed.
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    for text in [&stdout, &stderr] {
        assert!(!text.contains("\"available_parallelism\":"));
        assert!(!text.contains(&row.raw_payload_json));
        assert!(!text.contains(&row.output_digest_or_refs_json));
    }
    assert_eq!(json["backend_payload"], "withheld");

    // Independently reopen the memory store and read the derived projection.
    let config = MemoryConfig {
        base_dir: sandbox.join("mem"),
        ..Default::default()
    };
    let memory = MemoryStore::open_with_embedder(
        config.clone(),
        Box::new(MockEmbedder::new(config.embedding.dimensions)),
    )
    .unwrap();
    let found = memory
        .find_similar_tool_observations(TOOL_NAME, NAMESPACE, 10)
        .await
        .unwrap()
        .observations;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].receipt_id.as_deref(), Some(receipt_id.as_str()));
    assert_eq!(found[0].tool_name, TOOL_NAME);
    assert_eq!(found[0].scope, NAMESPACE);
    assert_eq!(found[0].status, "succeeded");
    assert_eq!(found[0].summary, format!("Tool {TOOL_NAME} succeeded"));
    assert!(!found[0].summary.contains("available_parallelism\":"));
    assert!(memory
        .find_similar_tool_observations(TOOL_NAME, "other", 10)
        .await
        .unwrap()
        .observations
        .is_empty());
}

#[test]
fn retry_on_existing_directory_never_re_executes() {
    let tmp = TempDir::new().unwrap();
    let sandbox = tmp.path().join("sandbox");

    let first = run_enabled(NAMESPACE, &sandbox);
    assert!(first.status.success());
    let first_json = stdout_json(&first);
    let receipt_id = first_json["receipt_id"].as_str().unwrap().to_owned();

    // Snapshot every file under the sandbox (path + bytes) after the first run.
    fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
        let mut entries = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    let rel = path.strip_prefix(root).unwrap().display().to_string();
                    entries.push((rel, std::fs::read(&path).unwrap()));
                }
            }
        }
        entries.sort();
        entries
    }
    let before = snapshot(&sandbox);
    assert!(!before.is_empty());

    for _ in 0..2 {
        let retry = run_enabled(NAMESPACE, &sandbox);
        assert_not_executed(&retry, "denied");
        let text = String::from_utf8(retry.stdout).unwrap();
        assert!(!text.contains(&receipt_id));
    }

    // Byte-for-byte unchanged: no second receipt, no rewritten store.
    assert_eq!(snapshot(&sandbox), before);
    let forge = ForgeStore::open(&sandbox.join("forge.db")).unwrap();
    let row = forge.get_tool_receipt(&receipt_id).unwrap().unwrap();
    assert_eq!(row.receipt_id, receipt_id);
}
