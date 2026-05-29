# Phase 05 Boundary Guardrail

Guardrail sources:

- `codex/manual-injections/PHASE_BOUNDARY_GUARDRAIL.md`
- `codex/manual_injections/AFTER_PHASE_05.md`

## Revalidation

1. Source-of-truth ownership: Python sidecar remains a binding/wrapper layer; Rust crates remain source of shape validation, pool build, decode, receipts, and accounting.
2. Duplicate abstraction or shadow implementation: Python wrappers do not implement compression, fallback, or pool semantics.
3. Silent schema widening, coercion, hidden fallback, fake compatibility: unsupported native paths fail through `PolyKvShapeError` or skip when native is absent; no zero-copy claim made.
4. Material operations and receipts: native functions return JSON-compatible Rust receipts; Python tests assert receipt fields when native is present.
5. Exact fallback: native sidecar build path delegates to `build_from_exact_blocks`.
6. Optional adapters: no Python adapter implementation added.
7. Tests/fixtures/assertions: import, receipt parity, shape rejection, and no-silent-copy tests exist.
8. Failed/skipped validation: native tests skipped because `_native` is not built and `maturin` is unavailable.
9. Scope exclusions: no daemon mode, governor, runtime authority, semantic-memory, Gloss, Recall, AiDENs, or ClaimLedger integration introduced.
10. Rollback path: remove Python sidecar files and workspace member as listed in Phase 04.

Decision: proceed to Phase 06.
