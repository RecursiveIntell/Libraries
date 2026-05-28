#!/usr/bin/env bash
set -euo pipefail
ROOT="${1:-$(pwd)}"
if [[ ! -f "$ROOT/Cargo.toml" ]]; then
  echo "error: run from AiDENs repo root or pass repo root" >&2
  exit 1
fi
mkdir -p "$ROOT/scripts" "$ROOT/docs/contract-ownership" "$ROOT/.codex_evidence/contract_ownership"
cp -r AGENTS.md CODEX_MASTER_PROMPT.md CODEX_PHASE_MANIFEST.yaml CODEX_PROMPTS "$ROOT"/
cp scripts/* "$ROOT/scripts/"
cp -r docs/* "$ROOT/docs/contract-ownership/" 2>/dev/null || true
chmod +x "$ROOT"/scripts/*.sh "$ROOT"/scripts/*.py 2>/dev/null || true
echo "Installed AiDENs contract ownership collapse bundle into $ROOT"
