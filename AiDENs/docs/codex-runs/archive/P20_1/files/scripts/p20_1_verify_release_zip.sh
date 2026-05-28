#!/usr/bin/env bash
set -euo pipefail
if [ $# -ne 1 ]; then
  echo "usage: $0 /path/to/aidens-release.zip" >&2
  exit 2
fi
ZIP="$1"
if [ ! -f "$ZIP" ]; then
  echo "zip not found: $ZIP" >&2
  exit 2
fi
TMP="${TMPDIR:-/tmp}/aidens-p20-1-zipcheck-$$"
rm -rf "$TMP"
mkdir -p "$TMP"
python3 - "$ZIP" "$TMP" <<'PY'
import sys, zipfile, pathlib
zip_path=pathlib.Path(sys.argv[1]); out=pathlib.Path(sys.argv[2])
with zipfile.ZipFile(zip_path) as z:
    z.extractall(out)
print(out)
PY
ROOT="$TMP"
count=$(find "$TMP" -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d ' ')
if [ "$count" = "1" ]; then
  ROOT=$(find "$TMP" -mindepth 1 -maxdepth 1 -type d | head -1)
fi
cd "$ROOT"
python3 scripts/p20_1_hard_code_audit.py --aidens-overlay-only --fail-on-blocking
python3 scripts/p20_validate_agency_cases.py evals/p20_agency_eval_cases.jsonl
python3 scripts/p20_1_validate_archive_manifest.py --root .
echo "release zip package checks passed: $ZIP"
