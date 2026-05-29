# Phase 06 Report

## Scope

Added Tier 0/Tier 1 benchmark and receipt harness scripts that emit raw JSON artifacts under the active run directory.

## Changed files

- `scripts/bench_rust_synthetic.py`
- `scripts/bench_boundary.py`
- `scripts/compare_receipts.py`
- `.codex-runs/20260522T045320Z-poly-kv-next/rust_synthetic_bench.json`
- `.codex-runs/20260522T045320Z-poly-kv-next/python_boundary_bench.json`
- `.codex-runs/20260522T045320Z-poly-kv-next/receipt_parity_report.json`

## Implementation

- `bench_rust_synthetic.py` runs a deterministic Rust synthetic q8 test and records elapsed wall-clock time, command, exit code, and raw tails. It makes no performance claim.
- `bench_boundary.py` checks Python package/native boundary availability and records skip reason when `_native` is unavailable.
- `compare_receipts.py` validates the harness output states and writes a parity report that accepts explicit Python native skips.

## Validation

Commands and results:

- `python3 scripts/bench_rust_synthetic.py --run-id 20260522T045320Z-poly-kv-next`: pass, wrote `rust_synthetic_bench.json`
- `python3 scripts/bench_boundary.py --run-id 20260522T045320Z-poly-kv-next`: pass, wrote `python_boundary_bench.json` with `status: skip`
- `python3 scripts/compare_receipts.py --run-id 20260522T045320Z-poly-kv-next`: pass, wrote `receipt_parity_report.json`
- `python -m json.tool` for all three emitted JSON files: pass

## Skips

Python boundary benchmark status is `skip` because `poly_kv._native` is not built; this is recorded in `python_boundary_bench.json` and `receipt_parity_report.json`.
