#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT/scripts/check_repo_surface.sh"
TMP_BASE="${TMPDIR:-$ROOT/.tmp}"
mkdir -p "$TMP_BASE"
TMP="$(mktemp -d "$TMP_BASE/check_repo_surface.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TMP/docs" "$TMP/scripts" "$TMP/release"
mkdir -p "$TMP/docs/archive/root_closeout_history"
cat >"$TMP/README.md" <<'EOF'
Start with PACK_README.md and MASTER_ISSUE_MATRIX.md and STATUS_DASHBOARD.md and SUPPORT_PROFILE.md and AGENTS.md and PROMPT.md and docs/README.md
EOF
cat >"$TMP/Makefile" <<'EOF'
# references PACK_README.md MASTER_ISSUE_MATRIX.md STATUS_DASHBOARD.md SUPPORT_PROFILE.md AGENTS.md PROMPT.md docs/README.md
EOF

  touch "$TMP/PACK_README.md" \
      "$TMP/MASTER_ISSUE_MATRIX.md" \
      "$TMP/SOURCE_BASIS.md" \
      "$TMP/SUPPORT_PROFILE.md" \
      "$TMP/IMPLEMENTATION_PLAYBOOK.md" \
      "$TMP/CONFORMANCE_GATES.md" \
      "$TMP/PHASED_EXECUTION_PLAN.md" \
      "$TMP/RISKS_AND_FORBIDDEN_SHORTCUTS.md" \
      "$TMP/STATUS_DASHBOARD.md" \
      "$TMP/STATUS_EVIDENCE_MANIFEST.json" \
      "$TMP/RELEASE_CHECKLIST.md" \
      "$TMP/release/closeout_receipt_v1.json" \
      "$TMP/AGENTS.md" \
      "$TMP/PROMPT.md" \
      "$TMP/docs/archive/root_closeout_history/README.md" \
      "$TMP/docs/archive/root_closeout_history/manifest.json" \
      "$TMP/docs/README.md" \
      "$TMP/docs/DIGEST_MIGRATION_RUNBOOK.md" \
      "$TMP/docs/REPO_SURFACE_REPAIR_SPEC.md" \
      "$TMP/docs/TOOL_RUNTIME_INTEGRATION_PLAN.md" \
      "$TMP/docs/TEST_STRATEGY_AND_FIXTURE_PLAN.md" \
      "$TMP/docs/COMPATIBILITY_BURNDOWN_PLAN.md" \
      "$TMP/scripts/check_repo_surface.sh" \
      "$TMP/scripts/test_check_repo_surface.sh" \
      "$TMP/scripts/generate_closeout_receipt.py" \
      "$TMP/scripts/check_manifest_truth.sh" \
      "$TMP/scripts/check_schema_compat.sh" \
      "$TMP/scripts/check_doc_truth.sh" \
      "$TMP/scripts/check_excluded_ecosystem_smoke.sh" \
      "$TMP/scripts/check_hotspot_budgets.sh" \
      "$TMP/scripts/check_public_type_drift.py" \
      "$TMP/scripts/public_type_drift_allowlist.json" \
      "$TMP/scripts/check_schema_registry_uniqueness.sh" \
      "$TMP/scripts/check_mirror_discipline.sh" \
      "$TMP/scripts/check_root_archive_manifest.py" \
      "$TMP/scripts/check_closeout_receipt.py" \
      "$TMP/scripts/check_public_api_docs.py"

bash "$SCRIPT" "$TMP"

rm "$TMP/MASTER_ISSUE_MATRIX.md"
if bash "$SCRIPT" "$TMP" >/dev/null 2>&1; then
  echo "negative self-test unexpectedly passed" >&2
  exit 1
fi

echo "repo surface self-tests passed"
