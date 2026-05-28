#!/usr/bin/env bash
set -euo pipefail
out="target/aidens-p20-2-audit"
mkdir -p "$out"
{
  echo "# P20.2 Audit Bundle"
  echo
  echo "Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo
  echo "## Required files"
  for f in evals/p20_agency_eval_cases.jsonl scripts/p20_2_verify.sh fixtures/test-agent/basic-agent.toml; do
    if [[ -e "$f" ]]; then echo "- present: $f"; else echo "- MISSING: $f"; fi
  done
  echo
  echo "## Logs"
  find "$out/logs" -maxdepth 1 -type f -printf '%f\n' 2>/dev/null | sort | sed 's/^/- /'
} > "$out/FINAL_AUDIT_REPORT.md"
mkdir -p handoffs/p20_2
cp "$out/FINAL_AUDIT_REPORT.md" handoffs/p20_2/FINAL_AUDIT_REPORT.md
