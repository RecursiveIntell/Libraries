#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found; cannot regenerate schemas in this environment" >&2
  exit 1
fi

cargo run -p contract-schema-gen -- schemas
cargo run -p contract-schema-gen -- --check schemas
