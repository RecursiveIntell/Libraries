#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

if [[ -f AiDENs/Cargo.toml && -f AiDENs/scripts/validate_learning_corpus_v2.py ]]; then
  echo "[verify-learning-agent] detected archive root; entering AiDENs/"
  cd AiDENs
fi

LOG_DIR="${AIDENS_LEARNING_VERIFY_LOG_DIR:-target/verify-learning-agent/${RUN_ID:-CURRENT}}"
mkdir -p "$LOG_DIR"

required_failures=0
live_skipped=0

check() {
  local name="$1"
  local required="$2"
  shift 2
  echo "[verify-learning-agent] $name: $*"
  set +e
  "$@" >"$LOG_DIR/${name}.stdout.log" 2>"$LOG_DIR/${name}.stderr.log"
  local code=$?
  set -e
  if [[ "$code" -eq 0 ]]; then
    echo "[verify-learning-agent] PASS $name"
  else
    echo "[verify-learning-agent] FAIL $name exit=$code" >&2
    tail -40 "$LOG_DIR/${name}.stderr.log" >&2 || true
    if [[ "$required" == "required" ]]; then
      required_failures=$((required_failures + 1))
    fi
  fi
}

skip() {
  local name="$1"
  local reason="$2"
  echo "[verify-learning-agent] SKIP $name ($reason)"
}

check completion_ledger required python3 -m pytest -q scripts/tests/test_learning_completion_ledger.py
check corpus_v2_tests required python3 -m pytest -q scripts/tests/test_learning_corpus_v2.py
check corpus_v2_validator required python3 scripts/validate_learning_corpus_v2.py fixtures/learning-coding-agent/v2

if command -v cargo >/dev/null 2>&1; then
  check aidens_cli_learning_tests required cargo test -p aidens-cli --lib learning_
else
  skip aidens_cli_learning_tests "cargo is unavailable"
  required_failures=$((required_failures + 1))
fi

# A static gate cannot certify the current-HEAD live lane. Keep this explicit
# even when all required local checks pass.
skip live_podman_closure_replay "not run; current-HEAD Podman evidence is required for a complete release claim"
live_skipped=1

if [[ "$required_failures" -ne 0 ]]; then
  echo "[verify-learning-agent] RELEASE_GATE FAIL required_failures=$required_failures"
  exit 1
fi

if [[ "$live_skipped" -ne 0 ]]; then
  echo "[verify-learning-agent] RELEASE_GATE INDETERMINATE live_podman_closure_replay=SKIP"
  exit 2
fi

echo "[verify-learning-agent] RELEASE_GATE PASS"
