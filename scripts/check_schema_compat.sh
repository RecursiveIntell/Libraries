#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

if [[ ! -d schemas ]]; then
  echo "schema compatibility check failed: missing canonical schemas directory" >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "schema compatibility check failed: cargo not available" >&2
  exit 1
fi

cargo run -p contract-schema-gen -- --check schemas >/dev/null

echo "schema compatibility check passed"
