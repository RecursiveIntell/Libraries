#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
echo "[check] no absolute Cargo path dependencies"

if grep -RIn --include='Cargo.toml' -E 'path\s*=\s*"/' "$ROOT"; then
  echo "ERROR: absolute Cargo path dependency detected" >&2
  exit 1
fi

echo "OK"
