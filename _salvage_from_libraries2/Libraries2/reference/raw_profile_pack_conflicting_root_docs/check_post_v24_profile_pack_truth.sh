#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"

need_files=(
  "00_START_HERE.md"
  "01_EXECUTIVE_SUMMARY.md"
  "03_NO_V25_RATIONALE_AND_PROFILE_LAYER_RULE.md"
  "04_PROFILE_SUITE_OVERVIEW.md"
  "06_PROFILE_MASTER_ISSUE_MATRIX.md"
  "07_SCHEMA_REGISTRY_AND_COMPATIBILITY_PLAN.md"
  "08_CRATE_BOUNDARY_AND_OWNERSHIP_MAP.md"
  "09_EXACT_FILE_TOUCH_MAP.md"
  "10_RELEASE_BAR_AND_ACCEPTANCE.md"
  "11_PER_CRATE_APPLY_PLAN.md"
  "14_REFERENCE_INTERPRETER_AND_CONFORMANCE_PLAN.md"
  "19_APPLY_PLAN.md"
  "20_CURRENT_CODE_SNAPSHOT_NOTES_20260315.md"
  "PACK_MANIFEST.json"
)

for rel in "${need_files[@]}"; do
  test -f "$ROOT/$rel" || { echo "missing required file: $rel" >&2; exit 1; }
done

spec_count="$(find "$ROOT" -maxdepth 1 -type f -name 'CANONICAL_STACK_PROFILE_SPEC_P*.md' | wc -l | tr -d ' ')"
schema_count="$(find "$ROOT/schemas" -maxdepth 1 -type f -name '*.schema.json' | wc -l | tr -d ' ')"
example_count="$(find "$ROOT/examples" -maxdepth 1 -type f -name '*.example.json' | wc -l | tr -d ' ')"
fixture_count="$(find "$ROOT/contracts/fixtures" -type f -name '*.bundle.json' | wc -l | tr -d ' ')"
conformance_count="$(find "$ROOT/conformance" -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d ' ')"

test "$spec_count" = "7" || { echo "expected 7 spec docs, found $spec_count" >&2; exit 1; }
test "$schema_count" = "28" || { echo "expected 28 schemas, found $schema_count" >&2; exit 1; }
test "$example_count" = "28" || { echo "expected 28 examples, found $example_count" >&2; exit 1; }
test "$fixture_count" = "14" || { echo "expected 14 fixtures, found $fixture_count" >&2; exit 1; }
test "$conformance_count" = "7" || { echo "expected 7 conformance dirs, found $conformance_count" >&2; exit 1; }

echo "post-v24 profile completion pack truth checks passed"
