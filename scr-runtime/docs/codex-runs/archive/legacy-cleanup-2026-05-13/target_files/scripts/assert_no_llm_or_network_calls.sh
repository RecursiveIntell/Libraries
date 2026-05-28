#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"

SCAN_PATHS=()
for p in "$ROOT/crates" "$ROOT/Cargo.toml"; do
  if [ -e "$p" ]; then
    SCAN_PATHS+=("$p")
  fi
done

if [ "${#SCAN_PATHS[@]}" -eq 0 ]; then
  echo "No Rust paths found."
  exit 0
fi

PATTERN='reqwest|ureq|hyper|tonic|openai|anthropic|ollama|openrouter|embedding|embeddings|tokenizers|candle|llama|gemini|claude|chat.completions|model_provider'

if grep -RInE "$PATTERN" "${SCAN_PATHS[@]}" --exclude-dir=target; then
  echo "FAIL: network/LLM/model dependency or call found. P0A must be deterministic and local."
  exit 1
fi

echo "PASS: no network/LLM/model patterns found."
