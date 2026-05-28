#!/usr/bin/env bash
set -euo pipefail
ROOT="${1:-${AIDENS_REPO_ROOT:-$(pwd)}}"
cd "$ROOT"

has_stack_deps=0
if [[ -f Cargo.toml ]] && grep -Eq 'stack-ids|semantic-memory|knowledge-runtime|forge-memory-bridge|verification-control|llm-tool-runtime' Cargo.toml; then
  has_stack_deps=1
fi

if [[ "$has_stack_deps" -eq 1 ]]; then
  if grep -RInE 'direct .*dependencies.*actual stack package names: \*\*0\*\*|Detected direct AiDENs dependencies on actual stack package names: \*\*0\*\*|do not directly depend on the actual stack crates' . --include='*.md' >/tmp/aidens_stale_docs.txt 2>/dev/null; then
    echo "ERROR: stale docs claim no direct stack dependencies despite Cargo stack deps:" >&2
    cat /tmp/aidens_stale_docs.txt >&2
    exit 1
  fi
fi
