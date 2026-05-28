#!/usr/bin/env bash
set -euo pipefail
ROOT="${1:-${AIDENS_REPO_ROOT:-$(pwd)}}"
cd "$ROOT"
fail=0
check_file_contains() {
  local file="$1"; shift
  if [[ ! -f "$file" ]]; then echo "WARN: missing $file" >&2; return; fi
  for needle in "$@"; do
    if ! grep -q "$needle" "$file"; then
      echo "ERROR: $file does not reference required canonical crate/token: $needle" >&2
      fail=1
    fi
  done
}
check_file_contains crates/aidens-memory-kit/Cargo.toml 'forge-memory-bridge' 'semantic-memory' 'knowledge-runtime' 'semantic-memory-forge'
check_file_contains crates/aidens-receipts/Cargo.toml 'llm-tool-runtime' 'semantic-memory-forge' 'verification-control'
check_file_contains crates/aidens-kernel-kit/Cargo.toml 'recursive-kernel-core' 'constraint-compiler' 'kernel-execution' 'kernel-oracles'
check_file_contains crates/aidens-governance-kit/Cargo.toml 'verification-control' 'verification-policy' 'verification-adjudication'
check_file_contains crates/aidens-provider-kit/Cargo.toml 'llm-tool-runtime'
check_file_contains crates/aidens-tool-kit/Cargo.toml 'llm-tool-runtime'

# Heuristic source-level delegation checks.
check_file_contains crates/aidens-memory-kit/src/lib.rs 'forge_memory_bridge' 'semantic_memory' 'knowledge_runtime'
check_file_contains crates/aidens-receipts/src/lib.rs 'llm_tool_runtime' 'semantic_memory_forge' 'verification_control'
check_file_contains crates/aidens-kernel-kit/src/lib.rs 'recursive_kernel_core' 'constraint_compiler' 'kernel_execution' 'kernel_oracles'
check_file_contains crates/aidens-governance-kit/src/lib.rs 'verification_control' 'verification_policy' 'verification_adjudication'
check_file_contains crates/aidens-provider-kit/src/lib.rs 'llm_tool_runtime'
check_file_contains crates/aidens-tool-kit/src/lib.rs 'llm_tool_runtime'
exit "$fail"
