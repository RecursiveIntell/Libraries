#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
required=(
  README.md
  00_START_HERE.md
  01_MASTER_ISSUE_TENSOR.json
  02_MASTER_ISSUE_MATRIX.md
  03_IMPLEMENTATION_PLAYBOOK.md
  04_EXACT_FILE_TOUCH_MAP.md
  05_TEST_AND_CONFORMANCE_PLAN.md
  06_RISK_REGISTER.md
  10_HOSTILE_AUDIT_CLAUDE.md
  11_HOSTILE_AUDIT_GPT.md
  CLAUDE.md
  PROMPT.md
  PACK_MANIFEST.json
)
missing=0
for f in "${required[@]}"; do
  if [[ ! -f "$ROOT/$f" ]]; then
    echo "pack truth: required doc $f not present" >&2
    missing=1
  fi
done
if (( missing )); then
  echo "pack truth check failed: missing required files" >&2
  exit 1
fi
PACK_ROOT="$ROOT" python3 - <<'PY'
from pathlib import Path
import json, os
root = Path(os.environ['PACK_ROOT'])
json.loads((root/'01_MASTER_ISSUE_TENSOR.json').read_text())
json.loads((root/'PACK_MANIFEST.json').read_text())
print('pack json ok')
PY
printf 'pack truth check passed\n'
