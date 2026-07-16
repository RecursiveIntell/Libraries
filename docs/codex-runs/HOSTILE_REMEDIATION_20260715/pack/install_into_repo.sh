#!/usr/bin/env bash
set -euo pipefail
REPO="${1:-}"; TARGET_REL="${2:-docs/codex-runs/HOSTILE_REMEDIATION_20260715/pack}"
[[ -n "$REPO" ]] || { echo "usage: install_into_repo.sh /path/to/Libraries [target-relative-path]" >&2; exit 2; }
REPO="$(cd "$REPO" && pwd)"; PACK_DIR="$(cd "$(dirname "$0")" && pwd)"; TARGET="$REPO/$TARGET_REL"
[[ ! -e "$TARGET" ]] || { echo "refusing overwrite: $TARGET" >&2; exit 1; }
python3 "$PACK_DIR/tools/verify_pack.py" --pack "$PACK_DIR"
mkdir -p "$(dirname "$TARGET")"; cp -R "$PACK_DIR" "$TARGET"; echo "$TARGET"
