#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
SCAN_PATHS=()

for p in "$ROOT/crates" "$ROOT/src" "$ROOT/policies" "$ROOT/schemas/generated"; do
  if [ -e "$p" ]; then
    SCAN_PATHS+=("$p")
  fi
done

if [ "${#SCAN_PATHS[@]}" -eq 0 ]; then
  echo "No production paths found to scan."
  exit 0
fi

PATTERN='FEUT|EEG|theta|gamma|alpha band|intelligence field|P[[:space:]]*=[[:space:]]*NP|Clay proof|Riemann proof|universal entropy law|black.?hole echo|neuro.?calibrated'

if grep -RInE "$PATTERN" "${SCAN_PATHS[@]}"; then
  echo "FAIL: forbidden FEUT/EEG/proof terminology found in production paths."
  exit 1
fi

echo "PASS: no forbidden FEUT/EEG/proof terminology in production paths."
