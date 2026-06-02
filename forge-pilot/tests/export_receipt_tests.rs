//! Tests that the `governance` feature wires `claim_ledger` into the
//! canonical export path and emits an `ExportReceipt` per roundtrip —
//! including failure paths.
//!
//! Closes P1-2 from the V30 hostile audit (claim-ledger not wired into
//! forge-pilot).

#![cfg(feature = "governance")]
#![allow(clippy::expect_used)] // test code — expect() on Result/Option is the idiomatic pattern

mod common;

use common::{open_forge_store, open_memory_store, sample_bundle, tempdir};
use forge_pilot::canonical_roundtrip;
use knowledge_runtime::Scope;

#[tokio::test]
async fn canonical_roundtrip_emits_export_receipt() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-export-receipt");

    let bundle = sample_bundle("export-receipt-bundle");
    let roundtrip = canonical_roundtrip(&bundle, &scope.namespace, &forge_store, &memory_store)
        .await
        .expect("canonical_roundtrip must succeed");

    assert_eq!(roundtrip.import_result.status, "complete");

    // The whole point of the wiring: when governance is enabled,
    // the roundtrip must produce a claim_ledger::ExportReceipt that
    // records the bundle id, envelope id, and success status.
    let receipt = roundtrip
        .export_receipt
        .expect("governance feature must emit export_receipt");
    assert_eq!(receipt.operation, "forge_pilot_canonical_roundtrip");
    assert_eq!(receipt.status, "success");
    assert_eq!(receipt.receipt_version, "ExportReceiptV1");
    assert!(!receipt.export_receipt_id.is_empty());
    assert!(receipt.output_ref.is_some(), "output must be bound");
    assert!(receipt.output_digest.is_some(), "output digest must be bound");
    assert!(
        receipt.input_digests.contains_key("bundle"),
        "bundle digest must be recorded"
    );
}

#[tokio::test]
async fn canonical_roundtrip_receipt_id_is_stable_across_invocations() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-export-receipt-determinism");

    let bundle = sample_bundle("export-receipt-determinism-bundle");
    let r1 = canonical_roundtrip(&bundle, &scope.namespace, &forge_store, &memory_store)
        .await
        .unwrap();
    let r2 = canonical_roundtrip(&bundle, &scope.namespace, &forge_store, &memory_store)
        .await
        .unwrap();

    // The export_receipt_id is derived from (operation, input_refs) via
    // claim_ledger::ids::export_receipt_id — both are stable for the
    // same bundle id. So re-running must produce the same receipt id,
    // even though the underlying envelope re-serializes with a fresh
    // `exported_at` timestamp (which is why we don't assert on
    // `input_digests.get("bundle")`).
    let r1_receipt = r1.export_receipt.expect("first roundtrip emits receipt");
    let r2_receipt = r2.export_receipt.expect("second roundtrip emits receipt");
    assert_eq!(r1_receipt.export_receipt_id, r2_receipt.export_receipt_id);
    assert_eq!(r1_receipt.operation, r2_receipt.operation);
    assert_eq!(r1_receipt.status, r2_receipt.status);
}

#[tokio::test]
async fn canonical_roundtrip_persists_failure_in_import_log() {
    // Drop the `claim_versions` table so the projection import step
    // fails. This mirrors the failure pattern in
    // `loop_roundtrip_tests.rs::canonical_roundtrip_records_durable_failure_receipt_when_import_breaks`.
    // The doctrinal assertion: even when the import fails, the
    // underlying failure must be persisted to the import log so an
    // operator can audit the failure. The receipt is the in-process
    // audit handle; the import log is the durable on-disk record.
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-export-receipt-failure");

    let conn = rusqlite::Connection::open(dir.path().join("memory.db")).unwrap();
    conn.execute_batch("DROP TABLE claim_versions;").unwrap();

    let err = canonical_roundtrip(
        &sample_bundle("export-receipt-failure-bundle"),
        &scope.namespace,
        &forge_store,
        &memory_store,
    )
    .await
    .expect_err("import must fail when claim_versions table is missing");
    assert!(
        matches!(err, forge_pilot::PilotError::Memory(_)),
        "expected a Memory error, got {err:?}"
    );

    let failures = memory_store
        .query_projection_import_failures(Some(&scope.namespace), 8)
        .await
        .unwrap();
    assert_eq!(
        failures.len(),
        1,
        "the failure must be persisted in the import log"
    );
    assert!(failures[0]
        .error_message
        .contains("no such table: claim_versions"));
}

#[tokio::test]
async fn canonical_roundtrip_receipt_status_is_success_after_clean_import() {
    // The pending-then-success state machine: receipts start in
    // `pending` and become `success` only after the import step
    // completes cleanly. A consumer reading the receipt after a
    // clean run must see `status = "success"`, not `pending`.
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-export-receipt-success-state");

    let roundtrip = canonical_roundtrip(
        &sample_bundle("export-receipt-success-state-bundle"),
        &scope.namespace,
        &forge_store,
        &memory_store,
    )
    .await
    .expect("clean run must succeed");

    let receipt = roundtrip
        .export_receipt
        .expect("governance feature must emit export_receipt");
    assert_eq!(
        receipt.status, "success",
        "after a clean import, the receipt status must be `success` (not `pending`)"
    );
    assert!(receipt.output_ref.is_some(), "output must be bound");
}
