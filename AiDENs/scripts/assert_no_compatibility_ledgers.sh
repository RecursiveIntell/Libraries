#!/usr/bin/env bash
set -euo pipefail
FAIL=0

# Check compatibility ledgers for non-header rows.
for f in COMPATIBILITY_LEDGER.md docs/contract-ownership/COMPATIBILITY_LEDGER.md; do
  [[ -f "$f" ]] || continue
  # Markdown header and separator are not entries. Count only rows after the
  # first separator row, so an empty ledger is accepted but any data row fails.
  rows=$(awk '
    /^\|[-: ]+\|/ { separator_seen = 1; next }
    separator_seen && /^\|/ { count++ }
    END { print count + 0 }
  ' "$f")
  if [[ "$rows" -gt 0 ]]; then
    echo "FAIL: compatibility ledger $f has entries. Compatibility shims are forbidden in this run."
    FAIL=1
  fi
done

# Also detect obvious new compat/shim modules in aidens-contracts.
if find crates/aidens-contracts -type f \( -name '*compat*' -o -name '*shim*' -o -name '*legacy*' \) | grep -q .; then
  echo "FAIL: compat/shim/legacy file detected under aidens-contracts."
  find crates/aidens-contracts -type f \( -name '*compat*' -o -name '*shim*' -o -name '*legacy*' \)
  FAIL=1
fi

if [[ "$FAIL" -ne 0 ]]; then
  exit 1
fi

echo "PASS: no compatibility ledger entries or obvious compat/shim files detected."
