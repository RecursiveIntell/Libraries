#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"
cd "$ROOT"

TARGET_DIRS=(
  effect-runtime/src
  verification-control/src
  verification-adjudication/src
  remote-oracle-admission/src
  federated-settlement/src
)

PATTERNS=(
  'allowed_run_modes'
  'required_preflight_checks'
  'required_observation_classes'
  'requires_compensation_plan_for'
  'max_delegation_depth'
  'forbidden_role_combinations'
  'required_assurance_sections'
  'required_monitor_classes'
  'continuity_exception_ttl_minutes'
  'requires_postmortem_for_severity'
  'allowed_execution_regions'
  'forbidden_transfer_classes'
  'lossy_fields'
  'break_glass_requires_post_hoc_review'
  'EffectPolicyProfileV1'
  'DelegationPolicyProfileV1'
  'ReleasePolicyProfileV1'
  'ContinuityPolicyProfileV1'
  'ResidencyPolicyProfileV1'
  'TenantBoundaryProfileV1'
)

joined="$(IFS='|'; echo "${PATTERNS[*]}")"
if rg -n --hidden --glob '*.rs' "$joined" "${TARGET_DIRS[@]}"; then
  echo "raw profile handling detected in a target consumer crate" >&2
  exit 1
fi

echo "no local recomposition patterns detected in target consumer crates"
