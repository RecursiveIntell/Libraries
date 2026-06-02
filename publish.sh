#!/usr/bin/env bash
# publish.sh — publish all 13 crates to crates.io in topological order.
# Run from the parent Libraries/ directory.
#
# Usage: ./publish.sh [phase]
#   ./publish.sh 1     # Phase 1 only (no internal deps)
#   ./publish.sh 2     # Phase 2 (depends on Phase 1)
#   ./publish.sh 3     # Phase 3 (the consumer)
#   ./publish.sh all   # All three phases
#
# Each `cargo publish` invocation needs:
#   1. An authenticated session (run `cargo login` first with a token that
#      has publish scope for the "sikmindz" account)
#   2. Network access to crates.io
#   3. The current directory to be the correct sub-workspace

set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

publish_in_workspace() {
    local workspace="$1"
    local crate="$2"
    local extra_args="${3:-}"
    echo "============================================================"
    echo "Publishing: $crate  (workspace: $workspace)"
    echo "============================================================"
    (cd "$workspace" && cargo publish -p "$crate" --allow-dirty $extra_args)
}

phase1() {
    # No internal-dep ordering requirements.
    # All from the parent Libraries/ workspace.
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
    # No internal-dep ordering within the parent Libraries/ workspace
    cd "$ROOT"
    publish_in_workspace . quant-eval
    publish_in_workspace . gpu-backend
    publish_in_workspace . turbo-quant
}

phase2b() {
    # Depends on phase 2a (gpu-backend)
    cd "$ROOT"
    publish_in_workspace . fib-quant
}

phase2c() {
    # Depends on phase 2a + 2b
    cd "$ROOT"
    publish_in_workspace . scr-runtime-compression
}

phase2d() {
    # Depends on phase 2a + 2b
    publish_in_workspace poly-kv poly-kv
}

phase3() {
    # The consumer
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
publish.sh — publish all 13 crates to crates.io in topological order.

Phases:
  1   boundary-compiler, bitemporal-runtime, stack-ids, semantic-memory-forge,
      forge-memory-bridge, quant-governor, quant-codec-core  (7 crates, no deps)
  2a  quant-eval, gpu-backend, turbo-quant  (3 crates, no internal deps)
  2b  fib-quant  (depends on gpu-backend)
  2c  scr-runtime-compression  (depends on fib+turbo+governor)
  2d  poly-kv  (depends on fib+turbo+gpu+core)
  3   semantic-memory  (depends on all)

Usage: ./publish.sh [phase]
  ./publish.sh 1   # only phase 1
  ./publish.sh 2   # all of phase 2
  ./publish.sh all # all phases

Pre-requisite:
  cargo login <token>   # token must have publish scope
EOF
        ;;
esac
