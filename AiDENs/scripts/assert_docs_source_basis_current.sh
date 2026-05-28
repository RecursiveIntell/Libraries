#!/usr/bin/env bash
set -euo pipefail
FAIL=0
SCAN_PATHS=(README.md STATUS.md SOURCE_BASIS.md SUPPORT_PROFILE.md docs)
# Old source-basis references are allowed only when explicitly labeled historical/superseded.
SOURCE_HITS="$(grep -RIn --exclude-dir=archive 'libraries-source-clean-20260426.zip\|2026-04-26.*current\|current.*2026-04-26' "${SCAN_PATHS[@]}" 2>/dev/null || true)"
if [[ -n "$SOURCE_HITS" ]]; then
  printf '%s\n' "$SOURCE_HITS"
  echo "FAIL: stale 2026-04-26 source basis appears to be current."
  FAIL=1
fi
LOC_HITS="$(grep -RIn --exclude-dir=archive '37 Rust files\|5,126 LOC\|5126 LOC' "${SCAN_PATHS[@]}" 2>/dev/null || true)"
if [[ -n "$LOC_HITS" ]]; then
  printf '%s\n' "$LOC_HITS"
  echo "FAIL: stale file/LOC count found."
  FAIL=1
fi
if [[ "$FAIL" -ne 0 ]]; then
  echo "Update docs to current 2026-04-28 source basis or explicitly mark old references historical/superseded."
  exit 1
fi
echo "PASS: no blocking stale source-basis docs detected."
