#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

required_files=(
  "README.md"
  "SUPPORT_PROFILE.md"
  "STATUS_DASHBOARD.md"
  "CONFORMANCE_GATES.md"
  "CLAUDE.md"
  "PACK_MANIFEST.json"
  "release/closeout_receipt_v1.json"
  "AGENTS.md"
  "PROMPT.md"
  "scripts/check_repo_surface.sh"
  "scripts/check_doc_truth.sh"
  "scripts/check_hotspot_budgets.sh"
  "scripts/check_public_type_drift.py"
  "scripts/public_type_drift_allowlist.json"
  "scripts/check_mirror_discipline.sh"
  "scripts/check_public_api_docs.py"
  "scripts/lane_manifest.json"
)

# Optional files that enhance but do not block
optional_files=(
  "MASTER_ISSUE_MATRIX.md"
  "SOURCE_BASIS.md"
  "IMPLEMENTATION_PLAYBOOK.md"
  "RISKS_AND_FORBIDDEN_SHORTCUTS.md"
  "STATUS_EVIDENCE_MANIFEST.json"
  "scripts/test_check_repo_surface.sh"
  "scripts/generate_closeout_receipt.py"
  "scripts/check_manifest_truth.sh"
  "scripts/check_schema_compat.sh"
  "scripts/check_excluded_ecosystem_smoke.sh"
  "scripts/check_schema_registry_uniqueness.sh"
  "scripts/check_root_archive_manifest.py"
  "scripts/check_closeout_receipt.py"
)
for f in "${optional_files[@]}"; do
  if [[ ! -f "$f" ]]; then
    echo "optional: $f (not present, not blocking)" >&2
  fi
done

missing=0
for f in "${required_files[@]}"; do
  if [[ ! -f "$f" ]]; then
    echo "missing: $f" >&2
    missing=1
  fi
done

# Optional front-door files to verify only if they exist
front_door_files=()
for candidate in README.md Makefile; do
  [[ -f "$candidate" ]] && front_door_files+=("$candidate")
done

if (( ${#front_door_files[@]} > 0 )); then
  for ref in STATUS_DASHBOARD.md SUPPORT_PROFILE.md; do
    found=0
    for f in "${front_door_files[@]}"; do
      if grep -q "$ref" "$f"; then
        found=1
        break
      fi
    done
    if (( found == 0 )); then
      echo "front-door reference missing for $ref in ${front_door_files[*]}" >&2
      missing=1
    fi
  done
fi

if (( missing != 0 )); then
  echo "repo surface check failed" >&2
  exit 1
fi

echo "repo surface check passed"
