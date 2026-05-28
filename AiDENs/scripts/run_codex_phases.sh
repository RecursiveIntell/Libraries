#!/usr/bin/env bash
set -euo pipefail

PACK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="${AIDENS_REPO_ROOT:-$(pwd)}"
PROMPT_DIR="$PACK_DIR/CODEX_PROMPTS"

phase_name() {
  case "$1" in
    00) echo "SOURCE_TRUTH_AND_LAYOUT";;
    01) echo "CONTRACT_COLLAPSE";;
    02) echo "CANONICAL_ADAPTER_SPINE";;
    03) echo "GOLDEN_VERTICAL_SLICE";;
    04) echo "FAILURE_HONESTY";;
    05) echo "MEMORY_RUNTIME_HARDENING";;
    06) echo "GOVERNANCE_PROMOTION";;
    07) echo "DAEMON_QUEUE_SCHEDULE_WAKE";;
    08) echo "KERNEL_ORACLE_INTEGRATION";;
    09) echo "RELEASE_AUDIT";;
    *) echo "";;
  esac
}

usage() {
  cat <<'EOF'
Usage: scripts/run_codex_phases.sh <command> [phase]

Commands:
  list          List phases
  preflight     Verify handoff files and run static preflight checks
  phase NN      Print the Codex prompt path/content for phase NN
  verify NN     Run available gates for phase NN
  status        Print repo/root status hints

Set AIDENS_REPO_ROOT=/path/to/AiDENs when running outside the repo root.
EOF
}

require_files() {
  local files=(README.md SOURCE_BASIS.md AGENTS.md CODEX_MASTER_PROMPT.md CODEX_PHASE_MANIFEST.yaml CANONICAL_OWNER_MAP.md SHADOW_SEMANTICS_AUDIT.md GOLDEN_VERTICAL_SLICE_SPEC.md ACCEPTANCE_GATES.md TESTKIT_TARGETS.md)
  for f in "${files[@]}"; do
    test -f "$PACK_DIR/$f" || { echo "missing bundle file: $f" >&2; exit 2; }
  done
}

run_if_exists() {
  local script="$PACK_DIR/scripts/$1"
  if [[ -x "$script" ]]; then
    (cd "$REPO_ROOT" && "$script" "$REPO_ROOT")
  else
    echo "missing/non-executable script: $script" >&2
    exit 2
  fi
}

cmd="${1:-}"
case "$cmd" in
  list)
    for n in 00 01 02 03 04 05 06 07 08 09; do echo "$n $(phase_name "$n")"; done
    ;;
  preflight)
    require_files
    echo "PACK_DIR=$PACK_DIR"
    echo "REPO_ROOT=$REPO_ROOT"
    run_if_exists assert_stack_paths.sh
    # Shadow truth is expected before Phase 1, so warn during preflight.
    (cd "$REPO_ROOT" && "$PACK_DIR/scripts/assert_no_shadow_truth.sh" --warn "$REPO_ROOT") || true
    run_if_exists assert_docs_match_cargo.sh
    run_if_exists assert_compat_is_finite.sh
    ;;
  phase)
    ph="${2:-}"; name="$(phase_name "$ph")"; test -n "$name" || { usage; exit 2; }
    file="$PROMPT_DIR/PHASE_${ph}_${name}.md"
    test -f "$file" || { echo "missing phase prompt: $file" >&2; exit 2; }
    echo "$file"
    echo "----"
    cat "$file"
    ;;
  verify)
    ph="${2:-}"; name="$(phase_name "$ph")"; test -n "$name" || { usage; exit 2; }
    run_if_exists assert_stack_paths.sh
    case "$ph" in
      00) run_if_exists assert_docs_match_cargo.sh ;;
      01) run_if_exists assert_no_shadow_truth.sh; run_if_exists assert_compat_is_finite.sh ;;
      02|03|04|06|08) run_if_exists assert_adapter_delegation.sh ;;
      05) run_if_exists assert_no_shadow_truth.sh ;;
      07) run_if_exists assert_stack_paths.sh ;;
      09) run_if_exists assert_no_shadow_truth.sh; run_if_exists assert_docs_match_cargo.sh; run_if_exists assert_adapter_delegation.sh; run_if_exists assert_compat_is_finite.sh ;;
    esac
    if command -v cargo >/dev/null 2>&1; then
      (cd "$REPO_ROOT" && cargo metadata --format-version 1 >/dev/null)
      echo "cargo metadata: ok"
    else
      echo "cargo unavailable: build/test certification skipped"
    fi
    ;;
  status)
    echo "PACK_DIR=$PACK_DIR"
    echo "REPO_ROOT=$REPO_ROOT"
    test -f "$REPO_ROOT/Cargo.toml" && echo "AiDENs Cargo.toml found" || echo "AiDENs Cargo.toml NOT found"
    command -v cargo >/dev/null 2>&1 && cargo --version || echo "cargo unavailable"
    ;;
  *) usage; exit 2 ;;
esac
