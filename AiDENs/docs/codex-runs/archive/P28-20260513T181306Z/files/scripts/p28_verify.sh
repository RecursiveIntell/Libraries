#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "[p28] superseded historical verifier; delegating to current verifier" >&2
exec bash "$SCRIPT_DIR/verify_current.sh" "$@"
