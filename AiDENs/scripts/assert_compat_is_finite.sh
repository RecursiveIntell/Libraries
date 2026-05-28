#!/usr/bin/env bash
set -euo pipefail
ROOT="${1:-${AIDENS_REPO_ROOT:-$(pwd)}}"
LEDGER="${AIDENS_COMPAT_LEDGER:-$ROOT/COMPATIBILITY_LEDGER.md}"
if [[ ! -f "$LEDGER" ]]; then
  # Also allow pack-local ledger.
  LEDGER="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/COMPATIBILITY_LEDGER.md"
fi
[[ -f "$LEDGER" ]] || { echo "ERROR: COMPATIBILITY_LEDGER.md not found" >&2; exit 1; }

if [[ -e "$ROOT/crates/aidens-compat" ]]; then
  echo "ERROR: aidens-compat crate is forbidden; remove the compatibility surface." >&2
  exit 1
fi

if grep -qE '^\| `' "$LEDGER"; then
  echo "ERROR: compatibility ledger must not retain shim rows." >&2
  exit 1
fi

if grep -RInE 'LegacyAidens|#\[deprecated' "$ROOT/crates" --include='*.rs' >/tmp/aidens_compat_refs.txt 2>/dev/null; then
  echo "ERROR: compatibility shim markers are forbidden:" >&2
  cat /tmp/aidens_compat_refs.txt >&2
  exit 1
fi

required_headers=('Shim name' 'Reason' 'Canonical replacement' 'Removal criterion' 'Tests proving compatibility' 'Non-authoritative')
for h in "${required_headers[@]}"; do
  grep -q "$h" "$LEDGER" || { echo "ERROR: ledger missing header: $h" >&2; exit 1; }
done
