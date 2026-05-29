# Benchmark Tiers

Benchmarks are recorded as receipts first. README text can state that harnesses exist, but must not infer speed, memory, or quality claims from a single local receipt.

| Tier | Scope | Current artifact |
|---|---|---|
| Tier 0 | Rust synthetic fixture validation | `.codex-runs/$RUN_ID/rust_synthetic_bench.json` |
| Tier 1 | Python package/native boundary | `.codex-runs/$RUN_ID/python_boundary_bench.json` |
| Receipt parity | Harness status comparison | `.codex-runs/$RUN_ID/receipt_parity_report.json` |
| Tier 2 | one small model fixture | planned |
| Tier 3 | larger model fixture | planned |
| Tier 4 | serving runtime adapter fixture | planned |

Current scripts:

```bash
python3 scripts/bench_rust_synthetic.py --run-id "$RUN_ID"
python3 scripts/bench_boundary.py --run-id "$RUN_ID"
python3 scripts/compare_receipts.py --run-id "$RUN_ID"
```

If Python native bindings are unavailable, Tier 1 must emit `status: "skip"` with an exact reason.
