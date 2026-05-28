#!/usr/bin/env bash
set -euo pipefail
OUT="${1:-target/p21/audit}"
mkdir -p "$OUT" handoffs/p21
{
  echo "# P21 Audit Bundle"
  echo
  echo "Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo
  echo "## Git status"
  git status --short 2>/dev/null || true
  echo
  echo "## Available logs"
  find target/p21 -maxdepth 2 -type f 2>/dev/null | sort || true
} > "$OUT/README.md"
cp "$OUT/README.md" handoffs/p21/AUDIT_BUNDLE_INDEX.md
