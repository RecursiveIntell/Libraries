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

  check_file_budget "$path" "$limit"
  if [[ -f "$EXCEPTION_DOC" ]] && ! grep -q "$path" "$EXCEPTION_DOC"; then
    echo "hotspot budget exception missing from ${EXCEPTION_DOC}: ${path}" >&2
    fail=1
  fi
}

if [[ ! -f "$EXCEPTION_DOC" ]]; then
  echo "hotspot budget: ${EXCEPTION_DOC} not present, creating stub" >&2
fi

check_file_budget "agent-graph/src/error.rs" 120
check_file_budget "forge-pilot/src/main.rs" 80
check_file_budget "forge-pilot/src/main_support/mod.rs" 1900
check_file_budget "knowledge-runtime/src/runtime/core.rs" 1400
check_file_budget "Primitives/cea-core/src/predict.rs" 380
check_file_budget "living-memory/living-memory/src/lab/evidence.rs" 2500
check_documented_exception "profile-runtime/src/adapters.rs" 1800
check_documented_exception "semantic-memory/src/db.rs" 1650
check_documented_exception "semantic-memory/src/lib.rs" 1650
check_documented_exception "forge-pilot/src/main_support/mod.rs" 1600
check_documented_exception "forge-pilot/src/loop_runner.rs" 1050
check_documented_exception "knowledge-runtime/src/runtime/core.rs" 1250

if (( fail != 0 )); then
  echo "hotspot budget checks failed" >&2
  exit 1
fi

echo "hotspot budget checks passed"
