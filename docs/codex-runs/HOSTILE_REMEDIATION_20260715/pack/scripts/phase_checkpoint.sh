#!/usr/bin/env bash
set -euo pipefail
REPO=""; PACK_DIR=""; RUN_DIR=""; PHASE=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo) REPO="$2"; shift 2 ;;
    --pack-dir) PACK_DIR="$2"; shift 2 ;;
    --run-dir) RUN_DIR="$2"; shift 2 ;;
    --phase) PHASE="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ -n "$REPO" && -n "$PACK_DIR" && -n "$RUN_DIR" && -n "$PHASE" ]] || {
 echo "usage: phase_checkpoint.sh --repo PATH --pack-dir PATH --run-dir PATH --phase N" >&2; exit 2; }
python3 "$PACK_DIR/tools/run_validation_matrix.py" --repo "$REPO" --pack-dir "$PACK_DIR" \
 --matrix "$PACK_DIR/config/validation_matrix.json" --output-dir "$RUN_DIR/evidence/phase-$PHASE" \
 --stage phase --continue-on-failure
git -C "$REPO" diff --check
git -C "$REPO" status --short
echo "Generate and validate the phase receipt before tagging."
