#!/usr/bin/env bash
set -euo pipefail
REPO=""; PACK_DIR=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo) REPO="$2"; shift 2 ;;
    --pack-dir) PACK_DIR="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ -n "$REPO" && -n "$PACK_DIR" ]] || { echo "usage: bootstrap_run.sh --repo PATH --pack-dir PATH" >&2; exit 2; }
REPO="$(cd "$REPO" && pwd)"; PACK_DIR="$(cd "$PACK_DIR" && pwd)"
RUN_DIR="$REPO/docs/codex-runs/HOSTILE_REMEDIATION_20260715"
python3 "$PACK_DIR/tools/verify_pack.py" --pack "$PACK_DIR"
[[ ! -e "$RUN_DIR" ]] || { echo "run directory already exists: $RUN_DIR" >&2; exit 1; }
mkdir -p "$RUN_DIR"/{run,evidence/baseline,handoffs,decisions}
cp -R "$PACK_DIR"/. "$RUN_DIR/pack/"
cp "$PACK_DIR/templates/RUN_STATE.json" "$RUN_DIR/run/state.json"
cp "$PACK_DIR/templates/DECISION_LOG.md" "$RUN_DIR/run/decision_log.md"
cp "$PACK_DIR/templates/RISK_REGISTER.md" "$RUN_DIR/run/risk_register.md"
python3 "$PACK_DIR/tools/workspace_inventory.py" --repo "$REPO" --output "$RUN_DIR/evidence/baseline/workspace_inventory.json" || true
python3 "$PACK_DIR/tools/check_id_authority.py" --repo "$REPO" --allowlist "$PACK_DIR/config/id_authority_allowlist.json" --json-output "$RUN_DIR/evidence/baseline/id_authority.json" || true
python3 "$PACK_DIR/tools/check_placeholder_codecs.py" --repo "$REPO" --json-output "$RUN_DIR/evidence/baseline/placeholder_codecs.json" || true
python3 "$PACK_DIR/tools/check_evidence_consistency.py" --repo "$REPO" --strict > "$RUN_DIR/evidence/baseline/evidence_consistency.json" || true
echo "$RUN_DIR"
