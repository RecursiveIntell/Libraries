use aidens_receipts::{CanonicalEventLog, CanonicalEventLogConfig};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn test_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "aidens-cli-repeated-process-receipts-{}-{nonce}",
        std::process::id()
    ))
}

fn run_cli(config: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aidens-cli"))
        .args([
            "run",
            "--config",
            config
                .to_str()
                .expect("test config path must be valid UTF-8"),
            "same no-tool prompt",
        ])
        .output()
        .expect("built aidens-cli binary must launch")
}

fn assert_success(label: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{label} failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn run_twice_in_separate_processes_appends_to_one_verified_canonical_chain() {
    let root = test_root();
    let receipt_root = root.join("receipts");
    let config = root.join("aidens.toml");
    std::fs::create_dir_all(&root).expect("create repeated-process fixture root");
    std::fs::write(
        &config,
        format!(
            r#"app_id = "repeated-process-no-tool"
memory_mode = "disabled"
receipt_level = "standard"

[provider]
kind = "mock"
mock_response = "same mock response"

[receipts]
store_root = "{}"
"#,
            receipt_root.display()
        ),
    )
    .expect("write repeated-process mock config");

    let first = run_cli(&config);
    assert_success("first aidens-cli process", &first);

    let second = run_cli(&config);
    assert_success("second aidens-cli process", &second);

    let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&receipt_root))
        .expect("open shared canonical receipt log");
    assert!(log.verify_chain().expect("verify shared canonical chain"));
    let records = log.list_records().expect("list shared canonical records");
    assert_eq!(
        records.len(),
        8,
        "each run must append four durable records"
    );
    let receipt_ids = records
        .iter()
        .map(|record| record.receipt_id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(receipt_ids.len(), records.len());
    let run_receipt_ids = records
        .iter()
        .filter(|record| record.schema_name == "run-report-v1")
        .map(|record| record.receipt_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(run_receipt_ids.len(), 2);
    assert_ne!(run_receipt_ids[0], run_receipt_ids[1]);
    let persisted = serde_json::to_string(&records).expect("serialize canonical records");
    assert!(!persisted.contains("local-process-seq"));

    std::fs::remove_dir_all(&root).expect("remove repeated-process fixture root");
}
