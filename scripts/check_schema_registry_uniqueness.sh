#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

if ! command -v jq >/dev/null 2>&1; then
  echo "schema registry check failed: jq is required" >&2
  exit 1
fi

shopt -s nullglob
manifests=(contracts/schemas/*/manifest.json)
if [[ ${#manifests[@]} -eq 0 ]]; then
  echo "schema registry check failed: no schema wave manifests found" >&2
  exit 1
fi

declare -A seen_wave=()
declare -A seen_owner=()
errors=0

for manifest_path in "${manifests[@]}"; do
  wave="$(basename "$(dirname "$manifest_path")")"
  owner_crate="$(jq -r '.owner_crate // .primary_owner // empty' "$manifest_path")"

  if [[ -z "$owner_crate" ]]; then
    echo "schema registry check failed: manifest missing owner_crate/primary_owner in $manifest_path" >&2
    errors=1
    continue
  fi

  mapfile -t schema_files < <(jq -r '(.schema_files // .schemas // [])[]?' "$manifest_path")
  if [[ ${#schema_files[@]} -eq 0 ]]; then
    echo "schema registry check failed: manifest missing schema_files/schemas in $manifest_path" >&2
    errors=1
    continue
  fi

  for schema_name in "${schema_files[@]}"; do
    if [[ -n "${seen_wave[$schema_name]:-}" ]]; then
      echo "schema ownership conflict: $schema_name appears in both ${seen_wave[$schema_name]} (owner ${seen_owner[$schema_name]}) and $wave (owner $owner_crate)" >&2
      errors=1
      continue
    fi

    if [[ ! -f "schemas/$schema_name" ]]; then
      echo "schema registry check failed: missing schema file schemas/$schema_name" >&2
      errors=1
      continue
    fi

    seen_wave["$schema_name"]="$wave"
    seen_owner["$schema_name"]="$owner_crate"
  done
done

if [[ "$errors" -ne 0 ]]; then
  exit 1
fi

echo "schema registry uniqueness checks passed"
