# Phase 02 Boundary Guardrail

Guardrail sources:

- `codex/manual-injections/PHASE_BOUNDARY_GUARDRAIL.md`
- `codex/manual_injections/AFTER_PHASE_02.md`

## Revalidation

1. Source-of-truth ownership: receipt/accounting changes remain in `poly-kv`; codec IDs, profile traits, shape types, and eval report remain in `quant-codec-core`.
2. Duplicate abstraction or shadow implementation: no TurboQuant/FibQuant math or duplicate codec framework introduced.
3. Silent schema widening, coercion, hidden fallback, fake compatibility: receipt schema fields are explicit; fallback receipts remain explicit; no compatibility layer added.
4. Material operations and receipts: build now stores compression eval receipts; decode receipts disclose full-block decode, returned values, scratch bytes, copy behavior, and fallback if used.
5. Exact fallback: unchanged and still required by `PoolBuilder::build_from_blocks`; fallback decode returns `FallbackReceiptV1`.
6. Optional adapters: unchanged; no external compatibility claim added.
7. Tests/fixtures/assertions: updated tests cover persisted eval receipts, canonical manifest byte length, mixed reader scratch budgets, and full-block decode receipts.
8. Failed/skipped validation: one failed run of `scripts/assert_realized_accounting.py` was caused by a baseline `SyntaxError`; repaired and rerun successfully.
9. Scope exclusions: no governor, runtime authority, semantic-memory, Gloss, Recall, AiDENs, or ClaimLedger integration introduced.
10. Rollback path: revert the Phase 02 changed files listed in `phase_02_report.md`.

Decision: proceed to Phase 03.
