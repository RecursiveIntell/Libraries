#!/usr/bin/env bash
set -euo pipefail
# Compatibility wrapper retained for legacy packaging checks.
exec bash scripts/p25_verify.sh "$@"
