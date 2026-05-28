#!/usr/bin/env bash
set -euo pipefail
FILE="crates/aidens-contracts/src/lib.rs"
if [[ ! -f "$FILE" ]]; then
  echo "error: $FILE not found" >&2
  exit 2
fi

FAIL=0
RISKY_TYPES=(
  BoundaryRepairReportV1
  JsonRepairReportV2
  SchemaValidationReportV1
  RuntimeViewRequestV1
  RetrievalPolicyV1
  QueryWideningReportV1
  DegradationEventV1
  ProjectionDigestV1
  ViewDisclosureReportV1
  RegionContractV1
  SyndromeV1
  ResidualV1
  SubtractionPlanV1
  SupportCoreV1
  RemovalFrontierV1
)

for ty in "${RISKY_TYPES[@]}"; do
  if grep -nE "pub (struct|enum|type) ${ty}\b" "$FILE" >/tmp/wrapper_hit.txt; then
    line="$(cut -d: -f1 /tmp/wrapper_hit.txt | head -1)"
    start=$(( line > 20 ? line - 20 : 1 ))
    end=$(( line + 80 ))
    window="$(sed -n "${start},${end}p" "$FILE" | tr '[:upper:]' '[:lower:]')"
    if ! grep -Eq 'canonical_backpointers|canonical_[a-z_]*(id|ids)|canonical_repair_record_ids|canonical_control_receipt_ids' <<<"$window"; then
      echo "FAIL: risky wrapper $ty lacks explicit canonical backpointer/id fields near definition."
      FAIL=1
    fi
  fi
done

if [[ "$FAIL" -ne 0 ]]; then
  echo "Risky local wrappers must carry canonical backpointers or be quarantined."
  exit 1
fi

echo "PASS: wrapper backpointer gate did not find blocking risky wrappers."
