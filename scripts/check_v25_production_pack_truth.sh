#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"

required_files=(
  "docs/v25/PRODUCTION_GAP_AUDIT_20260318.md"
  "docs/v25/PRODUCTION_GAP_AUDIT_20260318.json"
  "docs/v25/PRODUCTION_MASTER_ISSUE_MATRIX_20260318.md"
  "docs/v25/PRODUCTION_MASTER_ISSUE_MATRIX_20260318.csv"
  "docs/v25/PRODUCTION_MASTER_ISSUE_MATRIX_20260318.json"
  "docs/v25/PRODUCTION_EXACT_FILE_TOUCH_MAP_20260318.md"
  "docs/v25/PRODUCTION_CONSUMER_CITATION_SPEC_20260318.md"
  "docs/v25/PRODUCTION_SCHEMA_AND_EXAMPLE_BACKFILL_PLAN_20260318.md"
  "docs/v25/PRODUCTION_CI_AND_GATE_PLAN_20260318.md"
  "docs/v25/PRODUCTION_ACCEPTANCE_AND_COMMANDS_20260318.md"
  "docs/v25/PRODUCTION_CODEX_SEQUENCE_20260318.md"
  "docs/v25/PRODUCTION_RELEASE_BAR_AND_EXIT_CRITERIA_20260318.md"
  "docs/v25/PRODUCTION_NON_GOALS_AND_GUARDRAILS_20260318.md"
  "docs/v25/PRODUCTION_CI_WORKFLOW_SNIPPET_20260318.yml"
  "plans/v25-production-closure.execplan.md"
  "apply/v25/PRODUCTION_APPLY_SEQUENCE_20260318.md"
  "prompts/codex_finish_handoff_prompt_v25_production.txt"
  "prompts/codex_finish_operating_prompt_v25_production.md"
  "prompts/codex_short_prompt_v25_production.txt"
  "prompts/codex_workstream_01_effect_and_controls.md"
  "prompts/codex_workstream_02_policy_and_adjudication.md"
  "prompts/codex_workstream_03_remote_settlement_schemas_ci.md"
  "scripts/audit_v25_production_gap.py"
  "scripts/check_no_local_recomposition.sh"
  "scripts/check_v25_production_closure.py"
  "scripts/run_v25_production_pack_checks.sh"
)

for rel in "${required_files[@]}"; do
  test -f "$ROOT/$rel" || { echo "missing production pack file: $rel" >&2; exit 1; }
done

grep -q 'V25P-101' "$ROOT/docs/v25/PRODUCTION_MASTER_ISSUE_MATRIX_20260318.md" || { echo 'issue matrix missing V25P-101' >&2; exit 1; }
grep -q 'effect-runtime' "$ROOT/docs/v25/PRODUCTION_GAP_AUDIT_20260318.md" || { echo 'gap audit missing effect-runtime finding' >&2; exit 1; }
grep -q 'do not let any consumer crate inspect raw profile fields' "$ROOT/prompts/codex_finish_handoff_prompt_v25_production.txt" || { echo 'handoff prompt missing raw-profile guardrail' >&2; exit 1; }

echo "v25 production pack truth checks passed"
