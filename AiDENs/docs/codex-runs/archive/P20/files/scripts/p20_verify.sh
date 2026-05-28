#!/usr/bin/env bash
set -euo pipefail
# P20 compatibility wrapper retained for older docs/tools/tests that still invoke scripts/p20_verify.sh.
exec bash scripts/p20_1_verify.sh "$@"
