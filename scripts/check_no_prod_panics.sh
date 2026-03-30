#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

if ! command -v jq >/dev/null 2>&1; then
  echo "supported-lane panic audit failed: jq is required" >&2
  exit 1
fi

allowlist_path="scripts/prod_panic_allowlist.json"
if [[ ! -f "$allowlist_path" ]]; then
  echo "supported-lane panic audit failed: missing allowlist file $allowlist_path" >&2
  exit 1
fi

declare -A allowlist=()
while IFS= read -r entry; do
  [[ -n "$entry" ]] || continue
  allowlist["$entry"]=1
done < <(jq -r '.[]' "$allowlist_path")

pattern='\.unwrap[[:space:]]*\(|\.expect[[:space:]]*\(|\<panic![[:space:]]*\(|\<todo![[:space:]]*\(|\<unimplemented![[:space:]]*\('
errors=0
allowlisted_hits=0

mapfile -t supported_crates < <(python3 scripts/print_supported_lane.py)
if [[ "${#supported_crates[@]}" -eq 0 ]]; then
  echo "supported-lane panic audit failed: supported lane is empty" >&2
  exit 1
fi

for crate in "${supported_crates[@]}"; do
  src_dir="$crate/src"
  if [[ "$crate" == "forge-engine" ]]; then
    src_dir="living-memory/living-memory/src"
  fi
  if [[ ! -d "$src_dir" ]]; then
    echo "supported-lane panic audit failed: missing source directory $src_dir" >&2
    exit 1
  fi

  while IFS= read -r -d '' target; do
    base_name="$(basename "$target")"
    case "$base_name" in
      tests.rs|lib_tests.rs|*_tests.rs)
        continue
        ;;
    esac

    while IFS= read -r numbered_line; do
      line_number="${numbered_line%%:*}"
      code="${numbered_line#*:}"
      code="${code%%//*}"

      if [[ "$code" =~ $pattern ]]; then
        entry="$target:$line_number"
        if [[ -z "${allowlist[$entry]:-}" ]]; then
          echo "supported-lane panic audit failed: $entry" >&2
          errors=1
        else
          allowlisted_hits=$((allowlisted_hits + 1))
        fi
      fi
    done < <(awk '
      /^#\[cfg\(test\)\]/ { in_test=1; depth=0; next }
      in_test { for(i=1;i<=length($0);i++){c=substr($0,i,1); if(c=="{")depth++; if(c=="}")depth--}; if(depth<=0)in_test=0; next }
      {print}
    ' "$target" | nl -ba -w1 -s:)
  done < <(find "$src_dir" -type f -name '*.rs' -print0 | sort -z)
done

if [[ "$errors" -ne 0 ]]; then
  exit 1
fi

echo "supported-lane panic audit passed (${allowlisted_hits} allowlisted compatibility shortcut(s))"
