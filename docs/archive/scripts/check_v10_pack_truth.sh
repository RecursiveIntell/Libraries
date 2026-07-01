#!/usr/bin/env bash
set -euo pipefail

BASE_DIR="$(cd "$(dirname "$0")/.." && pwd)"
required=(
  "README.md"
  "00_START_HERE.md"
  "04_V10_MASTER_ISSUE_MATRIX.md"
  "04_V10_MASTER_ISSUE_MATRIX.json"
  "05_V10_IMPLEMENTATION_SEQUENCE.md"
  "09_GRAPH_GEOMETRY_PLAN.md"
  "10_INCREMENTAL_RECOMPUTATION_PLAN.md"
  "11_REPAIR_AND_SYNDROME_PLAN.md"
  "12_NUISANCE_STATE_AND_CALIBRATION_PLAN.md"
  "13_V10_BENCHMARK_PLAN.md"
  "PACK_MANIFEST.json"
)

for f in "${required[@]}"; do
  if [[ ! -f "$BASE_DIR/$f" ]]; then
    echo "missing: $f" >&2
    exit 1
  fi
done

python3 - <<'PY'
import json, pathlib
base = pathlib.Path(r"/mnt/data/v10_finishline_pack")
manifest = json.loads((base/"PACK_MANIFEST.json").read_text())
assert manifest["pack"] == "v10_finishline_pack"
print("manifest ok")
PY

echo "v10 pack truth check passed"
