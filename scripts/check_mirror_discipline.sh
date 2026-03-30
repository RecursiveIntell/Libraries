#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

sync_script="apply/v25/SYNC_LIBRARIES_SOURCE_MIRROR.sh"
mirror_dir="libraries-source"

if [[ ! -f "$sync_script" ]]; then
  echo "mirror discipline: sync script $sync_script not present (excluded from pack scope, skipping)" >&2
  echo "mirror discipline check passed (sync script excluded)"
  exit 0
fi

if [[ ! -x "$sync_script" ]]; then
  echo "mirror discipline failed: $sync_script must be executable" >&2
  exit 1
fi

if ! grep -Eq 'rsync -a --delete' "$sync_script"; then
  echo "mirror discipline failed: sync script must call rsync with --delete" >&2
  exit 1
fi

if ! grep -Eq "apply/v25" "$sync_script" || ! grep -Eq "libraries-source" "$sync_script"; then
  echo "mirror discipline failed: sync script must reference apply/v25 and libraries-source" >&2
  exit 1
fi

if [[ ! -d "$mirror_dir" ]]; then
  echo "mirror discipline check passed (no mirror directory present)"
  exit 0
fi

if [[ ! -f "$mirror_dir/README.md" ]]; then
  echo "mirror discipline failed: mirror is present but README.md is missing" >&2
  exit 1
fi

echo "mirror discipline check passed"
