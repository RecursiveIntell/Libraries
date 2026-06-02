#!/usr/bin/env bash
# publish.sh — publish all 14 crates to crates.io in topological order.
# Run from the parent Libraries/ directory.
#
# Usage: ./publish.sh [phase]
#   ./publish.sh 1     # Phase 1 only (no internal deps)
#   ./publish.sh 2a    # Phase 2a (no internal deps, but ordered for clarity)
#   ./publish.sh 2b    # Phase 2b (fib-quant, depends on gpu-backend)
#   ./publish.sh 2c    # Phase 2c (scr-runtime-compression)
#   ./publish.sh 2d    # Phase 2d (poly-kv)
#   ./publish.sh 3     # Phase 3 (semantic-memory — the consumer)
#   ./publish.sh all   # All phases
#
# Pre-requisite:
#   cargo login <token>   # token must have publish scope for sikmindz
#   Network access to crates.io
#
# The script runs each `cargo publish` in the correct sub-workspace.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

publish_in_workspace() {
    local workspace="$1"
    local crate="$2"
    echo "============================================================"
    echo "Publishing: $crate  (workspace: $workspace)"
    echo "============================================================"
    (cd "$workspace" && cargo publish -p "$crate" --allow-dirty)
}

phase1() {
    # No internal-dep ordering requirements.
    cd "$ROOT"
    publish_in_workspace . boundary-compiler
    publish_in_workspace . bitemporal-runtime
    publish_in_workspace . stack-ids
    publish_in_workspace . semantic-memory-forge
    publish_in_workspace . forge-memory-bridge
    publish_in_workspace . quant-governor

    # From the poly-kv sub-workspace
    publish_in_workspace poly-kv quant-codec-core
}

phase2a() {
    cd "$ROOT"
    publish_in_workspace . quant-eval
    publish_in_workspace . gpu-backend
    publish_in_workspace . turbo-quant
}

phase2b() {
    cd "$ROOT"
    publish_in_workspace . fib-quant
}

phase2c() {
    cd "$ROOT"
    publish_in_workspace . scr-runtime-compression
}

phase2d() {
    publish_in_workspace poly-kv poly-kv
}

phase3() {
    cd "$ROOT"
    publish_in_workspace . semantic-memory
}

case "${1:-help}" in
    1|phase1) phase1 ;;
    2a) phase2a ;;
    2b) phase2b ;;
    2c) phase2c ;;
    2d) phase2d ;;
    2) phase2a; phase2b; phase2c; phase2d ;;
    3) phase3 ;;
    all) phase1; phase2a; phase2b; phase2c; phase2d; phase3 ;;
    help|*)
        cat <<EOF
publish.sh — publish all 14 crates to crates.io in topological order.

Phases:
  1   boundary-compiler, bitemporal-runtime, stack-ids, semantic-memory-forge,
      forge-memory-bridge, quant-governor, quant-codec-core  (7 crates, no internal deps)
  2a  quant-eval, gpu-backend, turbo-quant  (3 crates, no internal deps)
  2b  fib-quant  (depends on gpu-backend)
  2c  scr-runtime-compression  (depends on fib+turbo+governor)
  2d  poly-kv  (depends on fib+turbo+gpu+core)
  3   semantic-memory  (the consumer, depends on everything)

Usage: ./publish.sh [phase]
  ./publish.sh 1   # only phase 1
  ./publish.sh 2   # all of phase 2
  ./publish.sh all # all phases

Pre-requisite:
  cargo login <token>   # token must have publish scope
EOF
        ;;
esac
