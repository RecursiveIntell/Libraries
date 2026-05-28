#!/usr/bin/env python3
"""Assert Phase 19 high-risk layers are explicitly quarantined and bug status is classified."""

from pathlib import Path
import csv
import json
import sys


MANIFEST = Path("P29_STATUS_EVIDENCE_MANIFEST.json")
QUARANTINE_DOC = Path("docs/super-pass/HIGH_RISK_LAYER_QUARANTINE.md")
KNOWN_LIMITS = Path("docs/super-pass/KNOWN_LIMITATIONS_REGISTER.md")
MATRIX = Path("matrices/SUPER_PASS_BACKLOG_1020.csv")
CLAUDE_INTEGRATION = Path("06_CLAUDE_AUDIT_INTEGRATION.md")

REQUIRED_LAYER_LABELS = [
    "forge-pilot",
    "effect-runtime",
    "verification pipeline",
    "federation",
    "attestation",
    "authority-delegation",
    "recursive-kernel-core",
]

REQUIRED_OWNER_PATHS = [
    "../forge-pilot",
    "../effect-runtime",
    "../verification-policy",
    "../verification-control",
    "../verification-calibration",
    "../verification-adjudication",
    "../federated-settlement",
    "../remote-oracle-admission",
    "../mechanism-runtime",
    "../attestation-exchange",
    "../authority-delegation",
    "../recursive-kernel-core",
]

REQUIRED_PHASE19_ROWS = {
    "CLAUDE-F-015": "fixed",
    "CLAUDE-F-016": "quarantined",
}


def fail(message: str, failures: list[str]) -> None:
    failures.append(message)


def load_manifest(failures: list[str]) -> dict:
    if not MANIFEST.exists():
        fail(f"missing manifest: {MANIFEST}", failures)
        return {}
    try:
        return json.loads(MANIFEST.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"manifest is not valid JSON: {exc}", failures)
        return {}


def assert_bug_classification(manifest: dict, failures: list[str]) -> None:
    if isinstance(manifest.get("open_bugs"), list):
        fail("manifest still exposes active flat open_bugs list", failures)
    if isinstance(manifest.get("quarantines"), list):
        fail("manifest still exposes active flat quarantines list", failures)

    classification = manifest.get("audit_bug_status_classification")
    if not isinstance(classification, dict):
        fail("manifest missing audit_bug_status_classification object", failures)
        return

    expected_keys = ["fixed", "quarantined", "deferred", "open_blocking"]
    for key in expected_keys:
        if not isinstance(classification.get(key), list):
            fail(f"audit_bug_status_classification.{key} must be a list", failures)

    if classification.get("open_blocking"):
        fail("audit_bug_status_classification.open_blocking must be empty for Phase 19 exit", failures)
    if not classification.get("replaces_flat_open_bugs"):
        fail("bug status classification must declare replaces_flat_open_bugs=true", failures)
    for bug_id in [f"BUG-{number:03d}" for number in range(190, 201)]:
        if bug_id not in classification.get("quarantined", []):
            fail(f"high-risk audit bug missing from quarantined bucket: {bug_id}", failures)


def assert_layer_quarantine(manifest: dict, failures: list[str]) -> None:
    quarantine = manifest.get("high_risk_layer_quarantine")
    if not isinstance(quarantine, dict):
        fail("manifest missing high_risk_layer_quarantine object", failures)
        return
    if quarantine.get("status") != "quarantined_from_supported_local_claims":
        fail("high_risk_layer_quarantine has incorrect status", failures)

    layer_entries = quarantine.get("layers", [])
    if not isinstance(layer_entries, list):
        fail("high_risk_layer_quarantine.layers must be a list", failures)
        return
    by_surface = {
        str(entry.get("surface", "")).lower(): entry
        for entry in layer_entries
        if isinstance(entry, dict)
    }
    for label in REQUIRED_LAYER_LABELS:
        entry = by_surface.get(label)
        if not entry:
            fail(f"missing high-risk layer quarantine entry: {label}", failures)
            continue
        if entry.get("status") != "quarantined":
            fail(f"high-risk layer is not quarantined: {label}", failures)


def assert_docs_and_paths(failures: list[str]) -> None:
    doc_text = QUARANTINE_DOC.read_text(encoding="utf-8", errors="ignore").lower() if QUARANTINE_DOC.exists() else ""
    if not doc_text:
        fail(f"missing quarantine document: {QUARANTINE_DOC}", failures)
    for label in REQUIRED_LAYER_LABELS:
        if label not in doc_text:
            fail(f"quarantine document missing layer label: {label}", failures)
    if doc_text.count("quarantined") < len(REQUIRED_LAYER_LABELS):
        fail("quarantine document does not explicitly quarantine every layer", failures)

    limits_text = KNOWN_LIMITS.read_text(encoding="utf-8", errors="ignore") if KNOWN_LIMITS.exists() else ""
    for row_id in REQUIRED_PHASE19_ROWS:
        if row_id not in limits_text:
            fail(f"known limitations missing Phase 19 row: {row_id}", failures)

    integration_text = CLAUDE_INTEGRATION.read_text(encoding="utf-8", errors="ignore") if CLAUDE_INTEGRATION.exists() else ""
    for row_id, status in REQUIRED_PHASE19_ROWS.items():
        if f"|{row_id}|" not in integration_text or f"|{status}|" not in integration_text:
            fail(f"Claude integration missing status {status} for {row_id}", failures)

    for rel in REQUIRED_OWNER_PATHS:
        if not Path(rel).exists():
            fail(f"expected sibling owner path is absent: {rel}", failures)


def assert_matrix_status(failures: list[str]) -> None:
    if not MATRIX.exists():
        fail(f"missing issue matrix: {MATRIX}", failures)
        return
    with MATRIX.open(newline="") as f:
        rows = {row.get("ID"): row for row in csv.DictReader(f)}
    for row_id, status in REQUIRED_PHASE19_ROWS.items():
        row = rows.get(row_id)
        if not row:
            fail(f"matrix missing row {row_id}", failures)
        elif row.get("Status") != status:
            fail(f"matrix row {row_id} has status {row.get('Status')}, expected {status}", failures)


def main() -> int:
    failures: list[str] = []
    manifest = load_manifest(failures)
    assert_bug_classification(manifest, failures)
    assert_layer_quarantine(manifest, failures)
    assert_docs_and_paths(failures)
    assert_matrix_status(failures)

    if failures:
        print("Phase 19 high-risk quarantine FAILED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print("Phase 19 high-risk quarantine OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
