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
if [[ "${P20_2_RELEASE_REQUIRE_CARGO:-0}" == "1" ]]; then
  P20_2_REQUIRE_CARGO=1 bash scripts/p20_2_verify.sh
fi
