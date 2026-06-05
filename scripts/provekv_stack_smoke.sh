#!/usr/bin/env bash
set -euo pipefail

cargo test -p semantic-memory --features poly-kv-pool --test pool_generation_types
cargo test -p semantic-memory --lib provekv_pool_generation_db_tests --features poly-kv-pool
cargo run -p semantic-memory --features poly-kv-pool --example provekv_pool_benchmark_receipt > /tmp/provekv_pool_benchmark_receipt.json
python3 scripts/validate_provekv_benchmark_receipt.py /tmp/provekv_pool_benchmark_receipt.json
cargo test -p forge-memory-bridge --test provekv_lifecycle_receipts
cargo test -p semantic-memory-forge --test audit_candidate_search
cargo test -p claim-ledger --test similar_claim_candidates
cargo test -p claim-ledger --test proof_packet_candidate_provenance
cargo check -p knowledge-runtime -p llm-tool-runtime -p agent-graph -p llm-pipeline -p kernel-execution -p kernel-oracles
python3 scripts/validate_provekv_integration_boundaries.py --root .
