# Target Final State

The next pass should leave the repository in this state:

```text
poly-kv/
  Cargo.toml
  AGENTS.md
  README.md
  crates/
    quant-codec-core/
      # hardened shape/ID/digest/eval core
    poly-kv/
      # hardened pool, receipts, accounting, exact fallback, q8 key path
    poly-kv-python/                 # new, optional, feature/bindings crate
      Cargo.toml
      src/lib.rs
  python/
    poly_kv/
      __init__.py
      _types.py
      adapters/
        __init__.py
        hf.py                        # prototype only; no universal compatibility claim
        torch.py                     # DLPack/import helper prototype only
      py.typed
      _native.pyi
    tests/
      test_import.py
      test_receipt_parity.py
      test_shape_rejection.py
      test_no_silent_copy.py
  docs/
    NEXT_RELEASE_PLAN.md
    PY_SIDECAR_SPEC.md
    BENCHMARK_TIERS.md
    GENERATED_SCHEMA_POLICY.md
    CLAIM_BOUNDARY.md
  scripts/
    preflight.sh
    run_rust_gates.sh
    run_python_gates.sh
    validate_final_state.py
    check_public_claims.py
    assert_receipt_integrity.py
    assert_realized_accounting.py
    assert_no_boundary_drift.py
  .codex-runs/<run-id>/
    startup_preflight.md
    source_inventory.md
    phase_00_report.md ... phase_10_report.md
    commands_run.log
    changed_files.txt
    validation_results.md
    invariant_report.md
    risk_register.md
    rollback_plan.md
    final_audit_report.md
    remaining_delta.md
```

## Required final properties

1. Rust core remains usable without Python dependencies.
2. Python sidecar is optional and does not contaminate core crate dependency graph.
3. Shape V2 models batch, query heads, KV heads, dtype, layout, and attention kind explicitly.
4. Existing alpha API is either preserved or superseded with clear deprecation and migration notes.
5. Lossy compression eval receipts are persisted/retrievable.
6. Manifest byte accounting is realized, not estimated.
7. Reader scratch accounting uses actual attached-reader budgets.
8. Decode receipts disclose full-block decode vs slice decode.
9. Exact fallback remains mandatory for alpha2 build fixtures.
10. `turbo-quant` / `fib-quant` adapters remain unsupported unless their APIs are inspected and tests pass.
11. Python API is bulk-oriented: build pool, attach reader, decode layer/slice, receipts. No per-token hot-path API.
12. Python sidecar emits receipts for GIL release, copy behavior, dtype/layout validation, and errors.
13. Tier 0 synthetic and Tier 1 Python smoke benchmarks emit JSON receipts.
14. README stays claim-bounded: no production-ready, no 2.91x end-to-end claim, no all-HF/vLLM/llama.cpp compatibility claim.
