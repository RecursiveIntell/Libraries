#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-before}"

run() {
  echo
  echo "### $*"
  "$@"
}

if ! command -v cargo >/dev/null 2>&1; then
  echo "missing required tool: cargo" >&2
  exit 2
fi

run cargo fmt --all -- --check
run cargo check --workspace --all-targets
run cargo test --workspace --all-targets
run cargo clippy --workspace --all-targets -- -D warnings

if [ -f scripts/validate_strict_schemas.py ]; then
  run python3 scripts/validate_strict_schemas.py
fi
if [ -f scripts/validate_schema_rust_parity.py ]; then
  run python3 scripts/validate_schema_rust_parity.py
fi
if [ -f scripts/scr_superpass_preflight.py ]; then
  run python3 scripts/scr_superpass_preflight.py "$MODE"
fi
if [ -f scripts/scr_superpass_static_gates.py ]; then
  run python3 scripts/scr_superpass_static_gates.py "$MODE"
fi

for f in \
  scripts/assert_no_opaque_signal_scanning.sh \
  scripts/assert_no_feut_contamination.sh \
  scripts/assert_no_llm_or_network_calls.sh \
  scripts/assert_no_durable_float_scores.sh \
  scripts/assert_no_naked_decision_booleans.sh \
  scripts/assert_no_shadow_truth.sh \
  scripts/assert_no_unexplained_golden_changes.sh
do
  if [ -f "$f" ]; then
    run bash "$f"
  fi
done

echo
echo "scr_superpass_run_all: complete"
