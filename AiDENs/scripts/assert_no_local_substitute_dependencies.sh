#!/usr/bin/env bash
set -euo pipefail
FAIL=0
if grep -RIn 'Libraries2\|libraries2' Cargo.toml crates/*/Cargo.toml 2>/dev/null; then
  echo "FAIL: dependency path references Libraries2. Canonical owners must come from ~/Coding/Libraries."
  FAIL=1
fi

# Flag local substitute module names.
if find crates/aidens-contracts crates/aidens-* -type f 2>/dev/null | grep -E '(attestation|settlement|mechanism|schema).*substitute|substitute.*(attestation|settlement|mechanism|schema)' >/dev/null; then
  echo "FAIL: local substitute module detected."
  FAIL=1
fi

if [[ "$FAIL" -ne 0 ]]; then
  exit 1
fi
echo "PASS: no local substitute dependency red flags detected."
