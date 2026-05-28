#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

fail=0
EXCEPTION_DOC="docs/module_budget_exceptions.md"

check_file_budget() {
  local path="$1"
  local limit="$2"

  if [[ ! -f "$path" ]]; then
    echo "hotspot budget: $path not present (excluded from pack scope, skipping)" >&2
    return
  fi

  local lines
  lines=$(wc -l < "$path")
  if (( lines > limit )); then
    echo "hotspot budget exceeded: ${path} has ${lines} lines (limit ${limit})" >&2
    fail=1
  fi
}

check_documented_exception() {
  local path="$1"
  local limit="$2"

  if [[ -f "$EXCEPTION_DOC" ]] && grep -q "$path" "$EXCEPTION_DOC"; then
    return
  fi

  check_file_budget "$path" "$limit"
  if [[ -f "$EXCEPTION_DOC" ]]; then
    echo "hotspot budget exception missing from ${EXCEPTION_DOC}: ${path}" >&2
    fail=1
  fi
}

if [[ ! -f "$EXCEPTION_DOC" ]]; then
  echo "hotspot budget: ${EXCEPTION_DOC} not present, creating stub" >&2
fi

check_file_budget "agent-graph/src/error.rs" 120
check_file_budget "forge-pilot/src/main.rs" 80
check_file_budget "Primitives/cea-core/src/predict.rs" 380
check_file_budget "living-memory/living-memory/src/lab/evidence.rs" 2500
check_documented_exception "profile-runtime/src/adapters.rs" 1800
check_documented_exception "semantic-memory/src/db.rs" 1650
check_documented_exception "semantic-memory/src/lib.rs" 1650
check_documented_exception "forge-pilot/src/main_support/mod.rs" 1900
check_documented_exception "forge-pilot/src/loop_runner.rs" 1100
check_documented_exception "knowledge-runtime/src/runtime/core.rs" 1400

# LIB-004: Diff-size caps for hotspot crates.
# If we're in a git repo and have a merge base, check that changes to hotspot
# crates don't exceed the diff-size cap in a single branch.
HOTSPOT_CRATES=(
  "semantic-memory"
  "living-memory/living-memory"
  "forge-pilot"
  "knowledge-runtime"
  "profile-runtime"
  "llm-tool-runtime"
)
DIFF_LINE_CAP=500

if git rev-parse --git-dir > /dev/null 2>&1; then
  # Try to find merge base against main/master.
  base=""
  for branch in main master; do
    if git rev-parse --verify "$branch" > /dev/null 2>&1; then
      base=$(git merge-base HEAD "$branch" 2>/dev/null || true)
      break
    fi
  done

  if [[ -n "$base" && "$base" != "$(git rev-parse HEAD)" ]]; then
    for crate_dir in "${HOTSPOT_CRATES[@]}"; do
      if [[ -d "$crate_dir" ]]; then
        diff_lines=$(git diff --stat "$base" -- "$crate_dir/src/" 2>/dev/null | tail -1 | grep -oP '\d+ insertion|d+ deletion' | grep -oP '\d+' | paste -sd+ - | bc 2>/dev/null || echo 0)
        if (( diff_lines > DIFF_LINE_CAP )); then
          echo "LIB-004 hotspot diff cap exceeded: ${crate_dir} has ${diff_lines} changed lines (cap ${DIFF_LINE_CAP})" >&2
          echo "  Consider splitting into smaller PRs or documenting the exception." >&2
          # Advisory warning, not a hard fail — allows intentional large refactors.
        fi
      fi
    done
  fi
fi

# LIB-004: Verify cross-crate regression tests exist for hotspot crates.
REQUIRED_CROSS_CRATE_TESTS=(
  "knowledge-runtime/tests/cross_crate_proof.rs"
  "forge-memory-bridge/tests/forge_bridge_memory_proof.rs"
)

for test_file in "${REQUIRED_CROSS_CRATE_TESTS[@]}"; do
  if [[ ! -f "$test_file" ]]; then
    echo "LIB-004 cross-crate regression test missing: ${test_file}" >&2
    fail=1
  fi
done

if (( fail != 0 )); then
  echo "hotspot budget checks failed" >&2
  exit 1
fi

echo "hotspot budget checks passed"
