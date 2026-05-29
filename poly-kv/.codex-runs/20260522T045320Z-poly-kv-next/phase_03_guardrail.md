# Phase 03 Boundary Guardrail

Guardrail sources:

- `codex/manual-injections/PHASE_BOUNDARY_GUARDRAIL.md`
- `codex/manual_injections/AFTER_PHASE_03.md`

## Revalidation

1. Source-of-truth ownership: fallback storage and pool build semantics remain in `poly-kv`; codec/shape primitives remain in `quant-codec-core`.
2. Duplicate abstraction or shadow implementation: no duplicate fallback type or codec math introduced.
3. Silent schema widening, coercion, hidden fallback, fake compatibility: the fallback convenience is explicitly named `build_from_exact_blocks`; adapter stubs remain unsupported.
4. Material operations and receipts: convenience path delegates to `build_from_blocks`, preserving build receipt, eval receipts, and manifest accounting.
5. Exact fallback: available for every built pool; convenience method derives it directly from exact blocks.
6. Optional adapters: inspected `crates/poly-kv/src/adapters/turbo_quant.rs` and `crates/poly-kv/src/adapters/fibquant.rs`; both return `UnsupportedAdapter`.
7. Tests/fixtures/assertions: added `builder_can_derive_exact_fallback_from_input_blocks`.
8. Failed/skipped validation: none in Phase 03.
9. Scope exclusions: no governor, runtime authority, semantic-memory, Gloss, Recall, AiDENs, or ClaimLedger integration introduced.
10. Rollback path: revert `crates/poly-kv/src/pool.rs` and `crates/poly-kv/tests/synthetic_roundtrip.rs` Phase 03 changes.

Decision: proceed to Phase 04.
