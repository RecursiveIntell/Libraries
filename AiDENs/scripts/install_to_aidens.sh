#!/usr/bin/env bash
set -euo pipefail
DEST="${1:-$HOME/Coding/Libraries/AiDENs}"
SRC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mkdir -p "$DEST"
rsync -a --exclude target "$SRC_DIR/" "$DEST/"
echo "Installed AiDENs bootstrap into $DEST"
