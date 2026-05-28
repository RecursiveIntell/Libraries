#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${ROOT}"

log() {
  printf '[p28-verify] %s\n' "$*"
}

require_file() {
  local path="$1"
  if [[ ! -e "${path}" ]]; then
    printf 'FAIL: required verifier input missing: %s\n' "${path}" >&2
    exit 2
  fi
}

require_file "P28_MASTER_PACKET.md"
require_file "P28_ACCEPTANCE_GATES.md"
require_file "P28_PHASE_PLAN.md"
require_file "STATUS.md"
require_file "SUPPORT_PROFILE.md"
require_file "SOURCE_BASIS.md"
require_file "scripts/assert_p28_zpy_safe_relative.py"
require_file "scripts/assert_p28_zpy_text_detection.py"
require_file "scripts/assert_p28_manifest_semantic_aggregate.py"
require_file "scripts/assert_p28_package_validation_paths.py"

log "current-run truth"
python3 scripts/assert_current_run_truth.py

log "P28 package/z.py regression guards"
python3 scripts/assert_p28_zpy_safe_relative.py
python3 scripts/assert_p28_zpy_text_detection.py
python3 scripts/assert_p28_manifest_semantic_aggregate.py
python3 scripts/assert_p28_package_validation_paths.py

if [[ "${P28_SKIP_CARGO:-0}" == "1" || "${P27_SKIP_CARGO:-0}" == "1" ]]; then
  log "cargo checks skipped by explicit P28_SKIP_CARGO/P27_SKIP_CARGO"
  if [[ "${P28_REQUIRE_CARGO:-0}" == "1" || "${P27_REQUIRE_CARGO:-0}" == "1" || "${P28_FINAL_STRICT:-0}" == "1" ]]; then
    printf 'FAIL: cargo checks are required but skip flag is set\n' >&2
    exit 2
  fi
  exit 0
fi

log "cargo fmt"
cargo fmt --all -- --check

log "cargo check"
cargo check --workspace --all-targets

log "cargo test"
cargo test --workspace --all-targets

if [[ "${P28_FINAL_STRICT:-0}" == "1" ]]; then
  log "cargo clippy"
  cargo clippy --workspace --all-targets -- -D warnings

  log "cargo doc"
  cargo doc --workspace --no-deps
fi

log "ok"
