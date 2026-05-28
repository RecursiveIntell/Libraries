#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="${1:-.}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PACK_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

git -C "${REPO_ROOT}" apply "${PACK_ROOT}/patches/0001-ci-hardening.patch"

echo "applied: ${PACK_ROOT}/patches/0001-ci-hardening.patch"
