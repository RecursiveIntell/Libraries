#!/usr/bin/env bash
set -euo pipefail

require() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required tool: $1" >&2
    exit 2
  fi
}

run_optional_package_checks() {
  if [[ -z "${SCR_RUNTIME_PACKAGE_ZIP:-}" ]]; then
    return 0
  fi

  local package_zip="$SCR_RUNTIME_PACKAGE_ZIP"
  local package_manifest="${SCR_RUNTIME_PACKAGE_MANIFEST:-${package_zip%.zip}.manifest.json}"

  if [[ ! -f "$package_zip" ]]; then
    echo "package zip not found: $package_zip" >&2
    return 2
  fi
  if [[ ! -f "$package_manifest" ]]; then
    echo "package manifest not found: $package_manifest" >&2
    return 2
  fi

  python3 scripts/verify_archive_manifest_parity.py "$package_zip" "$package_manifest"
  python3 scripts/assert_required_archive_paths.py "$package_zip"
}

require cargo
require python3
require bash
require rg

cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings

bash scripts/generate_schemas.sh
python3 scripts/validate_strict_schemas.py
python3 scripts/assert_existing_crate_boundaries.py
python3 scripts/assert_no_stale_surfaces.py
bash scripts/assert_no_opaque_signal_scanning.sh
bash scripts/assert_no_feut_contamination.sh
bash scripts/assert_no_llm_or_network_calls.sh
bash scripts/assert_no_durable_float_scores.sh
bash scripts/assert_no_naked_decision_booleans.sh
bash scripts/assert_no_shadow_truth.sh
bash scripts/assert_no_unexplained_golden_changes.sh

cargo run -p scr-cli -- verify-fixtures fixtures/audit/cases fixtures/audit/expected policies/audit_policy_v1.toml

run_optional_package_checks

echo "ok p31 completion checks"
