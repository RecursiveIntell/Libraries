#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

"$SCRIPT_DIR/assert_no_absolute_cargo_paths.sh" "$ROOT"

if [[ -d "$ROOT/semantic-memory" ]]; then
  "$SCRIPT_DIR/assert_no_shadow_codec.sh" "$ROOT"
  "$SCRIPT_DIR/assert_semantic_memory_integration.sh" "$ROOT" || true
fi

if [[ -d "$ROOT/turbo-quant" ]]; then
  "$SCRIPT_DIR/assert_turbo_quant_hardening.sh" "$ROOT/turbo-quant" || true
fi

echo "[check] cargo fmt/test commands to run manually or in Codex:"
cat <<'CMDS'
cargo fmt --check
cargo test -p semantic-memory --features hnsw
cargo test -p semantic-memory --features hnsw,turbo-quant-codec
(cd turbo-quant && cargo test)
CMDS
