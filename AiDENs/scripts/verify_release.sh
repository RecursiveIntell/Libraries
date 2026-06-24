#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

if [[ ! -f Cargo.toml && -f AiDENs/Cargo.toml ]]; then
  echo "[verify-release] detected archive root; entering AiDENs/"
  cd AiDENs
fi

RUN_ID="${RUN_ID:-P32}"
LOG_DIR="${AIDENS_VERIFY_LOG_DIR:-target/verify-release/$RUN_ID}"
mkdir -p "$LOG_DIR"

run() {
  local name="$1"; shift
  echo "[verify-release] $name: $*"
  set +e
  "$@" >"$LOG_DIR/${name}.stdout.log" 2>"$LOG_DIR/${name}.stderr.log"
  local code=$?
  set -e
  if [[ "$code" -ne 0 ]]; then
    echo "[verify-release] FAIL $name exit=$code" >&2
    echo "--- stdout tail ---" >&2
    tail -80 "$LOG_DIR/${name}.stdout.log" >&2 || true
    echo "--- stderr tail ---" >&2
    tail -80 "$LOG_DIR/${name}.stderr.log" >&2 || true
    return "$code"
  fi
  echo "[verify-release] PASS $name"
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
run p30_guard python3 scripts/p30_guard.py --repo .
run no_scaffold_promoted bash scripts/assert_no_scaffold_promoted.sh
run phase_gate_integrity python3 scripts/assert_phase_gate_integrity.py
run phase19_high_risk_quarantine python3 scripts/assert_phase19_high_risk_quarantine.py
run schema_generation_scope python3 scripts/assert_schema_generation_scope.py
run script_refs_strict python3 scripts/assert_script_refs_strict.py
run sibling_workspace_layout python3 scripts/assert_sibling_workspace_layout.py
run zpy_total_contract python3 scripts/assert_zpy_total_contract.py
run aidens_capability_contract python3 scripts/assert_aidens_capability_contract.py
run docs_match_cargo bash scripts/assert_docs_match_cargo.sh
run super_pass_docs_evidence_closure python3 scripts/assert_super_pass_docs_evidence_closure.py

if ! command -v cargo >/dev/null 2>&1; then
  echo "[verify-release] BLOCKER: cargo not found" >&2
  exit 2
fi

run cargo_metadata cargo metadata --locked --format-version 1
run cargo_fmt cargo fmt --all --check
run cargo_check cargo check --workspace --locked --all-targets
run cargo_test cargo test --workspace --locked --all-targets
run cargo_clippy cargo clippy --workspace --locked --all-targets -- -D warnings
run package_validation python3 scripts/assert_package_validation.py

LATEST_PACKAGE="$(ls -t target/p32/package/AiDENs-*.zip 2>/dev/null | head -1 || true)"
if [[ -z "$LATEST_PACKAGE" ]]; then
  echo "[verify-release] FAIL package_self_replay: no target/p32/package/AiDENs-*.zip" >&2
  exit 2
fi
run package_self_replay python3 scripts/assert_package_self_replay.py \
  --package "$LATEST_PACKAGE" \
  --expected-run "$RUN_ID" \
  --receipt-out "$LOG_DIR/package_self_replay_receipt.json"

python3 scripts/generate_release_gate_manifest.py --root . --run "$RUN_ID" --log-dir "$LOG_DIR" --out "$LOG_DIR/RELEASE_GATE_MANIFEST.json" >/dev/null

echo "[verify-release] PASS: all release gates passed"
