#!/usr/bin/env bash
set -euo pipefail
# Compatibility wrapper. Always delegate to the current-run verifier.
exec bash "$(dirname "${BASH_SOURCE[0]}")/verify_current.sh" "$@"
