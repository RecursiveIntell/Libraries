# Phase 07 — Docs and public claims

Update:

- README.md
- crates/poly-kv/README.md
- docs/NEXT_RELEASE_PLAN.md
- docs/PY_SIDECAR_SPEC.md
- docs/BENCHMARK_TIERS.md
- docs/CLAIM_BOUNDARY.md

Allowed claims only:

- Rust crate provides shared KV pool manifests, receipts, exact fallback, q8 key experiments.
- Python sidecar is alpha/experimental and bulk-oriented.
- HF/PyTorch adapters are prototype/experimental unless real tests prove more.

Forbidden claims:

- production-ready
- universal HF support
- vLLM/llama.cpp/Candle/Burn/tch compatibility
- 2.91x end-to-end storage
- no quality loss
- throughput/latency improvement
- benchmark superiority

Gate: `python3 scripts/check_public_claims.py` passes.
