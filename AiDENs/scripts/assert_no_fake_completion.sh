#!/usr/bin/env bash
set -euo pipefail
ROOT="${1:-.}"
FAIL=0
PATTERN='AiDENs placeholder response|placeholder runner output|wire provider implementation next|fake success|TODO runtime|not implemented but healthy'

echo "Checking for forbidden fake-completion patterns under $ROOT/crates"
if grep -RIn --exclude-dir target --exclude-dir .git -E "$PATTERN" "$ROOT/crates"; then
  FAIL=1
fi

# Normal runtime crates should not use placeholder-style answers.
if grep -RIn --exclude-dir target --exclude-dir .git -E 'format!\("AiDENs .*response to:' "$ROOT/crates/aidens-runner" 2>/dev/null; then
  FAIL=1
fi

if [ "$FAIL" -ne 0 ]; then
  echo "Forbidden fake completion pattern remains. Pass is not complete." >&2
  exit 1
fi

echo "No fake completion patterns found."
