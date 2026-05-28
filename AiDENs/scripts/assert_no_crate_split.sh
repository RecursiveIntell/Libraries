#!/usr/bin/env bash
set -euo pipefail
# This run is forbidden from splitting aidens-contracts.
if find crates -maxdepth 1 -type d -name 'aidens-contracts-*' | grep -q .; then
  echo "FAIL: aidens-contracts split crate detected. Split is forbidden in this run."
  find crates -maxdepth 1 -type d -name 'aidens-contracts-*'
  exit 1
fi
echo "PASS: no aidens-contracts split crates detected."
