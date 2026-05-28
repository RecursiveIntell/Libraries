#!/usr/bin/env bash
set -euo pipefail
AIDENS="${AIDENS_ROOT:-$HOME/Coding/Libraries/AiDENs}"
RECALL="${RECALL_ROOT:-$HOME/Coding/Recall}"
RECALL_CODING="${RECALL_CODING_ROOT:-$HOME/Coding/Recall-Coding}"
LIBRARIES="${LIBRARIES_ROOT:-$HOME/Coding/Libraries}"

echo "## AiDENs fake completion/current scaffold patterns"
grep -RIn --exclude-dir target --exclude-dir .git -E "placeholder|wire provider implementation next|fake success|TODO runtime|skeletal" "$AIDENS/crates" || true

echo "## Recall provider/tool symbols"
grep -RIn -E "ToolLoopRunner|run_openai_responses|run_openai_chat|run_ollama|run_anthropic|resolve_execution_mode|CompletionProvider|LlmPipelineProvider|ProviderCapabilities" \
  "$RECALL/recall-session/src" "$RECALL/deps/llm-pipeline/src" | head -180 || true

echo "## Recall path safety symbols"
grep -RIn -E "path safety|sandbox|traversal|canonical|sensitive|absolute" "$RECALL/recall-session/src/path_safety.rs" "$RECALL/recall-session/src/tools" | head -120 || true

echo "## Recall-Coding coding tool candidates"
find "$RECALL_CODING" -type f | grep -E 'workspace|coding|run_checks|patch|audit|shell|file' | head -120 || true

echo "## Library candidates"
find "$LIBRARIES" -maxdepth 4 -type f \( -path '*/llm-tool-runtime/src/lib.rs' -o -path '*/llm-pipeline/src/lib.rs' -o -path '*/job-queue/src/lib.rs' -o -path '*/stack-ids/src/lib.rs' \) -print || true
