#!/usr/bin/env bash
set -euo pipefail
PHASE="${1:?phase id required}"
DIR=".codex_evidence/contract_ownership/$PHASE"
mkdir -p "$DIR"
git status --short > "$DIR/git_status_after.txt" || true
git diff --stat > "$DIR/git_diff_stat.txt" || true
git diff --binary > "$DIR/git_diff.patch" || true
echo "Evidence collected in $DIR"
