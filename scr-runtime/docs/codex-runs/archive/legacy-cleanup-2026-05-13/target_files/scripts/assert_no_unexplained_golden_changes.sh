#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"

if [ ! -d "$ROOT/.git" ]; then
  echo "No git repo detected; skipping golden change guard."
  exit 0
fi

CHANGED="$(git -C "$ROOT" status --porcelain -- fixtures/audit/expected || true)"

if [ -n "$CHANGED" ] && [ ! -f "$ROOT/POLICY_CHANGE.md" ]; then
  echo "FAIL: golden expected fixtures changed without POLICY_CHANGE.md"
  echo "$CHANGED"
  exit 1
fi

echo "PASS: no unexplained golden fixture changes."
