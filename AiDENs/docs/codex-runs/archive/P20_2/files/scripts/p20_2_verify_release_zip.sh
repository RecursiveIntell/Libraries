#!/usr/bin/env bash
set -euo pipefail
zip_path="${1:?usage: scripts/p20_2_verify_release_zip.sh path/to/release.zip}"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
unzip -q "$zip_path" -d "$tmp"
root="$tmp"
# If the zip contains one top-level directory, use it.
count=$(find "$tmp" -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d ' ')
if [[ "$count" == "1" ]]; then
  root=$(find "$tmp" -mindepth 1 -maxdepth 1 -type d | head -1)
fi
cd "$root"
python3 scripts/p20_2_scan_package_integrity.py .
python3 scripts/p20_2_validate_agency_cases.py evals/p20_agency_eval_cases.jsonl
python3 scripts/p20_2_scan_testkit_purity.py . --require-integration-crate
python3 scripts/p20_scan_aidens.py --root . --out target/aidens-p20-2-audit/p20-scan --require-phase-reports-through 0 --aidens-overlay-only --fail-on-blocking
bash scripts/assert_no_shadow_truth.sh
bash scripts/assert_no_fake_completion.sh .
python3 scripts/p20_2_scanner_selftest.py
if [[ "${P20_2_RELEASE_REQUIRE_CARGO:-0}" == "1" ]]; then
  P20_2_REQUIRE_CARGO=1 bash scripts/p20_2_verify.sh
fi
echo "P20.2 release zip package checks passed: $zip_path"
