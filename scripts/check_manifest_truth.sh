#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

if ! command -v rg >/dev/null 2>&1; then
  echo "manifest truth check failed: rg is required" >&2
  exit 1
fi

extract_package_value() {
  local cargo_file="$1"
  local key="$2"
  awk -v key="$key" '
    /^\[package\]$/ { in_pkg=1; next }
    /^\[/ { if (in_pkg) exit }
    !in_pkg { next }
    $0 ~ "^[[:space:]]*" key "[[:space:]]*=" {
      sub("^[[:space:]]*" key "[[:space:]]*=[[:space:]]*", "", $0)
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", $0)
      gsub(/^"/, "", $0)
      gsub(/"$/, "", $0)
      print $0
      exit
    }
  ' "$cargo_file"
}

package_has_key() {
  local cargo_file="$1"
  local key="$2"
  awk -v key="$key" '
    /^\[package\]$/ { in_pkg=1; next }
    /^\[/ { if (in_pkg) exit }
    !in_pkg { next }
    $0 ~ "^[[:space:]]*" key "[[:space:]]*=" { found=1; exit }
    END { exit(found ? 0 : 1) }
  ' "$cargo_file"
}

mapfile -t cargo_files < <(rg --files -g 'Cargo.toml' -g '!_salvage_from_libraries2/**' | sort)
if [[ ${#cargo_files[@]} -eq 0 ]]; then
  echo "manifest truth check skipped (no Cargo.toml files found)"
  exit 0
fi

errors=0

if [[ -f Cargo.toml ]] && ! grep -q '\[workspace\.dependencies\]' Cargo.toml; then
  echo "Cargo.toml: missing [workspace.dependencies] table" >&2
  errors=1
fi

if [[ ! -f Primitives/Cargo.toml ]]; then
  echo "Primitives/Cargo.toml: missing Primitives workspace manifest" >&2
  errors=1
else
  if ! grep -q '\[workspace\]' Primitives/Cargo.toml; then
    echo "Primitives/Cargo.toml: missing [workspace] table" >&2
    errors=1
  fi
  if ! grep -q 'forge-policy' Primitives/Cargo.toml || ! grep -q 'check-runner' Primitives/Cargo.toml; then
    echo "Primitives/Cargo.toml: missing expected primitive workspace members" >&2
    errors=1
  fi
fi

for cargo in "${cargo_files[@]}"; do
  if [[ ! -r "$cargo" ]]; then
    echo "cannot read $cargo" >&2
    errors=1
    continue
  fi

  if ! grep -q '^\[package\]$' "$cargo"; then
    continue
  fi

  readme="$(extract_package_value "$cargo" "readme")"
  description="$(extract_package_value "$cargo" "description")"
  license_value="$(extract_package_value "$cargo" "license")"

  if [[ -n "$readme" && ! -f "$(dirname "$cargo")/$readme" ]]; then
    echo "$cargo: readme path missing -> $readme" >&2
    errors=1
  fi
  if package_has_key "$cargo" "description" && [[ -z "$description" ]]; then
    echo "$cargo: empty description" >&2
    errors=1
  fi
  if package_has_key "$cargo" "license" && [[ -z "$license_value" ]]; then
    echo "$cargo: empty license" >&2
    errors=1
  fi
done

if [[ "$errors" -ne 0 ]]; then
  echo "manifest truth check failed" >&2
  exit 1
fi

echo "manifest truth check passed"
