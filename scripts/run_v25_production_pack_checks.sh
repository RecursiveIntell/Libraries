#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"
MODE="pack"
if [[ "${1:-}" == "--final" ]]; then
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
  MODE="final"
fi
if [[ "${2:-}" == "--final" ]]; then
  MODE="final"
fi

bash "$ROOT/scripts/check_v25_production_pack_truth.sh" "$ROOT"
bash "$ROOT/scripts/check_no_local_recomposition.sh" "$ROOT"
python3 "$ROOT/scripts/audit_v25_production_gap.py" > /tmp/v25-production-gap.json

if [[ "$MODE" == "final" ]]; then
  python3 "$ROOT/scripts/check_v25_production_closure.py"
fi

echo "v25 production pack checks completed in mode: $MODE"
