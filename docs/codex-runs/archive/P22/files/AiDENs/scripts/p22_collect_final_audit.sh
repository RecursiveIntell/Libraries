#!/usr/bin/env bash
set -euo pipefail
mkdir -p target/p22/audit handoffs/p22
{
  echo "# P22 Command Log Summary"
  echo
  echo "Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo
  find target/p22/audit -maxdepth 1 -type f | sort | sed 's#^#- #'
} > target/p22/audit/COMMAND_LOG_SUMMARY.md

git status --short > target/p22/audit/git_status_short.txt || true
git diff --stat > target/p22/audit/git_diff_stat.txt || true
cp target/p22/audit/git_status_short.txt target/p22/audit/CHANGED_FILE_SUMMARY.md || true
: > target/p22/audit/UNRESOLVED_RISKS.md

echo "P22 audit collection complete"
