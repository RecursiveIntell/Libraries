# Benchmark and Harness Spec

## Goal

Prove correctness, accounting, and sidecar overhead before making performance claims.

## Tiers

| Tier | Scope | Gate |
|---|---|---|
| Tier 0 | Rust synthetic fixtures | exact fallback, deterministic digests, shape rejection, accounting reconciliation |
| Tier 1 | Python boundary smoke | import, parity, no-silent-copy, receipts, NumPy CPU path |
| Tier 2 | one HF small-model prototype | explicit model/tokenizer fingerprint, cache extract/build/decode/inject receipt |
| Tier 3 | 7B/8B benchmark | stride PPL, attention/logit drift, memory accounting, local receipts |
| Tier 4 | runtime adapters | only after separate vLLM/llama.cpp/Candle/Burn adapter designs |

## Required metrics

- exact fallback bitwise equality
- manifest digest determinism
- realized serialized bytes
- per-reader scratch bytes
- reconstruction MSE / cosine / max abs error
- attention/logit drift once model-backed tests exist
- PPL delta once model-backed tests exist
- Python boundary overhead
- copy/zero-copy disclosure
- replay success

## Benchmark commands

```bash
cargo test --workspace --all-targets
cargo bench -p poly-kv --features bench --bench synthetic_pool -- --save-baseline alpha2
python -m pytest -q python/tests
python3 scripts/bench_rust_synthetic.py --run-id "$RUN_ID"
python3 scripts/bench_boundary.py --run-id "$RUN_ID"
python3 scripts/compare_receipts.py --run-id "$RUN_ID"
```

## Claim rule

Benchmark results go into `docs/benchmarks/` as receipts first. README can only state that benchmarks exist unless a claim is explicitly supported by local reproducible receipts.
