#!/usr/bin/env bash
set -euo pipefail
if [[ $# -ne 1 ]]; then
  echo "usage: bash install_p20_overlay.sh /path/to/AiDENs" >&2
  exit 2
fi
SRC="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DST="$1"
mkdir -p "$DST/docs/p20" "$DST/docs/p20/reports" "$DST/scripts" "$DST/evals" "$DST/fixtures/p20" "$DST/templates/p20"

cp "$SRC/AGENTS.md" "$DST/AGENTS.md"
cp -r "$SRC/docs/." "$DST/docs/p20/"
mkdir -p "$DST/docs/p20/prompts"
cp -r "$SRC/prompts/." "$DST/docs/p20/prompts/"
cp -r "$SRC/scripts/." "$DST/scripts/"
cp -r "$SRC/evals/." "$DST/evals/"
cp -r "$SRC/fixtures/." "$DST/fixtures/p20/"
cp -r "$SRC/templates/." "$DST/templates/p20/"
cp -r "$SRC/tasks" "$DST/docs/p20/tasks"
cp -r "$SRC/supporting" "$DST/docs/p20/supporting"
chmod +x "$DST/scripts/p20_"*.py "$DST/scripts/p20_"*.sh 2>/dev/null || true

echo "Installed P20 v2 overlay into $DST"
echo "Next: cd $DST && bash scripts/p20_verify.sh"
