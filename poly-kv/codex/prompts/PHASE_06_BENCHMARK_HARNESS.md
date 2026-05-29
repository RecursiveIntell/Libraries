# Phase 06 — Tier 0/Tier 1 benchmark harness

Add scripts:

- `scripts/bench_rust_synthetic.py` or Rust bench JSON emitter
- `scripts/bench_boundary.py`
- `scripts/compare_receipts.py`

Emit JSON under `.codex-runs/$RUN_ID/`:

- `rust_synthetic_bench.json`
- `python_boundary_bench.json`
- `receipt_parity_report.json`

Do not put benchmark claims in README. Store raw receipts.

Gate: benchmark scripts run or skip with exact dependency reason; JSON schema validates.
