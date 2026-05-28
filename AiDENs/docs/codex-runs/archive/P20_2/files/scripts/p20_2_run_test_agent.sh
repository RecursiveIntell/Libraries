#!/usr/bin/env bash
set -euo pipefail
cargo test -p aidens-integration-tests test_agent_vertical_slice -- --nocapture
