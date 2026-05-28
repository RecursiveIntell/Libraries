#!/usr/bin/env bash
set -euo pipefail
mkdir -p target/aidens-p20-2-audit/logs
run() {
  local name="$1"; shift
  echo "== $name =="
  ( "$@" ) 2>&1 | tee "target/aidens-p20-2-audit/logs/${name}.log"
}
run package_integrity python3 scripts/p20_2_scan_package_integrity.py .
run agency_eval_validation python3 scripts/p20_2_validate_agency_cases.py evals/p20_agency_eval_cases.jsonl
if [[ "${P20_2_REQUIRE_INTEGRATION_CRATE:-0}" == "1" ]]; then
  run testkit_purity python3 scripts/p20_2_scan_testkit_purity.py . --require-integration-crate
else
  run testkit_purity python3 scripts/p20_2_scan_testkit_purity.py . || true
fi
if command -v cargo >/dev/null 2>&1; then
  run cargo_version cargo --version
  run cargo_fmt cargo fmt --all --check
  run cargo_check cargo check --workspace --all-targets --all-features
  run cargo_test cargo test --workspace --all-targets --all-features
  run cargo_clippy cargo clippy --workspace --all-targets --all-features -- -D warnings
  if [[ -f crates/aidens-integration-tests/Cargo.toml ]]; then
    run cargo_test_integration cargo test -p aidens-integration-tests --all-targets -- --nocapture
  fi
else
  echo "cargo not found" | tee target/aidens-p20-2-audit/logs/cargo_missing.log
  if [[ "${P20_2_REQUIRE_CARGO:-0}" == "1" ]]; then
    exit 1
  fi
fi
bash scripts/p20_2_generate_audit_bundle.sh
