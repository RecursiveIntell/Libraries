#!/usr/bin/env bash
set -euo pipefail
DEST="${1:-/home/sikmindz/Coding/Libraries/AiDENs}"
SRC="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$DEST"
cp -R "$SRC"/* "$DEST"/
echo "Installed P30 bundle into $DEST"
echo "Next: bash \"$DEST/scripts/p30_verify.sh\" \"$DEST\""
