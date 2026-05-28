#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"

if [ ! -d "$ROOT/crates" ]; then
  echo "No crates directory found."
  exit 0
fi

if grep -RInE '\b(f32|f64)\b' "$ROOT/crates" --exclude-dir=target; then
  echo "FAIL: f32/f64 found under crates. P0A durable score/control logic must be integer/fixed-point only."
  exit 1
fi

echo "PASS: no f32/f64 under crates."
