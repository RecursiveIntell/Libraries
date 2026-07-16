#!/usr/bin/env bash
set -euo pipefail
REPO=""; PACK_DIR=""; RUN_DIR=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo) REPO="$2"; shift 2 ;;
    --pack-dir) PACK_DIR="$2"; shift 2 ;;
    --run-dir) RUN_DIR="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ -n "$REPO" && -n "$PACK_DIR" ]] || { echo "usage: final_gate.sh --repo PATH --pack-dir PATH [--run-dir PATH]" >&2; exit 2; }
[[ -n "$RUN_DIR" ]] || RUN_DIR="$REPO/docs/codex-runs/HOSTILE_REMEDIATION_20260715"
python3 "$PACK_DIR/tools/verify_pack.py" --pack "$PACK_DIR"
python3 "$PACK_DIR/tools/run_validation_matrix.py" --repo "$REPO" --pack-dir "$PACK_DIR" \
 --matrix "$PACK_DIR/config/validation_matrix.json" --output-dir "$RUN_DIR/evidence/final" \
 --stage final --continue-on-failure
python3 "$PACK_DIR/tools/check_evidence_consistency.py" --repo "$REPO" --strict
git -C "$REPO" diff --check
git -C "$REPO" diff --exit-code
if [[ -n "$(git -C "$REPO" status --porcelain=v1)" ]]; then
  echo "final gate failed: repository is not clean" >&2; exit 1
fi
echo "final gate completed"
