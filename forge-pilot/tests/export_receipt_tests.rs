//! Tests that the `governance` feature wires `claim_ledger` into the
//! canonical export path and emits an `ExportReceipt` per roundtrip.
//!
//! Closes P1-2 from the V30 hostile audit (claim-ledger not wired into
//! forge-pilot).

#![cfg(feature = "governance")]

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
