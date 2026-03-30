#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not available; skipped excluded ecosystem smoke" >&2
  exit 0
fi

operational_satellites=(
  "LLM-Pipeline/Cargo.toml"
  "agent-graph/Cargo.toml"
  "job-queue/Cargo.toml"
  "Tauri-Queue/Cargo.toml"
  "AI-Batch-Queue/Cargo.toml"
)

utility_satellites=(
  "ComfyUI-RS/Cargo.toml"
  "Ollama-Vision-RS/Cargo.toml"
)

for manifest in "${operational_satellites[@]}"; do
  if [[ -f "$manifest" ]]; then
    echo "trace-smoke: $manifest"
    cargo test --manifest-path "$manifest"
  fi
done

for manifest in "${utility_satellites[@]}"; do
  if [[ -f "$manifest" ]]; then
    echo "utility-smoke: $manifest"
    cargo check --manifest-path "$manifest"
  fi
done

echo "excluded ecosystem smoke passed"
