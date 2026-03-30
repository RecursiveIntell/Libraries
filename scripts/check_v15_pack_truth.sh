#!/usr/bin/env bash
set -euo pipefail

BASE_DIR="$(cd "$(dirname "$0")/.." && pwd)"

required=(
  "README.md"
  "00_START_HERE.md"
  "05_V15_MASTER_ISSUE_MATRIX.md"
  "05_V15_MASTER_ISSUE_MATRIX.json"
  "06_V15_IMPLEMENTATION_SEQUENCE.md"
  "07_V15_TEST_AND_CONFORMANCE_PLAN.md"
  "08_V15_RELEASE_BAR_AND_GOVERNANCE.md"
  "10_CRATE_OWNERSHIP_AND_BOUNDARY_PLAN.md"
  "11_V14_COUNTERFACTUAL_RUNTIME_PLAN.md"
  "12_V15_ATTESTED_EXCHANGE_PLAN.md"
  "13_SCHEMA_REGISTRY_AND_COMPATIBILITY_PLAN.md"
  "14_REFERENCE_INTERPRETER_AND_CONFORMANCE_PLAN.md"
  "PACK_MANIFEST.json"
)

for f in "${required[@]}"; do
  if [[ ! -f "$BASE_DIR/$f" ]]; then
    echo "missing: $f" >&2
    exit 1
  fi
done

schema_count=$(find "$BASE_DIR/schemas" -maxdepth 1 -name '*.schema.json' | wc -l | tr -d ' ')
example_count=$(find "$BASE_DIR/examples" -maxdepth 1 -name '*.example.json' | wc -l | tr -d ' ')

if [[ "$schema_count" -lt 20 ]]; then
  echo "expected at least 20 schema files, found $schema_count" >&2
  exit 1
fi

if [[ "$example_count" -lt 20 ]]; then
  echo "expected at least 20 example files, found $example_count" >&2
  exit 1
fi

python3 - <<'PY'
import json, pathlib
base = pathlib.Path(r"/mnt/data/v15_endgame_pack")
manifest = json.loads((base/"PACK_MANIFEST.json").read_text())
assert manifest["pack"] == "v15_endgame_pack"
assert manifest["basis"]["workspace_members"] == 18
assert manifest["draft_schema_files"] >= 20
assert manifest["draft_example_files"] >= 20
print("manifest ok")
PY

echo "v15 pack truth check passed"
