#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "FAIL: cargo not found."
  exit 1
fi

if ! grep -R "verify-fixtures" "$ROOT/crates/scr-cli" >/dev/null 2>&1; then
  echo "FAIL: scr-cli verify-fixtures command not found."
  exit 1
fi

cargo run -p scr-cli -- verify-fixtures
