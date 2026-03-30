#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"
cd "$ROOT"

bash scripts/check_v25_repo_truth.sh "$ROOT"

if command -v cargo >/dev/null 2>&1; then
  cargo run -p contract-schema-gen -- schemas
  cargo run -p contract-schema-gen -- --check schemas
  cargo test -p stack-ids
  cargo test -p verification-policy
  cargo test -p profile-runtime
  cargo test -p knowledge-runtime
else
  echo "cargo not found; skipped Rust-specific checks" >&2
fi
