#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"

required_files=(
  "24_V25_SUPERSESSION_AND_CONSTITUTIONAL_CHANGE_NOTE_20260317.md"
  "CANONICAL_STACK_SPEC_V25_EFFECTIVE_CONSTITUTION_PROFILE_COMPOSITION_AND_OBLIGATION_FOLDING_RUNTIME.md"
  "CANONICAL_STACK_SPEC_V26_ADVISORY_CONSTITUTIONAL_SEARCH_MINIMAL_EXCEPTION_SYNTHESIS_AND_POLICY_COUNTERFACTUAL_RUNTIME.md"
  "docs/v25/README.md"
  "docs/v25/MASTER_ISSUE_MATRIX.md"
  "docs/v25/PER_CRATE_APPLY_PLAN.md"
  "docs/v25/SCHEMA_REGISTRY_AND_COMPATIBILITY_PLAN.md"
  "docs/v25/TEST_AND_CONFORMANCE_PLAN.md"
  "docs/v25/RELEASE_BAR_AND_ACCEPTANCE.md"
  "docs/v25/REPO_GAP_REPORT_20260317.md"
  "docs/v25/RISK_REGISTER.md"
  "docs/v25/CURRENT_CODE_SNAPSHOT_NOTES_20260317.md"
  "docs/v25/FILE_CREATION_BACKLOG.md"
  "plans/v25-effective-constitution.execplan.md"
  "scripts/check_v25_json_surface.py"
  "scripts/run_v25_local_checks.sh"
  "contracts/fixtures/v25/manifest.json"
  "conformance/v25/manifest.json"
  "profile-runtime/src/lib.rs"
)

for rel in "${required_files[@]}"; do
  test -f "$ROOT/$rel" || { echo "missing required file: $rel" >&2; exit 1; }
done

grep -q "Supersession note (2026-03-17)" "$ROOT/00_START_HERE.md" || { echo "00_START_HERE.md missing supersession note" >&2; exit 1; }
grep -q "Supersession note (2026-03-17)" "$ROOT/README.md" || { echo "README.md missing supersession note" >&2; exit 1; }
grep -q '"profile-runtime"' "$ROOT/Cargo.toml" || { echo "workspace missing profile-runtime" >&2; exit 1; }
grep -q 'write_schema::<profile_runtime::EffectiveConstitutionV1>' "$ROOT/contract-schema-gen/src/lib.rs" || { echo "contract-schema-gen missing v25 registration" >&2; exit 1; }
grep -q 'rsync -a --delete' "$ROOT/apply/v25/SYNC_LIBRARIES_SOURCE_MIRROR.sh" || { echo "mirror sync script not updated" >&2; exit 1; }

python3 "$ROOT/scripts/check_v25_json_surface.py"

echo "v25 repo truth checks passed"
