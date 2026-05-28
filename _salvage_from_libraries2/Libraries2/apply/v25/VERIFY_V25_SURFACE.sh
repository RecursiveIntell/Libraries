#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

bash scripts/check_v25_repo_truth.sh "$repo_root"

echo "v25 surface verification passed"
