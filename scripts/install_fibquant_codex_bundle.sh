#!/usr/bin/env bash
set -euo pipefail

BUNDLE="${1:-}"
if [[ -z "$BUNDLE" ]]; then
  echo "usage: $0 /path/to/fibquant_paper_core_codex_bundle_2026-05-16" >&2
  exit 2
fi
if [[ ! -d "$BUNDLE" ]]; then
  echo "bundle directory not found: $BUNDLE" >&2
  exit 2
fi

REPO="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO"

mkdir -p .agents/skills .codex/hooks docs/codex-runs/fibquant-paper-core scripts
cp -a "$BUNDLE/overlays/.agents/skills/fibquant-paper-core" .agents/skills/
cp -a "$BUNDLE/overlays/.codex/hooks/"*.py .codex/hooks/
chmod +x .codex/hooks/fibquant_*.py
cp -a "$BUNDLE/scripts/fibquant_startup_preflight.py" scripts/
cp -a "$BUNDLE/scripts/fibquant_final_assert.py" scripts/
chmod +x scripts/fibquant_*.py
cp -a "$BUNDLE"/*.md docs/codex-runs/fibquant-paper-core/
cp -a "$BUNDLE/phase_prompts" docs/codex-runs/fibquant-paper-core/
cp -a "$BUNDLE/manual_backstop_prompts" docs/codex-runs/fibquant-paper-core/
cp -a "$BUNDLE/matrices" docs/codex-runs/fibquant-paper-core/
cp -a "$BUNDLE/fixtures" docs/codex-runs/fibquant-paper-core/

if [[ -f .codex/hooks.json ]]; then
  echo "Existing .codex/hooks.json found; not overwriting."
  echo "Sample available at: $BUNDLE/overlays/.codex/hooks.json"
  echo "Merge manually or move existing file before installing hooks.json."
else
  cp -a "$BUNDLE/overlays/.codex/hooks.json" .codex/hooks.json
  echo "Installed .codex/hooks.json. Review with /hooks in Codex before trusting."
fi

echo "Installed FibQuant Codex bundle into $REPO"
echo "Next: start Codex, review /hooks, then paste docs/codex-runs/fibquant-paper-core/OPERATOR_PASTE_FIRST.md"
