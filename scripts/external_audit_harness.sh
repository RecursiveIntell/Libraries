#!/usr/bin/env bash
# LIB-007: Minimal reproducibility harness for external reviewers.
#
# This script lets an external auditor verify the core claims in
# release/closeout_receipt_v1.json without needing the full internal
# toolchain or sibling directory structure.
#
# Prerequisites:
#   - Rust toolchain with cargo (stable channel)
#   - Python 3.8+
#   - bash 4+
#
# Usage:
#   cd /path/to/Libraries
#   bash scripts/external_audit_harness.sh
#
# The script produces a JSON report at evidence/external_audit_report.json.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

REPORT_FILE="evidence/external_audit_report.json"
PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0
RESULTS=()

log() { echo "[audit] $*"; }
record() {
    local name="$1" status="$2" detail="${3:-}"
    RESULTS+=("{\"gate\":\"${name}\",\"status\":\"${status}\",\"detail\":\"${detail}\"}")
    case "$status" in
        pass) PASS_COUNT=$((PASS_COUNT + 1)) ;;
        fail) FAIL_COUNT=$((FAIL_COUNT + 1)) ;;
        skip) SKIP_COUNT=$((SKIP_COUNT + 1)) ;;
    esac
}

# --- Check prerequisites ---
log "Checking prerequisites..."

if ! command -v cargo >/dev/null 2>&1; then
    log "ERROR: cargo not found. Install the Rust toolchain first."
    exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
    log "WARNING: python3 not found. Python gates will be skipped."
    HAS_PYTHON=false
else
    HAS_PYTHON=true
fi

CARGO_VERSION="$(cargo --version)"
log "cargo: $CARGO_VERSION"

# --- Gate 1: Workspace compiles ---
log "Gate 1/7: cargo check --workspace"
if cargo check --workspace 2>&1; then
    record "cargo_check_workspace" "pass"
    log "  PASS"
else
    record "cargo_check_workspace" "fail" "cargo check failed"
    log "  FAIL"
fi

# --- Gate 2: Workspace tests pass ---
log "Gate 2/7: cargo test --workspace (no doc tests)"
if cargo test --workspace --lib --tests 2>&1; then
    record "cargo_test_workspace" "pass"
    log "  PASS"
else
    record "cargo_test_workspace" "fail" "some tests failed"
    log "  FAIL"
fi

# --- Gate 3: No production panic shortcuts ---
log "Gate 3/7: No production panic shortcuts"
if bash scripts/check_no_prod_panics.sh 2>&1; then
    record "no_prod_panics" "pass"
    log "  PASS"
else
    record "no_prod_panics" "fail" "production panic shortcuts found"
    log "  FAIL"
fi

# --- Gate 4: Schema registry uniqueness ---
log "Gate 4/7: Schema registry uniqueness"
if bash scripts/check_schema_registry_uniqueness.sh 2>&1; then
    record "schema_uniqueness" "pass"
    log "  PASS"
else
    record "schema_uniqueness" "fail" "duplicate schema IDs"
    log "  FAIL"
fi

# --- Gate 5: Manifest truth ---
log "Gate 5/7: Manifest truth"
if bash scripts/check_manifest_truth.sh 2>&1; then
    record "manifest_truth" "pass"
    log "  PASS"
else
    record "manifest_truth" "fail" "manifest mismatch"
    log "  FAIL"
fi

# --- Gate 6: Public type drift ---
log "Gate 6/7: Public type drift check"
if [ "$HAS_PYTHON" = true ]; then
    if python3 scripts/check_public_type_drift.py 2>&1; then
        record "public_type_drift" "pass"
        log "  PASS"
    else
        record "public_type_drift" "fail" "type drift detected"
        log "  FAIL"
    fi
else
    record "public_type_drift" "skip" "python3 not available"
    log "  SKIP (no python3)"
fi

# --- Gate 7: Closeout receipt self-check ---
log "Gate 7/7: Closeout receipt self-check"
if [ "$HAS_PYTHON" = true ] && [ -f scripts/check_closeout_receipt.py ]; then
    if python3 scripts/check_closeout_receipt.py 2>&1; then
        record "closeout_receipt" "pass"
        log "  PASS"
    else
        record "closeout_receipt" "fail" "receipt validation failed"
        log "  FAIL"
    fi
else
    record "closeout_receipt" "skip" "python3 not available or script missing"
    log "  SKIP"
fi

# --- Write report ---
TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
RESULTS_JSON=$(printf '%s\n' "${RESULTS[@]}" | paste -sd ',' -)

mkdir -p evidence
cat > "$REPORT_FILE" <<ENDJSON
{
  "harness": "external_audit_harness.sh",
  "issue_ref": "LIB-007",
  "captured_at": "${TIMESTAMP}",
  "cargo_version": "${CARGO_VERSION}",
  "summary": {
    "pass": ${PASS_COUNT},
    "fail": ${FAIL_COUNT},
    "skip": ${SKIP_COUNT},
    "total": $((PASS_COUNT + FAIL_COUNT + SKIP_COUNT))
  },
  "results": [${RESULTS_JSON}]
}
ENDJSON

log ""
log "=== External Audit Summary ==="
log "Pass: $PASS_COUNT  Fail: $FAIL_COUNT  Skip: $SKIP_COUNT"
log "Report written to: $REPORT_FILE"

if [ "$FAIL_COUNT" -gt 0 ]; then
    exit 1
fi
