# Phase 00 Boundary Guardrail

Guardrail sources:

- `codex/manual-injections/PHASE_BOUNDARY_GUARDRAIL.md`
- `codex/manual_injections/AFTER_PHASE_00.md`

## Revalidation

1. Source-of-truth ownership: no implementation files touched. Ownership remains `quant-codec-core` for codec/profile/digest/shape primitives and `poly-kv` for pool/manifest/receipt/fallback semantics.
2. Duplicate abstraction or shadow implementation: none introduced; only run audit files were added under `.codex-runs/20260522T045320Z-poly-kv-next/`.
3. Silent schema widening, coercion, hidden fallback, fake compatibility: none introduced.
4. Material operations and receipts: no material operation added or changed.
5. Exact fallback: unchanged; `crates/poly-kv/src/codecs/raw_exact.rs` remains the exact fallback owner.
6. Optional adapters: unchanged; no compatibility claim added.
7. Tests/fixtures/assertions: existing tests inventoried; no test changes in Phase 00.
8. Failed/skipped validation: none in Phase 00 preflight; dirty baseline recorded as existing state.
9. Scope exclusions: no governor, runtime authority, semantic-memory, Gloss, Recall, AiDENs, or ClaimLedger integration introduced.
10. Rollback path: remove `.codex-runs/20260522T045320Z-poly-kv-next/` to roll back Phase 00 audit artifacts.

Decision: proceed to Phase 01.
