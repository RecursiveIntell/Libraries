# Phase 04 Boundary Guardrail

Guardrail sources:

- `codex/manual-injections/PHASE_BOUNDARY_GUARDRAIL.md`
- `codex/manual_injections/AFTER_PHASE_04.md`

## Revalidation

1. Source-of-truth ownership: Python sidecar owns only bindings/wrappers; `quant-codec-core` remains owner of shapes and codec traits, `poly-kv` remains owner of pool/receipt semantics.
2. Duplicate abstraction or shadow implementation: sidecar delegates to Rust crates and does not reimplement codec math.
3. Silent schema widening, coercion, hidden fallback, fake compatibility: unsupported dtype/layout/attention fail closed; sidecar exposes bulk JSON APIs only.
4. Material operations and receipts: native build/decode functions return JSON-compatible Rust receipts.
5. Exact fallback: sidecar builds through `build_from_exact_blocks`, preserving exact fallback.
6. Optional adapters: Python adapter namespace is empty/reserved; no external compatibility claim added.
7. Tests/fixtures/assertions: import smoke test exists; native-dependent tests exist and explicitly skip if `_native` is unavailable.
8. Failed/skipped validation: `maturin` unavailable; `maturin build` skipped/failed with exact reason.
9. Scope exclusions: no daemon mode, governor, runtime authority, semantic-memory, Gloss, Recall, AiDENs, or ClaimLedger integration introduced.
10. Rollback path: remove `crates/poly-kv-python`, `python/`, `pyproject.toml`, and the workspace member entry in `Cargo.toml`.

Decision: proceed to Phase 05.
