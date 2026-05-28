#!/usr/bin/env bash
set -euo pipefail

# Codex must wire this to the chosen Rust schema generation mechanism.
# Required behavior:
#   1. generate schemas from Rust types
#   2. write to schemas/generated/
#   3. fail if generated output differs from checked-in files unless the developer intentionally updates them

echo "TODO: wire schema generation from Rust types. This script must not silently pass once implementation begins."
exit 1
