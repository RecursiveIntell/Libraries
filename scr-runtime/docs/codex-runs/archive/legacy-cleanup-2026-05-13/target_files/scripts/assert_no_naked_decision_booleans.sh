#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"

if [ ! -d "$ROOT/crates" ]; then
  echo "No crates directory found."
  exit 0
fi

PATTERN='pub[[:space:]]+fn[[:space:]]+(should_|can_|is_|allow|block|decide|evaluate)[A-Za-z0-9_]*[[:space:]]*\([^)]*\)[[:space:]]*->[[:space:]]*bool'

if grep -RInE "$PATTERN" "$ROOT/crates" --exclude-dir=target; then
  echo "FAIL: naked bool decision API found. Return a receipt-bearing decision or explicit error."
  exit 1
fi

echo "PASS: no naked bool decision APIs found."
