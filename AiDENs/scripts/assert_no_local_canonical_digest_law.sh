#!/usr/bin/env bash
set -euo pipefail
FILE="crates/aidens-contracts/src/lib.rs"
if [[ ! -f "$FILE" ]]; then
  echo "error: $FILE not found" >&2
  exit 2
fi

FAIL=0

# Exported or public canonical digest helpers are forbidden.
for pat in \
  'pub fn stable_json_digest' \
  'pub fn stable_text_digest' \
  'pub fn deterministic_artifact_id' \
  'pub struct CanonicalDigestV1' \
  'pub enum CanonicalDigestV1' \
  'pub type CanonicalDigestV1'; do
  if grep -RIn "$pat" "$FILE" >/tmp/aidens_digest_gate_hits.txt; then
    echo "FAIL: forbidden local canonical digest law: $pat"
    cat /tmp/aidens_digest_gate_hits.txt
    FAIL=1
  fi
done

# canonical_json_string may exist only if private and explicitly display/test scoped.
if grep -RIn 'pub fn canonical_json_string' "$FILE"; then
  echo "FAIL: canonical_json_string must not be exported from aidens-contracts"
  FAIL=1
fi

if [[ "$FAIL" -ne 0 ]]; then
  echo "Use stack-ids for canonical digest/identity semantics. Display-only digests must be renamed non-authoritative."
  exit 1
fi

echo "PASS: no exported local canonical digest law detected."
