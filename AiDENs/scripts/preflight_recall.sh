#!/usr/bin/env bash
set -euo pipefail
RECALL="${1:-$HOME/Coding/Recall}"
AIDENS="${2:-$HOME/Coding/Libraries/AiDENs}"

echo "AiDENs destination: $AIDENS"
echo "Recall source:       $RECALL"

test -d "$AIDENS" || { echo "Missing AiDENs destination" >&2; exit 1; }
test -d "$RECALL" || { echo "Missing Recall source" >&2; exit 1; }
for p in \
  recall-session/src/provider.rs \
  recall-session/src/session/tool_dispatch.rs \
  recall-session/src/config.rs \
  recall-session/tests/native_tool_conformance_tests.rs \
  recall-contracts/src/lib.rs; do
  test -f "$RECALL/$p" || { echo "Missing Recall file: $p" >&2; exit 1; }
done

echo "Preflight ok. Keep Recall read-only."
