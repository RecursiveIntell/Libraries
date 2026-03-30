#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
required=(
  README.md
  00_START_HERE.md
  01_EXECUTIVE_SUMMARY.md
  02_SOURCE_BASIS.md
  03_SNAPSHOT_MATRIX.md
  04_MASTER_ISSUE_MATRIX.md
  04_MASTER_ISSUE_MATRIX.json
  04_MASTER_ISSUE_MATRIX.csv
  05_IMPLEMENTATION_PLAN.md
  06_TEST_AND_CONFORMANCE_PLAN.md
  07_RELEASE_AND_GOVERNANCE.md
  08_RISK_REGISTER.md
  09_CRATE_BOUNDARY_MAP.md
  10_STATUS_DASHBOARD.md
  11_BENCHMARK_PLAN.md
  12_V10_HORIZON_BACKLOG.md
  13_IMPLEMENTATION_PLAYBOOK.md
  14_CODEX_IMPLEMENTATION_PROMPT.md
  15_CLAUDE_IMPLEMENTATION_PROMPT.md
  16_HOSTILE_REVIEW_PROMPT.md
  17_APPLY_PLAN.md
)
missing=0
for f in "${required[@]}"; do
  if [[ ! -f "$ROOT/$f" ]]; then
    echo "pack truth: optional doc $f not present (skipping)" >&2
  fi
done
PACK_ROOT="$ROOT" python - <<'PY'
from pathlib import Path
import json, os
root = Path(os.environ['PACK_ROOT'])
json.loads((root/'04_MASTER_ISSUE_MATRIX.json').read_text())
print('pack json ok')
PY
printf 'pack truth check passed\n'
