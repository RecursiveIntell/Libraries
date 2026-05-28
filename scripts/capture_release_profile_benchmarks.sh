#!/usr/bin/env bash
# LIB-006: Capture release-profile benchmark receipts.
#
# The existing dev-profile baseline (evidence/perf_baseline_20260330.json) is a
# regression alarm. This script captures the same canonical fixtures under
# --release so external reviewers can see throughput numbers that reflect
# optimized code, not debug instrumentation.
#
# Usage:
#   bash scripts/capture_release_profile_benchmarks.sh

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo not available; skipped release benchmark capture" >&2
    exit 0
fi

TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
DATE_TAG="$(date -u +%Y%m%d)"
REPORT_FILE="evidence/perf_release_${DATE_TAG}.json"

echo "[bench] Building kernel-conformance canonical_perf_snapshot in release mode..."
# Build first so we can separate compile time from execution time.
cargo build -p kernel-conformance --example canonical_perf_snapshot --release --quiet 2>&1

echo "[bench] Running release-profile benchmarks..."
# Capture the markdown output, then parse it into JSON.
MD_OUTPUT="$(cargo run -p kernel-conformance --example canonical_perf_snapshot --release --quiet 2>&1)"

# Parse the markdown table into JSON.
# Expected format: | fixture | effects | regions | delta | export | transform | import | compile | advisory |
mkdir -p evidence

python3 -c "
import json, sys, re

lines = '''${MD_OUTPUT}'''.strip().split('\n')
# Find table data rows (skip header and separator).
data_rows = [l for l in lines if l.startswith('|') and '---' not in l and 'Fixture' not in l]

fixtures = []
for row in data_rows:
    cols = [c.strip() for c in row.split('|')[1:-1]]
    if len(cols) >= 9:
        fixtures.append({
            'name': cols[0],
            'effect_records': int(cols[1]),
            'regions': int(cols[2]),
            'delta_regions': int(cols[3]),
            'export_ms': int(cols[4]),
            'transform_ms': int(cols[5]),
            'import_ms': int(cols[6]),
            'compile_ms': int(cols[7]),
            'advisory_ms': int(cols[8]),
        })

report = {
    'captured_at': '${TIMESTAMP}',
    'issue_ref': 'LIB-006',
    'profile': 'release (optimized)',
    'tool': 'kernel-conformance canonical_perf_snapshot --release',
    'note': 'Release-profile throughput numbers. Compare against evidence/perf_baseline_*.json (dev-profile) to see optimization impact.',
    'fixtures': fixtures,
}

with open('${REPORT_FILE}', 'w') as f:
    json.dump(report, f, indent=2)
    f.write('\n')

print(json.dumps(report, indent=2))
" 2>&1

echo ""
echo "[bench] Release benchmark report written to: ${REPORT_FILE}"
