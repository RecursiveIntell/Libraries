#!/usr/bin/env bash
set -euo pipefail
ROOT="${1:-.}"
cd "$ROOT"
mkdir -p target/p21
python3 scripts/p21_scan_package_integrity.py . | tee target/p21/package_integrity.json
python3 scripts/p21_scan_source_cross_refs.py . | tee target/p21/source_cross_refs.json
if [[ -f scripts/p20_2_validate_agency_cases.py && -f evals/p20_agency_eval_cases.jsonl ]]; then
  python3 scripts/p20_2_validate_agency_cases.py evals/p20_agency_eval_cases.jsonl | tee target/p21/agency_eval_validation.log
fi
if [[ "${P21_REQUIRE_CARGO:-0}" == "1" ]]; then
  cargo fmt --all --check | tee target/p21/fmt.log
  cargo check --workspace --all-targets --all-features | tee target/p21/check.log
  cargo test --workspace --all-targets --all-features | tee target/p21/test.log
  cargo clippy --workspace --all-targets --all-features -- -D warnings | tee target/p21/clippy.log
fi
echo "P21 verify completed"
