#!/usr/bin/env bash
set -euo pipefail

BASE_DIR="$(cd "$(dirname "$0")/.." && pwd)"
required=(
  "README.md"
  "00_START_HERE.md"
  "04_MASTER_ISSUE_MATRIX.md"
  "04_MASTER_ISSUE_MATRIX.json"
  "05_IMPLEMENTATION_SEQUENCE.md"
  "06_TEST_AND_CONFORMANCE_PLAN.md"
  "07_RELEASE_BAR_AND_CLOSEOUT.md"
  "17_APPLY_PLAN.md"
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
base = pathlib.Path(r"/mnt/data/zz6_v9_closeout_pack")
manifest = json.loads((base/"PACK_MANIFEST.json").read_text())
assert manifest["pack"] == "zz6_v9_closeout_pack"
print("manifest ok")
PY

echo "closeout pack truth check passed"
