#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not available; skipped canonical perf baseline capture" >&2
  exit 0
fi

mkdir -p docs
cargo run -p kernel-conformance --example canonical_perf_snapshot > docs/PERFORMANCE_BASELINE.md

echo "wrote docs/PERFORMANCE_BASELINE.md"
