#!/usr/bin/env bash
set -euo pipefail

KIT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPO_ROOT="${1:-}"

if [[ -z "$REPO_ROOT" ]]; then
  echo "usage: $0 /path/to/repo-root" >&2
  exit 1
fi

if [[ ! -d "$REPO_ROOT" ]]; then
  echo "repo root does not exist: $REPO_ROOT" >&2
  exit 1
fi

cp -R "$KIT_ROOT/repo_overlay/." "$REPO_ROOT/"
echo "overlay copied into $REPO_ROOT"
