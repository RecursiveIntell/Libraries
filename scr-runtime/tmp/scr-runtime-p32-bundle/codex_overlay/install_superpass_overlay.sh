#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-$(pwd)}"
SRC="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

copy_backup() {
  local src="$1"
  local dst="$2"
  mkdir -p "$(dirname "$dst")"
  if [ -e "$dst" ]; then
    cp -a "$dst" "$dst.bak.$(date -u +%Y%m%dT%H%M%SZ)"
  fi
  cp -a "$src" "$dst"
}

copy_backup "$SRC/AGENTS_SUPERPASS_APPEND.md" "$ROOT/AGENTS_P32_SUPERPASS_APPEND.md"
copy_backup "$SRC/.codex/hooks.json" "$ROOT/.codex/hooks.json"
mkdir -p "$ROOT/.codex/hooks"
for f in "$SRC"/.codex/hooks/*.py; do
  copy_backup "$f" "$ROOT/.codex/hooks/$(basename "$f")"
  chmod +x "$ROOT/.codex/hooks/$(basename "$f")"
done
mkdir -p "$ROOT/.agents/skills/scr-runtime-superpass"
copy_backup "$SRC/.agents/skills/scr-runtime-superpass/SKILL.md" "$ROOT/.agents/skills/scr-runtime-superpass/SKILL.md"

echo "Overlay installed with backups. Review changes before committing."
echo "Run hook self-test:"
echo "for f in .codex/hooks/*.py; do printf '{\"hook_event_name\":\"SelfTest\",\"cwd\":\"%s\",\"session_id\":\"selftest\"}' \"$PWD\" | python3 \"$f\" || exit 1; done"
