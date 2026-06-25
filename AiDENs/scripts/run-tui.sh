#!/usr/bin/env bash
# AiDENs TUI launcher — handles all args, prevents line-wrapping issues.
# Usage: ./run-tui.sh [model] [ollama-url]

set -euo pipefail

MODEL="${1:-gemma4:31b-cloud}"
OLLAMA_URL="${2:-http://127.0.0.1:11434}"
MEMORY_DIR="${HOME}/.hermes/semantic-memory.db"
QUEUE_DIR="/tmp/aidens-queue"
HTTP_URL="http://127.0.0.1:1738"
BINARY="${HOME}/Coding/Libraries/AiDENs/target/release/aidens-tui"

mkdir -p "$QUEUE_DIR"

exec "$BINARY" \
  --memory-dir "$MEMORY_DIR" \
  --queue-dir "$QUEUE_DIR" \
  --ollama-url "$OLLAMA_URL" \
  --ollama-model "$MODEL" \
  --http-base-url "$HTTP_URL"