#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"

if [ ! -d "$ROOT/crates" ]; then
  echo "No crates directory found."
  exit 0
fi

PATTERN='static[[:space:]]+mut|lazy_static!|OnceCell<.*(Mutex|RwLock)|OnceLock<.*(Mutex|RwLock)|GLOBAL_|SINGLETON|score_cache|policy_cache'

if grep -RInE "$PATTERN" "$ROOT/crates" --exclude-dir=target; then
  echo "FAIL: potential shadow-truth mutable global/cache pattern found."
  exit 1
fi

echo "PASS: no obvious shadow-truth mutable global/cache pattern found."
