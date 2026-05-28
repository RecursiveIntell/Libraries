#!/usr/bin/env bash
set -euo pipefail
mkdir -p target/p20-1
python3 scripts/p20_1_hard_code_audit.py --out target/p20-1/hard-code-audit.json --markdown target/p20-1/hard-code-audit.md --fail-on-blocking
python3 scripts/p20_validate_agency_cases.py evals/p20_agency_eval_cases.jsonl
python3 scripts/p20_1_validate_archive_manifest.py --root .
if command -v cargo >/dev/null 2>&1; then
  cargo fmt --all --check
  cargo check --workspace --all-targets --all-features
  cargo test --workspace --all-targets --all-features
  cargo clippy --workspace --all-targets --all-features -- -D warnings
else
  echo "cargo unavailable; cargo gates were NOT run in this environment" >&2
  if [ "${P20_1_REQUIRE_CARGO:-0}" = "1" ]; then
    exit 2
  fi
fi
