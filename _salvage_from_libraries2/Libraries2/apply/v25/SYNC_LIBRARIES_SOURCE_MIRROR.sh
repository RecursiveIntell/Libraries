#!/usr/bin/env bash
set -euo pipefail

apply_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
mirror_root="$repo_root/libraries-source"

# This apply/v25 utility syncs the active repo root into the derived libraries-source mirror.
if [[ "$apply_root" != *"/apply/v25" ]]; then
  echo "expected apply/v25 sync entrypoint, got $apply_root" >&2
  exit 1
fi

if [[ ! -d "$mirror_root" ]]; then
  echo "libraries-source mirror not present; nothing to sync"
  exit 0
fi

rsync -a --delete   --exclude 'libraries-source'   --exclude 'target'   "$repo_root/" "$mirror_root/"

echo "libraries-source mirror synchronized from active repo root"
