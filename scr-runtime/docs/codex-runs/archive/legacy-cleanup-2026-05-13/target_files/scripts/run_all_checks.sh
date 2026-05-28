#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

python3 scripts/validate_schemas.py
bash scripts/verify_golden_fixtures.sh
bash scripts/assert_no_feut_contamination.sh
bash scripts/assert_no_durable_float_scores.sh
bash scripts/assert_no_naked_decision_booleans.sh
bash scripts/assert_no_shadow_truth.sh
bash scripts/assert_no_llm_or_network_calls.sh
bash scripts/assert_no_unexplained_golden_changes.sh
