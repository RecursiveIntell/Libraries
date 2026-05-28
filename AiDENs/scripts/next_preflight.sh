#!/usr/bin/env bash
set -euo pipefail
AIDENS="${AIDENS_ROOT:-$HOME/Coding/Libraries/AiDENs}"
RECALL="${RECALL_ROOT:-$HOME/Coding/Recall}"
RECALL_CODING="${RECALL_CODING_ROOT:-$HOME/Coding/Recall-Coding}"
LIBRARIES="${LIBRARIES_ROOT:-$HOME/Coding/Libraries}"

echo "AiDENs:        $AIDENS"
echo "Recall:        $RECALL"
echo "Recall-Coding: $RECALL_CODING"
echo "Libraries:     $LIBRARIES"

for d in "$AIDENS" "$RECALL" "$RECALL_CODING" "$LIBRARIES"; do
  test -d "$d" || { echo "Missing directory: $d" >&2; exit 1; }
done

required_files=(
  "$AIDENS/crates/aidens-runner/src/lib.rs"
  "$AIDENS/crates/aidens-provider-kit/src/lib.rs"
  "$AIDENS/crates/aidens-tool-kit/src/lib.rs"
  "$AIDENS/crates/aidens-cli/src/lib.rs"
  "$AIDENS/crates/aidens-app-kit/src/lib.rs"
  "$AIDENS/crates/aidens-contracts/src/lib.rs"
  "$RECALL/recall-session/src/provider.rs"
  "$RECALL/recall-session/src/provider_bridge.rs"
  "$RECALL/recall-session/src/session/tool_dispatch.rs"
  "$RECALL/recall-session/src/tool_catalog.rs"
  "$RECALL/deps/llm-pipeline/src/tool_loop.rs"
)

for f in "${required_files[@]}"; do
  test -f "$f" || { echo "Missing required file: $f" >&2; exit 1; }
done

echo "Next-run preflight ok."
