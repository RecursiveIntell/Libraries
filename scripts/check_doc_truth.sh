#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

errors=0

for path in README.md CONFORMANCE_GATES.md CLAUDE.md PROMPT.md SOURCE_BASIS.md SUPPORT_PROFILE.md; do
  if [[ ! -f "$path" ]]; then
    echo "missing doc: $path" >&2
    errors=1
  fi
done

if [[ -f README.md ]] && ! grep -q 'make gate' README.md; then
  echo "README.md must point to make gate" >&2
  errors=1
fi

# PACK_README.md archived to docs/archive/superseded_packs/ in V29

if [[ -f CONFORMANCE_GATES.md ]] && ! grep -q 'cargo run -p contract-schema-gen -- schemas.generated' CONFORMANCE_GATES.md; then
  echo "CONFORMANCE_GATES.md must mention schema regeneration" >&2
  errors=1
fi

if [[ -f CONFORMANCE_GATES.md ]] && ! grep -q 'cargo clippy --workspace --all-targets --all-features -- -D warnings' CONFORMANCE_GATES.md; then
  echo "CONFORMANCE_GATES.md must describe warnings-fatal clippy" >&2
  errors=1
fi

# CLAUDE_AUDIT_RECONCILIATION.md and MASTER_ISSUE_MATRIX.md archived in V29
# Audit reconciliation is now covered by 10_HOSTILE_AUDIT_CLAUDE.md and 11_HOSTILE_AUDIT_GPT.md

# V6/V7 canonical specs archived in V29 — V25/V26 are the current specs
# RELEASE_CHECKLIST.md archived in V29 — gate verification is now via make gate

if (( errors != 0 )); then
  echo "doc truth check failed" >&2
  exit 1
fi

echo "doc truth check passed"
