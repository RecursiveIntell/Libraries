#!/usr/bin/env bash
set -euo pipefail
if [ $# -ne 1 ]; then
  echo "usage: $0 /path/to/AiDENs" >&2
  exit 2
fi
TARGET="$1"
if [ ! -d "$TARGET" ]; then
  echo "target not found: $TARGET" >&2
  exit 2
fi
ROOT="$(cd "$(dirname "$0")" && pwd)"
for d in audit docs prompts scripts evals fixtures supporting tasks .codex; do
  if [ -d "$ROOT/$d" ]; then
    mkdir -p "$TARGET/$d"
    cp -R "$ROOT/$d/." "$TARGET/$d/"
  fi
done
cp "$ROOT/AGENTS.md" "$TARGET/AGENTS.md"
cp "$ROOT/CODEX_START_HERE.md" "$TARGET/CODEX_START_HERE_P20_1.md"
echo "Installed P20.1 overlay into $TARGET"
echo "Run: bash scripts/p20_1_verify.sh"
