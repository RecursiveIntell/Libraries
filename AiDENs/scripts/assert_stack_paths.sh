#!/usr/bin/env bash
set -euo pipefail
ROOT="${1:-${AIDENS_REPO_ROOT:-$(pwd)}}"
cd "$ROOT"

fail=0
# Only inspect Rust manifests for actual dependency wiring. Documentation is
# allowed to mention forbidden paths while explaining the rule.
if find . -name Cargo.toml -print0 2>/dev/null | xargs -0 grep -nE 'Libraries ?2/stack-ids|[Ll]ibraries2/stack-ids|\.\./\.\./Libraries ?2/stack-ids|\.\./Libraries ?2/stack-ids|repo_overlay/stack-ids|scaffolds/stack-ids' >/tmp/aidens_bad_stackids.txt 2>/dev/null; then
  echo "ERROR: forbidden libraries2 stack-ids dependency found in Cargo.toml:" >&2
  cat /tmp/aidens_bad_stackids.txt >&2
  fail=1
fi

if [[ -f Cargo.toml ]]; then
  if ! grep -q 'stack-ids.*path *= *"\.\./stack-ids"' Cargo.toml; then
    echo "WARN: root Cargo.toml does not show stack-ids path = ../stack-ids. Verify current layout." >&2
  fi
else
  echo "WARN: no Cargo.toml at $ROOT; run this from ~/Coding/Libraries/AiDENs or set AIDENS_REPO_ROOT." >&2
fi

# These warnings are layout hints, not failures, because the handoff bundle may
# be executed outside the actual repository. In the real repo they should exist.
for p in ../stack-ids ../semantic-memory-forge ../forge-memory-bridge ../semantic-memory ../knowledge-runtime ../llm-tool-runtime; do
  if [[ ! -e "$p/Cargo.toml" ]]; then
    echo "WARN: expected canonical sibling missing from AiDENs root: $p" >&2
  fi
done

exit "$fail"
