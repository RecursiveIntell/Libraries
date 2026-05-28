#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

# Support both real repo root and archive root where AiDENs/ is nested.
if [[ ! -f Cargo.toml && -f AiDENs/Cargo.toml ]]; then
  echo "[verify-current] detected archive root; entering AiDENs/"
  cd AiDENs
fi

mkdir -p handoffs
LOG_DIR="handoffs/p31a_verify_logs"
mkdir -p "$LOG_DIR"

run() {
  local name="$1"; shift
  echo "[verify-current] $name: $*"
  set +e
  "$@" >"$LOG_DIR/${name}.stdout.log" 2>"$LOG_DIR/${name}.stderr.log"
  local code=$?
  set -e
  if [[ "$code" -ne 0 ]]; then
    echo "[verify-current] FAIL $name exit=$code" >&2
    echo "--- stdout tail ---" >&2
    tail -80 "$LOG_DIR/${name}.stdout.log" >&2 || true
    echo "--- stderr tail ---" >&2
    tail -80 "$LOG_DIR/${name}.stderr.log" >&2 || true
    return "$code"
  fi
  echo "[verify-current] PASS $name"
}

run release_ledger_schema python3 scripts/assert_release_ledger_schema.py
run current_run_truth python3 scripts/assert_current_run_truth.py
run release_truth_consistency python3 scripts/assert_release_truth_consistency.py
run root_markdown_archive_policy python3 scripts/assert_root_markdown_archive_policy.py
run codex_artifact_classification python3 scripts/assert_codex_artifact_classification.py
run support_claims_have_evidence python3 scripts/assert_support_claims_have_evidence.py

run no_fake_completion bash scripts/assert_no_fake_completion.sh .
run no_shadow_truth bash scripts/assert_no_shadow_truth.sh
run adapter_delegation bash scripts/assert_adapter_delegation.sh
run tool_runtime_delegation bash scripts/assert_tool_runtime_delegation.sh
run no_canonical_type_duplicates python3 scripts/assert_no_canonical_type_duplicates.py
run no_local_substitute_dependencies bash scripts/assert_no_local_substitute_dependencies.sh
run p30_guard_fail_broad python3 scripts/p30_guard.py --repo . --fail-broad

if ! command -v cargo >/dev/null 2>&1; then
  echo "[verify-current] BLOCKER: cargo not found; build_certified must remain false" >&2
  exit 2
fi

run cargo_metadata cargo metadata --locked --format-version 1
run cargo_fmt cargo fmt --all --check
run cargo_check cargo check --workspace --locked --all-targets
run cargo_test cargo test --workspace --locked --all-targets
run cargo_clippy cargo clippy --workspace --locked --all-targets -- -D warnings

echo "[verify-current] PASS: release truth, classification, invariants, and build gates passed"
