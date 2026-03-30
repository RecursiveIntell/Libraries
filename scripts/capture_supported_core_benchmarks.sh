#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

tmp_file="$(mktemp)"
trap 'rm -f "$tmp_file"' EXIT

cargo run -p kernel-conformance --example canonical_perf_snapshot --quiet > "$tmp_file"
mkdir -p docs
mv "$tmp_file" docs/PERFORMANCE_BASELINE.md

echo "supported core benchmark snapshot captured in docs/PERFORMANCE_BASELINE.md"
