#!/usr/bin/env bash
set -euo pipefail

PHASE="${1:-manual}"
ROOT="$(pwd)"
EVIDENCE_DIR="$ROOT/.codex_evidence/contract_ownership/$PHASE"
mkdir -p "$EVIDENCE_DIR"

run_capture() {
  local name="$1"
  shift
  {
    echo "COMMAND: $*"
    echo "WORKING_DIRECTORY: $ROOT"
    echo "START_TIME: $(date -Is)"
    set +e
    "$@"
    local status=$?
    set -e
    echo "EXIT_STATUS: $status"
    echo "END_TIME: $(date -Is)"
    return "$status"
  } 2>&1 | tee "$EVIDENCE_DIR/${name}.txt"
}

run_gate() {
  local name="$1"
  shift
  run_capture "$name" "$@" || FAIL=1
}

run_phase_01_expected_duplicates() {
  python3 scripts/make_type_ownership_inventory.py
  python3 - <<'PY'
import csv
import sys
from pathlib import Path

path = Path("docs/contract-ownership/CANONICAL_DUPLICATE_FINDINGS.csv")
expected = {
    "AttestationEnvelopeV1",
    "SharedDispositionV1",
    "SettlementCaseV1",
    "TheoryRefuterSuiteV1",
    "TheoryVersionV1",
    "HypothesisLibraryV1",
}

if not path.exists():
    print(f"FAIL: {path} missing")
    sys.exit(1)

rows = list(csv.DictReader(path.open()))
found = {row["type_name"] for row in rows if row.get("severity") == "P0"}
missing = sorted(expected - found)
if missing:
    print("FAIL: generated duplicate gate did not detect expected P0 duplicates:")
    for name in missing:
        print(f"- {name}")
    sys.exit(1)

print("PASS: generated ownership inventory detected all expected P0 duplicates.")
for row in rows:
    if row.get("type_name") in expected:
        print(
            f"{row['severity']}: {row['type_name']} local "
            f"{row['aidens_file']}:{row['aidens_line']} duplicates "
            f"{row['canonical_owner']} {row['canonical_file']}:{row['canonical_line']}"
        )
PY
}

git status --short > "$EVIDENCE_DIR/git_status_before_gate.txt" || true

FAIL=0

case "$PHASE" in
  00)
    run_gate assert_no_crate_split bash scripts/assert_no_crate_split.sh
    run_gate assert_docs_source_basis_current bash scripts/assert_docs_source_basis_current.sh
    ;;
  01)
    run_gate phase_01_expected_duplicates run_phase_01_expected_duplicates
    ;;
  02)
    run_gate assert_no_canonical_type_duplicates python3 scripts/assert_no_canonical_type_duplicates.py
    run_gate assert_no_compatibility_ledgers bash scripts/assert_no_compatibility_ledgers.sh
    ;;
  03)
    run_gate assert_no_canonical_type_duplicates python3 scripts/assert_no_canonical_type_duplicates.py
    run_gate assert_no_local_substitute_dependencies bash scripts/assert_no_local_substitute_dependencies.sh
    ;;
  04)
    run_gate assert_no_local_canonical_digest_law bash scripts/assert_no_local_canonical_digest_law.sh
    ;;
  05)
    run_gate assert_schema_generation_scope python3 scripts/assert_schema_generation_scope.py
    ;;
  06)
    run_gate assert_tool_runtime_delegation bash scripts/assert_tool_runtime_delegation.sh
    run_gate assert_wrapper_backpointers bash scripts/assert_wrapper_backpointers.sh
    ;;
  07|final|manual)
    run_gate assert_no_crate_split bash scripts/assert_no_crate_split.sh
    run_gate assert_docs_source_basis_current bash scripts/assert_docs_source_basis_current.sh
    run_gate assert_no_canonical_type_duplicates python3 scripts/assert_no_canonical_type_duplicates.py
    run_gate assert_no_local_canonical_digest_law bash scripts/assert_no_local_canonical_digest_law.sh
    run_gate assert_schema_generation_scope python3 scripts/assert_schema_generation_scope.py
    run_gate assert_tool_runtime_delegation bash scripts/assert_tool_runtime_delegation.sh
    run_gate assert_wrapper_backpointers bash scripts/assert_wrapper_backpointers.sh
    run_gate assert_no_compatibility_ledgers bash scripts/assert_no_compatibility_ledgers.sh
    run_gate assert_no_local_substitute_dependencies bash scripts/assert_no_local_substitute_dependencies.sh
    ;;
  *)
    echo "error: unknown contract ownership phase '$PHASE'" >&2
    exit 2
    ;;
esac

git status --short > "$EVIDENCE_DIR/git_status_after_gate.txt" || true
git diff --stat > "$EVIDENCE_DIR/git_diff_stat.txt" || true
git diff --binary > "$EVIDENCE_DIR/git_diff.patch" || true

if [[ "$FAIL" -ne 0 ]]; then
  echo "FAIL: one or more contract ownership gates failed. Evidence in $EVIDENCE_DIR" >&2
  exit 1
fi

echo "PASS: contract ownership verification passed. Evidence in $EVIDENCE_DIR"
