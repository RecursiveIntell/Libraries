#!/usr/bin/env bash
set -euo pipefail
# Lightweight grep-based guard until a real cargo-metadata boundary checker is added.
fail=0
check_forbidden() {
  local crate="$1" forbidden="$2"
  if grep -R "${forbidden//-/_}\|$forbidden" "crates/$crate/src" >/dev/null 2>&1; then
    echo "Boundary violation: $crate references $forbidden" >&2
    fail=1
  fi
}
check_forbidden aidens-contracts aidens-app-kit
check_forbidden aidens-contracts aidens-runner
check_forbidden aidens-runner aidens-tauri-kit
check_forbidden aidens-runner aidens-daemon-kit
check_forbidden aidens-tool-kit aidens-app-kit
check_forbidden aidens-provider-kit aidens-tool-kit
exit "$fail"
